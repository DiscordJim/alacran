use std::{collections::HashMap, fmt::Debug, hash::Hash, iter::Sum, ops::{Add, Mul}};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Currency(&'static str);

pub struct ConversionTable {
    mappings: HashMap<Currency, HashMap<Currency, f64>>
}

impl ConversionTable {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new()
        }
    }
    pub fn add_conversion(&mut self, source: impl Into<Currency>, target: impl Into<Currency>, factor: f64) {
        let source = source.into();
        let target = target.into();
        if !self.mappings.contains_key(&source) {
            self.mappings.insert(source, HashMap::new());
        }
        if !self.mappings.contains_key(&target) {
            self.mappings.insert(target, HashMap::new());
        }
        self.mappings.get_mut(&source).unwrap().insert(target, factor);
        self.mappings.get_mut(&target).unwrap().insert(source,  1.0 /factor);
    }
    pub fn convert(&'static self, value: Value, target: Currency) -> Option<Value> {
        Some(Value {
            amount: self.mappings.get(&value.currency)?.get(&target)? * value.amount,
            currency: target,
            conversion_table: Some(&self)
        })
    }
    pub fn free(self) -> &'static Self {
        Box::leak(Box::new(self))
    }
}

#[derive(Clone)]
pub struct Value {
    currency: Currency,
    conversion_table: Option<&'static ConversionTable>,
    amount: f64
}

impl Sum for Value {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum = 0.0;
        let mut c = 0.0;
        let mut curr= Currency("NA");
        let mut table = None;
        for i in iter {
            let y = i.amount + c;
            curr = i.currency;
            table = i.table();
            let t = sum + y;

            c = (t - sum) - y;
            sum = t;
        }
        Self {
            amount: sum,
            currency: curr,
            conversion_table: table
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        // println!("Bruh: {:?}", self.amount);
        let main = (self.amount).abs().floor() as usize;
        let principal = main.to_string().as_bytes().rchunks(3).rev().map(std::str::from_utf8).collect::<Result<Vec<&str>, _>>().unwrap().join(",");

        let cents= ((self.amount.abs() - (main as f64)) * 100.0).floor() as usize;

        if self.amount < 0.0 {
            write!(f, "-{principal}.{cents}{}", self.currency.0)
        } else {
            write!(f, "{principal}.{cents}{}", self.currency.0)
        }
        
    }
}


impl Value {
    pub fn dummy<C: Into<Currency>, F: Into<f64>>(cur: C, amount: F) -> Self {
        Self::new(cur, amount, ConversionTable::new().free())
    }
    pub fn new<C: Into<Currency>, F: Into<f64>>(cur: C, amount: F, table: &'static ConversionTable) -> Self {
        Self {
            amount: amount.into(),
            currency: cur.into(),
            conversion_table: Some(table)
        }
    }
    pub fn zero<C: Into<Currency>>(cur: C, table: &'static ConversionTable) -> Self {
        Self::new(cur, 0.0, table)
    }
    pub fn negate(&self) -> Self {
        Self {
            amount: self.amount * -1.0,
            conversion_table: self.conversion_table,
            currency: self.currency
        }
    }
    pub fn table(&self) -> Option<&'static ConversionTable> {
        self.conversion_table
    }

    pub fn non_decimal(&self) -> i128 {
        if self.amount < 0.0 {
            self.amount.ceil() as i128
        } else {
            self.amount.floor() as i128
        }
    }

  
  
}

impl Mul<f64> for Value {
    type Output = Value;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            amount: self.amount * rhs,
            conversion_table: self.conversion_table,
            currency: self.currency
        }
    }    
}

impl Mul<f64> for &Value {
    type Output = Value;
    fn mul(self, rhs: f64) -> Self::Output {
        Value {
            amount: self.amount * rhs,
            conversion_table: self.conversion_table,
            currency: self.currency
        }
    }    
}

impl Add<Value> for Value {
    type Output = Value;
    fn add(self, rhs: Value) -> Self::Output {
        if self.currency == rhs.currency {
            Self {
                amount: self.amount + rhs.amount,
                conversion_table: self.conversion_table,
                currency: rhs.currency
            }
        } else {
            self.conversion_table.unwrap().convert(rhs, self.currency).unwrap() + self
        }
    }
}




impl Into<Currency> for &'static str {
    fn into(self) -> Currency {
        Currency(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{ConversionTable, Value};



    #[test]
    pub fn test_conversion() {
        let mut table = ConversionTable::new();
        table.add_conversion("CAD", "COP", 2911.98);
        let table = table.free();

        let bob = Value::new("CAD", 28.0, table);
        let alice = Value::new("COP", 600000.0, table);


        let total = bob + alice;
        assert!((total.amount - (206.0 + 28.0)).abs() < 0.1)
       
    }
}