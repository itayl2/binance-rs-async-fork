use std::fmt;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BinanceErrorWithSend {
    pub message: String,
}

impl BinanceErrorWithSend {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

// so that we could create wrapper methods for methods of the dydx client and use this error to allow these methods to also be called from tokio spawn
unsafe impl Send for BinanceErrorWithSend {}

impl fmt::Display for BinanceErrorWithSend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BinanceErrorWithSend {}

impl From<Box<dyn std::error::Error>> for BinanceErrorWithSend {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        BinanceErrorWithSend::new(&format!("{err:?}"))
    }
}

impl From<anyhow::Error> for BinanceErrorWithSend {
    fn from(err: anyhow::Error) -> Self {
        BinanceErrorWithSend::new(&err.to_string())
    }
}