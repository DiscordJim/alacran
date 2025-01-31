use std::borrow::Borrow;

use chrono::{DateTime, Duration, NaiveDateTime, TimeDelta, TimeZone, Utc};
use slotmap::SlotMap;

use super::{item::Item, value::{Currency, Value}, Assesible};

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
        self.entries.get_mut(parent).unwrap().add_child(key);
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





#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

    use crate::instruments::{book::Book, item::{Interest, Item}, value::Value, Assesible};



    pub fn make_credit_card(principal: usize, interest: f64) -> Item {
        Item::basic_debt(
            Value::new(
                "CAD",
                -1.0 * principal as f64,
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
            book_value: Value::new("CAD", 150000),
            children: vec![],
            inception: Utc.with_ymd_and_hms(2000, 01, 01, 1, 1, 1).unwrap(),
            interest: Some(Interest::new(0.04, Duration::days(365))),
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
