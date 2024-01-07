use std::ops::{Range, RangeInclusive};
use std::rc::Rc;

use crate::date::DateKey;
use crate::fiscalyear::FiscalYear;
use crate::indic::{SALES_CODE, CASH_CODE, SLC, NET_DEBT_CODE, FY, EBITDA_CODE, EBITA_CODE};
use crate::data::IndicatorInputData;

pub fn fake_context(default_month: u8, all_years: Vec<i32>, initial_month: Option<u8>) {
   
}

pub fn fake_years<'y>() -> Vec<FiscalYear> {
    const START: u8 = 3;
    const DELAY: u8 = 8;
    let mut year = 2019;
    let mut years: Vec<FiscalYear> = vec![];
    
    let mut rg = START..=DELAY;
    let y: Vec<DateKey> = rg.into_iter().map(|x| DateKey::new(x, year)).collect();
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
            DateKey::new( m as u8, year)
        }).collect();
        years.push(FiscalYear::build(Rc::new(y)));
    }

    years
}

pub fn indicator_data() -> Vec<IndicatorInputData> {
    let codes = [&SALES_CODE, &EBITDA_CODE, &EBITA_CODE, &CASH_CODE, &NET_DEBT_CODE];
    let mut data: Vec<IndicatorInputData> = vec![];
    let mut year;
    for c in codes {
        year = 2019;
        for m in 3..=8 {
            data.push(build_month_input(c, m, year));
            if m == 5 || m == 8 {
                data.push(build_span_input(c, m, year, Some(&SLC)));
            }
        }
        data.push(build_span_input(c, 8, year, Some(&FY)));
        for m in 9..=12 {
            data.push(build_month_input(c, m, year));
            if m == 11 {
                data.push(build_span_input(c, m, year, Some(&SLC)));
            }
        }
        year += 1;
        for m in 1..=8 {
            data.push(build_month_input(c, m, year));
            if m == 2 || m == 5 || m == 8 {
                data.push(build_span_input(c, m, year, Some(&SLC)));
            }
        }
        data.push(build_span_input(c, 8, year, Some(&FY)));
        for m in 9..=12 {
            data.push(build_month_input(c, m, year));
            if m == 11 {
                data.push(build_span_input(c, m, year, Some(&SLC)));
            }
        }
        year += 1;
        for m in 1..=8 {
            data.push(build_month_input(c, m, year));
            if m == 2 || m == 5 || m == 8 {
                data.push(build_span_input(c, m, year, Some(&SLC)));
            }
        }
        data.push(build_span_input(c, 8, year, Some(&FY)));
    }
    
    data
}

pub fn build_month_input(code: &'static isize, month: u8, year: i32) -> IndicatorInputData {
    let float: Option<f64> = rand::random();
    let mut f: f32 = rand::random();
    f *= 10f32;
    let int: u32 = 10u32.pow(f.abs().trunc() as u32);
    let val: Option<f64>;
    match float {
        Some(f) => {
            val = Some(f * int as f64);
        },
        None => {
            val = None;
        }
    };
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

pub fn build_span_input(code: &'static isize, month: u8, year: i32, span: Option<&'static str>) -> IndicatorInputData {
    IndicatorInputData {
        code,
        context: 1, 
        author: "Nobody".to_string(),
        span: span,
        month,
        year,
        computed: None,
        inputed: None
    }
}