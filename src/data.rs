use crate::{ComputedIndicator, Indicator, SALES_CODE, ComputeMode, NET_DEBT_CODE, CASH_CODE, EBITA_CODE, EBITDA_CODE, IndicatorInput, UserInput, ComputeKey, DateKey, fiscalyear::FiscalYear};
use std::{collections::HashMap, rc::Rc, cell::RefCell};

use self::mock::indicator_data;

pub fn get_all_inputs() -> Vec<IndicatorInput> {
    build_inputs(indicator_data())
}

pub fn load_context<'y>(context_id: isize) -> Vec<FiscalYear> {
    mock::fake_context()
}

fn build_inputs(data: Vec<IndicatorInputData>) -> Vec<IndicatorInput> {
    data.iter().map(|input| {
        build_input(input)
    }).collect()
}

fn build_input(input: &IndicatorInputData) -> IndicatorInput {
    let dt = DateKey::build(input.month, input.year);
    IndicatorInput {
        context: input.context,
        code: input.code,
        input: RefCell::new(UserInput { inputed: input.inputed, computed: input.computed, author: input.author.to_string() }),
        key: Rc::new(ComputeKey { date: dt, span: input.span })
    }
}

pub fn get_config() -> HashMap<&'static isize, ComputeMode> {
    let mut config = HashMap::new();
    config.insert(&SALES_CODE, ComputeMode::AddUp);
    config.insert(&EBITDA_CODE, ComputeMode::AddUp);
    config.insert(&EBITA_CODE, ComputeMode::AddUp);
    config.insert(&CASH_CODE, ComputeMode::Default);
    config.insert(&NET_DEBT_CODE, ComputeMode::Default);
    config
}

pub struct IndicatorInputData {
    pub code: &'static isize,
    pub context: isize,
    pub span: Option<&'static str>,
    pub month: u8,
    pub year: i32,
    pub inputed: Option<f64>,
    pub computed: Option<f64>,
    pub author: String
}

pub mod inputs;
pub mod mock;