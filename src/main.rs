extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::env;
use std::path::Path;

use clap::{App, Arg};

mod utils;
mod config;
mod deploy;


fn main() {
    let matchs = App::new("DeployTool").version("1.0")
        .author("Rookie. <gb880327@189.cn>")
        .about("
        配置文件使用toml配置格式，private_key和password二选一，优先使用private_key登陆！
        配置信息说明：
            [[servers]]
                name = 'test_server'                #服务名称
                host = '127.0.0.1'                  #服务器地址
                port = 22                           #SSH端口
                user = 'root'                       #服务器用户名
                password = '1'                      #服务器密码(填写了private_key此项可为空)
                private_key = ''                    #秘钥文件路径(免密登陆)
            [[projects]]
                name = ''                           #项目名称
                source_dir = ''                     #项目路径
                remote_dir = ''                     #服务器部署路径
                target_name = ''                    #部署文件名称
                deploy_cmd.before = ['']            #部署前操作(即文件上传前操作，例如执行项目编译压缩等操作)
                deploy_cmd.after = ['']             #部署操作(即文件上传完成后再服务器上需要完成的操作)
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
