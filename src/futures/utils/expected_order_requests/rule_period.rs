use std::time::Duration;
use strum_macros::{Display, EnumString};
use crate::errors::{Error, Result};

const MIN_RULE_PERIOD_DURATION_SECS: u64 = 120;

#[derive(Deserialize, Serialize, Display, EnumString, PartialEq, Eq, Debug, Clone, Hash)]
pub enum RulePeriod {
    Seconds(u64),
    Minutes(u64),
    Hours(u64),
    Days(u64),
    Weeks(u64),
}

impl Default for RulePeriod {
    fn default() -> Self {
        Self::Seconds(0)
    }
}

impl RulePeriod {
    fn get_duration(&self) -> Duration {
        match self {
            RulePeriod::Seconds(seconds) => Duration::from_secs(*seconds),
            RulePeriod::Minutes(minutes) => Duration::from_secs(*minutes * 60),
            RulePeriod::Hours(hours) => Duration::from_secs(*hours * 60 * 60),
            RulePeriod::Days(days) => Duration::from_secs(*days * 24 * 60 * 60),
            RulePeriod::Weeks(weeks) => Duration::from_secs(*weeks * 7 * 24 * 60 * 60),
        }
    }
    
    pub fn get_validated_duration(&self) -> Result<Duration> {
        let duration = self.get_duration();
        let duration_secs = duration.as_secs();
        if duration_secs < MIN_RULE_PERIOD_DURATION_SECS {
            return Err(Error::ExpectedOrdersRuleViolated(format!("Rule period duration must be at least {MIN_RULE_PERIOD_DURATION_SECS} seconds. Duration: {duration_secs}")));
        }
        
        Ok(duration)
    }
    
    pub fn get_min_nanos_timestamp(&self, now_nanos: u64) -> Result<u64> {
        let duration = self.get_validated_duration()?;
        let selected_nanos = duration.as_nanos() as u64;
        Ok(now_nanos - selected_nanos)
    }
}