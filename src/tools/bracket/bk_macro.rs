#[macro_export]
macro_rules! bk_config {
    (($ident:ident),*) => (
        pub struct BracketConfig2 {
            $(
            pub $ident: &str
            )*
        }
    );
}