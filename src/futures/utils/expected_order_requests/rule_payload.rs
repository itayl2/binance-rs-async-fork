use crate::errors;
use crate::errors::Error;
use crate::futures::utils::expected_order_requests::rule_period::RulePeriod;
use crate::futures::utils::expected_order_requests::rule_size_params::ExpectedOrderRequestsRuleSizeParams;
use crate::futures::utils::order_tracker::get_gte_timestamp;
use crate::futures::utils::order_tracking_item::OrderTrackingItem;
use crate::futures::utils::top_n::TopNEntry;
use crate::rest_model::OrderSide;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
pub struct ExpectedOrderRequestsRulePayload {
    pub symbol: String,
    pub size_params: ExpectedOrderRequestsRuleSizeParams,
    pub max_count: u64,
    pub period: RulePeriod,
    pub side: OrderSide,
}

impl ExpectedOrderRequestsRulePayload {
    pub fn validate(&self, order_request: &TopNEntry<OrderTrackingItem>) -> errors::Result<()> {
        if order_request.item.size.is_zero() {
            return Err(Error::ExpectedOrdersRuleViolated("Order size is zero".to_string()));
        }
        
        if self.max_count == 0 {
            return Err(Error::ExpectedOrdersRuleViolated("Max count is zero".to_string()));
        }
        
        let min_timestamp = match self.period.get_min_nanos_timestamp(order_request.timestamp) {
            Ok(timestamp) => timestamp,
            Err(error) => return Err(Error::ExpectedOrdersRuleViolated(format!("Failed to get min timestamp for order request: {}", error.get_msg()))),
        };
        let orders_in_period = match get_gte_timestamp(&self.symbol, min_timestamp) {
            Some(orders) => orders,
            None => return Err(Error::ExpectedOrdersRuleViolated(format!("Order tracker is not tracking symbol {}", self.symbol))),
        };
        let mut counter = 0;
        let mut found_submitted_order_in_tracker = false;
        for tracking_wrapper in orders_in_period.iter() {
            if self.matches_order(tracking_wrapper) {
                continue;
            }
            
            if tracking_wrapper.item.id == *order_request.item.id {
                found_submitted_order_in_tracker = true;
            }
            
            counter += 1;
        }
        
        if !found_submitted_order_in_tracker {
            return Err(Error::ExpectedOrdersRuleViolated("Expected the submitted order to be in the tracker but it was not found by id".to_string()));
        }
        if counter > self.max_count {
            return Err(Error::ExpectedOrdersRuleViolated("The submitted order violates the max count rule".to_string()));
        }
        
        Ok(())
    }
    
    pub fn matches_order(&self, order: &TopNEntry<OrderTrackingItem>) -> bool {
        if order.item.side != self.side {
            return false;
        }
        
        match self.size_params {
            ExpectedOrderRequestsRuleSizeParams::Min(min_size) => order.item.size >= min_size,
            ExpectedOrderRequestsRuleSizeParams::Max(max_size) => order.item.size <= max_size,
            ExpectedOrderRequestsRuleSizeParams::MinMax { min, max } => order.item.size >= min && order.item.size <= max,
        }
    }
}