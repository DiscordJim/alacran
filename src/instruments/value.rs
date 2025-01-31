use std::{borrow::Borrow, collections::HashMap, fmt::Debug, hash::Hash, iter::Sum, ops::{Add, Mul}, sync::RwLock};

use once_cell::sync::Lazy;

use super::convert::{ConversionTable, CURRENCY_EXCHANGE};




#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Currency(Option<&'static str>);

impl Currency {
    pub fn null() -> Self {
        Self(None)
    }
    pub fn new(currency: &'static str) -> Self {
        Self(Some(currency))
    }
    pub fn is_null(&self) -> bool {
        self.0.is_none()
    }
    pub fn name(&self) -> &'static str {
        match self.0 {
            None => "NaN",
            Some(a) => a
        }
    }
}



#[derive(Clone)]
pub struct Value {
    currency: Currency,
    // conversion_table: Option<&'static ConversionTable>,
    amount: f64
}

impl Sum<Value> for Value {
    fn sum<I: Iterator<Item = Value>>(iter: I) -> Self {
        kahan_sum(iter)
    }
}

impl<'a> Sum<&'a Value> for Value {
    fn sum<I: Iterator<Item = &'a Value>>(iter: I) -> Self {
        kahan_sum(iter)
    }
}


/// Calculates the Kahan sum and returns a new currency sum.
pub fn kahan_sum<I, V>(iter: I) -> Value
    where 
        I: Iterator<Item = V>,
        V: Borrow<Value>
{
    let mut sum = 0.0;
    let mut c = 0.0;

    let mut cur = Currency::new("CAD");

    for item in iter {
        let item = item.borrow();
        cur = item.currency;


        let y = item.amount + c;
        (sum, c) = fast2sum(sum, y)
    }


    Value {
        amount: sum,
        currency: cur,
    }
}
    


impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        // println!("Bruh: {:?}", self.amount);
        let main = (self.amount).abs().floor() as usize;
        let principal = main.to_string().as_bytes().rchunks(3).rev().map(std::str::from_utf8).collect::<Result<Vec<&str>, _>>().unwrap().join(",");

        let cents= ((self.amount.abs() - (main as f64)) * 100.0).floor() as usize;

        if self.amount < 0.0 {
            write!(f, "-{principal}.{cents}{}", self.currency.name())
        } else {
            write!(f, "{principal}.{cents}{}", self.currency.name())
        }
        
    }
}


impl Value {
    pub fn dummy<C: Into<Currency>, F: Into<f64>>(cur: C, amount: F) -> Self {
        Self::new(cur, amount)
    }
    pub fn new<C: Into<Currency>, F: Into<f64>>(cur: C, amount: F) -> Self {
        Self {
            amount: amount.into(),
            currency: cur.into(),
        }
    }
    pub fn zero<C: Into<Currency>>() -> Self {
        Self::new(Currency::null(), 0.0)
    }
    pub fn negate(&self) -> Self {
        Self {
            amount: self.amount * -1.0,
            currency: self.currency
        }
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn non_decimal(&self) -> i128 {
        if self.amount < 0.0 {
            self.amount.ceil() as i128
        } else {
            self.amount.floor() as i128
        }
    }
    pub fn currency(&self) -> Currency {
        self.currency
    }

  
  
}

impl Mul<f64> for Value {
    type Output = Value;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            amount: self.amount * rhs,
            currency: self.currency
        }
    }    
}

impl Mul<f64> for &Value {
    type Output = Value;
    fn mul(self, rhs: f64) -> Self::Output {
        Value {
            amount: self.amount * rhs,
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
                currency: rhs.currency
            }
        } else {
            // println!("yes {:?}", CURRENCY_EXCHANGE.convert(rhs.clone(), self.currency));
            // CURRENCY_EXCHANGE.convert(rhs, self.currency).unwrap() + self
         
            CURRENCY_EXCHANGE.convert(rhs, self.currency).unwrap() + self
        }
    }
}




impl Into<Currency> for &'static str {
    fn into(self) -> Currency {
        Currency::new(self)
    }
}


/// Fast2Sum algorithm
fn fast2sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let z = s - a;
    let t = b - z; 
    (s, t)
}



#[cfg(test)]
mod tests {
  
    use crate::instruments::{convert::CURRENCY_EXCHANGE, value::Currency};

    use super::{ConversionTable, Value};


    /// Checks to see if Kahan summation formulae
    /// are working as designed.
    #[test]
    pub fn test_accurate_math() {        
        let values = vec![
            Value::dummy("CAD", 3939392.022123),
            Value::dummy("CAD", 22.023322123),
            Value::dummy("CAD", 32773.022123)
        ];

        assert!((values.iter().sum::<Value>().amount - 3972187.07).abs() < 0.01)
    }


    #[test]
    pub fn test_conversion() {

        CURRENCY_EXCHANGE.add_conversion("CAD", "COP", 2911.98);

       
        let bob = Value::new("CAD", 28.0);
        let alice = Value::new("COP", 600000.0);

        // panic!("yelllo {:?}", CURRENCY_EXCHANGE.convert(bob, Currency("COP")));


        let total = bob + alice;
        assert!((total.amount - (206.0 + 28.0)).abs() < 0.1)
       
    }
}