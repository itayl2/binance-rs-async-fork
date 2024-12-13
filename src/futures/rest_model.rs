use rust_decimal::Decimal;
use crate::rest_model::string_or_bool;
pub use crate::rest_model::{string_or_u64, Asks, Bids, BookTickers, KlineSummaries, KlineSummary,
                            OrderSide, OrderStatus, RateLimit, ServerTime, SymbolPrice, SymbolStatus, Tickers,
                            TimeInForce};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInformation {
    pub timezone: String,
    pub server_time: u64,
    pub futures_type: String,
    pub rate_limits: Vec<RateLimit>,
    pub exchange_filters: Vec<Filters>,
    pub assets: Vec<AssetDetail>,
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetDetail {
    pub asset: String,
    pub margin_available: bool,
    pub auto_asset_exchange: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
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
    pub settle_plan: u64,
    pub trigger_protect: Decimal,
    pub filters: Vec<Filters>,
    pub order_types: Vec<OrderType>,
    pub time_in_force: Vec<TimeInForce>,
    pub min_order_size: Decimal,
    pub step_size: Decimal,
    pub tick_size: Decimal,
}

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
            pub settle_plan: u64,
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
        };
        symbol_object.min_order_size = symbol_object.get_min_order_size();
        symbol_object.tick_size = symbol_object.get_tick_size();
        symbol_object.step_size = symbol_object.get_step_size();
        Ok(symbol_object)
    }
}

impl Symbol {
    pub fn get_min_order_size(&self) -> Decimal {
        for filter in self.filters.iter() {
            match filter {
                Filters::LotSize { min_qty, .. } => return *min_qty,
                _ => {},
            }
        }
        panic!("No lot size filter found for min_order_size")
    }

    pub fn get_tick_size(&self) -> Decimal {
        for filter in self.filters.iter() {
            match filter {
                Filters::PriceFilter { tick_size, .. } => return *tick_size,
                _ => {},
            }
        }
        panic!("No price filter found for tick_size")
    }

    pub fn get_step_size(&self) -> Decimal {
        for filter in self.filters.iter() {
            match filter {
                Filters::LotSize { step_size, .. } => return *step_size,
                _ => {},
            }
        }
        panic!("No price filter found for step_size")
    }

    pub fn round_order_size(&self, order_size: Decimal) -> Decimal {
        let quotient = order_size / self.step_size;
        let floored_quotient = quotient.floor();
        floored_quotient * self.step_size
    }

