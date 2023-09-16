
use std::cell::RefCell;

use crate::{fiscalyear::FiscalYear, indic::{IndicatorInput, SLC, FY, LTM, ComputerMode}, ComputeError, ComputeKey, date::DateKey};
use super::inputs::InputContext;


pub struct InputMonitoring {
    context: Box<InputContext>,
    years: Box<Vec<FiscalYear>>
}

impl InputMonitoring {
    pub fn build(context: InputContext, years: Vec<FiscalYear>) -> InputMonitoring {
        let mut monitor = InputMonitoring {
            context: Box::new(context),
            years: Box::new(years)
        };

        for fy in monitor.years.as_mut() {
            fy.build_slices(3);
        }

        return monitor;
    }

    pub fn compute<'a>(&self, inputs: &'a mut Vec<IndicatorInput>) -> Result<&'a Vec<IndicatorInput>, ComputeError> {
        let compute_keys = FiscalYear::get_keys(&self.years);

        for k in compute_keys {
            println!("\nComputing {} {}", k.span.unwrap_or("month"), k.date.to_string());
            if let Err(e) = self.compute_by_key(inputs, &k) {
                let mut message = String::from("Unable to compute fiscal year ");
                message.push_str(&k.date.to_string()[..]);
                message.push_str(", key: ");
                message.push_str(&k.span.unwrap().to_string()[..]);
                message.push_str(". Error: ");
                message.push_str(e);
                return Err(ComputeError { details: message });
            }
        }

        Ok(inputs)
    }

    pub fn compute_by_key(&self, inputs: &mut Vec<IndicatorInput>, key: &ComputeKey) -> Result<(), &'static str> {
        if inputs.len() == 0 {
            return Err("Indicator inputs list is empty");
        }
        if self.years.len() == 0 {
            return Err("There was no available fiscal years");
        }
        let span;
        match key.span {
            Some(c) => span = c,
            None => return Err("Invalid span")
        };
        
        let fy = match FiscalYear::find(&self.years, &key.date) {
            Ok(value) => value,
            Err(value) => return value
        };
    
        match span {
            FY => self.compute_fy(inputs, key, fy),
            SLC => {
                let slice: Vec<DateKey>;
                match fy.find_slice(&key.date) {
                    Ok(s) => slice = s,
                    Err(e) => return Err(e)
                };
                return self.compute_slice(inputs, &slice, key, Some(&SLC), &vec![None]);
            },
            LTM => {
                let slice: Vec<DateKey>;
                match FiscalYear::find_ltm_slice(&self.years, &key.date) {
                    Ok(s) => slice = s,
                    Err(e) => return Err(e)
                };
                return self.compute_slice(inputs, &slice, key, Some(&LTM), &vec![Some(&SLC), None]);
            },
            _ => Err("Unknown span")
        }
    }

    fn compute_slice(&self, inputs: &mut Vec<IndicatorInput>, slice: &Vec<DateKey>, key: &ComputeKey, span: Option<&str>, children_spans: &Vec<Option<&str>>) -> Result<(), &'static str> {
        let mut slice_inputs = self.extract_inputs(inputs, slice, span, children_spans);
        // emulate group by
        let mut codes:Vec<&'static isize> = slice_inputs.iter().map(|i| i.code).collect();
        while codes.len() > 0 {
            // get group key
            let _k = codes.first();
            let code: &'static isize;
            match _k {
                Some(x) => code = x,
                None => break
            }
            if span == Some(&LTM) {
                if let Some(value) = self.compute_ltm_of_indicator(&slice_inputs, slice, &key.date, code) {
                    return value;
                }
            }
            else if let Some(value) = self.compute_slice_of_indicator(&slice_inputs, code, span, children_spans) {
                return value;
            }
            // keep other inputs
            slice_inputs.retain(|i| i.code != code);
            // keep other keys (even if they appear multiple times)
            codes.retain(|x| *x != code);
        }
    
        Ok(())
    }

    fn compute_fy(&self, inputs: &mut Vec<IndicatorInput>, key: &ComputeKey, fy: &FiscalYear) -> Result<(), &'static str> {
        let max_date = fy.max();
        if let Err(e) = max_date {
            return Err(e);
        }
        if let Some(value) = self.compute_each_slice(inputs, key, fy) {
            return value;
        }
    
        if let Err(e) = self.compute_slice(inputs, &fy.get_months(), key, Some(&FY), &vec![Some(&SLC)]) {
            return Err(e);
        }
    
        Ok(())
    }

    fn compute_each_slice(&self, inputs: &mut Vec<IndicatorInput>, key: &ComputeKey, fy: &FiscalYear) -> Option<Result<(), &'static str>> {
        for s in FiscalYear::get_slices(fy) {
            let slice: Vec<DateKey>;
            match fy.find_slice(s.first().unwrap()) {
                Ok(s) => slice = s,
                Err(e) => return Some(Err(e))
            };
            if let Err(e) = self.compute_slice(inputs, &slice, key, Some(&SLC), &vec![None]) {
                return Some(Err(e));
            }
        }
    
        None
    }

    fn compute_slice_of_indicator(
        &self,
        slice_inputs: &Vec<&mut IndicatorInput>, 
        code: &'static isize, 
        target_span: Option<&str>, 
        item_spans: &Vec<Option<&str>>) -> Option<Result<(), &'static str>> {
        let indic_inputs: Vec<&&mut IndicatorInput> = slice_inputs.iter()
            .filter(|i| i.code == code)
            .collect();
        if indic_inputs.len() == 0 {
            return None;
        }
        let computer = indic_inputs.first().unwrap().get_computer(&self.context.configuration);
        let target_input = indic_inputs
            .iter()
            .filter(|&&i| i.key.span == target_span)
            .next();
        match target_input {
            None => {
                return Some(Err("Missing target input"));
            },
            _ => {}
        }
        let input_values = self.extract_values(&indic_inputs, item_spans[0]);
        
        match computer.compute(&input_values) {
            Ok(x) => {
                target_input.map(|val| val.input.borrow_mut().computed = Some(x));
            },
            Err(e) => {
                if e.details.len() > 0 {
                    println!("{}", e.details);
                    return Some(Err("Compute failed"));
                }
                else {
                    target_input.map(|val| val.input.borrow_mut().computed = None);
                }
            }
        }
        None
    }

    fn compute_ltm_of_indicator(&self, slice_inputs: &Vec<&mut IndicatorInput>, slice: &Vec<DateKey>, date: &DateKey, code: &'static isize) -> Option<Result<(), &'static str>> {
        let indic_inputs: Vec<&&mut IndicatorInput> = slice_inputs.iter()
            .filter(|i| i.code == code)
            .collect();
        if indic_inputs.len() == 0 {
            return None;
        }
        let target_input = indic_inputs.iter()
            .filter(|&&i| i.key.span == None && i.key.date == *date)
            .next();
        match target_input {
            None => {
                return Some(Err("Missing target input"));
            },
            _ => {}
        }
        let mode = self.context.configuration.get(code).expect("Unable to determine compute mode");
        // direct sum
        // if let ComputerMode::AddUp = mode {
        //     *target_input.unwrap().ltm.borrow_mut() = self.compute_ltm_values(date, &indic_inputs);
        //     return None;
        // }
        // let input_values = self.extract_ltm_values(date, &indic_inputs);
        let input_values = match mode {
            ComputerMode::AddUp | ComputerMode::Avg => self.extract_ltm_combinable_values(date, slice, &indic_inputs, mode),
            _ => self.extract_ltm_values(date, &indic_inputs)
        };
        
        let computer = indic_inputs.first().unwrap().get_computer(&self.context.configuration);
        match computer.compute(&input_values) {
            Ok(x) => {
                target_input.map(|val| *val.ltm.borrow_mut() = Some(x));
            },
            Err(e) => {
                if e.details.len() > 0 {
                    println!("{}", e.details);
                    return Some(Err("Compute failed"));
                }
                else {
                    target_input.map(|val| val.ltm.take());
                }
            }
        }
        None
    }
    
    fn extract_inputs<'a>(&self, inputs: &'a mut Vec<IndicatorInput>, slice: &Vec<DateKey>, parent_span: Option<&str>, children_spans: &Vec<Option<&str>>) -> Vec<&'a mut IndicatorInput> {
        let slice_inputs: Vec<_> = inputs.iter_mut()
            .filter(|i| i.key.span == parent_span || children_spans.iter().any(|s| i.key.span == *s))
            .filter(|i| slice.iter().any(|d| i.key.date == *d))
            .collect();
    
        slice_inputs
    }

    fn extract_values(&self, indic_inputs: &Vec<&&mut IndicatorInput>, span: Option<&str>) -> Vec<Box<f64>> {
        let mut input_values: Vec<Box<f64>> = Vec::new();
        indic_inputs
            .iter()
            .filter(|&&i| i.key.span == span)
            .for_each(|&i| {
                let mut o = i.input.borrow().inputed;
                if o == None {
                    o = i.input.borrow().computed;
                }
                match o {
                    Some(f) => input_values.push(Box::new(f)),
                    None => {}
                }
            });
        input_values
    }

    fn extract_ltm_values(&self, end_date: &DateKey, indic_inputs: &Vec<&&mut IndicatorInput>) -> Vec<Box<f64>> {
        let mut start_date = *end_date;
        start_date.add_months(-12);
        let mut month_inputs: Vec<_> = indic_inputs.iter()
            .filter(|i| start_date <= i.key.date && i.key.date <= *end_date)
            .filter(|i| i.key.span == None)
            .collect();

        month_inputs.sort_by(|&&a, &&b| a.key.date.cmp(&b.key.date));
        
        let mut input_values: Vec<Box<f64>> = Vec::new();
        month_inputs.iter().for_each(|i| {
            let mut o = i.input.borrow().inputed;
            if o == None {
                o = i.input.borrow().computed;
            }
            match o {
                Some(f) => input_values.push(Box::new(f)),
                None => {}
            }
        });
        input_values
    }

    fn extract_ltm_combinable_values(&self, end_date: &DateKey, slice: &Vec<DateKey>, indic_inputs: &Vec<&&mut IndicatorInput>, mode: &ComputerMode) -> Vec<Box<f64>> {
        let x: Vec<&IndicatorInput> = indic_inputs.iter().map(|i| &***i).collect();
        let mut ltm = LtmSumHandler::new(end_date, slice, &x, mode);
        let ltm = ltm.verify();
        if let Err(e) = ltm {
            panic!("{e}");
        }
        let ltm = ltm.unwrap().collect_values();
        if let Err(e) = ltm {
            panic!("{e}");
        }
        ltm.unwrap().get_values().unwrap().borrow().to_vec()
    }
    
}

