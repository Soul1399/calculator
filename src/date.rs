
use std::{error::Error, cmp::Ordering, ops::RangeInclusive};
use regex::Regex;
use lazy_static::lazy_static;

const MAX_MONTH: u8 = 12;
const MAX_COMMON_DAY: u8 = 28;
const FEB_MONTH: u8 = 2;
const PATTERN_YEAR: &str = r"\d{4}";
const PATTERN_MONTH: &str = r"1[0-2]|0?\d";
const PATTERN_DAY: &str = r"[12]\d|3[01]|0?\d";
const ERR_INVALID_DATE_STR: &str = "Invalid date format";

lazy_static! {
    static ref MONTH_RANGE: RangeInclusive<u8> = 1..=MAX_MONTH;
    static ref DAY_MAP: Vec<u8> = vec![
        MAX_COMMON_DAY + 3,
        MAX_COMMON_DAY,
        MAX_COMMON_DAY + 3,
        MAX_COMMON_DAY + 2,
        MAX_COMMON_DAY + 3,
        MAX_COMMON_DAY + 2,
        MAX_COMMON_DAY + 3,
        MAX_COMMON_DAY + 3,
        MAX_COMMON_DAY + 2,
        MAX_COMMON_DAY + 3,
        MAX_COMMON_DAY + 2,
        MAX_COMMON_DAY + 3
    ];
    
    static ref RGX_DATEYMD: Regex = Regex::new(format!(r"^(?<y>{})(?<m>{})(?<d>{})$", PATTERN_YEAR, PATTERN_MONTH, PATTERN_DAY).as_str()).unwrap();
    
    static ref RGX_DATEYMD_H: Regex = Regex::new(format!(r"^(?<y>{})-(?<m>{})-(?<d>{})$", PATTERN_YEAR, PATTERN_MONTH, PATTERN_DAY).as_str()).unwrap();
    static ref RGX_DATEYMD_S: Regex = Regex::new(format!(r"^(?<y>{})/(?<m>{})/(?<d>{})$", PATTERN_YEAR, PATTERN_MONTH, PATTERN_DAY).as_str()).unwrap();
    static ref RGX_DATEYMD_D: Regex = Regex::new(format!(r"^(?<y>{})\.(?<m>{})\.(?<d>{})$", PATTERN_YEAR, PATTERN_MONTH, PATTERN_DAY).as_str()).unwrap();

    static ref RGX_DATEDMY_H: Regex = Regex::new(format!(r"^(?<d>{})-(?<m>{})-(?<y>{})$", PATTERN_DAY, PATTERN_MONTH, PATTERN_YEAR).as_str()).unwrap();
    static ref RGX_DATEDMY_S: Regex = Regex::new(format!(r"^(?<d>{})/(?<m>{})/(?<y>{})$", PATTERN_DAY, PATTERN_MONTH, PATTERN_YEAR).as_str()).unwrap();
    static ref RGX_DATEDMY_D: Regex = Regex::new(format!(r"^(?<d>{})\.(?<m>{})\.(?<y>{})$", PATTERN_DAY, PATTERN_MONTH, PATTERN_YEAR).as_str()).unwrap();

    static ref RGX_DATEMDY_H: Regex = Regex::new(format!(r"^(?<m>{})-(?<d>{})-(?<y>{})$", PATTERN_MONTH, PATTERN_DAY, PATTERN_YEAR).as_str()).unwrap();
    static ref RGX_DATEMDY_S: Regex = Regex::new(format!(r"^(?<m>{})/(?<d>{})/(?<y>{})$", PATTERN_MONTH, PATTERN_DAY, PATTERN_YEAR).as_str()).unwrap();
    static ref RGX_DATEMDY_D: Regex = Regex::new(format!(r"^(?<m>{})\.(?<d>{})\.(?<y>{})$", PATTERN_MONTH, PATTERN_DAY, PATTERN_YEAR).as_str()).unwrap();
}

#[derive(Clone, Copy, Debug, Eq, Default)]
pub struct DateKey {
    month: u8,
    year: i32
}

impl DateKey {
    pub fn new(m: u8, y: i32) -> Self {
        let o = DateKey::build(m, y);
        if let Err(e) = o {
            panic!("{}", e.details);
        }
        o.unwrap()
    }

