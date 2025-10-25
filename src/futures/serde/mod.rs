use serde::{Deserialize, Deserializer};
use rust_decimal::Decimal;
use serde_json::Value;
use crate::futures::rest_model::{ContractType, Filters, OrderType, Symbol};
use crate::rest_model::{SymbolStatus, TimeInForce};

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Define a temporary struct that matches the JSON structure
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct TempObject {
            pub symbol: String,
            pub pair: String,
            pub contract_type: ContractType,
            pub delivery_date: u64,
            pub onboard_date: u64,
            pub status: SymbolStatus,
            pub maint_margin_percent: Decimal,
            pub required_margin_percent: Decimal,
            pub base_asset: String,
            pub quote_asset: String,
            pub price_precision: u16,
            pub quantity_precision: u16,
            pub base_asset_precision: u64,
            pub quote_precision: u64,
            pub underlying_type: String,
            pub underlying_sub_type: Vec<String>,
            pub settle_plan: Option<u64>,
            pub trigger_protect: Decimal,
            pub filters: Vec<Filters>,
            pub order_types: Vec<OrderType>,
            pub time_in_force: Vec<TimeInForce>,
        }

        // Deserialize into the temporary struct
        let temp = TempObject::deserialize(deserializer)?;

        let mut symbol_object = Symbol {
            symbol: temp.symbol,
            pair: temp.pair,
            contract_type: temp.contract_type,
            delivery_date: temp.delivery_date,
            onboard_date: temp.onboard_date,
            status: temp.status,
            maint_margin_percent: temp.maint_margin_percent,
            required_margin_percent: temp.required_margin_percent,
            base_asset: temp.base_asset,
            quote_asset: temp.quote_asset,
            price_precision: temp.price_precision,
            quantity_precision: temp.quantity_precision,
            base_asset_precision: temp.base_asset_precision,
            quote_precision: temp.quote_precision,
            underlying_type: temp.underlying_type,
            underlying_sub_type: temp.underlying_sub_type,
            settle_plan: temp.settle_plan,
            trigger_protect: temp.trigger_protect,
            filters: temp.filters,
            order_types: temp.order_types,
            time_in_force: temp.time_in_force,
            min_order_size: Decimal::ZERO,
            step_size: Decimal::ZERO,
            tick_size: Decimal::ZERO,
            step_scale: 0,
            tick_scale: 0,
        };
        symbol_object.min_order_size = symbol_object.get_min_order_size();
        symbol_object.tick_size = symbol_object.get_tick_size();
        symbol_object.step_size = symbol_object.get_step_size();
        symbol_object.step_scale = symbol_object.step_size.scale();
        symbol_object.tick_scale = symbol_object.tick_size.scale();
        Ok(symbol_object)
    }
}

pub fn null_to_empty_string<'de, D>(de: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::Deserialize;
    Option::<String>::deserialize(de).map(|opt| opt.unwrap_or_default())
}

/// Deserializes a field that can be a string, any number, or null.
/// - Strings are kept as-is.
/// - Numbers (int, float, etc.) are converted to a string.
/// - `null` becomes an empty string "".
///
/// Use with `#[serde(deserialize_with = "flexible_to_string", default)]`
pub fn flexible_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize into a generic JSON Value
    let v = Value::deserialize(deserializer)?;

    // Match on the type of Value
    match v {
        // "order_id": "12345"
        Value::String(s) => Ok(s),

        // "order_id": 12345 (or 123.45, or -10, etc.)
        // This is the key: Value::Number handles *all* numeric types.
        Value::Number(n) => Ok(n.to_string()),

        // "order_id": null
        Value::Null => Ok("".to_string()),

        // "order_id": true, "order_id": [...] "order_id": {...}
        _ => Err(serde::de::Error::custom(format!("expected string, number, or null, found {v:?}"))),
    }
}