use std::path::Path;
use std::process::exit;
use dialoguer::{MultiSelect, Select};
use crate::config::{Config, Project, Server};
use crate::utils;
use dialoguer::console::Term;

pub struct DeployUtil {
    pub cmd: utils::CmdUtil,
    pub ssh: utils::SshUtil,
    pub servers: Vec<Server>,
    pub projects: Vec<Project>,
    pub term: Term,
}

impl DeployUtil {
    pub fn new(config_path: String) -> DeployUtil {
        let cmd = utils::CmdUtil::new();
        let ssh = utils::SshUtil::new();
        let config = Config::read_config(config_path);
        let servers = config.servers;
        let projects = config.projects;
        let term = Term::stdout();
        DeployUtil { cmd, ssh, servers, projects, term }
    }

    fn status(&mut self, code: i32) {
        if code == 1 || code == 2 || code == 126 || code == 127 || code == 128 {
            panic!("命令执行错误！");
        }
    }

    fn deploy(&mut self, project: &Project, server: &Server) {
        self.term.write_line(&*format!("{} 部署开始！", server.label)).unwrap();
        self.ssh.login_with_pwd(server.host.clone(), server.port.clone(), server.user.clone(), server.password.clone());
        let file_path = Path::new(&project.source_dir).join(&project.target_name);
        let target_path = Path::new(&project.remote_dir);
        self.ssh.check_dir(target_path);
        self.ssh.upload_file(file_path.as_path(), target_path.join(&project.target_name).as_path());
        std::fs::remove_file(file_path).unwrap();
        for cmd in &project.deploy_after_cmd {
            let code = self.ssh.exec(String::from(cmd));
            self.status(code);
        }
        self.term.write_line(&*format!("{} 部署完成！", server.label)).unwrap();
    }

    fn before_deploy(&mut self, project: &Project){
        self.term.write_line("开始部署前置操作").unwrap();
        let source_dir = project.source_dir.clone();
        self.cmd.change_path(source_dir);
        for cmd in &project.deploy_before_cmd {
            let code = self.cmd.exec(String::from(cmd));
            self.status(code);
        }
        self.term.write_line("完成部署前置操作!").unwrap();
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
        let select: Vec<usize> = MultiSelect::new().items(&items).with_prompt("请选择目标服务器").interact().unwrap();
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

        self.before_deploy(project);

        for index in server_index {
            self.deploy(project, servers.get(index).unwrap());
        }
        exit(0);
    }
}