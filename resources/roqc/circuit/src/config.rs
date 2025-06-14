use derive_more::Display;
use itertools::iproduct;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::layer::Layout;

#[derive(Deserialize, Debug, Clone, Serialize, Display, PartialEq, Eq)]
pub enum Cost {
    Depth,
    Gate,
    Mixed,
}
#[derive(Deserialize, Debug, Clone, Serialize, Display, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum TimeOut {
    // seconds
    PerGate(f64),
    PerSegment(f64),
}

#[derive(Deserialize, Debug, Clone, Serialize, Display, PartialEq, Eq)]
pub enum Gateset {
    Nam,
    CliffordT,
}

#[derive(Deserialize, Debug, Clone, Serialize, Display, PartialEq)]
#[display("QuartzConfig(cost={cost}, timeout={timeout})")]
pub struct QuartzConfig {
    pub cost: Cost,
    pub timeout: TimeOut,
    pub ecc_path: String,
    pub gateset: Gateset,
    pub n_threads: usize,
}

#[derive(Deserialize, Debug, Clone, Serialize, Display, PartialEq, Eq)]
pub struct VoqcConfig {}

#[derive(Deserialize, Debug, Clone, Serialize, Display, PartialEq, Eq)]
pub struct RoqcConfig {}

#[derive(Deserialize, Debug, Clone, Serialize, Display, PartialEq, Eq)]
pub struct TketConfig {}

#[derive(Deserialize, Debug, Clone, Serialize, Display, PartialEq, Eq)]
pub struct QiskitConfig {}

#[derive(Deserialize, Debug, Clone, Serialize, Display, PartialEq)]
pub enum OracleName {
    Quartz(QuartzConfig),
    Voqc(VoqcConfig),
    Roqc(RoqcConfig),
    Tket(TketConfig),
    Qiskit(QiskitConfig),
}

#[derive(Deserialize, Debug, Clone, Serialize, Display, PartialEq, Eq)]
pub enum PreprocessConfig {
    None,
}

macro_rules! config_structs {
    ($($field:ident: $ftype:ty),*) => {
        #[derive(Deserialize, Debug, Clone, Serialize)]
        pub struct MultipleConfigs {
            $(pub $field: Vec<$ftype>,)*
        }

        #[derive(Deserialize, Debug, Clone, Serialize)]
        pub struct SingleConfig {
            $(pub $field: $ftype,)*
        }

        impl MultipleConfigs {
            pub fn unique_config_elements(&self) -> HashMap<AllConfigKeys, bool> {
                let mut unique_elements = HashMap::new();
                $(
                    unique_elements.insert(AllConfigKeys::$field, self.$field.len() == 1);
                )*
                unique_elements
            }

            pub fn to_single_configs(&self) -> Vec<SingleConfig> {
                let combinations = iproduct!(
                    $(self.$field.iter()),*
                );
                let mut single_configs = Vec::new();
                for combination in combinations {
                    let ( $( $field, )* ) = combination;
                    single_configs.push(SingleConfig {
                        $( $field: $field.clone(), )*
                    });
                }
                single_configs
            }
            pub fn print_unique_elements(&self) {
                let unique_elements = self.unique_config_elements();
                $(
                    if unique_elements.get(&AllConfigKeys::$field).copied().unwrap_or(false) {
                        println!("{}: {:?}", stringify!($field), self.$field[0]);
                    }
                )*
            }

        }
        impl SingleConfig {
            pub fn print_non_unique_elements(&self, unique_elements: &HashMap<AllConfigKeys, bool>) {
                $(
                    if !unique_elements.get(&AllConfigKeys::$field).copied().unwrap_or(true) {
                        println!("{}: {:?}", stringify!($field), &self.$field);
                    }
                )*
            }
            pub fn non_unique_elements(&self, unique_elements: &HashMap<AllConfigKeys, bool>)->String {
                let mut result = String::new();
                $(
                    if !unique_elements.get(&AllConfigKeys::$field).copied().unwrap_or(true) {
                        result.push_str(&format!("{}: {:?}, ", stringify!($field), &self.$field));
                    }
                )*
                result
            }
        }
        #[allow(non_camel_case_types)]
        #[derive(PartialEq, Eq, Hash)]
        pub enum AllConfigKeys {
            $( $field, )*
        }
    };
}
config_structs! {
    circuit_path: String,
    use_soam:bool, //Directly send it to oracle or use soam
    omega: usize,
    oracle_name: OracleName,
    preprocess_config: PreprocessConfig,
    cost: Cost,
    gateset: Gateset,
    n_threads: usize,
    layout: Layout
}
impl MultipleConfigs {
    pub fn read_config(config_path: &String) -> MultipleConfigs {
        let config_string =
            std::fs::read_to_string(config_path).expect("failed to read config file");
        let config: MultipleConfigs =
            toml::from_str(&config_string).expect("failed to parse config file");
        config
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_save_config() {
        let config = MultipleConfigs {
            circuit_path: vec!["circuit1".to_string()],
            use_soam: vec![true],
            omega: vec![10],
            oracle_name: vec![
                OracleName::Quartz(QuartzConfig {
                    cost: Cost::Depth,
                    timeout: TimeOut::PerGate(0.1),
                    ecc_path: "ecc_path".to_string(),
                    gateset: Gateset::Nam,
                    n_threads: 1,
                }),
                OracleName::Quartz(QuartzConfig {
                    cost: Cost::Depth,
                    timeout: TimeOut::PerGate(0.1),
                    ecc_path: "ecc_path".to_string(),
                    gateset: Gateset::Nam,
                    n_threads: 1,
                }),
            ],
            preprocess_config: vec![PreprocessConfig::None],
            cost: vec![Cost::Depth],
            gateset: vec![Gateset::Nam],
            n_threads: vec![1],
            layout: vec![Layout::Dense],
        };
        let config_string = toml::to_string(&config).expect("Failed to serialize config");
        std::fs::write("config.toml", config_string).expect("Failed to write config file");
    }
}
