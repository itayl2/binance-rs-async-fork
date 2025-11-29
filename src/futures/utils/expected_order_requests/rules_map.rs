use std::collections::HashSet;
use anyhow::anyhow;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use crate::futures::utils::expected_order_requests::rule::ExpectedOrderRequestsRule;
use crate::errors::{Error, Result};
use crate::futures::utils::order_tracking_item::OrderTrackingItem;
use crate::futures::utils::top_n::TopNEntry;

type OrderRequestSymbol = String;
type ExpectedOrderRequestsRuleMap = DashMap<OrderRequestSymbol, HashSet<ExpectedOrderRequestsRule>>;

static EXPECTED_ORDER_REQUESTS_RULES: Lazy<ExpectedOrderRequestsRuleMap> = Lazy::new(ExpectedOrderRequestsRuleMap::new);

fn validate_rule_set_durations(symbol: &OrderRequestSymbol, rules: &HashSet<ExpectedOrderRequestsRule>) -> Result<()> {
    let mut durations = vec![];
    for rule in rules.iter() {
        let duration = match rule.get_duration() {
            Ok(duration) => duration,
            Err(error) => return Err(Error::ExpectedOrdersRuleViolated(format!("Failed to get duration for {symbol} rule: {}. Rule: {rule:?}", error.get_msg()))),
        };
        durations.push(duration);
    }

    let mut found_at_least_52_weeks = false;
    let mut found_hours_until_two_days = false;
    let min_hours = 1;
    let max_hours = 48;
    for duration in durations.iter() {
        let secs = duration.as_secs();
        let duration_in_weeks = secs / 60 / 60 / 24 / 7;
        if duration_in_weeks >= 52 {
            found_at_least_52_weeks = true;
        }

        let duration_in_hours = secs / 60 / 60;
        if duration_in_hours >= min_hours && duration_in_hours <= max_hours {
            found_hours_until_two_days = true;
        }
    }
    if !found_at_least_52_weeks {
        return Err(Error::ExpectedOrdersRuleViolated(format!("No rule found to cover at least 52 weeks for symbol {symbol}")));
    }
    if !found_hours_until_two_days {
        return Err(Error::ExpectedOrdersRuleViolated(format!("No rule found to cover somewhere between {min_hours} and {max_hours} hours for symbol {symbol}")));
    }
    Ok(())
}

fn validate_rule_set(symbol: &OrderRequestSymbol, rules: &HashSet<ExpectedOrderRequestsRule>) -> Result<()> {
    let found_global_rules = rules.iter().any(|rule| matches!(rule, ExpectedOrderRequestsRule::Global(_)));
    if !found_global_rules {
        return Err(Error::ExpectedOrdersRuleViolated(format!("No global expected order requests rules found for symbol {symbol}")));
    }

    validate_rule_set_durations(symbol, rules)?;
    Ok(())
}

pub fn validate_order_request(symbol: &OrderRequestSymbol, tracking_item_wrapper: &TopNEntry<OrderTrackingItem>) -> Result<Vec<ExpectedOrderRequestsRule>> {
    let rules = match EXPECTED_ORDER_REQUESTS_RULES.get(symbol) {
        Some(rules) => rules.clone(),
        None => return Err(Error::ExpectedOrdersRuleViolated(format!("No expected order requests rules found for symbol {symbol}")))
    };
    validate_rule_set(symbol, &rules)?;
    
    let matching_rules = rules.iter().filter(|rule| rule.matches_order(tracking_item_wrapper)).collect::<Vec<&ExpectedOrderRequestsRule>>();
    if matching_rules.is_empty() {
        return Err(Error::ExpectedOrdersRuleViolated(format!("No matching expected order requests rules found for order request {tracking_item_wrapper:?}")));
    }
    
    let mut validated_rules = vec![];
    for rule in matching_rules.into_iter() {
        if let Err(error) = rule.validate(tracking_item_wrapper) {
            let error_msg = error.get_msg();
            return Err(Error::ExpectedOrdersRuleViolated(format!("{symbol} order request violates rule. Error: {error_msg}. Order: {tracking_item_wrapper:?}, Rule: {rule:?}")));
        } else {
            validated_rules.push(rule.clone());
        }
    }
    Ok(validated_rules)
}

pub fn set_rules_for_symbol(symbol: OrderRequestSymbol, rules: HashSet<ExpectedOrderRequestsRule>) -> Result<()> {
    if rules.is_empty() {
        return Err(anyhow!("No rules provided for symbol {symbol}").into());
    }
    
    let mut existing_rules = EXPECTED_ORDER_REQUESTS_RULES.entry(symbol.clone()).or_default();
    if existing_rules.is_empty() {
        return Err(anyhow!("No rules found for symbol {symbol}. Should be hardcoded before calling this function").into());
    }
    
    let mut global_rules = vec![];
    for rule in existing_rules.iter() {
        if let ExpectedOrderRequestsRule::Global(_) = rule {
            global_rules.push(rule.clone());
        }
    }
    
    if global_rules.is_empty() {
        return Err(anyhow!("No global rules found for symbol {symbol}. Should be hardcoded and set into some static before calling this function").into());
    }
    
    let mut new_set: HashSet<ExpectedOrderRequestsRule> = HashSet::new();
    for rule in global_rules.iter() {
        new_set.insert(rule.clone());
    }
    
    for rule in rules.iter() {
        if let ExpectedOrderRequestsRule::Global(_) = rule {
            return Err(anyhow!("Global rules should not be submitted into this function, they should be hardcoded").into())
        }
        new_set.insert(rule.clone());
    };
    
    if new_set.is_empty() {
        return Err(anyhow!("Logic bug, new set of rules is empty for symbol {symbol}").into());
    }
    *existing_rules = new_set;
    Ok(())
}