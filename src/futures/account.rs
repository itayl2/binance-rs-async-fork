use std::collections::{BTreeMap, HashMap};
use super::rest_model::{AccountBalance, AccountInformation, AccountInformationV3, AccountTrade, CanceledOrderResponse, Order, Position, PositionSide, PositionV3, SupportedOrderType, Symbol, Transaction, WorkingType};
use crate::account::{OrderCancellation, OrderCancellationWithU64};
use crate::client::Client;
use crate::errors::*;
use crate::rest_model::{OrderSide, TimeInForce};
use crate::rest_model::{PairAndWindowQuery, PairQuery};
use crate::util::*;
use serde::Serializer;
use std::fmt;
use anyhow::anyhow;
use rust_decimal::Decimal;
use crate::futures::utils::expected_order_requests::rules_map::validate_order_request;
use crate::futures::utils::order_tracker::add_order_tracking_item;

#[derive(Clone, Debug)]
pub struct FuturesAccount {
    pub client: Client,
    pub recv_window: u64,
}

/// Serialize bool as str
fn serialize_as_str<S, T>(t: &T, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
    T: fmt::Display,
{
    serializer.collect_str(t)
}

/// Serialize opt bool as str
fn serialize_opt_as_uppercase<S, T>(t: &Option<T>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToString,
{
    match *t {
        Some(ref v) => serializer.serialize_some(&v.to_string().to_uppercase()),
        None => serializer.serialize_none(),
    }
}

#[derive(Serialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetOrderRequest {
    pub symbol: String,
    #[serde(rename = "orderId")]
    pub order_id: Option<String>,
    #[serde(rename = "origClientOrderId")]
    pub orig_client_order_id: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub position_side: Option<PositionSide>,
    #[serde(rename = "type")]
    pub order_type: SupportedOrderType,
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "quantity")]
    pub quantity: Option<Decimal>,
    pub reduce_only: Option<bool>,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub close_position: Option<bool>,
    pub activation_price: Option<Decimal>,
    pub callback_rate: Option<Decimal>,
    pub working_type: Option<WorkingType>,
    #[serde(serialize_with = "serialize_opt_as_uppercase")]
    pub price_protect: Option<bool>,
    pub new_client_order_id: Option<String>,
}

impl From<OrderRequestMandatoryClientId> for OrderRequest {
    fn from(order: OrderRequestMandatoryClientId) -> Self {
        Self {
            symbol: order.symbol,
            side: order.side,
            position_side: order.position_side,
            order_type: order.order_type,
            time_in_force: order.time_in_force,
            quantity: match order.quantity {
                Some(quantity) => Some(quantity.normalize()),
                None => None,
            },
            reduce_only: order.reduce_only,
            price: match order.price {
                Some(price) => Some(price.normalize()),
                None => None,
            },
            stop_price: match order.stop_price {
                Some(stop_price) => Some(stop_price.normalize()),
                None => None,
            },
            close_position: order.close_position,
            activation_price: order.activation_price,
            callback_rate: order.callback_rate,
            working_type: order.working_type,
            price_protect: order.price_protect,
            new_client_order_id: Some(order.new_client_order_id),
        }
    }
}

#[derive(Serialize, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequestMandatoryClientId {
    pub symbol: String,
    pub side: OrderSide,
    pub position_side: Option<PositionSide>,
    #[serde(rename = "type")]
    pub order_type: SupportedOrderType,
    pub time_in_force: Option<TimeInForce>,
    #[serde(rename = "quantity")]
    pub quantity: Option<Decimal>,
    pub reduce_only: Option<bool>,
    pub price: Option<Decimal>,
    #[serde(skip_serializing, default)]
    pub intended_price: Decimal,
    pub stop_price: Option<Decimal>,
    pub close_position: Option<bool>,
    pub activation_price: Option<Decimal>,
    pub callback_rate: Option<Decimal>,
    pub working_type: Option<WorkingType>,
    #[serde(serialize_with = "serialize_opt_as_uppercase")]
    pub price_protect: Option<bool>,
    pub new_client_order_id: String,
}

