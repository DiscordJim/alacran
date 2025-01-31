use std::borrow::Borrow;

use chrono::{DateTime, Duration, NaiveDateTime, TimeDelta, TimeZone, Utc};
use slotmap::SlotMap;

use super::{value::{Currency, Value}, Assesible};

#[derive(Default)]
pub struct Book {
    entries: SlotMap<ItemKey, Item>,
}

impl Book {
    /// Adds a new item to the book.
    pub fn add(&mut self, item: Item) -> ItemKey {
        self.entries.insert(item)
    }
    /// Adds an item to the book with a parent relationship to another entity.
    pub fn add_child(&mut self, new: Item, parent: ItemKey) -> ItemKey {
        let key = self.entries.insert(new);
        self.entries.get_mut(parent).unwrap().children.push(key);
        key
    }
}


impl Assesible for Book {
    fn assess(&self, time: DateTime<Utc>) -> Value {
        self.entries
            .iter()
            .map(|(_, v)| v.assess(time))
            .sum::<Value>()
    }
    fn currency(&self) -> Currency {
        self.entries.iter().nth(0).unwrap().1.currency()
    }
}

slotmap::new_key_type! {
    pub struct ItemKey;
}


/// An item to be put on the books.
pub struct Item {
    book_value: Value,
    interest: Option<Interest>,
    inception: DateTime<Utc>,
    // parent: Option<ItemKey>,
    children: Vec<ItemKey>,

    /// Changes, these typically correspond to payments and stuff of the like.
    deltas: Vec<(DateTime<Utc>, Value)>,

    /// Does this item have any sort of recurring payout of a fixed amount?
    payouts: Vec<Payout>


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

    use std::str::FromStr;

    use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

    use crate::instruments::{value::{ConversionTable, Value}, Assesible};

    use super::{Book, Interest, Item};

    pub fn make_credit_card(principal: usize, interest: f64) -> Item {
        Item::basic_debt(
            Value::new(
                "CAD",
                -1.0 * principal as f64,
                ConversionTable::new().free(),
            ),
            interest,
            Duration::days(365),
            Utc.with_ymd_and_hms(2008, 01, 01, 1, 1, 1).unwrap(),
        )
    }

    #[test]
    pub fn test_basic_debt() {
        let credit_card = make_credit_card(15000, 0.20);

        let current_debt =
            credit_card.assess(Utc.with_ymd_and_hms(2025, 01, 28, 11, 7, 0).unwrap());

        // panic!("wow {:?}", current_debt);

        assert_eq!(current_debt.non_decimal(), -338224);
    }

    #[test]
    pub fn test_basic_book() {
        let credit_card = make_credit_card(10000, 0.20);
        let credit_card_2 = make_credit_card(100, 0.02);

        // let house = Item::fixed(Value::new("CAD", 150000, ConversionTable::new().free()), Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0));

        let house = Item {
            book_value: Value::new("CAD", 150000, ConversionTable::new().free()),
            children: vec![],
            inception: Utc.with_ymd_and_hms(2000, 01, 01, 1, 1, 1).unwrap(),
            interest: Some(Interest {
                percent: 0.04,
                period: Duration::days(365),
            }),
            // parent: None,
            deltas: vec![],
            payouts: vec![]
        };

        let mut book = Book::default();
        book.add(credit_card);
        book.add(credit_card_2);
        book.add(house);

        assert_eq!(
            book.assess(Utc.with_ymd_and_hms(2025, 01, 28, 11, 7, 0).unwrap())
                .non_decimal(),
            175733
        );
    }

    #[test]
    pub fn partially_paid_credit_card() {
        // Standard credit card with 1000 of debt and a 20% interest.
        let mut credit = make_credit_card(1000, 0.20);

        // Pay off $1000 after having the card for one month.
        credit.add_delta(
            Utc.with_ymd_and_hms(2008, 02, 01, 1, 1, 1).unwrap(),
            Value::dummy("CAD", 1000),
        );

        // 7 years later there should be about $55.11CAD on the card.
        let value = credit.assess(Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap());
        assert_eq!(value.non_decimal(), -55);
    }
}
