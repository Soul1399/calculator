use std::{rc::Rc , collections::HashMap};

use crate::date::DateKey;

pub struct FiscalYear {
    slices: HashMap<u8, Vec<DateKey>>,
    pub months: Rc<Vec<DateKey>>
}

impl FiscalYear {
    pub fn build(mths: Rc<Vec<DateKey>>) -> FiscalYear {
        FiscalYear { slices: HashMap::new(), months: mths }
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
    pub fn get_slice(&mut self, pos: u8, size: Option<u8>) -> Result<Vec<DateKey>, &'static str> {
        if pos < 1 {
            return Err("Invalid position: expected a position greater than zero");
        }
        self.build_slices(size.unwrap_or(3));
        match self.slices.get(&pos) {
            Some(d) => Ok(d.to_vec()),
            None => Err("Invalid position")
        }
    }

    pub fn find<'a>(v: &'a Vec<Self>, d: &'a DateKey) -> Option<&'a FiscalYear> {
        v.iter().find(|fy| fy.min() <= Ok(d) && Ok(d) <= fy.max())
    }

    fn build_slices(&mut self, size: u8) {
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
}

#[cfg(test)]
mod tests {
    use crate::date::DateKey;
    use super::*;

    #[test]
    fn slices_1() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        let slice = fy.get_slice(1, None).unwrap();
        assert_eq!(vec![DateKey::build(1, 2023), DateKey::build(2, 2023), DateKey::build(3, 2023)], slice)
    }

    #[test]
    fn slices_3() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        let slice = fy.get_slice(3, None).unwrap();
        assert_eq!(vec![DateKey::build(7, 2023), DateKey::build(8, 2023), DateKey::build(9, 2023)], slice)
    }

    #[test]
    fn slice_invalid() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        let slice = fy.get_slice(45, None);
        assert_eq!(Err("Invalid position"), slice)
    }

    #[test]
    fn no_slices() {
        let months: Vec<DateKey> = vec![];
        let mut fy = FiscalYear::build(Rc::new(months));
        let slice = fy.get_slice(3, None);
        assert_eq!(Err("Invalid position"), slice)
    }

    #[test]
    fn fy_min() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        assert_eq!(DateKey::build(1, 2023), *fy.min().unwrap())
    }

    #[test]
    fn fy_max() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        assert_eq!(DateKey::build(12, 2023), *fy.max().unwrap())
    }
}