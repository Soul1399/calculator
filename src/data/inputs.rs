
use std::collections::HashMap;

use crate::{ComputeKey, IndicatorInput, fiscalyear::FiscalYear, FY, date::DateKey, SLC, ComputedIndicator, Indicator};

pub fn compute_from(inputs: &mut Vec<IndicatorInput>, y: &mut Vec<FiscalYear>, key: &ComputeKey) -> Result<(), &'static str> {
    if inputs.len() == 0 {
        return Err("Indicator inputs list is empty");
    }
    if y.len() == 0 {
        return Err("There was no available fiscal years");
    }
    let span;
    match key.span {
        Some(c) => span = c,
        None => return Err("Invalid span")
    };

    match span {
        FY => compute_fullyear(inputs, y, &key.date),
        SLC => compute_slice(inputs, y, &key.date),
        _ => Err("Unknown span")
    }
}

fn compute_fullyear(inputs: &mut Vec<IndicatorInput>, y: &mut Vec<FiscalYear>, date: &DateKey) -> Result<(), &'static str> {
    let fy = match FiscalYear::find(y, date) {
        Ok(value) => value,
        Err(value) => return value,
    };
    
    let mut fy_inputs: Vec<&mut IndicatorInput> = inputs.into_iter()
        .filter(|i| *fy.min().unwrap() <= i.key.date && i.key.date <= *fy.max().unwrap())
        .collect();


    Ok(())
}

fn compute_slice(inputs: &mut Vec<IndicatorInput>, y: &mut Vec<FiscalYear>, date: &DateKey) -> Result<(), &'static str> {
    let fy = match FiscalYear::find(y, date) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let slice: Vec<DateKey>;
    match fy.find_slice(date, None) {
        Ok(s) => slice = s,
        Err(e) => return Err(e)
    };
    let mut slice_inputs = extract_slice_inputs(inputs, slice);
    
    let config = crate::data::get_config();
    let mut keys:Vec<&isize> = slice_inputs.iter().map(|i| i.code).collect();
    while keys.len() > 0 {
        let _k = keys.first();
        let key: &isize;
        match _k {
            Some(x) => key = x,
            None => break
        }
        let mut indic_inputs: Vec<&&mut IndicatorInput> = slice_inputs.iter()
            .filter(|i| i.code == key)
            .collect();
        let computer = indic_inputs.first().unwrap().info(&config);
        //computer.compute(inputs)
        slice_inputs.retain(|i| i.code == key);
        keys.retain(|x| *x != key);
    }

    Ok(())
}

fn extract_slice_inputs(inputs: &mut Vec<IndicatorInput>, slice: Vec<DateKey>) -> Vec<&mut IndicatorInput> {
    let mut slice_inputs: Vec<_> = inputs.iter_mut()
        .filter(|i| i.key.span == Some(&SLC) || i.key.span == None)
        .filter(|i| slice.iter().any(|d| i.key.date == *d))
        .collect();

    slice_inputs
}

#[cfg(test)]
mod tests {
    use crate::{ComputeKey, FY, IndicatorInput, date::DateKey, fiscalyear::FiscalYear};
    use super::*;

    #[test]
    fn compute_from_works() {
        let mut v: Vec<IndicatorInput> = vec![];
        let mut y: Vec<FiscalYear> = vec![];
        let key = ComputeKey { span: Some(&FY), date: DateKey::build(1, 1) };
        compute_from(&mut v, &mut y, &key);
        assert!(v.iter().filter(|i| i.key.span == Some(&FY)).count() > 0);
    }

    #[test]
    #[should_panic(expected = "Indicator inputs list is empty")]
    fn compute_from_empty() {
        let mut v: Vec<IndicatorInput> = vec![];
        let mut y: Vec<FiscalYear> = vec![];
        let key = ComputeKey { span: Some(&FY), date: DateKey::build(1, 1) };
        match compute_from(&mut v, &mut y, &key) {
            Err(e) => panic!("{e}"),
            _ => {}
        };
    }
}