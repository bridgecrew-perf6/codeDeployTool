use std::env;
use std::path::Path;

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
        path = match arg.contains("debug") {
            true => Path::new(env!("CARGO_MANIFEST_DIR")).join("config.yml").to_str().unwrap().parse().unwrap(),
            false => Path::new(&arg).join("config.yml").to_str().unwrap().parse().unwrap()
        }
    }
    let mut deploy = deploy::DeployUtil::new(path);
    deploy.run();
}
