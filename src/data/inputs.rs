
use std::{collections::HashMap, rc::Rc, cell::RefCell, borrow::BorrowMut};

use crate::{ComputeKey, IndicatorInput, fiscalyear::FiscalYear, FY, date::DateKey, SLC, UserInput};

pub fn compute_by_key(inputs: &mut Vec<IndicatorInput>, y: &mut Vec<FiscalYear>, key: &ComputeKey) -> Result<(), &'static str> {
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
    
    if let Some(value) = prepare_fy(y, key) {
        return value;
    }

    let fy = match FiscalYear::find(y, &key.date) {
        Ok(value) => value,
        Err(value) => return value
    };

    match span {
        FY => compute_fullyear(inputs, fy),
        SLC => compute_slice(inputs, fy, &key.date),
        _ => Err("Unknown span")
    }
}

fn prepare_fy(y: &mut Vec<FiscalYear>, key: &ComputeKey) -> Option<Result<(), &'static str>> {
    let fy: &mut FiscalYear;
    match FiscalYear::find(y, &key.date) {
        Ok(value) => {
            fy = value;
        },
        Err(value) => {
            return Some(value);
        }
    }
    fy.build_slices(3);

    None
}

fn compute_fullyear(inputs: &mut Vec<IndicatorInput>, fy: &FiscalYear) -> Result<(), &'static str> {
    for s in FiscalYear::get_slices(fy) {
        if let Err(e) = compute_slice(inputs, fy, s.first().unwrap()) {
            return Err(e);
        }
    }

    Ok(())
}

fn compute_slice(inputs: &mut Vec<IndicatorInput>, fy: &FiscalYear, date: &DateKey) -> Result<(), &'static str> {
    let slice: Vec<DateKey>;
    match fy.find_slice(date, None) {
        Ok(s) => slice = s,
        Err(e) => return Err(e)
    };
    
    let max_of_slice = *slice.iter().max().unwrap();
    let mut slice_inputs = extract_slice_inputs(inputs, slice);
    // emulate group by
    let config = crate::data::get_config();
    let mut keys:Vec<&'static isize> = slice_inputs.iter().map(|i| i.code).collect();
    while keys.len() > 0 {
        // get group key
        let _k = keys.first();
        let key: &'static isize;
        match _k {
            Some(x) => key = x,
            None => break
        }
        if let Some(value) = compute_slice_of_indicator(&slice_inputs, key, &config, max_of_slice) {
            return value;
        }
        // keep other inputs
        slice_inputs.retain(|i| i.code != key);
        // keep other keys (even if they appear multiple times)
        keys.retain(|x| *x != key);
    }

    Ok(())
}

fn compute_slice_of_indicator(slice_inputs: &Vec<&mut IndicatorInput>, key: &'static isize, config: &HashMap<&'static isize, crate::ComputeMode>, max_of_slice: DateKey) -> Option<Result<(), &'static str>> {
    let indic_inputs: Vec<&&mut IndicatorInput> = slice_inputs.iter()
        .filter(|i| i.code == key)
        .collect();
    if indic_inputs.len() == 0 {
        return None;
    }
    let computer = indic_inputs.first().unwrap().info(config);
    let mut slice_input = indic_inputs
        .iter()
        .filter(|&&i| i.key.span == Some(&SLC))
        .next();
    match slice_input {
        None => {
            return Some(Err("Missing main slice input"));
        },
        _ => {}
    }
    let input_values = extract_values(&indic_inputs);
    println!("\nComputing {} in {}", key, max_of_slice.to_string());
    match computer.compute(&input_values) {
        Ok(x) => {
            if slice_input.is_none() {
                slice_input = Some(&&&mut IndicatorInput {
                    code: key, 
                    input: RefCell::new(UserInput { author: String::new(), computed: None, inputed: None }),
                    context: 1,
                    key: Rc::new(ComputeKey { span: Some(&SLC), date: max_of_slice })
                });
            }
            else {
                slice_input.map(|val| val.input.borrow_mut().computed = Some(x));
            }
        },
        Err(e) => {
            if e.details.len() > 0 {
                println!("{}", e.details);
                return Some(Err("Compute failed"));
            }
            else {
                slice_input.map(|val| val.input.borrow_mut().computed = None);
            }
        }
    }
    None
}

fn extract_values(indic_inputs: &Vec<&&mut IndicatorInput>) -> Vec<Rc<f64>> {
    let mut input_values: Vec<Rc<f64>> = Vec::new();
    indic_inputs
        .iter()
        .filter(|&&i| i.key.span == None)
        .for_each(|&i| {
            let mut o = i.input.borrow().inputed;
            if o == None {
                o = i.input.borrow().computed;
            }
            match o {
                Some(f) => input_values.push(Rc::new(f)),
                None => {}
            }
        });
    input_values
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
        compute_by_key(&mut v, &mut y, &key);
        assert!(v.iter().filter(|i| i.key.span == Some(&FY)).count() > 0);
    }

    #[test]
    #[should_panic(expected = "Indicator inputs list is empty")]
    fn compute_from_empty() {
        let mut v: Vec<IndicatorInput> = vec![];
        let mut y: Vec<FiscalYear> = vec![];
        let key = ComputeKey { span: Some(&FY), date: DateKey::build(1, 1) };
        match compute_by_key(&mut v, &mut y, &key) {
            Err(e) => panic!("{e}"),
            _ => {}
        };
    }
}