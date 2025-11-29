use rust_decimal::Decimal;
use crate::rest_model::OrderSide;

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct OrderTrackingItem {
    #[serde(rename = "s")]
    pub size: Decimal,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "sd")]
    pub side: OrderSide,
    #[serde(rename = "i")]
    pub id: String,
}

impl PartialEq for OrderTrackingItem {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for OrderTrackingItem {}

impl Ord for OrderTrackingItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
            .then_with(|| self.price.cmp(&other.price))
            .then_with(|| self.size.cmp(&other.size))
            .then_with(|| self.side.cmp(&other.side))
    }
}

impl PartialOrd for OrderTrackingItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

