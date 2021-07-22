use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;

use anyhow::{anyhow, Result};
use regex::Regex;
use toml::Value;
use toml::value::Table;

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
    pub identity_file: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub name: String,
    pub source_dir: String,
    pub remote_dir: String,
    pub target_name: String,
    pub before: HashMap<String, Vec<String>>,
    pub after: HashMap<String, Vec<String>>,
}


impl Config {
    fn replace_with_reg(reg: &Regex, value: String, replace: String) -> String {
        reg.replace_all(&*value, replace.as_str()).to_string()
    }

    fn get_table(value: &Value, key: String) -> Option<&Table> {
        match value.get(key) {
            Some(val) => Some(val.as_table().unwrap()),
            None => None
        }
    }

    fn get_str(value: &Value, key: &str) -> String {
        match value.get(key) {
            Some(val) => val.as_str().unwrap().to_string(),
            None => "".to_string()
        }
    }

    fn get_int(value: &Value, key: &str) -> i64 {
        match value.get(key) {
            Some(val) => val.as_integer().unwrap(),
            None => 0
        }
    }

    fn get_map(value: &Value, key: &str) -> HashMap<String, Vec<String>> {
        let target_name_reg = Regex::new(r"(\{target_name\})").unwrap();
        let remote_dir_reg = Regex::new(r"(\{remote_dir\})").unwrap();
        let source_dir_reg = Regex::new(r"(\{source_dir\})").unwrap();

        let source_dir = Config::get_str(value, "source_dir");
        let remote_dir = Config::get_str(value, "remote_dir");
        let target_name = Config::get_str(value, "target_name");

        match value.get(key) {
            Some(val) => {
                let mut data = HashMap::new();
                let table = val.as_table().unwrap();
                for sub_key in table.keys() {
                    let item = table.get(sub_key).unwrap().as_array().unwrap();
                    let vec: Vec<String> = item.iter().map(|x| x.as_str().unwrap().to_string())
                        .map(|x| Config::replace_with_reg(&target_name_reg, x.clone(), target_name.clone()))
                        .map(|x| Config::replace_with_reg(&remote_dir_reg, x.clone(), remote_dir.clone()))
                        .map(|x| Config::replace_with_reg(&source_dir_reg, x.clone(), source_dir.clone()))
                        .collect();
                    data.insert(sub_key.to_string(), vec);
                }
                data
            }
            None => HashMap::new()
        }
    }

    pub fn read_config(path: String) -> Result<Config> {
        match OpenOptions::new().read(true).open(path) {
            Ok(mut fs) => {
                let mut config_str = String::new();
                fs.read_to_string(&mut config_str)?;
                let value = config_str.parse::<toml::Value>()?;
                let server = Config::get_table(&value, "server".to_string()).unwrap();
                let project = Config::get_table(&value, "project".to_string()).unwrap();
                let mut servers: Vec<Server> = vec![];
                let mut projects: Vec<Project> = vec![];

                for key in server.keys() {
                    let item = server.get(key).unwrap();
                    servers.push(Server {
                        name: key.to_string(),
                        host: Config::get_str(&item, "host"),
                        port: Config::get_int(&item, "port"),
                        user: Config::get_str(&item, "user"),
                        password: Config::get_str(&item, "password"),
                        private_key: Config::get_str(&item, "private_key"),
                        identity_file: Config::get_str(&item, "identity_file"),
                    });
                }
                for key in project.keys() {
                    let item = project.get(key).unwrap();
                    let before_cmd = Config::get_map(&item, "before");
                    let after_cmd = Config::get_map(&item, "after");

                    projects.push(Project {
                        name: key.to_string(),
                        source_dir: Config::get_str(&item, "source_dir"),
                        remote_dir: Config::get_str(&item, "remote_dir"),
                        target_name: Config::get_str(&item, "target_name"),
                        before: before_cmd,
                        after: after_cmd,
                    });
                }

                Ok(Config { servers, projects })
            }
            Err(err) => Err(anyhow!(err.to_string()))
        }
    }
}