#[derive(Clone)]
struct LtmInputs<'a> {
    start_date: DateKey,
    end_date: DateKey,
    dates: Vec<DateKey>,
    month_inputs: Vec<&'a IndicatorInput>,
    slice_inputs: Vec<&'a IndicatorInput>,
    compute_mode: &'a ComputerMode,
    values: RefCell<Vec<Box<f64>>>,
    buffer: RefCell<Vec<f64>>,
    bypass_buffer: RefCell<Vec<f64>>,
    min_slc: usize,
    is_unavailable: bool
}
pub trait LtmState {
    fn get_values(&self) -> Option<RefCell<Vec<Box<f64>>>> {
        None
    }
}

pub struct LtmInit<'a> {
    ltm_data: LtmInputs<'a>
}

pub struct LtmSumHandler<'a> {
    ltm_data: LtmInputs<'a>
}

pub struct LtmCollector<'a> {
    ltm_data: LtmInputs<'a>
}

impl<'a> LtmState for LtmInit<'a> {}
impl<'a> LtmState for LtmSumHandler<'a> {}
impl<'a> LtmState for LtmCollector<'a> {
    fn get_values(&self) -> Option<RefCell<Vec<Box<f64>>>> {
        Some(RefCell::clone(&self.ltm_data.values))
    }
}

