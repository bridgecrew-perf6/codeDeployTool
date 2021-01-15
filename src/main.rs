use std::env;
use std::path::Path;
use std::env::consts::OS as target_os;

mod utils;
mod config;
mod deploy;


fn main() {
    let args: Vec<String> = env::args().collect();
    let path;
    if args.len() > 1 {
        let arg: String = args.get(1).unwrap().to_string();
        path = Path::new(&arg).to_str().unwrap().parse().unwrap();
    } else {
        let mut config_path = env::current_exe().unwrap();
        config_path.pop();
        let arg: String = config_path.to_str().unwrap().parse().unwrap();
        path = match arg.contains(if cfg!(target_os = "windows") { "\\target\\debug" } else { "/target/debug" }) {
            true => Path::new(env!("CARGO_MANIFEST_DIR")).join("config.yml").to_str().unwrap().parse().unwrap(),
            false => Path::new(&arg).join("config.yml").to_str().unwrap().parse().unwrap()
        }
    }
    let mut deploy = deploy::DeployUtil::new(path);
    deploy.run();
}
