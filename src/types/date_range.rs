use time::Month;

use crate::Error;

use super::{Date, DateTime};

#[derive(Clone, Debug)]
pub struct DateRange {
    pub start: Date,
    pub end: Date,
}

impl DateRange {
    /// Return a range with the start as the 1st day ofthe current month and the end as the last day of the current month
    pub fn get_current_month_inclusive() -> Result<Self, Error> {
        let now = DateTime::now_utc().date();
        let start = now.replace_day(1)?;
        let start_of_next_month = if start.month() == Month::December {
            start.replace_month(time::Month::January)?.replace_year(start.year() + 1)?
        }
        else {
            start.replace_month(start.month().next())?
        };
        let end = start_of_next_month.replace_ordinal(start_of_next_month.ordinal() - 1)?;

        Ok(Self { start: Date::from(start), end: Date::from(end) })
    }
}