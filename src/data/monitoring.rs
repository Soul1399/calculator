
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

    fn compute_ltm_values(&self, end_date: &DateKey, indic_inputs: &Vec<&&mut IndicatorInput>) -> Option<f64> {
        let mut month_inputs: Vec<_> = indic_inputs.iter().filter(|i| i.key.span == None).collect();
        let slc_inputs: Vec<_> = indic_inputs.iter().filter(|i| i.key.span == Some(&SLC)).collect();
        if slc_inputs.iter()
            .filter(|i| i.input.borrow().computed != None || i.input.borrow().inputed != None)
            .count() < 4 {
                return None;
            }
        let mut start_date = *end_date;
        start_date.add_months(-12);
        month_inputs.sort_by(|&&a, &&b| a.key.date.cmp(&b.key.date));
        let mut result: Option<f64> = None;
        let mut buffer: Vec<f64> = vec![];
        let mut out_buffer: Vec<f64> = vec![];
        month_inputs.iter().for_each(|ii| {
            let s = slc_inputs.iter()
                .filter(|i| i.key.date == ii.key.date)
                .next();
            if ii.key.date < start_date {
                if let Some(v) = ii.get_value() {
                    out_buffer.push(v);
                }
            }
            else {
                if let Some(v) = ii.get_value() {
                    if ii.key.date <= *end_date {
                        buffer.push(v);
                    }
                    else {
                        out_buffer.push(v);
                    }
                }
                if let Some(v) = s {
                    if let Some(x) = v.input.borrow().inputed {
                        if v.key.date < *end_date {
                            let f = result.unwrap_or_default();
                            result = Some((x / (out_buffer.len() + buffer.len()) as f64) * buffer.len() as f64 + f);
                        }
                    }
                    else if buffer.len() > 0 {
                        let f = result.unwrap_or_default();
                        buffer.push(f);
                        result = Some(fsum::FSum::new().add_all(&buffer).value());
                    }
                    buffer.clear();
                    out_buffer.clear();
                }
            }
        });
        
        result
    }

    fn extract_ltm_combinable_values(&self, end_date: &DateKey, slice: &Vec<DateKey>, indic_inputs: &Vec<&&mut IndicatorInput>, mode: &ComputerMode) -> Vec<Box<f64>> {
        let mut month_inputs: Vec<_> = indic_inputs.iter().filter(|i| i.key.span == None).collect();
        let slc_inputs: Vec<_> = indic_inputs.iter().filter(|i| i.key.span == Some(&SLC)).collect();
        let min_slc = if let &ComputerMode::Avg = mode { 1 } else { 4 };
        if slc_inputs.iter()
            .filter(|i| i.input.borrow().computed != None || i.input.borrow().inputed != None)
            .count() < min_slc {
                return vec![];
            }
        let mut start_date = *end_date;
        start_date.add_months(-12);
        month_inputs.sort_by(|&&a, &&b| a.key.date.cmp(&b.key.date));
        let mut values: Vec<Box<f64>> = vec![];
        let mut buffer: Vec<f64> = vec![];
        let mut out_buffer: Vec<f64> = vec![];
        let mut dates: Vec<&DateKey> = slice.iter().collect();
        dates.sort();
        let mut current_date: &DateKey;
        while dates.len() > 0 {
            current_date = dates.first().unwrap();
            let s = slc_inputs.iter()
                .filter(|i| i.key.date == *current_date)
                .next();
            let m = month_inputs.iter()
                .filter(|i| i.key.date == *current_date)
                .next();

            if *current_date < start_date || current_date > end_date {
                out_buffer.push(0.0);
                if let Some(ii) = m {
                    if let Some(v) = ii.get_value() {
                        out_buffer.pop();
                        out_buffer.push(v);
                    }
                }
            }
            else {
                buffer.push(0.0);
                if let Some(ii) = m {
                    if let Some(v) = ii.get_value() {
                        buffer.pop();
                        buffer.push(v);
                    }
                }
            }

            if let Some(v) = s {
                if let Some(x) = v.input.borrow().inputed {
                    if out_buffer.len() == 0 {
                        values.push(Box::new(x));
                    }
                    else {
                        values.push(Box::new((x / (out_buffer.len() + buffer.len()) as f64) * buffer.len() as f64));
                    }
                }
                else if let &ComputerMode::Avg = mode {
                    if let Some(x) = v.input.borrow().computed {
                        values.push(Box::new(x));
                    }
                }
                else if buffer.len() > 0 {
                    values.extend(buffer.iter().map(|x| Box::new(*x)));
                }
                buffer.clear();
                out_buffer.clear();
            }

            // to avoid duplicate values
            dates.retain(|d| *d != current_date);
        }
        
        values
    }
    
}