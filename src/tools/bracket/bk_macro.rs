#[macro_export]
macro_rules! bk_config {
    (($ident:ident),*) => (
        pub struct BracketConfig {
            $(
            pub $ident: &str
            )*
        }
    );
}