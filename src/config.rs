use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::io::Error;
use toml;

// values for each key
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigTomlInit {
    pub type_: Option<String>,
    pub trigger: Option<String>,
    pub code: Option<String>,
}
// values for each key
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigTomlDetectFlag {
    pub type_: Option<String>,
    pub trigger: Option<String>,
    pub code: Option<String>,
}
// values for each key
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigTomlSimpleTrigger {
    pub type_: Option<String>,
    pub trigger: Option<String>,
    pub code: Option<String>,
}

//struct that contain the config.toml information as parameters
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigToml {
    // write the keys here
    pub init: Option<ConfigTomlInit>,
    pub detect_flag: Option<ConfigTomlDetectFlag>,
    pub simple_trigger: Option<ConfigTomlSimpleTrigger>,
}

#[derive(Debug)]
pub struct Config {
    pub not_important: String,
}

impl Config {
    pub fn new() -> Result<ConfigToml, Error> {
        let config_filepaths: [&str; 2] = ["./config.toml", "./Config.toml"];
        let mut content: String = "".to_owned();
        for filepath in &config_filepaths {
            if let Ok(result) = fs::read_to_string(filepath) {
                content = result;
                break;
            }
        }

        let config_toml: ConfigToml = toml::from_str(&content).unwrap();

        Ok(config_toml)
    }

    pub fn extract_init_values(config_toml: ConfigToml) -> (String, String, String) {
        let (key_type, key_trigger, key_code): (String, String, String) = match config_toml.init {
            Some(key) => {
                let key_type: String = key.type_.unwrap_or_else(|| {
                    println!("Missing field user type_ in table Init.");
                    "unknown".to_owned()
                });
                let key_trigger: String = key.trigger.unwrap_or_else(|| {
                    println!("Missing field tridgger in table Init.");
                    "unknown".to_owned()
                });
                let key_code: String = key.code.unwrap_or_else(|| {
                    println!("Missing field code in table Init.");
                    "unknown".to_owned()
                });
                (key_type, key_trigger, key_code)
            }
            // in case no values for the keys are specified in the toml file
            None => {
                println!("Missing table database.");
                (
                    "unknown".to_owned(),
                    "unknown".to_owned(),
                    "unknown".to_owned(),
                )
            }
        };
        (key_type, key_trigger, key_code)
    }
}
