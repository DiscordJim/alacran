use chrono::{DateTime, Utc};
use value::{Currency, Value};

pub mod book;
pub mod value;
pub mod delta;
pub mod risk;
pub mod convert;
pub mod item;

pub trait Assesible {
    /// Asseses the value of an asset at a certain time.
    fn assess(&self, time: DateTime<Utc>) -> Value;
    /// Get the primary currency type of the asset.
    fn currency(&self) -> Currency;
}

pub struct AssessmentResult {
    /// The primary value of the assessment.
    value: Value,
    /// The excess amounts, this is usually just the sum of money from payouts
    /// and as added to cash on the books.
    cash: Value
}

impl AssessmentResult {
    pub fn new(value: Value, cash: Value) -> Self {
        Self { value, cash }
    }
}