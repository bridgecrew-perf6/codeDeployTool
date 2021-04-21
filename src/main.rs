extern crate clap;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::env;
use std::path::Path;
use clap::{App, Arg};

mod utils;
mod deploy;
mod config;


fn main() {
    let matchs = App::new("DeployTool").version("1.0")
        .author("Rookie. <gb880327@189.cn>")
        .about("
        配置文件使用toml配置格式，private_key和password二选一，优先使用private_key登陆！
        before 和 after 有多个配置时会使用选择的配置，当只有一个配置时默认使用不需选择(多个配置时before和after的配置项名称必须相同)
        配置信息说明：
            [server.test_server]                    #服务器名称
                host = '127.0.0.1'                  #服务器地址
                port = 22                           #SSH端口
                user = 'root'                       #服务器用户名
                password = '1'                      #服务器密码(填写了private_key此项可为空)
                private_key = ''                    #秘钥文件路径(免密登陆)
            [project.demo]                          #项目名称
                source_dir = ''                     #项目路径
                remote_dir = ''                     #服务器部署路径
                target_name = ''                    #部署文件名称
                [project.demo.before]               #部署前操作(即文件上传前操作，例如执行项目编译压缩等操作)
                 test = ['ls']                      #不同情况不同配置
                [project.demo.after]                #部署操作(即文件上传完成后再服务器上需要完成的操作)
                 test = ['ls']
        ")
        .arg(Arg::with_name("config").short("c").long("config").value_name("FILE").help("指定自定义配置文件"))
        .get_matches();

    let path;
    match matchs.value_of("config") {
        Some(config) => path = config.to_string(),
        None => {
            let mut config_path = env::current_exe().unwrap();
            config_path.pop();
            let arg: String = config_path.to_str().unwrap_or("").parse().unwrap();
            path = match arg.contains(if cfg!(target_os = "windows") { "\\target\\debug" } else { "/target/debug" }) {
                true => Path::new(env!("CARGO_MANIFEST_DIR")).join("config.toml").to_str().unwrap().parse().unwrap(),
                false => Path::new(&arg).join("config.toml").to_str().unwrap().parse().unwrap()
            }
        }
    }
    let mut deploy = deploy::DeployUtil::new(path);
    deploy.run().unwrap();
}
