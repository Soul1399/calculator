use std::{collections::HashMap, rc::Rc};

use crate::{date::DateKey, indic::{FY, LTM}, compute::ComputeKey};


pub struct FiscalYear {
    slices: HashMap<u8, Vec<DateKey>>,
    months: Rc<Vec<DateKey>>
}

impl FiscalYear {
    pub fn build(mths: Rc<Vec<DateKey>>) -> FiscalYear {
        const MAX_MONTHS: u8 = 24;
        if mths.len() > MAX_MONTHS as usize {
            panic!("Fiscal year does not allow having more than {} months", MAX_MONTHS);
        }
        FiscalYear { slices: HashMap::new(), months: mths }
    }
    pub fn get_months(&self) -> Vec<DateKey> {
        self.months.as_ref().to_vec()
    }
    pub fn nb_months(&self) -> u8 {
        self.months.len() as u8
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

    pub fn find_ltm_slice(list: &Vec<Self>, end_date: &DateKey) -> Result<Vec<DateKey>, &'static str> {
        let mut start_date = *end_date;
        start_date.add_months(-12);

        let _s: Vec<_> = list.iter().flat_map(|y| y.slices.iter().map(|x| x.1))
            .filter(|&s| s.iter().any(|d| d == end_date || d == &start_date))
            .collect();

        if _s.len() == 0 {
            return Err("LTM date not found");
        }

        let mut dates: Vec<DateKey> = list.iter()
            .flat_map(|y| y.months.as_ref())
            .map(|m| *m)
            .filter(|m| start_date <= *m && m <= end_date)
            .collect();

        dates.extend(_s.iter().flat_map(|s| *s));

        if dates.len() > 0 {
            dates.sort();
            return Ok(dates);
        }
        Err("Slice not found")
    }

    pub fn find<'a>(v: &'a Vec<Self>, d: &DateKey) -> Result<&'a FiscalYear, Result<(), &'static str>> {
        let _y = v.iter().find(|fy| fy.min() <= Ok(d) && Ok(d) <= fy.max());
        match _y {
            None => return Err(Err("Date was not found in any fiscal years")),
            Some(x) => Ok(x)
        }
    }

    pub fn find_mut<'a>(v: &'a mut Vec<Self>, d: &DateKey) -> Result<&'a mut FiscalYear, Result<(), &'static str>> {
        let _y = v.iter_mut().find(|fy| fy.min() <= Ok(d) && Ok(d) <= fy.max());
        match _y {
            None => return Err(Err("Date was not found in any fiscal years")),
            Some(x) => Ok(x)
        }
    }

    pub fn get_keys(years: &Vec<Self>) -> Vec<ComputeKey> {
        let mut keys: Vec<ComputeKey> = years.iter()
            .map(|y| ComputeKey { date: *y.max().unwrap(), span: Some(&FY) })
            .collect();

        keys.extend(years.iter().flat_map(|y| y.months.as_ref()).map(|m| ComputeKey { date: *m, span: Some(&LTM) }));

        return keys;
    }

    pub fn build_slices(&mut self, size: u8) {
        if self.slices.len() > 0 {
            return;
        }
        if size < 1 {
            panic!("Unable to create slice of size 0");
        }
        if size as usize > self.months.len() {
            panic!("{}", format!("Too big slice size ({size})"));
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
    fn count_months() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        assert_eq!(fy.months.len(), 12);
        let months: Vec<DateKey> = (6..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        assert_eq!(fy.months.len(), 7);
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

    #[test]
    #[should_panic(expected="Date was not found in any fiscal years")]
    fn find_none_before() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        if let Err(Err(e)) = FiscalYear::find(&vec![fy], &DateKey::build(1, 2000)) {
            panic!("{}", e);
        }
    }

    #[test]
    #[should_panic(expected="Date was not found in any fiscal years")]
    fn find_none_after() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        if let Err(Err(e)) = FiscalYear::find(&vec![fy], &DateKey::build(1, 2030)) {
            panic!("{}", e);
        }
    }

    #[test]
    fn find_fy() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        let list = vec![fy];
        let r = FiscalYear::find(&list, &DateKey::build(4, 2023));
        let f = r.unwrap();
        assert_eq!(f.min(), list[0].min());
        assert_eq!(f.max(), list[0].max());
    }

    #[test]
    fn build_keys() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        let keys = FiscalYear::get_keys(&vec![fy]);
        assert!(keys.len() == 13);
        assert!(keys.iter().filter(|x| x.span == Some(&FY)).count() == 1);
        assert!(keys.iter().filter(|x| x.span == Some(&LTM)).count() == 12);
    }

    #[test]
    fn build_keys_multiple() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        let mut list = vec![fy];
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2022)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        list.push(fy);
        let months: Vec<DateKey> = (7..=12).into_iter().map(|m| DateKey::build(m, 2021)).collect();
        let fy = FiscalYear::build(Rc::new(months));
        list.push(fy);
        let keys = FiscalYear::get_keys(&list);
        assert!(keys.len() == 33);
        assert_eq!(keys.iter().filter(|x| x.span == Some(&FY)).count(), 3);
        assert_eq!(keys.iter().filter(|x| x.span == Some(&LTM)).count(), 30);
    }

    #[test]
    #[should_panic(expected="Too big slice size (30)")]
    fn build_slice() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        fy.build_slices(30);

    }

    #[test]
    #[should_panic(expected="Unable to create slice of size 0")]
    fn build_0_slice() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        fy.build_slices(0);
    }

    #[test]
    fn build_n_slices() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        fy.build_slices(3);
        assert_eq!(4, fy.slices.len());
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        fy.build_slices(4);
        assert_eq!(3, fy.slices.len());
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        fy.build_slices(5);
        assert_eq!(3, fy.slices.len());
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        fy.build_slices(2);
        assert_eq!(6, fy.slices.len());
    }

    #[test]
    fn find_no_slice() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        fy.build_slices(3);
        let r = fy.find_slice(&DateKey::build(1, 2020));
        if let Err(e) = r {
            assert_eq!("Slice not found", e);
        }
        panic!("Test failed!");
    }

    #[test]
    fn find_one_slice() {
        let months: Vec<DateKey> = (1..=12).into_iter().map(|m| DateKey::build(m, 2023)).collect();
        let mut fy = FiscalYear::build(Rc::new(months));
        fy.build_slices(3);
        let r = fy.find_slice(&DateKey::build(3, 2023)).unwrap();
        assert_eq!(r.len(), 3);
        assert!(r.iter().any(|d| d.month() == 3 && d.year() == 2023));
        assert!(r.iter().any(|d| d.month() == 2 && d.year() == 2023));
        assert!(r.iter().any(|d| d.month() == 1 && d.year() == 2023));
        assert!(!r.iter().any(|d| d.month() == 4 && d.year() == 2023));
        assert!(!r.iter().any(|d| d.month() == 3 && d.year() == 2021));
    }

}