    pub fn build(m: u8, y: i32) -> Result<Self, DateError> {
        let mut dt: DateKey = Default::default();
        dt.apply(m, y)?;
        Ok(dt)
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
        let max_m = MAX_MONTH as i32;
        let mut new_m: i32;
        if let 1..=11 = n.abs() {
            new_m = (self.month as i32) + n;
        }
        else if n % max_m == 0 {
            new_m = self.month as i32;
            self.year += n / max_m;
        }
        else {
            self.year += f32::trunc((n / max_m) as f32) as i32;
            new_m = (self.month as i32) + n % max_m;
        }
        if new_m < 1 {
            new_m = max_m + new_m;
            self.year -= 1;
        }
        else if new_m > max_m {
            new_m = new_m - max_m;
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

impl PartialOrd for DayDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.date_key.partial_cmp(&other.date_key) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        
        self.num.partial_cmp(&other.num)
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

impl Ord for DayDate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Less)
    }
}

impl ToString for DayDate {
    fn to_string(&self) -> String {
        let slice = vec![
            self.year().to_string(),
            self.month().to_string(),
            self.day().to_string()
        ];
        slice.join("-")
    }
}

impl DayDate {
    pub fn day(&self) -> u8 {
        self.num
    }

    pub fn month(&self) -> u8 {
        self.date_key.month()
    }

    pub fn year(&self) -> i32 {
        self.date_key.year()
    }

    pub fn new(y: i32, m: u8, d: u8) -> Self {
        let o = DayDate::build(y, m, d);
        if let Err(e) = o {
            panic!("{}", e.details);
        }
        o.unwrap()
    }

    pub fn build(y: i32, m: u8, d: u8) -> Result<Self, DateError> {
        let mut date: DayDate = Default::default();
        date.apply(y, m, d)?;
        Ok(date)
    }

    pub fn apply(&mut self, y: i32, m: u8, d: u8) -> Result<(), DateError> {
        let num: u8;
        self.date_key.apply(m, y)?;
        self.date_key.month = m;
        self.date_key.year = y;
        let max = max_day_of_month(m, y);
        
        if (1..=max).contains(&d) {
            num = d;
        }
        else {
            return Err(DateError { details: format!("Day of date should be between 1 and {}", max) });
        }

        self.num = num;
        Ok(())
    }

    pub fn add_days(&mut self, n: i32) {
        if n == 0 { return }
        if n < 0 {
            self.subtract_days(n);
        }
        else {
            self.append_days(n);
        }
    }

    pub fn add_months(&mut self, n: i32) {
        self.date_key.add_months(n);
    }

    pub fn add_years(&mut self, n: i32) {
        self.add_months(n * MAX_MONTH  as i32);
    }

    fn subtract_days(&mut self, n: i32) {
        assert!(n < 0);
        if n + self.num as i32 > 0 {
            self.num += n as u8;
            return;
        }

        let mut num = n + self.num as i32;
        let mut new_key = self.date_key.clone();
        new_key.add_months(-1);
        let mut max = max_day_of_month(new_key.month, new_key.year);
        while num < 0 {
            num += max as i32;
            new_key.add_months(-1);
            max = max_day_of_month(new_key.month, new_key.year);
        }
        self.date_key = new_key;
        self.num = max + num as u8;
    }

    fn append_days(&mut self, n: i32) {
        assert!(n > 0);
        let mut max = max_day_of_month(self.date_key.month, self.date_key.year);
        if n + self.num as i32 <= max as i32 {
            self.num += n as u8;
            return;
        }
        
        let rest = max - self.num;
        let mut num = n - rest as i32;
        let mut new_key = self.date_key.clone();
        
        new_key.add_months(1);
        max = max_day_of_month(new_key.month, new_key.year);
        while num > max as i32 {
            num -= max as i32;
            new_key.add_months(1);
            max = max_day_of_month(new_key.month, new_key.year);
        }

        self.date_key = new_key;
        self.num = num as u8;
    }

