use crate::{ComputedIndicator, Indicator, SALES_CODE, ComputeMode, NET_DEBT_CODE, CASH_CODE, EBITA_CODE, EBITDA_CODE, IndicatorInput, UserInput, ComputeKey, DateKey, fiscalyear::FiscalYear};
use std::{collections::HashMap, rc::Rc, cell::RefCell};

use self::mock::indicator_data;

pub fn get_indicators() -> Vec<IndicatorInput> {
    build_inputs(indicator_data(), get_config())
}

pub fn load_context<'y>(context_id: isize) -> Vec<FiscalYear> {
    mock::fake_context()
}

fn build_inputs(data: Vec<IndicatorInputData>, conf: HashMap<isize, ComputeMode>) -> Vec<IndicatorInput> {
    data.iter().map(|input| {
        build_input(&conf, input)
    }).collect()
}

fn build_input(conf: &HashMap<isize, ComputeMode>, input: &IndicatorInputData) -> IndicatorInput {
    let dt = DateKey::build(input.month, input.year);
    IndicatorInput {
        indic: match conf.get(input.code) {
            Some(ComputeMode::AddUp) => ComputedIndicator::AddUp(Rc::new(Indicator::build(input.context, *input.code))),
            Some(ComputeMode::Default) => ComputedIndicator::Default(Rc::new(Indicator::build(input.context, *input.code))),
            Some(ComputeMode::Avg) => ComputedIndicator::Avg(Rc::new(Indicator::build(input.context, *input.code))),
            Some(ComputeMode::Complex) => ComputedIndicator::Complex(Rc::new(Indicator::build(input.context, *input.code))),
            None => panic!("Input was undefined")
        },
        input: RefCell::new(UserInput { inputed: input.inputed, computed: input.computed, author: input.author.to_string() }),
        key: Rc::new(ComputeKey { date: dt, span: input.span })
    }
}

fn get_config() -> HashMap<isize, ComputeMode> {
    let mut config = HashMap::new();
    config.insert(SALES_CODE, ComputeMode::AddUp);
    config.insert(EBITDA_CODE, ComputeMode::AddUp);
    config.insert(EBITA_CODE, ComputeMode::AddUp);
    config.insert(CASH_CODE, ComputeMode::Default);
    config.insert(NET_DEBT_CODE, ComputeMode::Default);
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