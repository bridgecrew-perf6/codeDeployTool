use std::path::Path;
use std::process::exit;

use anyhow::{anyhow, Result};
use dialoguer::{MultiSelect, Select};
use dialoguer::console::{style, Term};
use crate::config::{Config, Project, Server};
use crate::utils;
use crate::utils::SshUtil;

pub struct DeployUtil {
    pub cmd: utils::CmdUtil,
    pub config: Config,
    pub term: Term,
}

impl DeployUtil {
    pub fn new(config_path: String) -> DeployUtil {
        let cmd = utils::CmdUtil::new();
        let config = Config::read_config(config_path);
        let term = Term::stdout();

        DeployUtil { cmd, config, term }
    }

    fn login_server(&mut self, host: &String, port: &i64, user: &String, password: &String) -> Result<SshUtil> {
        match SshUtil::new(host.clone(), port.clone()) {
            Ok(mut ssh) => {
                match self.config.private_key.is_empty() {
                    true => {
                        ssh.login_with_pwd(user.clone(), password.clone())?;
                    }
                    false => {
                        let private_key = Path::new(&self.config.private_key);
                        ssh.login_with_pubkey(user.clone(), private_key)?;
                    }
                }
                Ok(ssh)
            }
            Err(err) => Err(anyhow!(err.to_string()))
        }
    }

    fn deploy(&mut self, project: &Project, server: &Server) -> Result<()> {
        self.term.write_line(&format!("{} 部署开始！", server.label))?;
        match self.login_server(&server.host, &server.port, &server.user, &server.password) {
            Err(err) => Err(anyhow!(err.to_string())),
            Ok(mut ssh) => {
                let file_path = Path::new(&project.source_dir).join(&project.target_name);
                let target_path = Path::new(&project.remote_dir);
                ssh.check_dir(target_path)?;
                ssh.upload_file(file_path.as_path(), target_path.join(&project.target_name).as_path())?;
                std::fs::remove_file(file_path)?;
                for cmd in &project.deploy_after_cmd {
                    ssh.exec(String::from(cmd))?;
                }
                self.term.write_line(&format!("{} 部署完成！", server.label))?;
                Ok(())
            }
        }
    }

    fn before_deploy(&mut self, project: &Project) -> Result<()> {
        self.term.write_line("开始部署前置操作")?;
        let source_dir = project.source_dir.clone();
        let target_file = Path::new(&source_dir).join(&project.target_name);
        if target_file.exists() {
            std::fs::remove_file(target_file)?;
        }

        self.cmd.change_path(source_dir);
        for cmd in &project.deploy_before_cmd {
            self.cmd.exec(String::from(cmd))?;
        }
        self.term.write_line("完成部署前置操作!")?;
        Ok(())
    }

    fn choose_project_and_server(projects: &Vec<Project>, servers: &Vec<Server>) -> (usize, Vec<usize>) {
        let mut items: Vec<String> = Vec::new();
        for i in 0..projects.len() {
            items.push(format!("{}\n", projects.get(i).unwrap().label));
        }
        let select_project = Select::new().items(&items).default(0)
            .with_prompt("请选择需要部署的项目(默认选择第一个)").interact().unwrap();

        let mut select_server: Vec<usize> = Vec::new();
        items.clear();
        for i in 0..servers.len() {
            items.push(format!("{}", servers.get(i).unwrap().label));
        }
        let mut select: Vec<usize> = MultiSelect::new().items(&items).with_prompt("请选择目标服务器").interact().unwrap();
        while select.is_empty() {
            select = MultiSelect::new().items(&items).with_prompt("请选择目标服务器").interact().unwrap();
        }
        for i in select {
            select_server.push(i);
        }
        (select_project, select_server)
    }

    pub fn run(&mut self) -> Result<()> {
        let projects = self.config.projects.to_vec();
        let servers = self.config.servers.to_vec();
        let (project_index, server_index) = DeployUtil::choose_project_and_server(&projects, &servers);
        let project = projects.get(project_index).unwrap();

        match self.before_deploy(project) {
            Err(err) => self.term.write_line(&style(format!("{}", err.to_string())).red().cyan().to_string())?,
            Ok(()) => {}
        };

        for index in server_index {
            let server = servers.get(index).unwrap();
            match self.deploy(project, server) {
                Err(err) => {
                    self.term.write_line(&style(format!("服务器 {} 部署失败！({})", &server.label, err.to_string())).red().cyan().to_string())?;
                    continue;
                }
                Ok(()) => {}
            }
        }
        exit(0);
    }
}