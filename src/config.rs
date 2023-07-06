use std::{fs, path::Path};
use serde::{Serialize, Deserialize};


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: String,
    pub root: String,
    pub username: Option<String>,
    pub password: Option<String>
}

impl ServerConfig {
    pub fn from_file(path: &str) -> ServerConfig {
        let file = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => panic!("Could not read file! {e}")
        };
        
        let mut config: ServerConfig = match serde_yaml::from_str(&file) {
            Ok(res) => res,
            Err(e) => panic!("Could not parse YAML! {e}")
        };
        
        config.root = match fs::canonicalize(Path::new(&config.root)) {
            Ok(path) => path.to_str().unwrap().into(),
            Err(e) => panic!("Config error: {e}")
        };

        config
    }
}
