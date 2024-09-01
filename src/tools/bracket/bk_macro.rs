
#[macro_export]
macro_rules! bk_config {
    ($($ident:ident),* ) => {
        $(
            paste::paste! {
                const [<"BKCONF" _ $ident:upper>]: &str = stringify!($ident);
            }
        )*

        paste::paste! {
            lazy_static::lazy_static! {
                $(
                    static ref [<"CONFIG" _ $ident:upper>]: String = str::replace(stringify!($ident), "_", " ");
                )*
            }
        }

        #[derive(Debug, Clone)]
        pub struct BracketConfig {
            $(
            pub $ident: String,
            )*
        }

        impl Default for BracketConfig {
            fn default() -> Self {
                BracketConfig {
                    $(
                        $ident: Default::default(),
                    )*
                }
            }
        }
    };
}