impl Default for OrderRequestMandatoryClientId {
    fn default() -> Self {
        Self {
            symbol: String::default(),
            side: OrderSide::Buy,
            position_side: None,
            order_type: SupportedOrderType::Limit,
            time_in_force: None,
            quantity: None,
            reduce_only: None,
            price: None,
            intended_price: Decimal::ZERO,
            stop_price: None,
            close_position: None,
            activation_price: None,
            callback_rate: None,
            working_type: None,
            price_protect: None,
            new_client_order_id: String::default(),
        }
    }
}

impl OrderRequestMandatoryClientId {
    pub fn has_stop_price(&self) -> bool {
        self.stop_price.is_some()
    }

    pub fn set_stop_price(&mut self, raw_stop_price: Decimal, symbol: &Symbol) {
        self.stop_price = Some(symbol.get_order_price(raw_stop_price));
    }

    pub fn set_size(&mut self, size: Decimal) {
        self.quantity = Some(size);
    }

    pub fn get_size(&self) -> Decimal {
        self.quantity.unwrap_or(Decimal::ZERO)
    }

    pub fn get_price(&self) -> Option<Decimal> {
        self.price
    }

    pub fn get_market(&self) -> String {
        self.symbol.clone()
    }

    pub fn get_client_id(&self) -> String {
        self.new_client_order_id.clone()
    }

    pub fn set_client_id(&mut self, client_id: String) {
        self.new_client_order_id = client_id;
    }
    
    pub fn get_intended_price(&self) -> Decimal {
        self.intended_price
    }
}

// #[derive(Serialize)]
// #[serde(rename_all = "camelCase")]
// struct ChangePositionModeRequest {
//     #[serde(serialize_with = "serialize_as_str")]
//     pub dual_side_position: bool,
// }

impl FuturesAccount {
    /// Get an order
    pub async fn get_order(&self, order: Option<GetOrderRequest>) -> Result<Order> {
        self.client
            .get_signed_p("/fapi/v1/order", order, self.recv_window)
            .await
    }

    /// Place an order
    #[cfg(not(feature = "backtest"))]
    pub async fn place_order(&self, order: OrderRequest) -> Result<Transaction> {
        let top_n_entry = add_order_tracking_item(&order)?;
        let validated_rules = validate_order_request(&order.symbol, &top_n_entry)?;
        if validated_rules.is_empty() {
            return Err(anyhow!("Expected some validated rule but got none").into());
        }
        match self.client
            .post_signed_p::<Transaction, OrderRequest>("/fapi/v1/order", order, self.recv_window)
            .await {
            Ok(mut transaction) => {
                transaction.validated_rules = validated_rules;
                Ok(transaction)
            },
            Err(error) => Err(error)
        }
    }

    #[cfg(not(feature = "backtest"))]
    pub async fn place_order_with_key(&self, order: OrderRequest, private_key: &str) -> Result<Transaction> {
        let top_n_entry = add_order_tracking_item(&order)?;
        let validated_rules = validate_order_request(&order.symbol, &top_n_entry)?;
        if validated_rules.is_empty() {
            return Err(anyhow!("Expected some validated rule but got none").into());
        }
        match self.client
            .post_signed_p_with_key::<Transaction, OrderRequest>("/fapi/v1/order", order, self.recv_window, private_key)
            .await {
            Ok(mut transaction) => {
                transaction.validated_rules = validated_rules;
                Ok(transaction)
            },
            Err(error) => Err(error)
        }
    }


    /// Place an order
    #[cfg(feature = "backtest")]
    pub async fn place_order(&self, order: serde_json::Value) -> Result<Order> {
        self.client
            .post_signed_p("/fapi/v1/order", order, self.recv_window)
            .await
    }

    /// Get currently open orders
    pub async fn get_open_orders(&self, symbol: Option<impl Into<String>>) -> Result<Vec<Order>> {
        let dummy_hashmap: HashMap<String, String> = HashMap::new();
        let payload = match symbol {
            Some(symbol) => build_signed_request_p(PairQuery { symbol: symbol.into() }, self.recv_window)?,
            None => build_signed_request_p(dummy_hashmap, self.recv_window)?,
        };
        self.client.get_signed("/fapi/v1/openOrders", &payload).await
    }