impl<'a> LtmSumHandler<'a> {
    pub fn new(date: &'a DateKey, slice: &'a Vec<DateKey>, inputs: &'a Vec<&'a IndicatorInput>, mode: &'a ComputerMode) -> LtmInit<'a> {
        LtmInit { ltm_data: LtmInputs::build(date, slice, inputs, mode) }
    }

    pub fn collect_values(&self) -> Result<LtmCollector<'a>, &'static str> {
        if !self.ltm_data.is_unavailable {
            let mut current_date: &DateKey;
            let mut dates: Vec<&DateKey> = self.ltm_data.dates.iter().collect();
            dates.sort();
            while dates.len() > 0 {
                current_date = dates.first().unwrap();
                self.buffer(current_date);
                self.collect(current_date);
                // to avoid duplicate values
                dates.retain(|d| *d != current_date);
            }
        }
        Ok(LtmCollector { ltm_data: self.ltm_data.clone() })
    }

    fn buffer(&self, current_date: &DateKey) {
        let m = self.ltm_data.month_inputs.iter()
            .filter(|i| i.key.date == *current_date)
            .next();
        
        self.ltm_data.buffer.borrow_mut().push(Default::default());
        if let Some(ii) = m {
            if let Some(v) = ii.get_value() {
                self.ltm_data.buffer.borrow_mut().pop();
                self.ltm_data.buffer.borrow_mut().push(v);
            }
        }
        if *current_date < self.ltm_data.start_date || *current_date > self.ltm_data.end_date {
            self.ltm_data.bypass_buffer.borrow_mut().push(self.ltm_data.buffer.borrow_mut().pop().unwrap());
        }
    }

    fn collect(&self, current_date: &DateKey) {
        let s = self.ltm_data.slice_inputs.iter()
            .filter(|i| i.key.date == *current_date)
            .next();
        if let Some(v) = s {
            let nb_bypass = self.ltm_data.bypass_buffer.borrow().len();
            let nb_buffered = self.ltm_data.buffer.borrow().len();
            if let Some(x) = v.input.borrow().inputed {
                if nb_bypass == 0 {
                    self.ltm_data.values.borrow_mut().push(Box::new(x));
                }
                else {
                    self.ltm_data.values.borrow_mut().push(Box::new((x / (nb_bypass + nb_buffered) as f64) * nb_buffered as f64));
                }
            }
            else if let &ComputerMode::Avg = self.ltm_data.compute_mode {
                if let Some(x) = v.input.borrow().computed {
                    if nb_bypass == 0 {
                        self.ltm_data.values.borrow_mut().push(Box::new(x));
                    }
                    else {
                        self.ltm_data.values.borrow_mut().push(Box::new(self.ltm_data.buffer.borrow().iter().map(|f| *f).sum()));
                    }
                }
            }
            else if nb_buffered > 0 {
                self.ltm_data.values.borrow_mut().extend(self.ltm_data.buffer.borrow().iter().map(|x| Box::new(*x)));
            }
            self.ltm_data.buffer.borrow_mut().clear();
            self.ltm_data.bypass_buffer.borrow_mut().clear();
        }
    }
}

impl<'a> LtmInit<'a> {
    pub fn verify(&mut self) -> Result<LtmSumHandler, &'static str> {
        if self.ltm_data.slice_inputs.len() == 0 {
            return Err("Missing slice inputs");
        }

