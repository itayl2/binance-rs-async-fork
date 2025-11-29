use strum_macros::{Display, EnumString};

#[derive(Deserialize, Serialize, Display, EnumString, PartialEq, Eq, Debug, Clone, Hash)]
pub enum ExpectedOrderRequestsRuleType {
    Global,
    PerGrid,
}

impl Default for ExpectedOrderRequestsRuleType {
    fn default() -> Self {
        Self::PerGrid
    }
}