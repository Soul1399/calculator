
use std::{error::Error, rc::Rc};
use calculator::{Descriptive, Indicator, ComputedIndicator, CASH_CODE, SALES_CODE, data};

fn main() -> Result<(), Box<dyn Error>> {
    println!("\nCalcultor is running\n");
    
    let context = data::load_context(1);

    // context.iter().for_each(|fy| {
    //     match fy.get_quarter(2) {
    //         Ok(x) => println!("{}", x.to_string()),
    //         Err(e) => println!("Error: {}", e)
    //     };
    // });

    let list = data::get_indicators();

    // list.iter().for_each(|x| println!("Indicator: {}", x.indic.indicator().unwrap().default_name()));
    // list.iter().for_each(|x| println!("{}", x.key.span.unwrap_or_default()));

    Ok(())
}

fn test1(inputs: Vec<Option<&f64>>) -> Result<(), Box<dyn Error>> {
    let sales = Indicator::build(1, SALES_CODE);
    let cash = Indicator::build(1, CASH_CODE);
    let name = sales.default_name();
    println!("{name}");
    let add_up = ComputedIndicator::AddUp(Rc::new(sales));
    let nadd_up = ComputedIndicator::Default(Rc::new(cash));
    let result = add_up.compute(&inputs)?;
    
    println!("Total {} {result}", add_up.indicator().unwrap_or_default().default_name());
    let result = nadd_up.compute(&inputs)?;
    println!("Total {} {result}", nadd_up.indicator().unwrap_or_default().default_name());
    Ok(())
}

