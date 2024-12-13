use rust_decimal::Decimal;
use crate::futures::rest_model::{MarginType, OrderType, PositionSide, WorkingType};
use crate::rest_model::{ExecutionType, OrderSide, OrderStatus, TimeInForce};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "e")]
pub enum WebsocketEvent {
    AccountUpdate(Box<AccountUpdate>),
    OrderTradeUpdate(Box<OrderTradeUpdate>),
}

#[derive(Debug, Deserialize)]
pub struct AccountUpdate {
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "T")]
    pub transaction_time: u64,
    #[serde(rename = "a")]
    pub account: Account,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    #[serde(rename = "m")]
    pub reason_type: ReasonType,
    #[serde(rename = "B")]
    pub balances: Vec<Balance>,
    #[serde(rename = "P")]
    pub positions: Vec<Position>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReasonType {
    Deposit,
    Withdraw,
    Order,
    FundingFee,
    WithdrawReject,
    Adjustment,
    InsuranceClear,
    AdminDeposit,
    AdminWithdraw,
    MarginTransfer,
    MarginTypeChange,
    AssetTransfer,
    OptionsPremiumFee,
    OptionsSettleProfit,
    AutoExchange,
    CoinSwapDeposit,
    CoinSwapWithdraw,
}

#[derive(Debug, Deserialize)]
pub struct Balance {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "wb")]
    pub wallet_balance: Decimal,
    #[serde(rename = "cw")]
    pub cross_wallet_balance: Decimal,
    #[serde(rename = "bc")]
    pub balance_change: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct Position {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "pa")]
    pub position_amount: Decimal,
    #[serde(rename = "ep")]
    pub entry_price: Decimal,
    #[serde(rename = "bep")]
    pub breakeven_price: Decimal,
    #[serde(rename = "cr")]
    pub accumulated_realized: Decimal,
    #[serde(rename = "up")]
    pub unrealized_profit: Decimal,
    #[serde(rename = "mt")]
    pub margin_type: MarginType,
    #[serde(rename = "iw")]
    pub isolated_wallet: Decimal,
    #[serde(rename = "ps")]
    pub position_side: PositionSide,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OrderTradeUpdate {
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "T")]
    pub transaction_time: u64,
    #[serde(rename = "o")]
    pub order: Order,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Order {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "c")]
    pub client_order_id: String,
    #[serde(rename = "S")]
    pub side: OrderSide,
    #[serde(rename = "o")]
    pub order_type: OrderType,
    #[serde(rename = "f")]
    pub time_in_force: TimeInForce,
    #[serde(rename = "q")]
    pub quantity: Decimal,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "ap")]
    pub average_price: Decimal,
    #[serde(rename = "sp")]
    pub stop_price: Decimal,
    #[serde(rename = "x")]
    pub execution_type: ExecutionType,
    #[serde(rename = "X")]
    pub order_status: OrderStatus,
    #[serde(rename = "i")]
    pub order_id: u64,
    #[serde(rename = "l")]
    pub order_last_filled_quantity: Decimal,
    #[serde(rename = "z")]
    pub order_filled_accumulated_quantity: Decimal,
    #[serde(rename = "L")]
    pub last_filled_price: Decimal,
    #[serde(default, rename = "n")]
    pub commission: Option<Decimal>,
    #[serde(rename = "N")]
    pub commission_asset: Option<String>,
    #[serde(rename = "T")]
    pub order_trade_time: u64,
    #[serde(rename = "t")]
    pub trade_id: u64,
    #[serde(rename = "b")]
    pub bid_notional: Decimal,
    #[serde(rename = "a")]
    pub ask_notional: Decimal,
    #[serde(rename = "m")]
    pub is_maker: bool,
    #[serde(rename = "R")]
    pub is_reduce: bool,
    #[serde(rename = "wt")]
    pub working_type: WorkingType,
    #[serde(rename = "ot")]
    pub original_order_type: OrderType,
    #[serde(rename = "ps")]
    pub position_side: PositionSide,
    #[serde(rename = "cp")]
    pub close_position: bool,
    #[serde(default, rename = "AP")]
    pub activation_price: Option<Decimal>,
    #[serde(default, rename = "cr")]
    pub callback_rate: Option<Decimal>,
    #[serde(rename = "pP")]
    pub price_protect: bool,
    #[serde(rename = "rp")]
    pub realized_profit: Decimal,
    #[serde(rename = "V")]
    pub stp_mode: SelfTradePreventionMode,
    #[serde(rename = "pm")]
    pub price_match: PriceMatch,
    #[serde(rename = "gtd")]
    pub good_till_date: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PriceMatch {
    /// No price match
    None,
    /// Counterparty best price
    Opponent,
    /// The 5th best price from the counterparty
    Opponent5,
    /// The 10th best price from the counterparty
    Opponent10,
    /// The 20th best price from the counterparty
    Opponent20,
    /// The best price on the same side of the order book
    Queue,
    /// The 5th best price on the same side of the order book
    Queue5,
    /// The 10th best price on the same side of the order book
    Queue10,
    /// The 20th best price on the same side of the order book
    Queue20,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SelfTradePreventionMode {
    /// No Self-Trade Prevention
    None,
    /// Expire taker order when STP trigger
    ExpireTaker,
    /// Expire taker and maker order when STP trigger
    ExpireBoth,
    /// Expire maker order when STP trigger
    ExpireMaker,
}
