use chrono::{DateTime, Utc};
use value::{Currency, Value};

pub mod book;
pub mod value;
pub mod delta;
pub mod risk;

pub trait Assesible {
    /// Asseses the value of an asset at a certain time.
    fn assess(&self, time: DateTime<Utc>) -> Value;
    /// Get the primary currency type of the asset.
    fn currency(&self) -> Currency;
}