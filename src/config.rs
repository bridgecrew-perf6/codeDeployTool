extern crate regex;

use std::fs::File;
use std::io::Read;
use regex::{Regex};
use rusty_yaml::Yaml;


#[derive(Debug)]
pub struct Config {
    pub servers: Vec<Server>,
    pub projects: Vec<Project>,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub label: String,
    pub host: String,
    pub port: i64,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub label: String,
    pub source_dir: String,
    pub remote_dir: String,
    pub target_name: String,
    pub deploy_before_cmd: Vec<String>,
    pub deploy_after_cmd: Vec<String>,
}


impl Config {
    fn get_str(yaml: &Yaml, name: String) -> String {
        Config::replace_str(yaml.get_section(name).unwrap().to_string())
    }
    fn get_int(yaml: &Yaml, name: String) -> i64 {
        yaml.get_section(name).unwrap().to_string().parse().unwrap()
    }
    fn replace_str(mut x: String) -> String {
        x = x.replace("- ", "");
        if x.starts_with("\"") {
            x.remove(0);
            x.remove(x.len() - 1);
        }
        x
    }
    fn replace_with_reg(reg: Regex, value:String, replace:String)-> String{
        reg.replace_all(&*value, replace.as_str()).to_string()
    }
    fn get_vec(yaml: &Yaml, name: String, target_name:String, remote_dir: String, source_dir: String) -> Vec<String> {
        let doc = yaml.get_section(name).unwrap().to_string();
        doc.split("\n").map(|x| Config::replace_str(x.parse().unwrap()))
            .map(|x| Config::replace_with_reg(Regex::new(r"(\{targetName\})").unwrap(),x.clone(), target_name.clone()))
            .map(|x| Config::replace_with_reg(Regex::new(r"(\{remoteDir\})").unwrap(),x.clone(), remote_dir.clone()))
            .map(|x| Config::replace_with_reg(Regex::new(r"(\{sourceDir\})").unwrap(),x.clone(), source_dir.clone()))
            .collect()
    }

    pub fn read_config(path: String) -> Config {
        let mut buffer = String::new();
        File::open(path).expect("配置文件读取错误！")
            .read_to_string(&mut buffer).unwrap();
        let doc = Yaml::from(&*buffer);

        let mut servers: Vec<Server> = Vec::new();
        match doc.has_section("server") {
            true => {
                let server_doc = doc.get_section("server").unwrap();
                for name in server_doc.get_section_names().unwrap() {
                    let label = name.clone();
                    let server_item = server_doc.get_section(name).unwrap();
                    let host = Config::get_str(&server_item, "host".to_string());
                    let port = Config::get_int(&server_item, "port".to_string());
                    let user = Config::get_str(&server_item, "user".to_string());
                    let password = Config::get_str(&server_item, "password".to_string());
                    servers.push(Server { label, host, port, user, password });
                }
            }
            false => panic!("请添加服务器配置信息！")
        };
        let mut projects: Vec<Project> = Vec::new();
        match doc.has_section("project") {
            true => {
                let project_doc = doc.get_section("project").unwrap();
                for name in project_doc.get_section_names().unwrap() {
                    let label = name.clone();
                    let project_item = project_doc.get_section(name).unwrap();
                    let source_dir = Config::get_str(&project_item, "sourceDir".to_string());
                    let remote_dir = Config::get_str(&project_item, "remoteDir".to_string());
                    let target_name = Config::get_str(&project_item, "targetName".to_string());
                    let deploy_cmd = project_item.get_section("deployCmd").unwrap();
                    let deploy_before_cmd = Config::get_vec(&deploy_cmd, "before".to_string(), target_name.clone(), remote_dir.clone(), source_dir.clone());
                    let deploy_after_cmd = Config::get_vec(&deploy_cmd, "after".to_string(),target_name.clone(), remote_dir.clone(), source_dir.clone());
                    projects.push(Project { label, source_dir, remote_dir, target_name, deploy_before_cmd, deploy_after_cmd });
                }
            }
            false => panic!("请添加项目配置信息！")
        };
        Config { servers, projects }
    }
}
