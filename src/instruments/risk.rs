use chrono::{DateTime, TimeDelta, Utc};
use super::Assesible;

/// Adds an element of risk to an [Assesible] item,
/// the exact function of these depends heavily on the
/// mechanism selected (the enum variant!)
pub enum Risk<A: Assesible> {
    /// The asset will drop by value by a certain amount.
    CertainLossPercentage { asset: A, percent: f64 },
    /// Loses value as a function of time, starting at some date.
    LosePercentOverTime {
        asset: A,
        percent: f64,
        period: TimeDelta,
        starting: DateTime<Utc>,
    },
}

impl<A: Assesible> Assesible for Risk<A> {
    fn assess(&self, time: chrono::DateTime<chrono::Utc>) -> super::value::Value {
        match self {
            Risk::CertainLossPercentage { asset, percent } => asset.assess(time) * *percent,
            Risk::LosePercentOverTime {
                asset,
                percent,
                period,
                starting,
            } => {
                if *starting > time {
                    // The interest has not started going down yet.
                    return asset.assess(time);
                }

                // Count how many periods of interest have passed
                let periods = (time - *starting).num_nanoseconds().unwrap() as f64
                    / period.num_nanoseconds().unwrap() as f64;
                
                // Count the loss multiplier to multiply the underlying value by.
                let loss_factor = (1.0 - *percent).powf(periods);

                asset.assess(time) * loss_factor
            }
        }
    }
    fn currency(&self) -> super::value::Currency {
        match self {
            Risk::CertainLossPercentage { asset, .. } => asset.currency(),
            Risk::LosePercentOverTime { asset, .. } => asset.currency(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeDelta, TimeZone, Utc};

    use crate::instruments::{item::Item, risk::Risk, value::Value, Assesible};

    #[test]
    pub fn test_always_fail() {
        let main = Item::basic_debt(
            Value::dummy("CAD", 10.0),
            0.20,
            TimeDelta::days(30),
            Utc::now(),
        );
        assert_eq!(main.assess(Utc::now()).non_decimal(), 10);

        let risky = Risk::CertainLossPercentage {
            asset: main,
            percent: 0.5,
        };
        assert_eq!(risky.assess(Utc::now()).non_decimal(), 5);
    }

    /// Checks that no interest is being applied before start.
    ///
    /// This may be useful for stuff like student loans where they only
    /// start generating interest once you are out of school.
    #[test]
    pub fn test_no_interest_before_start() {
        let interest_start = Utc.with_ymd_and_hms(2002, 1, 1, 0, 0, 0).unwrap();

        // Add a family car that devalues 10% about once a year.
        let family_car = Risk::LosePercentOverTime {
            asset: Item::fixed(Value::dummy("CAD", 50_000.00), interest_start),
            percent: 0.10,
            period: TimeDelta::days(365),
            starting: interest_start,
        };

        assert_eq!(
            family_car
                .assess(Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap())
                .non_decimal(),
            50_000
        );
    }

    #[test]
    pub fn test_lose_over_time_percent() {
        // The purchase date of the car and when we bring it in for inspecion.
        let purchase_date = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let inspect_date = Utc.with_ymd_and_hms(2004, 1, 1, 0, 0, 0).unwrap();

        // Add a family car that devalues 10% about once a year.
        let family_car = Risk::LosePercentOverTime {
            asset: Item::fixed(Value::dummy("CAD", 50_000.00), purchase_date),
            percent: 0.10,
            period: TimeDelta::days(365),
            starting: purchase_date,
        };

        // Should have devaluated to $32,795. Don't forget that 2000 is a leap year,
        // which is taken into account.
        assert_eq!(family_car.assess(inspect_date).non_decimal(), 32795);
    }
}
