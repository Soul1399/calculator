
use std::{error::Error, rc::Rc};
use calculator::{Descriptive, Indicator, ComputedIndicator, CASH_CODE, SALES_CODE, data, ComputeKey, date::DateKey, FY, SLC};

fn main() -> Result<(), Box<dyn Error>> {
    println!("\nCalcultor is running\n");
    
    let mut context = data::load_context(1);
    let config = crate::data::get_config();
    let mut list = data::get_all_inputs();

    // for y in context {
    //     println!("fiscal year {} - {}", y.min().unwrap().to_string(), y.max().unwrap().to_string());
    // }

    let key = ComputeKey { date: DateKey::build(8, 2020), span: Some(&FY) };

    data::inputs::compute_by_key(&mut list, &mut context, &key)?;

    for i in list {
        let info = i.info(&config);
        let value: String;
        match i.input.borrow().inputed {
            Some(f) => value = f.to_string(),
            None => match i.input.borrow().computed {
                Some(f) => value = f.to_string(),
                None => value = String::from("None")
            }
        }
        if i.key.span == Some(&FY) || i.key.span == Some(&SLC) {
            println!(
                "input {} {} {} {}", 
                i.key.date.to_string(),
                i.key.span.unwrap(),
                info.indicator().unwrap().default_name(),
                value);
        }
    }

    Ok(())
}

