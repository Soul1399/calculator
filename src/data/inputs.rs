use crate::{ComputeKey, IndicatorInput};


pub fn compute_from(inputs: &mut Vec<IndicatorInput>, key: &ComputeKey) -> Result<(), &'static str> {
    if inputs.len() == 0 {
        return Err("Indicator inputs list is empty");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{ComputeKey, FY, IndicatorInput, date::DateKey};
    use super::compute_from;

    #[test]
    //#[should_panic(expected = "Indicator inputs list is empty")]
    fn compute_from_works() {
        let mut v: Vec<IndicatorInput> = vec![];
        let key = ComputeKey { class: Some(&FY), date: DateKey::build(1, 1) };
        compute_from(&mut v, &key);
        assert!(v.len() > 0);
    }
}