    pub fn parse(string: &str) -> Result<DayDate, DateError> {
        let regex_list: Vec<&Regex> = vec![
            &RGX_DATEYMD,
            &RGX_DATEYMD_H, &RGX_DATEYMD_D, &RGX_DATEYMD_S,
            &RGX_DATEDMY_H, &RGX_DATEDMY_D, &RGX_DATEDMY_S,
            &RGX_DATEMDY_H, &RGX_DATEMDY_D, &RGX_DATEMDY_S
        ];
        let o = regex_list.into_iter().find(|rgx| rgx.is_match(string));
        if o.is_none() {
            return Err(DateError { details: ERR_INVALID_DATE_STR.to_owned() })
        }
        let rg = o.unwrap();
        let mut date: DayDate = Default::default();
        let captures = rg.captures(string).unwrap();
        let parsed_day = str::parse::<u8>(captures.name("d").unwrap().as_str()).unwrap();
        let parsed_month = str::parse::<u8>(captures.name("m").unwrap().as_str()).unwrap();
        let parsed_year = str::parse::<i32>(captures.name("y").unwrap().as_str()).unwrap();
        date.apply(parsed_year, parsed_month, parsed_day)?;
        Ok(date)
    }
}

pub fn is_leap_year(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

pub fn max_day_of_xmonth(index_m: usize, y:i32) -> u8 {
    if index_m as u8 + 1 == FEB_MONTH && is_leap_year(y) {
        return DAY_MAP[index_m] + 1
    }
    DAY_MAP[index_m]
}

pub fn max_day_of_month(m: u8, y:i32) -> u8 {
    assert!(MONTH_RANGE.contains(&m));
    max_day_of_xmonth(m as usize - 1, y)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Month has to be between 1 and 12")]
    fn apply_invalid() {
        let _ = DateKey::new(40, 2000);
    }

    #[test]
    fn add_months() {
        let mut d = DateKey::new(1, 2000);
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

    #[test]
    fn add_days_tests() {
        let mut d = DayDate::build(2000, 1, 1).unwrap();
        d.add_days(30);
        assert_eq!(d.num, 31);
        assert_eq!(d.date_key.month, 1);
        assert_eq!(d.date_key.year, 2000);
        
        d.add_days(1);
        assert_eq!(d.num, 1);
        assert_eq!(d.date_key.month, 2);
        assert_eq!(d.date_key.year, 2000);

        d.add_days(-1);
        assert_eq!(d.num, 31);
        assert_eq!(d.date_key.month, 1);
        assert_eq!(d.date_key.year, 2000);

        d.add_months(12);
        assert_eq!(d.num, 31);
        assert_eq!(d.date_key.month, 1);
        assert_eq!(d.date_key.year, 2001);

        d.add_years(1);
        assert_eq!(d.num, 31);
        assert_eq!(d.date_key.month, 1);
        assert_eq!(d.date_key.year, 2002);
    }

    #[test]
    fn invalid_day() {
        let mut x: DayDate = Default::default();
        let mut result  = x.apply(1, 0, 1);
        if let Err(e) = result {
            assert!(e.details.starts_with("Month "));
        }

        result = x.apply(1, 1, 90);
        if let Err(e) = result {
            assert!(e.details.starts_with("Day "));
        }

        result = x.apply(1, 1, 0);
        if let Err(e) = result {
            assert!(e.details.starts_with("Day "));
        }

        result = x.apply(1, 13, 21);
        if let Err(e) = result {
            assert!(e.details.starts_with("Month "));
        }

        result = x.apply(1, 2, 30);
        if let Err(e) = result {
            assert!(e.details.starts_with("Day "));
        }
    }

    #[test]
    fn test_parse() {
        let mut s = "20231010";
        let mut d = DayDate::parse(s).unwrap();
        assert_eq!(d, DayDate::build(2023, 10, 10).unwrap());
        
        s = "2023-10-10";
        d = DayDate::parse(s).unwrap();
        assert_eq!(d, DayDate { num: 10, date_key: DateKey::new(10, 2023) });

        s = "21-12-2023";
        d = DayDate::parse(s).unwrap();
        assert_eq!(d.year(), 2023);
        assert_eq!(d.month(), 12);
        assert_eq!(d.day(), 21);

        s = "200011";
        d = DayDate::parse(s).unwrap();
        assert_eq!(d, DayDate { num: 1, date_key: DateKey::new(1, 2000) });

        s = "09-30-2000";
        d = DayDate::parse(s).unwrap();
        assert_eq!(d, DayDate { num: 30, date_key: DateKey::new(9, 2000) });

        s = "09-30-xxx";
        let e = DayDate::parse(s).err().unwrap();
        assert_eq!(e.details, ERR_INVALID_DATE_STR);
    }
}