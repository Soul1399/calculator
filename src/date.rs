use std::error::Error;

#[derive(Eq, Clone)]
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
        if n % 12 == 0 {
            self.year += n / 12;
        }
        else {
            self.year += f32::trunc((n / 12) as f32) as i32;
            let m = (self.month as i32) + n % 12;
            if m < 1 {
                self.month = 12 + m as u8;
                self.year -= 1;
            }
            else if m < 12 {
                self.month = m as u8 - 12;
                self.year += 1;
            }
        }
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
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.year.partial_cmp(&other.year) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.month.partial_cmp(&other.month)
    }

    fn lt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(std::cmp::Ordering::Less))
    }

    fn le(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(core::cmp::Ordering::Less | core::cmp::Ordering::Equal))
    }

    fn gt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(std::cmp::Ordering::Greater))
    }

    fn ge(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(core::cmp::Ordering::Greater | core::cmp::Ordering::Equal))
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

