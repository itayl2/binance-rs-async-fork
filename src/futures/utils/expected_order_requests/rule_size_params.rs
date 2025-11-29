use strum_macros::{Display, EnumString};
use rust_decimal::Decimal;

#[derive(Deserialize, Serialize, Display, EnumString, PartialEq, Eq, Debug, Clone, Hash)]
pub enum ExpectedOrderRequestsRuleSizeParams {
    Min(Decimal),
    Max(Decimal),
    MinMax {
        min: Decimal,
        max: Decimal,
    },
}

impl Default for ExpectedOrderRequestsRuleSizeParams {
    fn default() -> Self {
        Self::MinMax {
            min: Decimal::ZERO,
            max: Decimal::MAX,
        }
    }
}