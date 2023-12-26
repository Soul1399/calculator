use std::{error::Error, cmp::Ordering};

#[derive(Clone, Copy, Debug, Eq)]
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
        if m < 1 || m > 12 {
            return Err(DateError { details: String::from("Month has to be between 1 and 12") });
        }
        self.month = m;
        self.year = y;
        Ok(())
    }

    pub fn add_months(&mut self, n: i32) {
        if n == 0 {
            return;
        }
        let mut new_m: i32 = 0;
        if let 1..=11 = n.abs() {
            new_m = (self.month as i32) + n;
        }
        else if n % 12 == 0 {
            new_m = self.month as i32;
            self.year += n / 12;
        }
        else {
            self.year += f32::trunc((n / 12) as f32) as i32;
            new_m = (self.month as i32) + n % 12;
        }
        if new_m < 1 {
            new_m = 12 + new_m;
            self.year -= 1;
        }
        else if new_m > 12 {
            new_m = new_m - 12;
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