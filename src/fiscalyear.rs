use std::{rc::Rc, fmt::format};

use crate::date::DateKey;

pub struct FiscalYear {
    pub months: Rc<Vec<DateKey>>
}

impl FiscalYear {
    pub fn min(&self) -> Result<&DateKey, &'static str> {
        let mut v: Vec<&DateKey> = self.months.iter().collect();
        v.sort();
        match v.first() {
            Some(m) => Ok(*m),
            None => Err("Empty year")
        }
    }
    pub fn max(&self) -> Result<&DateKey, &'static str> {
        let mut v: Vec<&DateKey> = self.months.iter().collect();
        v.sort();
        match v.last() {
            Some(m) => Ok(*m),
            None => Err("Empty year")
        }
    }
    pub fn get_quarter(&self, pos: u8) -> Result<&DateKey, &'static str> {
        if pos < 1 {
            return Err("Invalid position: expected a position between 1 and 8");
        }
        if pos > 8 {
            return Err("Invalid position: cannot have more than 8 quarters");
        }
        let mut v: Vec<&DateKey> = self.months.iter().collect();
        v.sort();
        let mut sorted_v = v.iter();
        if pos > 1 {
            for p in 1..pos {
                for _ in 1..=3 {
                    match sorted_v.next() {
                        None => return Err("Invalid position: connot find quarter"),
                        _ => {}
                    }
                }
            }
        }
        let mut temp: Option<&DateKey> = None;
        for _ in 1..=3 {
            match sorted_v.next() {
                Some(x) => temp = Some(*x),
                None => break
            }
        }
        match temp {
            Some(v) => Ok(v),
            None => Err("Invalid position: connot find quarter")
        }
    }

    pub fn find<'a>(v: &'a Vec<Self>, d: &'a DateKey) -> Option<&'a FiscalYear> {
        v.iter().find(|fy| fy.min() <= Ok(d) && Ok(d) <= fy.max())
    }
}