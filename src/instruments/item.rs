use std::borrow::Borrow;

use chrono::{DateTime, TimeDelta, Utc};

use super::{book::ItemKey, value::{Currency, Value}, Assesible};


// / An item to be put on the books.
pub struct Item {
    pub book_value: Value,
    pub interest: Option<Interest>,
    pub inception: DateTime<Utc>,
    // parent: Option<ItemKey>,
    pub children: Vec<ItemKey>,

    /// Changes, these typically correspond to payments and stuff of the like.
    pub deltas: Vec<(DateTime<Utc>, Value)>,

    /// Does this item have any sort of recurring payout of a fixed amount?
    pub payouts: Vec<Payout>


}



pub enum Payout {
    OneTime {
        amount: Value,
        time: DateTime<Utc>
    },
    InterestOneTime {
        principal: Value,
        time: DateTime<Utc>,
        interest: Interest
    },
    FixedRecurring {
        amount: Value,
        start: DateTime<Utc>,
        frequency: TimeDelta
    },
    InterestRecurring {
        principal: Value,
        start: DateTime<Utc>,
        frequency: TimeDelta,
        interest: Interest
    }
}



impl Item {
    pub fn fixed(value: Value, inception: DateTime<Utc>) -> Self {
        Self {
            book_value: value,
            children: vec![],
            // parent: None,
            inception,
            interest: None,
            deltas: vec![],
            payouts: vec![]
        }
    }
    pub fn basic_debt(
        value: Value,
        interest: f64,
        period: TimeDelta,
        inception: DateTime<Utc>,
    ) -> Self {
        Self {
            book_value: value,
            children: vec![],
            // parent: None,
            inception,
            interest: Some(Interest {
                percent: interest,
                period,
            }),
            deltas: vec![],
            payouts: vec![]
        }
    }
    pub fn add_delta(&mut self, time: DateTime<Utc>, value: Value) {
        self.deltas.push((time, value));
        self.deltas.sort_by_key(|(f, _)| *f);
    }
    pub fn add_child(&mut self, key: ItemKey) {
        self.children.push(key)
    }
    
}

impl Assesible for Item {
    fn assess(&self, time: DateTime<Utc>) -> Value {
        if self.deltas.is_empty() {
            if self.interest.is_some() {
                self.interest
                    .as_ref()
                    .unwrap()
                    .apply(self.inception, time, &self.book_value)
            } else {
                self.book_value.clone()
            }
        } else {
            if self.interest.is_some() {
                let mut book = self.book_value.clone();
                let mut incep = self.inception;
                for (rtime, payment) in &self.deltas {
                    if *rtime > time {
                        // If we have hit the maximum time stop applying payments, we do not want to factor this into our calculations.
                        break;
                    }

                    let assessed = self.interest.as_ref().unwrap().apply(incep, *rtime, book);
                    book = assessed + payment.clone();
                    incep = *rtime;
                }

                self.interest.as_ref().unwrap().apply(incep, time, book)
            } else {
                self.book_value.clone() + self.deltas.iter().map(|(_, i)| i).sum()
            }
        }
    }
    fn currency(&self) -> Currency {
        self.book_value.currency()
    }
}


pub struct Interest {
    percent: f64,  
    period: TimeDelta,
}

impl Interest {
    pub fn new(percent: f64, period: TimeDelta) -> Self {
        Self { percent, period }
    }
    /// Apply the interest formula to the value, this
    /// introspects on the settings of this [Interest] object
    /// to calculate it.
    pub fn apply<R: Borrow<Value>>(
        &self,
        inception: DateTime<Utc>,
        current_time: DateTime<Utc>,
        value: R,
    ) -> Value {
        let delta = current_time - inception;
        let periods = delta.num_nanoseconds().unwrap() as f64
            / self.period.num_nanoseconds().unwrap() as f64;
        value.borrow() * (1.0 + self.percent).powf(periods)
    }
    /// This returns the actual interest amounts.
    pub fn interest<R: Borrow<Value>>(
        &self,
        inception: DateTime<Utc>,
        current_time: DateTime<Utc>,
        value: R,
    ) -> Value {
        self.apply(inception, current_time, value.borrow()) + value.borrow().negate()
    }
}


#[cfg(test)]
mod tests {

    #[test]
    pub fn test_item_fixed() {

    }
}