    /// Get all orders
    pub async fn get_all_orders(&self, symbol: impl Into<String>) -> Result<Vec<Order>> {
        let payload = build_signed_request_p(PairQuery { symbol: symbol.into() }, self.recv_window)?;
        self.client.get_signed("/fapi/v1/allOrders", &payload).await
    }

    /// Place a test order    
    pub async fn place_order_test(&self, order: OrderRequest) -> Result<Transaction> {
        self.client
            .post_signed_p("/fapi/v1/order/test", order, self.recv_window)
            .await
    }

    /// Get account trades
    pub async fn get_account_trades(&self, symbol: impl Into<String>, order_id: Option<String>) -> Result<Vec<AccountTrade>> {
        let mut data: HashMap<String, String> = HashMap::from([("symbol".to_string(), symbol.into())]);
        if let Some(order_id) = order_id {
            data.insert("orderId".to_string(), order_id.to_string());
        }
        let payload = build_signed_request_p(data, self.recv_window)?;
        self.client.get_signed("/fapi/v1/userTrades", &payload).await
    }

    /// Place a limit buy order
    #[cfg(not(feature = "backtest"))]
    pub async fn limit_buy(
        &self,
        symbol: impl Into<String>,
        qty: impl Into<Decimal>,
        price: Decimal,
        time_in_force: TimeInForce,
    ) -> Result<Transaction> {
        let order = OrderRequest {
            symbol: symbol.into(),
            side: OrderSide::Buy,
            position_side: None,
            order_type: SupportedOrderType::Limit,
            time_in_force: Some(time_in_force),
            quantity: Some(qty.into()),
            reduce_only: None,
            price: Some(price),
            stop_price: None,
            close_position: None,
            activation_price: None,
            callback_rate: None,
            working_type: None,
            price_protect: None,
            new_client_order_id: None,
        };
        self.place_order(order).await
    }

    /// Place a limit sell order
    #[cfg(not(feature = "backtest"))]
    pub async fn limit_sell(
        &self,
        symbol: impl Into<String>,
        qty: impl Into<Decimal>,
        price: Decimal,
        time_in_force: TimeInForce,
    ) -> Result<Transaction> {
        let order = OrderRequest {
            symbol: symbol.into(),
            side: OrderSide::Sell,
            position_side: None,
            order_type: SupportedOrderType::Limit,
            time_in_force: Some(time_in_force),
            quantity: Some(qty.into()),
            reduce_only: None,
            price: Some(price),
            stop_price: None,
            close_position: None,
            activation_price: None,
            callback_rate: None,
            working_type: None,
            price_protect: None,
            new_client_order_id: None,
        };
        self.place_order(order).await
    }

    /// Place a Market buy order
    #[cfg(not(feature = "backtest"))]
    pub async fn market_buy<S, F>(&self, symbol: S, qty: F) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<Decimal>,
    {
        let order = OrderRequest {
            symbol: symbol.into(),
            side: OrderSide::Buy,
            position_side: None,
            order_type: SupportedOrderType::Market,
            time_in_force: None,
            quantity: Some(qty.into()),
            reduce_only: None,
            price: None,
            stop_price: None,
            close_position: None,
            activation_price: None,
            callback_rate: None,
            working_type: None,
            price_protect: None,
            new_client_order_id: None,
        };
        self.place_order(order).await
    }

    /// Place a Market sell order
    #[cfg(not(feature = "backtest"))]
    pub async fn market_sell<S, F>(&self, symbol: S, qty: F) -> Result<Transaction>
    where
        S: Into<String>,
        F: Into<Decimal>,
    {
        let order: OrderRequest = OrderRequest {
            symbol: symbol.into(),
            side: OrderSide::Sell,
            position_side: None,
            order_type: SupportedOrderType::Market,
            time_in_force: None,
            quantity: Some(qty.into()),
            reduce_only: None,
            price: None,
            stop_price: None,
            close_position: None,
            activation_price: None,
            callback_rate: None,
            working_type: None,
            price_protect: None,
            new_client_order_id: None,
        };
        self.place_order(order).await
    }

