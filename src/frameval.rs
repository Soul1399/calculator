use std::{cell::RefCell, rc::Rc, ops::Deref};

#[derive(Clone, Copy, Debug)]
struct CellPosition {
    pub line: u32,
    pub column: u32
}

pub trait FigureValues {
    fn get_computed(self: &Self) -> Option<f64>;
}

struct CellValue {
    pub position: CellPosition,
    pub input: Option<f64>,
    pub figures: Option<Box<dyn FigureValues>>
}

impl CellValue {
    pub fn get_value(self: &Self) -> Box<Option<f64>> {
        if self.input.is_none() {
            if self.figures.is_none() {
                return Box::new(None);
            }
            return Box::new(self.figures.as_ref().unwrap().get_computed());
        }
        Box::new(self.input)
    }
}

// impl Deref for CellValue {
//     type Target = CellValue;

//     fn deref(&self) -> &Self::Target {
//         &self
//     }
// }

struct Grid {
    pub values: Vec<Rc<CellValue>>
}

