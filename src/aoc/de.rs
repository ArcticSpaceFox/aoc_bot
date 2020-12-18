//! Custom deserializers for response fields of the AoC API.

use std::fmt;

use chrono::{DateTime, TimeZone, Utc};
use serde::de::{self, Deserializer, Visitor};

/// Deserialize a UNIX timestamp that is encoded as string instead of an integer.
pub fn string_timestamp<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(TimestampVisitor)
}

struct TimestampVisitor;

impl<'de> Visitor<'de> for TimestampVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a UNIX timestamp encoded as string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let ts = v.parse::<i64>().map_err(|e| E::custom(e.to_string()))?;
        Ok(Utc.timestamp(ts, 0))
    }
}
