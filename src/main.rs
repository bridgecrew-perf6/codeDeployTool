use std::env;

mod utils;
mod config;
mod deploy;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut path = String::from("./config.yml");
    if args.len() > 1 {
        path = args.get(1).unwrap().to_string();
    }

    let mut deploy = deploy::DeployUtil::new(path);
    deploy.run();

}
