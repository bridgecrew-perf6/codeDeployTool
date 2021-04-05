extern crate regex;

use std::fs::OpenOptions;
use std::io::Read;
use anyhow::{anyhow, Result};
use regex::Regex;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub servers: Vec<Server>,
    pub projects: Vec<Project>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub name: String,
    pub host: String,
    pub port: i64,
    pub user: String,
    pub password: String,
    pub private_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub name: String,
    pub source_dir: String,
    pub remote_dir: String,
    pub target_name: String,
    pub deploy_cmd: DeployCmd,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeployCmd {
    pub before: Vec<String>,
    pub after: Vec<String>,
}


impl Config {

    pub fn replace_reg(cmd: String, project: &Project) -> Result<String> {
        let mut reg = Regex::new(r"(\{target_name\})")?;
        let mut value = reg.replace_all(cmd.as_str(), project.target_name.clone().as_str()).to_string();
        reg = Regex::new(r"(\{remote_dir\})")?;
        value = reg.replace_all(value.as_str(), project.remote_dir.clone().as_str()).to_string();
        reg = Regex::new(r"(\{source_dir\})")?;
        value = reg.replace_all(value.as_str(), project.source_dir.clone().as_str()).to_string();
        Ok(value)
    }

    pub fn read_config(path: String) -> Result<Config> {
        match OpenOptions::new().read(true).open(path) {
            Ok(mut fs) => {
                let mut config_str = String::new();
                fs.read_to_string(&mut config_str)?;
                let config: Config = toml::from_str(&*config_str)?;
                Ok(config)
            }
            Err(err) => Err(anyhow!(err.to_string()))
        }
    }
}