    pub fn get_order_price(&self, price: Decimal) -> Decimal {
        let quotient = price / self.tick_size;
        let floored_quotient = quotient.floor();
        floored_quotient * self.tick_size
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContractType {
    Perpetual,
    CurrentMonth,
    NextMonth,
    CurrentQuarter,
    NextQuarter,
    #[serde(rename = "CURRENT_QUARTER DELIVERING")]
    CurrentQuarterDelivery,
    PerpetualDelivering,
    #[serde(rename = "")]
    Empty,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    Market,
    Stop,
    StopMarket,
    TakeProfit,
    TakeProfitMarket,
    TrailingStopMarket,
}

/// By default, use market orders
impl Default for OrderType {
    fn default() -> Self { Self::Market }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionSide {
    Both,
    Long,
    Short,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkingType {
    MarkPrice,
    ContractPrice,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MarginType {
    Isolated,
    Cross,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "filterType")]
pub enum Filters {
    #[serde(rename = "PRICE_FILTER")]
    #[serde(rename_all = "camelCase")]
    PriceFilter {
        min_price: Decimal,
        max_price: Decimal,
        tick_size: Decimal,
    },
    #[serde(rename = "LOT_SIZE")]
    #[serde(rename_all = "camelCase")]
    LotSize {
        min_qty: Decimal,
        max_qty: Decimal,
        step_size: Decimal,
    },
    #[serde(rename = "MARKET_LOT_SIZE")]
    #[serde(rename_all = "camelCase")]
    MarketLotSize {
        min_qty: String,
        max_qty: String,
        step_size: String,
    },
    #[serde(rename = "MAX_NUM_ORDERS")]
    #[serde(rename_all = "camelCase")]
    MaxNumOrders { limit: u16 },
    #[serde(rename = "MAX_NUM_ALGO_ORDERS")]
    #[serde(rename_all = "camelCase")]
    MaxNumAlgoOrders { limit: u16 },
    #[serde(rename = "MIN_NOTIONAL")]
    #[serde(rename_all = "camelCase")]
    MinNotional {
        notional: Decimal,
    },
    #[serde(rename = "PERCENT_PRICE")]
    #[serde(rename_all = "camelCase")]
    PercentPrice {
        multiplier_up: Decimal,
        multiplier_down: Decimal,
        multiplier_decimal: Decimal,
    },
    #[serde(other)]
    Others,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    pub last_update_id: u64,
    // Undocumented
    #[serde(rename = "E")]
    pub event_time: u64,
    // Undocumented
    #[serde(rename = "T")]
    pub trade_order_time: u64,
    pub bids: Vec<Bids>,
    pub asks: Vec<Asks>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceStats {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub weighted_avg_price: String,
    pub last_price: Decimal,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub last_qty: Decimal,
    pub open_time: u64,
    pub close_time: u64,
    pub first_id: u64,
    pub last_id: u64,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Trades {
    AllTrades(Vec<Trade>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub id: u64,
    pub is_buyer_maker: bool,
    pub price: Decimal,
    pub qty: Decimal,
    pub quote_qty: Decimal,
    pub time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum AggTrades {
    AllAggTrades(Vec<AggTrade>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AggTrade {
    #[serde(rename = "T")]
    pub time: u64,
    #[serde(rename = "a")]
    pub agg_id: u64,
    #[serde(rename = "f")]
    pub first_id: u64,
    #[serde(rename = "l")]
    pub last_id: u64,
    #[serde(rename = "m")]
    pub maker: bool,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "q")]
    pub qty: Decimal,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// #[serde(untagged)]
// pub enum MarkPrices {
//     AllMarkPrices(Vec<MarkPrice>),
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarkPrice {
    pub symbol: String,
    pub mark_price: Decimal,
    pub index_price: Decimal,
    pub estimated_settle_price: Decimal,
    pub last_funding_rate: Decimal,
    pub next_funding_time: u64,
    pub interest_rate: Decimal,
    pub time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum LiquidationOrders {
    AllLiquidationOrders(Vec<LiquidationOrder>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LiquidationOrder {
    pub average_price: Decimal,
    pub executed_qty: Decimal,
    pub orig_qty: Decimal,
    pub price: Decimal,
    pub side: String,
    pub status: String,
    pub symbol: String,
    pub time: u64,
    pub time_in_force: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenInterest {
    pub open_interest: Decimal,
    pub symbol: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub client_order_id: String,
    pub cum_quote: Decimal,
    pub executed_qty: Decimal,
    pub order_id: u64,
    pub avg_price: Decimal,
    pub orig_qty: Decimal,
    pub price: Decimal,
    pub side: OrderSide,
    pub reduce_only: bool,
    pub position_side: PositionSide,
    pub status: OrderStatus,
    #[serde(default = "default_stop_price")]
    pub stop_price: Decimal,
    pub close_position: bool,
    pub symbol: String,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub orig_type: OrderType,
    #[serde(default = "default_activation_price")]
    pub activate_price: Decimal,
    #[serde(default = "default_price_rate")]
    pub price_rate: Decimal,
    pub update_time: u64,
    pub working_type: WorkingType,
    pub price_protect: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub client_order_id: String,
    pub cum_qty: Decimal,
    pub cum_quote: Decimal,
    pub executed_qty: Decimal,
    pub order_id: u64,
    pub avg_price: Decimal,
    pub orig_qty: Decimal,
    pub reduce_only: bool,
    pub side: OrderSide,
    pub position_side: PositionSide,
    pub status: OrderStatus,
    pub stop_price: Decimal,
    pub close_position: bool,
    pub symbol: String,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub type_name: OrderType,
    pub orig_type: OrderType,
    #[serde(default)]
    pub activate_price: Option<Decimal>,
    #[serde(default)]
    pub price_rate: Option<Decimal>,
    pub update_time: u64,
    pub working_type: WorkingType,
    price_protect: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CanceledOrder {
    pub client_order_id: String,
    pub cum_qty: Decimal,
    pub cum_quote: Decimal,
    pub executed_qty: Decimal,
    pub order_id: u64,
    pub orig_qty: Decimal,
    pub orig_type: String,
    pub price: Decimal,
    pub reduce_only: bool,
    pub side: String,
    pub position_side: String,
    pub status: String,
    pub stop_price: Decimal,
    pub close_position: bool,
    pub symbol: String,
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub activate_price: Option<Decimal>,
    #[serde(default)]
    pub price_rate: Option<Decimal>,
    pub update_time: u64,
    pub working_type: String,
    price_protect: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub entry_price: Decimal,
    pub margin_type: MarginType,
    #[serde(with = "string_or_bool")]
    pub is_auto_add_margin: bool,
    pub isolated_margin: Decimal,
    #[serde(with = "string_or_u64")]
    pub leverage: u64,
    pub liquidation_price: Decimal,
    pub mark_price: Decimal,
    pub max_notional_value: Decimal,
    #[serde(rename = "positionAmt")]
    pub position_amount: Decimal,
    pub symbol: String,
    #[serde(rename = "unRealizedProfit")]
    pub unrealized_profit: Decimal,
    pub position_side: PositionSide,
    pub update_time: u64,
    pub notional: Decimal,
    pub isolated_wallet: Decimal,
}

// https://binance-docs.github.io/apidocs/futures/en/#account-information-v2-user_data
// it has differences from Position returned by positionRisk endpoint
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountPosition {
    pub symbol: String,
    pub initial_margin: Decimal,
    #[serde(rename = "maintMargin")]
    pub maintenance_margin: Decimal,
    pub unrealized_profit: Decimal,
    pub position_initial_margin: Decimal,
    pub open_order_initial_margin: Decimal,
    #[serde(with = "string_or_u64")]
    pub leverage: u64,
    pub isolated: bool,
    pub entry_price: Decimal,
    pub max_notional: Decimal,
    pub bid_notional: Decimal,
    pub ask_notional: Decimal,
    pub position_side: PositionSide,
    #[serde(rename = "positionAmt")]
    pub position_amount: Decimal,
    pub update_time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountAsset {
    pub asset: String,
    pub wallet_balance: Decimal,
    pub unrealized_profit: Decimal,
    pub margin_balance: Decimal,
    pub maint_margin: Decimal,
    pub initial_margin: Decimal,
    pub position_initial_margin: Decimal,
    pub open_order_initial_margin: Decimal,
    pub cross_wallet_balance: Decimal,
    #[serde(rename = "crossUnPnl")]
    pub cross_unrealized_pnl: Decimal,
    pub available_balance: Decimal,
    pub max_withdraw_amount: Decimal,
    pub margin_available: bool,
    pub update_time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountInformation {
    pub fee_tier: u64,
    pub can_trade: bool,
    pub can_deposit: bool,
    pub can_withdraw: bool,
    pub update_time: u64,
    pub multi_assets_margin: bool,
    pub total_initial_margin: Decimal,
    #[serde(rename = "totalMaintMargin")]
    pub total_maintenance_margin: Decimal,
    pub total_wallet_balance: Decimal,
    pub total_unrealized_profit: Decimal,
    pub total_margin_balance: Decimal,
    pub total_position_initial_margin: Decimal,
    pub total_open_order_initial_margin: Decimal,
    pub total_cross_wallet_balance: Decimal,
    #[serde(rename = "totalCrossUnPnl")]
    pub total_cross_unrealized_pnl: Decimal,
    pub available_balance: Decimal,
    pub max_withdraw_amount: Decimal,
    pub assets: Vec<AccountAsset>,
    pub positions: Vec<AccountPosition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalance {
    pub account_alias: String,
    pub asset: String,
    pub balance: Decimal,
    pub cross_wallet_balance: Decimal,
    #[serde(rename = "crossUnPnl")]
    pub cross_unrealized_pnl: Decimal,
    pub available_balance: Decimal,
    pub max_withdraw_amount: Decimal,
    pub margin_available: bool,
    pub update_time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeLeverageResponse {
    pub leverage: u8,
    pub max_notional_value: Decimal,
    pub symbol: String,
}

fn default_stop_price() -> Decimal {
    Decimal::ZERO
}
fn default_activation_price() -> Decimal {
    Decimal::ZERO
}
fn default_price_rate() -> Decimal {
    Decimal::ZERO
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HistoryQuery {
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub from_id: Option<u64>,
    pub limit: u16,
    pub symbol: String,
    pub interval: Option<String>,
    pub period: Option<String>,
}

impl HistoryQuery {
    pub fn validate(&self) -> crate::errors::Result<()> {
        if let Some(period) = &self.period {
            if !PERIODS.contains(&period.as_str()) {
                return Err(crate::errors::Error::InvalidPeriod(period.clone()));
            }
        }
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IndexQuery {
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: u16,
    pub pair: String,
    pub interval: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FundingRate {
    pub symbol: String,
    pub funding_time: u64,
    pub funding_rate: Decimal,
}

pub static PERIODS: &[&str] = &["5m", "15m", "30m", "1h", "2h", "4h", "6h", "12h", "1d"];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenInterestHistory {
    pub symbol: String,
    pub sum_open_interest: Decimal,
    pub sum_open_interest_value: Decimal,
    pub timestamp: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LongShortRatio {
    pub symbol: String,
    pub long_account: Decimal,
    pub long_short_ratio: Decimal,
    pub short_account: Decimal,
    pub timestamp: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeverageBracket {
    pub bracket: u8,
    pub initial_leverage: u8,
    pub notional_cap: u64,
    pub notional_floor: u64,
    pub maint_margin_ratio: Decimal,
    pub cum: Decimal,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolBrackets {
    pub symbol: String,
    pub notional_coef: Option<Decimal>,
    pub brackets: Vec<LeverageBracket>,
}
