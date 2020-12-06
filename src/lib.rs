use std::collections::HashMap;

use serde::{
    // de::{Error, Visitor},
    Deserialize,
};

/* #[derive(Debug)]
pub struct DeserializableString(String);

impl<'de> Deserialize<'de> for DeserializableString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Vis;

        impl<'de> Visitor<'de> for Vis {
            type Value = String;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("something that can be converted to a String")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(format!("{}", v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(format!("{}", v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(v)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(v.into())
            }
        }

        deserializer.deserialize_any(Vis).map(Self)
    }
} */

#[derive(Clone)]
#[derive(Deserialize, Debug)]
pub struct YearEvent {
    pub timestamp: Option<String>,
    #[serde(rename = "event")]
    pub year: String,
    pub owner_id: String,
    pub members: HashMap<String, User>,
}

#[derive(Clone)]
#[derive(Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub stars: u32,
    pub local_score: u32,
    pub global_score: u32,
    pub completion_day_level: Completion,
}

impl Eq for User {
    
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.local_score == other.local_score
    }
}

impl PartialOrd for User {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for User {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.local_score.cmp(&other.local_score)
    }
}

#[derive(Clone)]
#[derive(Deserialize, Debug)]
pub struct Completion {
    pub days: Option<HashMap<String, Day>>,
}

#[derive(Clone)]
#[derive(Deserialize, Debug)]
pub struct Day {
    pub challenges: Vec<Challenge>,
}

#[derive(Clone)]
#[derive(Deserialize, Debug)]
pub struct Challenge {
    pub get_star_ts: String,
}
