use std::rc::Rc;

use crate::date::DateKey;
use crate::fiscalyear::FiscalYear;
use crate::{SALES_CODE, CASH_CODE};
use crate::data::IndicatorInputData;


pub fn fake_context() -> Vec<FiscalYear> {
    const START: u8 = 3;
    const DELAY: u8 = 8;
    let mut year = 2019;
    let mut years: Vec<FiscalYear> = vec![];
    
    let mut rg = START..=DELAY;
    let y: Vec<DateKey> = rg.into_iter().map(|x| DateKey::build(x, year)).collect();
    years.push(FiscalYear{months: Rc::new(y)});
    let delay: i8 = DELAY as i8 - 12;
    for _ in 0..2 {
        rg = 1..=12;
        let y: Vec<DateKey> = rg.into_iter().map(|x| {
            let mut m: i8 = (x as i8) + delay;
            if m <= 0 {
                m += 12;
            }
            else if m == 1 {
                year += 1;
            }
            DateKey::build( m as u8, year)
        }).collect();
        years.push(FiscalYear{months: Rc::new(y)});
    }

    years
}

pub fn indicator_data() -> Vec<IndicatorInputData> {
    let data = vec![
        IndicatorInputData {
            code: &SALES_CODE,
            context: 1, 
            author: "John Smille".to_string(),
            span: None,
            month: 1,
            year: 2020,
            computed: None,
            inputed: Some(234.665)
        },
        IndicatorInputData {
            code: &SALES_CODE,
            context: 1, 
            author: "John Smille".to_string(),
            span: None,
            month: 2,
            year: 2020,
            computed: None,
            inputed: None
        },
        IndicatorInputData {
            code: &SALES_CODE,
            context: 1, 
            author: "John Smille".to_string(),
            span: None,
            month: 5,
            year: 2020,
            computed: None,
            inputed: Some(34.0)
        },
        IndicatorInputData {
            code: &CASH_CODE,
            context: 1, 
            author: "John Smille".to_string(),
            span: None,
            month: 1,
            year: 2020,
            computed: None,
            inputed: Some(1.88)
         },
         IndicatorInputData {
            code: &CASH_CODE,
            context: 1, 
            author: "John Smille".to_string(),
            span: None,
            month: 9,
            year: 2020,
            computed: None,
            inputed: Some(3.81)
         }];
    data
}