use std::{rc::Rc, cell::RefCell, collections::HashMap};

use enum_iterator::Sequence;
use fsum::FSum;
use strum_macros::FromRepr;

use crate::{Descriptive, ComputeError, data::inputs::UserInput, ComputeKey};

pub const SALES_CODE: isize = 37;
pub const EBITDA_CODE: isize = 48;
pub const EBITA_CODE: isize = 50;
pub const CASH_CODE: isize = 25;
pub const NET_DEBT_CODE: isize = 30;

pub const FY: &str = "Full Year";
pub const LTM: &str = "Last Twelve Months";
pub const SLC: &str = "Slice";
pub const YTD: &str = "Year To Date";

#[repr(isize)]
#[derive(Debug, PartialEq, Sequence, strum_macros::Display, FromRepr)]
pub enum IndicatorName {
    None = 0,
    Sales = SALES_CODE,
    EBITDA = EBITDA_CODE,
    EBITA = EBITA_CODE,
    Cash = CASH_CODE,
    #[strum(serialize="Net Debt")]
    NetDebt = NET_DEBT_CODE
}

impl Default for IndicatorName {
    fn default() -> Self {
        IndicatorName::None
    }
}

// TECHNIQUE TO GET repr of isize
// impl IndicatorName {
//     fn discriminant(&self) -> isize {
//         unsafe { *(self as *const Self as *const isize) }
//     }
// }


#[derive(Debug, Clone)]
pub enum ComputerMode {
    Default,
    AddUp,
    Avg,
    Complex(fn(Vec<ComputeItem>) -> f64)
}

pub struct ComputeItem {
    pub code: String,
    pub value: Box<f64>
}

impl ComputerMode {
    pub fn compute(&self, inputs: &Vec<Box<f64>>) -> Result<f64, ComputeError> {
        let length = inputs.len();
        if length == 0 {
            return Err(ComputeError { details: String::new() });
        }

        let values = inputs.iter()
            .map(|f| f.as_ref());
        
        match self {
            Self::Default => Ok(*values.last().unwrap()),
            Self::AddUp => Ok(FSum::new().add_all(values).value()),
            Self::Avg => { Ok(FSum::new().add_all(values).value() / length as f64) },
            Self::Complex(_) => Err(ComputeError { details: "Not yet implemented".to_string() })
        }
    }
}

#[derive(Debug)]
pub struct BaseIndicator {
    code: &'static isize,
}

impl Default for BaseIndicator {
    fn default() -> Self {
        Self { code: &SALES_CODE }
    }
}

impl Descriptive for BaseIndicator {
    fn default_name(&self) -> String {
        let i = IndicatorName::from_repr(*self.code);
        i.unwrap_or_default().to_string()
    }

    fn name(&self) -> String {
        self.default_name()
    }
}

#[derive(Debug)]
pub struct Indicator {
    context: isize,
    base: BaseIndicator,
}

impl Default for Indicator {
    fn default() -> Self {
        Self { context: Default::default(), base: Default::default() }
    }
}

impl Indicator {
    pub fn build(context: isize, code: &'static isize) -> Self {
        Indicator { context, base: BaseIndicator { code } } 
    }

    pub fn get_code(&self) -> &isize {
        self.base.code
    }
}

impl Descriptive for Indicator {
    fn default_name(&self) -> String {
        self.base.default_name()
    }

    fn name(&self) -> String {
        self.base.name()
    }
}

pub struct IndicatorInput {
    pub input: RefCell<UserInput>,
    pub ltm: RefCell<Option<f64>>,
    pub code: &'static isize,
    pub context: isize,
    pub key: Rc<ComputeKey>
}

impl IndicatorInput {
    pub fn get_indicator(&self) -> Indicator {
        Indicator::build(self.context, self.code)
    }

    pub fn get_computer<'a>(&self, conf: &'a HashMap<&'static isize, ComputerMode>) -> &'a ComputerMode {
        match conf.get(self.code) {
            Some(c) => c,
            None => panic!("Input was undefined")
        }
    }

    pub fn get_value(&self) -> Option<f64> {
        if let Some(_) = self.input.borrow().inputed {
            return self.input.borrow().inputed;
        }
        if let Some(_) = self.input.borrow().computed {
            return self.input.borrow().computed;
        }
        None
    }
}