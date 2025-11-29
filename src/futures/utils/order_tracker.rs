use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::anyhow;
use crate::futures::utils::order_tracking_item::OrderTrackingItem;
use crate::futures::utils::top_n::{TopN, TopNEntry};
use once_cell::sync::Lazy;
use dashmap::DashMap;
use uuid::Uuid;
use crate::errors::Result;
use crate::futures::account::OrderRequest;

const TOP_N_ORDER_TRACKING_CAPACITY: usize = 3000;
type OrderSymbol = String;

impl Default for TopN<OrderTrackingItem> {
    fn default() -> Self {
        Self::new(TOP_N_ORDER_TRACKING_CAPACITY, None)
    }
}

static SYMBOL_ORDER_TRACKING: Lazy<DashMap<OrderSymbol, TopN<OrderTrackingItem>>> =
    Lazy::new(DashMap::new);


fn get_file_path(order_symbol: &OrderSymbol) -> PathBuf {
    PathBuf::from(format!("order_tracker_{}.json", order_symbol))
}

pub fn add_order_tracking_item(order_request: &OrderRequest) -> Result<TopNEntry<OrderTrackingItem>> {
    let symbol = order_request.symbol.clone();
    let file_path = get_file_path(&symbol);

    let timestamp_nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos() as u64,
        Err(error) => return Err(anyhow!("Failed to get current time for order tracking item: {error:?}").into())
    };
    let tracking_item = OrderTrackingItem {
        size: match order_request.quantity {
            Some(quantity) => quantity,
            None => return Err(anyhow!("Quantity is required to add order tracking item").into())
        },
        price: match order_request.price {
            Some(price) => price,
            None => return Err(anyhow!("Price is required to add order tracking item").into())
        },
        side: order_request.side.clone(),
        id: format!("{}-{timestamp_nanos}", Uuid::new_v4()),
    };
    let new_item_entry = TopNEntry {
        timestamp: timestamp_nanos,
        item: tracking_item.clone(),
    };

    // --- LOCK ACQUIRED HERE ---
    // .or_default() gets the existing entry or inserts a
    // *new, empty* TopN, and returns a write-lock (RefMut).
    let mut top_n_ref = SYMBOL_ORDER_TRACKING
        .entry(symbol.clone())
        .or_default();

    //
    // This is your "or_else" logic:
    //
    // If the TopN is empty, it *might* be brand new.
    // Try to load it from disk.
    if top_n_ref.is_empty() && file_path.exists() {
        // Try to load
        match load_top_n_from_file(&file_path, TOP_N_ORDER_TRACKING_CAPACITY) {
            Ok(loaded_top_n) => {
                // Success! Replace the empty TopN with the loaded one.
                // We are still under the lock, so this is safe.
                *top_n_ref = loaded_top_n;
            }
            Err(error) => return Err(anyhow!("Failed to load order tracker file from {} when adding item {symbol} {new_item_entry:?}: {error:?}", file_path.display()).into())
        }
    }

    // Now, whether it was loaded or was pre-existing,
    // insert the new item (in-memory only).
    top_n_ref.insert(new_item_entry.clone());

    if let Err(error) = save_top_n_to_file(&top_n_ref, &file_path) {
        return Err(anyhow!("Failed to write TopN set to file when adding item {symbol} {new_item_entry:?}: {error:?}").into());
    }
    Ok(new_item_entry)
}

pub fn get_all_tracking_items(symbol: &str) -> Option<Vec<TopNEntry<OrderTrackingItem>>> {
    SYMBOL_ORDER_TRACKING
        .get(symbol)
        .map(|top_n| top_n.get_all()) // .get() returns a Ref<...>
}

pub fn get_gte_timestamp(symbol: &OrderSymbol, timestamp: u64) -> Option<Vec<TopNEntry<OrderTrackingItem>>> {
    SYMBOL_ORDER_TRACKING
        .get(symbol)
        .map(|top_n| top_n.get_gte_timestamp(timestamp))
}

fn save_top_n_to_file(top_n: &TopN<OrderTrackingItem>, save_file_path: &PathBuf) -> Result<()>
{
    let as_json = top_n.get_set_json()?;
    if let Err(error) = fs::write(save_file_path, as_json) {
        return Err(anyhow!("Failed to save top n to file: {error:?}").into())
    }
    Ok(())
}

// New function to load the set from a file
fn load_top_n_from_file(save_file_path: &PathBuf, capacity: usize) -> Result<TopN<OrderTrackingItem>>
{
    let data = match fs::read_to_string(save_file_path) {
        Ok(data) => data,
        Err(error) => return Err(anyhow!("Failed to read top n from file: {error:?}").into())
    };
    let set: BTreeSet<TopNEntry<OrderTrackingItem>> = match serde_json::from_str(&data) {
        Ok(set) => set,
        Err(error) => return Err(anyhow!("Failed to deserialize top n from json: {error:?}").into())
    };

    Ok(TopN::new(capacity, Some(set)))
}