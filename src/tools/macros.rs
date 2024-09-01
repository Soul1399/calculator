#[macro_export]
macro_rules! build_entities {
    (json $path:expr) => {
        println!($path);
    };
    (valueof $i:ident) => {
        println!("{}", $i);
    };
    (listof $($i:ident),*) => {
        {
            let mut v: Vec<String> = Vec::new();
            $(
                v.push(stringify!($i).to_owned());
            )*
            v
        }
    }
}

#[macro_export]
macro_rules! show_name {
    ($tt:tt) => {
        println!("{}", stringify!($tt));
    };
}