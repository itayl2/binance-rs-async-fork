use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Debug;
use anyhow::anyhow;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use crate::errors;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopNEntry<T> {
    #[serde(rename = "ts")]
    pub timestamp: u64,
    #[serde(rename = "i")]
    pub item: T,
}

// T must now implement Ord using both the timestamp and the inner item because when we do .contains(), the btreeset will call .cmp(), and if the timestamp is identical and inner items are different, checking just the timestamp will determine they are equal and .contains() will return true even though the inner items are different.
impl<T: Eq + Ord> Ord for TopNEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Primary sort: timestamp (ascending)
        self.timestamp.cmp(&other.timestamp)
            // Secondary sort (tie-breaker): item
            .then_with(|| self.item.cmp(&other.item))
    }
}

impl<T: Eq + Ord> PartialOrd for TopNEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Implement PartialEq manually to be consistent with Ord
impl<T: Eq + Ord> PartialEq for TopNEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<T: Eq + Ord> Eq for TopNEntry<T> {}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Eq + Ord + Serialize",
    deserialize = "T: Eq + Ord + DeserializeOwned"
))]
pub struct TopN<T: Eq + Ord + Debug + DeserializeOwned> { // Add T: Ord
    set: BTreeSet<TopNEntry<T>>,
    pub capacity: usize,
}

impl<T> TopN<T>
where
    T: Eq + Ord + Clone + Debug + Serialize + DeserializeOwned,
{
    pub fn new(capacity: usize, set: Option<BTreeSet<TopNEntry<T>>>) -> Self {
        Self {
            set: set.unwrap_or_else(|| BTreeSet::new()),
            capacity,
        }
    }
    
    pub fn get_set_json(&self) -> errors::Result<String> {
        match serde_json::to_string(&self.set) {
            Ok(json) => Ok(json),
            Err(error) => Err(anyhow!("Failed to serialize top n to json: {error:?}").into()),
        }
    }

    pub fn insert(&mut self, item: TopNEntry<T>) {
        if self.set.contains(&item) {
            return;
        }
        
        self.set.insert(item);

        if self.set.len() > self.capacity {
            self.set.pop_first();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }
    
    pub fn get_capacity(&self) -> usize {
        self.capacity
    }
    
    pub fn get_gte_timestamp(&self, timestamp: u64) -> Vec<TopNEntry<T>> {
        self.set.iter().filter(|entry| entry.timestamp >= timestamp).cloned().collect()
    }
    
    pub fn get_lte_timestamp(&self, timestamp: u64) -> Vec<TopNEntry<T>> {
        self.set.iter().filter(|entry| entry.timestamp <= timestamp).cloned().collect()
    }
    
    pub fn get_all(&self) -> Vec<TopNEntry<T>> {
        self.set.iter().cloned().collect()
    }
    
    pub fn clear(&mut self) {
        self.set.clear();
    }
}