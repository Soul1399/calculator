use crate::{ComputeKey, IndicatorInput, fiscalyear::FiscalYear, FY, date::DateKey};


pub fn compute_from(inputs: &mut Vec<IndicatorInput>, y: &Vec<FiscalYear>, key: &ComputeKey) -> Result<(), &'static str> {
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
        FY => compute_fullyear(inputs, &y, &key.date),
        _ => Err("Unknown span")
    }
}

fn compute_fullyear(inputs: &mut Vec<IndicatorInput>, y: &Vec<FiscalYear>, date: &DateKey) -> Result<(), &'static str> {
    let fy = FiscalYear::find(y, date);
    match fy {
        None => return Err("Date was not found in any fiscal years"),
        _ => {}
    }
    

    Ok(())
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
        compute_from(&mut v, &y, &key);
        assert!(v.iter().filter(|i| i.key.span == Some(&FY)).count() > 0);
    }

    #[test]
    #[should_panic(expected = "Indicator inputs list is empty")]
    fn compute_from_empty() {
        let mut v: Vec<IndicatorInput> = vec![];
        let mut y: Vec<FiscalYear> = vec![];
        let key = ComputeKey { span: Some(&FY), date: DateKey::build(1, 1) };
        match compute_from(&mut v, &y, &key) {
            Err(e) => panic!("{e}"),
            _ => {}
        };
    }
}