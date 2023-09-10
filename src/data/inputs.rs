
use std::collections::HashMap;

use crate::indic::ComputerMode;

use super::get_config;

pub struct UserInput {
    pub inputed: Option<f64>,
    pub computed: Option<f64>,
    pub author: String
}

pub struct InputContext {
    pub id: isize,
    pub configuration: Box<HashMap<&'static isize, ComputerMode>>
}

impl InputContext {
    pub fn build(id: isize) -> InputContext {
        InputContext {
            id,
            configuration: Box::new(get_config())
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{data::{mock, build_inputs, get_config}, indic::{SALES_CODE, CASH_CODE}};

//     use super::*;

//     #[test]
//     fn compute_from_works() {
//         let mut v: Vec<IndicatorInput> = vec![];
//         let mut y: Vec<FiscalYear> = vec![];
//         let key = ComputeKey { span: Some(&FY), date: DateKey::build(1, 1) };
//         compute_by_key(&mut v, &mut y, &InputContext::build(1), &key);
//         assert!(v.iter().filter(|i| i.key.span == Some(&FY)).count() > 0);
//     }

//     #[test]
//     #[should_panic(expected = "Indicator inputs list is empty")]
//     fn compute_from_empty() {
//         let mut v: Vec<IndicatorInput> = vec![];
//         let mut y: Vec<FiscalYear> = vec![];
//         let key = ComputeKey { span: Some(&FY), date: DateKey::build(1, 1) };
//         match compute_by_key(&mut v, &mut y, &InputContext::build(1), &key) {
//             Err(e) => panic!("{e}"),
//             _ => {}
//         };
//     }

//     #[test]
//     fn extract_inputs_fy() {
//         let data_inputs = vec![
//             mock::build_span_input(&SALES_CODE, 12, 2023, Some(&FY)),
//             mock::build_span_input(&SALES_CODE, 3, 2023, Some(&SLC)),
//             mock::build_month_input(&SALES_CODE, 3, 2023),
//             mock::build_month_input(&SALES_CODE, 2, 2023),
//             mock::build_span_input(&CASH_CODE, 12, 2023, Some(&FY)),
//             mock::build_span_input(&CASH_CODE, 3, 2023, Some(&SLC)),
//             mock::build_month_input(&CASH_CODE, 3, 2023),
//             mock::build_month_input(&CASH_CODE, 2, 2023),
//             mock::build_span_input(&CASH_CODE, 12, 2024, Some(&FY)),
//             mock::build_span_input(&CASH_CODE, 3, 2024, Some(&SLC)),
//             mock::build_month_input(&CASH_CODE, 3, 2024),
//             mock::build_month_input(&CASH_CODE, 2, 2024)
//         ];
//         let mut inputs = build_inputs(data_inputs);
//         let slice = (1..13).into_iter().map(|m| DateKey::build(m as u8, 2023)).collect();
//         let extracted_inputs = extract_inputs(&mut inputs, slice, Some(&FY), &vec![Some(&SLC)]);
//         assert!(extracted_inputs.len() == 4);
//         assert!(!extracted_inputs.iter().any(|i| i.key.span == None));
//         assert_eq!(extracted_inputs.iter().filter(|i| i.key.span == Some(&FY)).count(), 2);
//         assert_eq!(extracted_inputs.iter().filter(|i| i.key.span == Some(&SLC)).count(), 2);
//     }
// }