    /// Place a cancellation order
    pub async fn cancel_order(&self, o: OrderCancellation) -> Result<CanceledOrderResponse> {
        let recv_window = o.recv_window.unwrap_or(self.recv_window);
        if let Some(order_id) = o.order_id {
            let as_u64 = OrderCancellationWithU64 {
                symbol: o.symbol,
                order_id: Some(match order_id.parse::<u64>() {
                    Ok(id) => id,
                    Err(_) => 0,
                }),
                orig_client_order_id: o.orig_client_order_id,
                new_client_order_id: o.new_client_order_id,
                recv_window: o.recv_window,
            };
            self.client.delete_signed_p("/fapi/v1/order", &as_u64, recv_window).await
        } else {
            self.client.delete_signed_p("/fapi/v1/order", &o, recv_window).await
        }
    }

    /// Get current position risk for the symbol
    pub async fn position_information<S>(&self, symbol: S) -> Result<Vec<Position>>
    where
        S: Into<String>,
    {
        self.client
            .get_signed_p(
                "/fapi/v2/positionRisk",
                Some(PairAndWindowQuery {
                    symbol: symbol.into(),
                    recv_window: self.recv_window,
                }),
                self.recv_window,
            )
            .await
    }

    /// Get current position risk for the symbol
    pub async fn position_information_v3<S>(&self, symbol: S) -> Result<Vec<PositionV3>>
    where
        S: Into<String>,
    {
        self.client
            .get_signed_p(
                "/fapi/v3/positionRisk",
                Some(PairAndWindowQuery {
                    symbol: symbol.into(),
                    recv_window: self.recv_window,
                }),
                self.recv_window,
            )
            .await
    }

    pub async fn all_position_information_v3(&self) -> Result<Vec<PositionV3>> {
        let payload: Option<HashMap<String, String>> = None;
        self.client
            .get_signed_p(
                "/fapi/v3/positionRisk",
                payload,
                self.recv_window,
            )
            .await
    }

    /// Return general [`AccountInformation`]
    pub async fn account_information(&self) -> Result<AccountInformation> {
        // needs to be changed to smth better later
        let payload = build_signed_request(BTreeMap::<String, String>::new(), self.recv_window)?;
        self.client.get_signed_d("/fapi/v2/account", &payload).await
    }

    // but its positions are missing entry_price which we need
    pub async fn account_information_v3(&self) -> Result<AccountInformationV3> {
        // needs to be changed to smth better later
        let payload = build_signed_request(BTreeMap::<String, String>::new(), self.recv_window)?;
        self.client.get_signed_d("/fapi/v3/account", &payload).await
    }

    /// Return account's [`AccountBalance`]
    pub async fn account_balance(&self) -> Result<Vec<AccountBalance>> {
        let parameters = BTreeMap::<String, String>::new();
        let request = build_signed_request(parameters, self.recv_window)?;
        self.client.get_signed_d("/fapi/v2/balance", request.as_str()).await
    }

    // commented out to be safe since I don't expect to use it
    // /// Change the initial leverage for the symbol
    // pub async fn change_initial_leverage<S>(&self, symbol: S, leverage: u8) -> Result<ChangeLeverageResponse>
    // where
    //     S: Into<String>,
    // {
    //     let mut parameters: BTreeMap<String, String> = BTreeMap::new();
    //     parameters.insert("symbol".into(), symbol.into());
    //     parameters.insert("leverage".into(), leverage.to_string());
    // 
    //     let request = build_signed_request(parameters, self.recv_window)?;
    //     self.client.post_signed_d("/fapi/v1/leverage", request.as_str()).await
    // }

    // commented out to be safe since I don't expect to use it
    /// Change the dual position side
    // pub async fn change_position_mode(&self, dual_side_position: bool) -> Result<()> {
    //     self.client
    //         .post_signed_p(
    //             "/fapi/v1/positionSide/dual",
    //             ChangePositionModeRequest { dual_side_position },
    //             self.recv_window,
    //         )
    //         .await?;
    //     Ok(())
    // }

    /// Cancel all open orders on this symbol
    pub async fn cancel_all_open_orders<S>(&self, symbol: S) -> Result<()>
    where
        S: Into<String>,
    {
        self.client
            .delete_signed_p(
                "/fapi/v1/allOpenOrders",
                PairQuery { symbol: symbol.into() },
                self.recv_window,
            )
            .await?;
        Ok(())
    }
}
