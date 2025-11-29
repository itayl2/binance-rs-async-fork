use strum_macros::{Display, EnumString};
use crate::errors;
use crate::futures::utils::expected_order_requests::rule_payload::ExpectedOrderRequestsRulePayload;
use crate::futures::utils::order_tracking_item::OrderTrackingItem;
use std::time::Duration;
use crate::futures::utils::top_n::TopNEntry;

#[derive(Deserialize, Serialize, Display, EnumString, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ExpectedOrderRequestsRule {
    Global(ExpectedOrderRequestsRulePayload),
    PerGrid(ExpectedOrderRequestsRulePayload),
}

impl ExpectedOrderRequestsRule {
    pub fn validate(&self, order_request: &TopNEntry<OrderTrackingItem>) -> errors::Result<()> {
        match self {
            ExpectedOrderRequestsRule::Global(global_rule) => global_rule.validate(order_request),
            ExpectedOrderRequestsRule::PerGrid(per_grid_rule) => per_grid_rule.validate(order_request),
        }
    }
    
    pub fn matches_order(&self, order: &TopNEntry<OrderTrackingItem>) -> bool {
        match self {
            ExpectedOrderRequestsRule::Global(global_rule) => global_rule.matches_order(order),
            ExpectedOrderRequestsRule::PerGrid(per_grid_rule) => per_grid_rule.matches_order(order),
        }
    }
    
    pub fn get_duration(&self) -> errors::Result<Duration> {
        match self {
            ExpectedOrderRequestsRule::Global(global_rule) => global_rule.period.get_validated_duration(),
            ExpectedOrderRequestsRule::PerGrid(per_grid_rule) => per_grid_rule.period.get_validated_duration(),
        }
    }
}