        if self.ltm_data.slice_inputs.iter().filter(|i| i.input.borrow().computed != None || i.input.borrow().inputed != None).count() < self.ltm_data.min_slc {
            self.ltm_data.is_unavailable = true;
        }

        Ok(LtmSumHandler { ltm_data: self.ltm_data.clone() })
    }
}

impl<'a> LtmInputs<'a> {
    fn build(date: &'a DateKey, slice: &'a Vec<DateKey>, inputs: &'a Vec<&'a IndicatorInput>, mode: &'a ComputerMode) -> LtmInputs<'a> {
        let mut start_date = *date;
        start_date.add_months(-12);
        let ltm = LtmInputs { 
            start_date, 
            end_date: *date, 
            dates: slice.into_iter().map(|d| *d).collect(), 
            month_inputs: inputs.iter().filter(|i| i.key.span == None).map(|i| *i).collect(),
            slice_inputs: inputs.iter().filter(|i| i.key.span == Some(&SLC)).map(|i| *i).collect(),
            compute_mode: mode,
            min_slc: if let &ComputerMode::Avg = mode { 1 } else { 4 },
            buffer: RefCell::new(Vec::with_capacity(slice.len())),
            bypass_buffer: RefCell::new(Vec::with_capacity(slice.len())),
            values: RefCell::new(vec![]),
            is_unavailable: false
        };
        
        ltm
    }
}

#[cfg(test)]
mod tests {
    use crate::{date::DateKey, indic::{ComputerMode, SALES_CODE}, data::inputs::UserInput};
    use std::{cell::RefCell, rc::Rc, collections::HashMap};
    use super::*;

