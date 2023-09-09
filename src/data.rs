use std::{collections::HashMap, rc::Rc, cell::RefCell};

use crate::{indic::{IndicatorInput, ComputerMode, SALES_CODE, EBITDA_CODE, EBITA_CODE, CASH_CODE, NET_DEBT_CODE}, fiscalyear::FiscalYear, ComputeKey, date::DateKey};

use self::{mock::indicator_data, inputs::UserInput};

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

pub fn get_config() -> HashMap<&'static isize, ComputerMode> {
    let mut config = HashMap::new();
    config.insert(&SALES_CODE, ComputerMode::AddUp);
    config.insert(&EBITDA_CODE, ComputerMode::AddUp);
    config.insert(&EBITA_CODE, ComputerMode::AddUp);
    config.insert(&CASH_CODE, ComputerMode::Default);
    config.insert(&NET_DEBT_CODE, ComputerMode::Default);
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