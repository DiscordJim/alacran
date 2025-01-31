use std::sync::RwLock;
use lazy_static::lazy_static;
use super::value::{Currency, Value};

lazy_static! {
    /// A current exchange.
    pub static ref CURRENCY_EXCHANGE: ConversionTable = ConversionTable::new();
}

#[derive(Debug)]
pub struct ConversionTable {
    mappings: RwLock<Vec<(Currency, Currency, f64)>>
}

impl ConversionTable {
    pub fn new() -> Self {
        Self {
            mappings: RwLock::default()
        }
    }
    pub fn add_conversion(&self, source: impl Into<Currency>, target: impl Into<Currency>, factor: f64) {
        let mut mappings = self.mappings.write().unwrap();
        let (source, target) = (source.into(), target.into());

        mappings.push((source, target, factor));
        mappings.push((target, source, 1.0 / factor));
    }
    /// Convert a piece of currency.
    pub fn convert(&self, value: Value, target: Currency) -> Option<Value> {
        let handle = self.mappings.read().unwrap();
        let (_, _, converted) = handle.iter().find(|(from, to, _)| *from == value.currency() && *to == target)?;
        Some(Value::dummy(target, value.amount() * *converted))
    }
    
}