    #[test]
    #[should_panic(expected="Missing slice inputs")]
    fn new_ltm()  {
        let (date, slice, mode) = init_ltm_data();
        let inputs = vec![];
        let mut ltm = LtmSumHandler::new(&date, &slice, &inputs, &mode);
        let ltm = ltm.verify();
        assert!(ltm.unwrap().ltm_data.is_unavailable);
    }

    #[test]
    fn is_unavailable_ltm()  {
        let (date, slice, mode) = init_ltm_data();
        let list = build_inputs(HashMap::new());
        let inputs = list.iter().collect();

        let mut ltm = LtmSumHandler::new(&date, &slice, &inputs, &mode);
        let ltm = ltm.verify();
        assert!(ltm.unwrap().ltm_data.is_unavailable);
    }

    #[test]
    fn is_unavailable_ltm2()  {
        let (date, slice, mode) = init_ltm_data();
        let mut values = HashMap::new();
        values.insert(9, 6.7);
        let list = build_inputs(values);
        let inputs = list.iter().collect();

        let mut ltm = LtmSumHandler::new(&date, &slice, &inputs, &mode);
        let ltm = ltm.verify();
        assert!(ltm.unwrap().ltm_data.is_unavailable);
    }

    #[test]
    fn is_available_ltm()  {
        let (date, slice, mode) = init_ltm_data();
        let mut values = HashMap::new();
        values.insert(9, 6.7);
        values.insert(12, 34.0);
        values.insert(3, 1.7);
        values.insert(6, 0.0);
        let list = build_inputs(values);
        let inputs = list.iter().collect();

        let mut ltm = LtmSumHandler::new(&date, &slice, &inputs, &mode);
        let ltm = ltm.verify();
        assert!(!ltm.unwrap().ltm_data.is_unavailable);
    }

    fn init_ltm_data<'a>() -> (DateKey, Vec<DateKey>, ComputerMode) {
        let date = DateKey::build(1, 2023);
        let mut slice: Vec<DateKey> = (1..7).into_iter().map(|x| DateKey::build(x, 2023)).collect();
        slice.extend((7..13).into_iter().map(|x| DateKey::build(x, 2022)));
        let mode = ComputerMode::AddUp;
        (date, slice, mode)
    }

    fn build_inputs(values: HashMap<u8, f64>) -> Vec<IndicatorInput> {
        vec![9, 12, 3, 6].into_iter().map(|m| {
            let y = if let 9 | 12 = m { 2022 } else { 2023 };
            let value = match values.get(&m) {
                Some(x) => Some(*x),
                None => None
            };
            IndicatorInput { 
                code: &SALES_CODE, 
                input: RefCell::new(UserInput { author: String::new(), computed: None, inputed: value }), 
                ltm: RefCell::new(None), 
                context: 1, 
                key: Rc::new(ComputeKey { date: DateKey::build(m, y), span: Some(&SLC) })
            }
        }).collect()
    }
}