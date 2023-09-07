
use std::{error::Error, rc::{Rc, Weak}, cell::RefCell, ops::Add, collections::HashMap};
use date::DateKey;
use enum_iterator::{all, Sequence};
use fsum::FSum;

pub const SALES_CODE: isize = 37;
pub const EBITDA_CODE: isize = 48;
pub const EBITA_CODE: isize = 50;
pub const CASH_CODE: isize = 25;
pub const NET_DEBT_CODE: isize = 30;

pub const FY: &str = "Full Year";
pub const LTM: &str = "Last Twelve Months";
pub const SLC: &str = "Slice";

#[repr(isize)]
#[derive(Debug, PartialEq, Sequence)]
pub enum IndicatorName {
    None = 0,
    Sales = SALES_CODE,
    Ebitda = EBITDA_CODE,
    Ebita = EBITA_CODE,
    Cash = CASH_CODE,
    NetDebt = NET_DEBT_CODE
}

impl Default for IndicatorName {
    fn default() -> Self {
        IndicatorName::None
    }
}

impl ToString for IndicatorName {
    fn to_string(&self) -> String {
        match self {
            IndicatorName::Sales => "Sales".to_string(),
            IndicatorName::Ebitda => "EBITDA".to_string(),
            IndicatorName::Ebita => "EBITA".to_string(),
            IndicatorName::Cash => "Cash".to_string(),
            IndicatorName::NetDebt => "Net Debt".to_string(),
            _ => {
                panic!("Impossible value of IndicatorName")
            }
        }
    }
}

impl IndicatorName {
    fn discriminant(&self) -> isize {
        unsafe { *(self as *const Self as *const isize) }
    }
}


impl From<isize> for IndicatorName {
    fn from(value: isize) -> Self {
        for i in all::<IndicatorName>() {
            if i.discriminant() == value {
                return i;
            }
        }
        IndicatorName::default()
    }
}

pub trait Descriptive {
    fn default_name(&self) -> String;
    fn name(&self) -> String;
}

#[repr(isize)]
pub enum ComputeMode {
    Default,
    AddUp,
    Avg,
    Complex
}

pub enum ComputedIndicator<T: Descriptive> {
    Default(Rc<T>),
    AddUp(Rc<T>),
    Avg(Rc<T>),
    Complex(Rc<T>)
}

#[derive(Debug)]
pub struct ComputeError {
    details: String
}

impl Error for ComputeError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl std::fmt::Display for ComputeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl<T: Descriptive> ComputedIndicator<T> {
    pub fn compute(&self, inputs: &Vec<Option<&f64>>) -> Result<f64, ComputeError> {
        let length = inputs.iter()
            .filter(|&f| f.is_some()).count();

        let values = inputs.iter()
            .filter(|&f| f.is_some())
            .map(|&f| f.unwrap());
        
        match self {
            Self::AddUp(x) => Ok(FSum::new().add_all(values).value()),
            Self::Default(x) => Ok(*values.last().unwrap()),
            Self::Avg(x) => { Ok(FSum::new().add_all(values).value() / length as f64) },
            _ => Err(ComputeError { details: "Unable to compute this indicator".to_string() })
        }
    }

    pub fn indicator(&self) -> Option<Rc<T>> {
        match self {
            Self::Default(x) => Some(Rc::clone(x)),
            Self::AddUp(x) => Some(Rc::clone(x)),
            Self::Avg(x) => Some(Rc::clone(x)),
            Self::Complex(x) => Some(Rc::clone(x)),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct BaseIndicator {
    code: isize,
}

impl Default for BaseIndicator {
    fn default() -> Self {
        Self { code: Default::default() }
    }
}

impl Descriptive for BaseIndicator {
    fn default_name(&self) -> String {
        let i = IndicatorName::from(self.code);
        i.to_string()
    }

    fn name(&self) -> String {
        todo!()
    }
}

#[derive(Debug)]
pub struct Indicator {
    context: isize,
    base: Rc<BaseIndicator>,
}

impl Default for Indicator {
    fn default() -> Self {
        Self { context: Default::default(), base: Rc::default() }
    }
}

impl Indicator {
    pub fn build(context: isize, code: isize) -> Self {
        Indicator { context, base: Rc::new(BaseIndicator { code }) } 
    }

    pub fn get_code(&self) -> isize {
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
    pub code: &'static isize,
    pub context: isize,
    pub key: Rc<ComputeKey>
}

impl IndicatorInput {
    pub fn info(&self, conf: &HashMap<isize, ComputeMode>) -> ComputedIndicator<Indicator> {
        let idc = Rc::new(Indicator::build(self.context, *self.code));
        match conf.get(self.code) {
            Some(ComputeMode::AddUp) => ComputedIndicator::AddUp(Rc::clone(&idc)),
            Some(ComputeMode::Default) => ComputedIndicator::Default(Rc::clone(&idc)),
            Some(ComputeMode::Avg) => ComputedIndicator::Avg(Rc::clone(&idc)),
            Some(ComputeMode::Complex) => ComputedIndicator::Complex(Rc::clone(&idc)),
            None => panic!("Input was undefined")
        }
    }
}

pub struct UserInput {
    pub inputed: Option<f64>,
    pub computed: Option<f64>,
    pub author: String
}

pub struct ComputeKey {
    pub date: DateKey,
    pub span: Option<&'static str>
}

pub mod data;
pub mod date;
pub mod tools;
pub mod fiscalyear;