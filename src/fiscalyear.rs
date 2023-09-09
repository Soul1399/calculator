use std::{collections::HashMap, rc::Rc};

use crate::{date::DateKey, ComputeKey, indic::FY};


pub struct FiscalYear {
    slices: HashMap<u8, Vec<DateKey>>,
    months: Rc<Vec<DateKey>>
}

impl FiscalYear {
    pub fn build(mths: Rc<Vec<DateKey>>) -> FiscalYear {
        FiscalYear { slices: HashMap::new(), months: mths }
    }
    pub fn get_months(&self) -> Vec<DateKey> {
        self.months.as_ref().to_vec()
    }
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
    pub fn get_slice(&self, pos: u8) -> Result<Vec<DateKey>, &'static str> {
        if pos < 1 {
            return Err("Invalid position: expected a position greater than zero");
        }
        
        match self.slices.get(&pos) {
            Some(d) => Ok(d.to_vec()),
            None => Err("Invalid position")
        }
    }
    pub fn find_slice(&self, date: &DateKey) -> Result<Vec<DateKey>, &'static str> {
        let slice = self.slices.iter()
            .filter(|x| x.1.iter().any(|d| d == date))
            .map(|x| x.1)
            .next();

        match slice {
            Some(x) => Ok(x.to_vec()),
            None => Err("Slice not found")
        }
    }

    pub fn find<'a>(v: &'a mut Vec<Self>, d: &DateKey) -> Result<&'a mut FiscalYear, Result<(), &'static str>> {
        let _y = v.iter_mut().find(|fy| fy.min() <= Ok(d) && Ok(d) <= fy.max());
        match _y {
            None => return Err(Err("Date was not found in any fiscal years")),
            Some(x) => Ok(x)
        }
    }

    pub fn get_keys(years: &Vec<Self>) -> Vec<ComputeKey> {
        years.iter()
            .map(|y| ComputeKey { date: *y.max().unwrap(), span: Some(&FY) })
            .collect()
    }

    pub fn build_slices(&mut self, size: u8) {
        if self.slices.len() > 0 {
            return;
        }
        let mut v: Vec<DateKey> = self.months.iter().map(|x| *x).collect();
        v.sort();
        let chunks = v.chunks(size as usize);
        let mut x = 0;
        for chunk in chunks {
            x += 1;
            self.slices.insert(x, chunk.to_vec());
        }
    }

    pub fn max_nb_slices() -> u8 { 8 }

    pub fn get_slices(fy: &Self) -> Vec<&Vec<DateKey>> {
        fy.slices.iter().map(|s| s.1).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::date::DateKey;
    use super::*;

    #[test]
    fn slices_1() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        let slice = fy.get_slice(1).unwrap();
        assert_eq!(vec![DateKey::build(1, 2023), DateKey::build(2, 2023), DateKey::build(3, 2023)], slice)
    }

    #[test]
    fn slices_3() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        let slice = fy.get_slice(3).unwrap();
        assert_eq!(vec![DateKey::build(7, 2023), DateKey::build(8, 2023), DateKey::build(9, 2023)], slice)
    }

    #[test]
    fn slice_invalid() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        let slice = fy.get_slice(45);
        assert_eq!(Err("Invalid position"), slice)
    }

    #[test]
    fn no_slices() {
        let months: Vec<DateKey> = vec![];
        let fy = FiscalYear::build(Rc::new(months));
        let slice = fy.get_slice(3);
        assert_eq!(Err("Invalid position"), slice)
    }

    #[test]
    fn fy_min() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        assert_eq!(DateKey::build(1, 2023), *fy.min().unwrap())
    }

    #[test]
    fn fy_max() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        assert_eq!(DateKey::build(12, 2023), *fy.max().unwrap())
    }
}