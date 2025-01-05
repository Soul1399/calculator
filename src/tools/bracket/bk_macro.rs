
#[macro_export]
macro_rules! bk_config {
    ($($ident:ident),* ) => {
        lazy_static::lazy_static! {
            static ref ConfigProps: Vec<String> = {
                let mut v: Vec<String> = Default::default();
                $(
                    v.push(String::from(stringify!($ident)));
                )*
                v
            };
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

        impl BracketConfig {
            pub fn get_config_value(&mut self, key: &str) -> &str {
                $(
                    if key == stringify!($ident) {
                        return &self.$ident;
                    }
                )*
                Default::default()
            }
            
            pub fn set_config(&mut self, key: &str, value: &str) {
                $(
                    if key == stringify!($ident) {
                        paste::paste! {
                        self.[<set _ $ident>](value);
                        }
                    }
                )*
            }

            pub fn is_empty(&self) -> bool {
                $(
                    if self.$ident.len() > 0 {
                        return false;
                    }
                )*
                true
            }

            $(
                paste::paste! {
                    fn [<set _ $ident>](&mut self, value: &str) {
                        self.$ident = value.to_owned();
                    }
                }
            )*
        }
    };
}