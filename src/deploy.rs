use std::path::Path;
use std::process::exit;
use dialoguer::{MultiSelect, Select};

use crate::config::{Config, Project, Server};
use crate::utils;
use dialoguer::theme::{ColorfulTheme};

pub struct DeployUtil {
    pub cmd: utils::CmdUtil,
    pub ssh: utils::SshUtil,
    pub servers: Vec<Server>,
    pub projects: Vec<Project>,
}

impl DeployUtil {
    pub fn new(config_path: String) -> DeployUtil {
        let cmd = utils::CmdUtil::new();
        let ssh = utils::SshUtil::new();
        let config = Config::read_config(config_path);
        let servers = config.servers;
        let projects = config.projects;
        DeployUtil { cmd, ssh, servers, projects }
    }

    fn status(&mut self, code: i32) {
        if code == 1 || code == 2 || code == 126 || code == 127 || code == 128 {
            panic!("命令执行错误！");
        }
    }

    fn deploy(&mut self, project: &Project, server: &Server) {
        let source_dir = project.source_dir.clone();
        self.cmd = utils::CmdUtil { current_dir: source_dir };
        let mut code = 0;
        for cmd in &project.deploy_before_cmd {
            code = self.cmd.exec(String::from(cmd));
            self.status(code);
        }
        self.ssh.login_with_pwd(server.host.clone(), server.port, server.user.clone(), server.password.clone());
        let file_path = Path::new(&project.source_dir).join(&project.target_name);
        let target_path = Path::new(&project.remote_dir);
        self.ssh.check_dir(target_path);
        println!("开始文件上传！");
        self.ssh.upload_file(file_path.as_path(), target_path.join(&project.target_name).as_path());

        std::fs::remove_file(file_path).unwrap();
        for cmd in &project.deploy_after_cmd {
            code = self.ssh.exec(String::from(cmd));
            self.status(code);
        }
    }

    fn choose_project_and_server(projects: &Vec<Project>, servers: &Vec<Server>) -> (usize, Vec<usize>) {
        let mut items: Vec<String> = Vec::new();
        for i in 0..projects.len() {
            items.push(format!("{}\n", projects.get(i).unwrap().label));
        }
        let theme = ColorfulTheme::default();
        let select_project = Select::with_theme(&theme).items(&items).default(0)
            .with_prompt("请选择需要部署的项目(默认选择第一个)").interact().unwrap();

        let mut select_server: Vec<usize> = Vec::new();
        items.clear();
        for i in 0..servers.len() {
            items.push(format!("{}", servers.get(i).unwrap().label));
        }
        let select: Vec<usize> = MultiSelect::with_theme(&theme).items(&items).with_prompt("请选择需要部署的项目(默认选择第一个)")
            .defaults(vec![true].as_slice()).interact().unwrap();
        for i in select {
            select_server.push(i);
        }
        (select_project, select_server)
    }

    pub fn run(&mut self) {
        let projects = self.projects.to_vec();
        let servers = self.servers.to_vec();
        let (project_index, server_index) = DeployUtil::choose_project_and_server(&projects, &servers);
        let project = projects.get(project_index).unwrap();
        for index in server_index {
            self.deploy(project, servers.get(index).unwrap());
        }
        exit(0);
    }
}