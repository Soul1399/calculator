use std::rc::Rc;

use crate::date::DateKey;
use crate::fiscalyear::FiscalYear;
use crate::{SALES_CODE, CASH_CODE, SLC, NET_DEBT_CODE};
use crate::data::IndicatorInputData;


pub fn fake_context<'y>() -> Vec<FiscalYear> {
    const START: u8 = 3;
    const DELAY: u8 = 8;
    let mut year = 2019;
    let mut years: Vec<FiscalYear> = vec![];
    
    let mut rg = START..=DELAY;
    let y: Vec<DateKey> = rg.into_iter().map(|x| DateKey::build(x, year)).collect();
    years.push(FiscalYear::build(Rc::new(y)));
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
        years.push(FiscalYear::build(Rc::new(y)));
    }

    years
}

pub fn indicator_data() -> Vec<IndicatorInputData> {
    let codes = [&SALES_CODE, &CASH_CODE, &NET_DEBT_CODE];
    let mut data: Vec<IndicatorInputData> = vec![];
    let mut year = 0;
    for c in codes {
        year = 2019;
        for m in 3..=8 {
            data.push(build_month_input(c, m, year));
            if m == 5 || m == 8 {
                data.push(build_slice_input(c, m, year));
            }
        }
        for m in 9..=12 {
            data.push(build_month_input(c, m, year));
            if m == 11 {
                data.push(build_slice_input(c, m, year));
            }
        }
        year += 1;
        for m in 1..=8 {
            data.push(build_month_input(c, m, year));
            if m == 2 || m == 5 || m == 8 {
                data.push(build_slice_input(c, m, year));
            }
        }
        for m in 9..=12 {
            data.push(build_month_input(c, m, year));
            if m == 11 {
                data.push(build_slice_input(c, m, year));
            }
        }
        year += 1;
        for m in 1..=8 {
            data.push(build_month_input(c, m, year));
            if m == 2 || m == 5 || m == 8 {
                data.push(build_slice_input(c, m, year));
            }
        }
    }
    
    data
}

fn build_month_input(code: &'static isize, month: u8, year: i32) -> IndicatorInputData {
    let mut float: f64 = rand::random();
    let int: i32 = rand::random();
    float /= 1000.00;
    let val = Some(float * int as f64);
    IndicatorInputData {
        code,
        context: 1, 
        author: "Nobody".to_string(),
        span: None,
        month,
        year,
        computed: None,
        inputed: val
    }
}

fn build_slice_input(code: &'static isize, month: u8, year: i32) -> IndicatorInputData {
    IndicatorInputData {
        code,
        context: 1, 
        author: "Nobody".to_string(),
        span: Some(&SLC),
        month,
        year,
        computed: None,
        inputed: None
    }
}