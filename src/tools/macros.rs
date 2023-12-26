#[macro_export]
macro_rules! build_entities {
    (json $path:expr) => {
        println!($path);
    };
    (valueof $i:ident) => {
        println!("{}", $i);
    };
}

