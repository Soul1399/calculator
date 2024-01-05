
use std::{error::Error, cmp::Ordering, ops::RangeInclusive};

use lazy_static::lazy_static;

const MAX_MONTH: u8 = 12;
const MAX_MONTH32: i32 = MAX_MONTH as i32;
const MAX_DAY: u8 = 31;
const MAX_COMMON_DAY: u8 = 28;
const GUESS_DAY: u8 = MAX_COMMON_DAY + 1;
const GUESS_MONTH: u8 = 2;

lazy_static! {
    static ref MONTH_RANGE: RangeInclusive<u8> = 1..=MAX_MONTH;
    static ref BASIC_DAY_RANGE: RangeInclusive<u8> = 1..=MAX_COMMON_DAY;
    static ref GREATER_DAY_RANGE: RangeInclusive<u8> = 1..=MAX_DAY;
    static ref DAY_MAP: Vec<u8> = vec![MAX_DAY, MAX_COMMON_DAY, MAX_DAY, MAX_DAY - 1, MAX_DAY, MAX_DAY - 1, MAX_DAY, MAX_DAY, MAX_DAY - 1, MAX_DAY, MAX_DAY - 1, MAX_DAY];
}

#[derive(Clone, Copy, Debug, Eq, Default)]
pub struct DateKey {
    month: u8,
    year: i32
}

impl DateKey {
    pub fn build(m: u8, y: i32) -> Self {
        let mut dt = DateKey { month: 1, year: 1 };
        match dt.apply(m, y) {
            Err(e) => panic!("{}", e.details),
            Ok(()) => {}
        }
        dt
    }

    pub fn month(&self) -> u8 {
        self.month
    }

    pub fn year(&self) -> i32 {
        self.year
    }

    pub fn apply(&mut self, m: u8, y: i32) -> Result<(), DateError> {
        if !MONTH_RANGE.contains(&m) {
            return Err(DateError { details: String::from(format!("Month has to be between 1 and {}", MONTH_RANGE.end())) });
        }
        self.month = m;
        self.year = y;
        Ok(())
    }

    pub fn add_months(&mut self, n: i32) {
        if n == 0 {
            return;
        }
        let mut new_m: i32;
        if let 1..=11 = n.abs() {
            new_m = (self.month as i32) + n;
        }
        else if n % MAX_MONTH32 == 0 {
            new_m = self.month as i32;
            self.year += n / MAX_MONTH32;
        }
        else {
            self.year += f32::trunc((n / MAX_MONTH32) as f32) as i32;
            new_m = (self.month as i32) + n % MAX_MONTH32;
        }
        if new_m < 1 {
            new_m = MAX_MONTH32 + new_m;
            self.year -= 1;
        }
        else if new_m > MAX_MONTH32 {
            new_m = new_m - MAX_MONTH32;
            self.year += 1;
        }
        self.month = new_m as u8;
    }
}

impl ToString for DateKey {
    fn to_string(&self) -> String {
        let slice = vec![
            self.month.to_string(),
            String::from('/'),
            self.year.to_string()
        ];
        slice.concat()
    }
}

#[derive(Debug)]
pub struct DateError {
    pub details: String
}

impl Error for DateError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl std::fmt::Display for DateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl PartialOrd for DateKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.year.partial_cmp(&other.year) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.month.partial_cmp(&other.month)
    }

    fn lt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Less))
    }

    fn le(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Less | Ordering::Equal))
    }

    fn gt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Greater))
    }

    fn ge(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Greater | Ordering::Equal))
    }
}

impl PartialEq for DateKey {
    fn eq(&self, other: &Self) -> bool {
        self.month == other.month && self.year == other.year
    }
}

impl Ord for DateKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Less)
    }
}

#[derive(Clone, Copy, Debug, Eq, Default)]
pub struct DayDate {
    num: u8,
    date_key: DateKey
}

impl PartialEq for DayDate {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num
        && self.date_key.partial_cmp(&other.date_key) == Some(Ordering::Equal)
    }
}

impl DayDate {
    pub fn build(d: u8, m: u8, y: i32) -> Result<Self, DateError> {
        let mut date: DayDate = Default::default();
        date.apply(d, m, y)?;
        Ok(date)
    }

    pub fn apply(&mut self, d: u8, m: u8, y: i32) -> Result<(), DateError> {
        let mut num: u8 = 0;
        if let Err(e) = self.date_key.apply(m, y) {
            return Err(e);
        }
        self.date_key.month = m;
        self.date_key.year = y;
        
        if BASIC_DAY_RANGE.contains(&d) {
            num = d;
        }
        else if d == GUESS_DAY && m == GUESS_MONTH {
            if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { num = d; }
        }
        else if GREATER_DAY_RANGE.contains(&d) {
            if let 1 | 3 | 5 | 7 | 8 | 10 | 12 = m { num = d; }
        }
        else if d <= (GUESS_DAY + 1) {
            if let 4 | 6 | 9 | 11 = m { num = d; }
        }

        if num == 0 {
            return Err(DateError { details: "Day of date should be between 1 and ...".to_owned() });
        }

        self.num = num;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Month has to be between 1 and 12")]
    fn apply_invalid() {
        let _ = DateKey::build(40, 2000);
    }

    #[test]
    fn add_months() {
        let mut d = DateKey::build(1, 2000);
        d.add_months(-12);
        assert_eq!(d.month, 1);
        assert_eq!(d.year, 1999);

        d.add_months(3);
        assert_eq!(d.month, 4);
        assert_eq!(d.year, 1999);

        d.add_months(11);
        assert_eq!(d.month, 3);
        assert_eq!(d.year, 2000);

        d.add_months(-100);
        assert_eq!(d.month, 11);
        assert_eq!(d.year, 1991);

        d.add_months(120);
        assert_eq!(d.month, 11);
        assert_eq!(d.year, 2001);
    }
}