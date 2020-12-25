use std::env::consts::OS as target_os;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};

use indicatif::{ProgressBar, ProgressStyle};
use ssh2::*;

#[derive(Clone)]
pub struct SshUtil {
    pub session: Session,
}

impl SshUtil {
    pub fn new() -> SshUtil {
        SshUtil { session: Session::new().unwrap() }
    }

    fn init(&mut self, host: String, port: i64) {
        let mut server = String::from(host);
        server.push(':');
        server.push_str(&*port.to_string());
        let tcp = TcpStream::connect(server).unwrap();
        self.session.set_tcp_stream(tcp);
        self.session.set_compress(true);
        self.session.set_timeout(30000);
        self.session.handshake().unwrap();
    }
    pub fn login_with_pwd(&mut self, host: String, port: i64, name: String, password: String) {
        self.init(host, port);
        self.session.userauth_password(&name, &password).expect("服务器登陆失败！");
    }

    pub fn exec(&mut self, cmd: String) -> i32 {
        writeln!(std::io::stdout(), "执行命令：{}", cmd).unwrap();
        let mut channel = self.session.channel_session().unwrap();
        channel.exec(&cmd).expect("shell执行出错！");
        let mut result = String::new();
        channel.read_to_string(&mut result).unwrap();
        write!(std::io::stdout(), "{}", result).unwrap();
        result.clear();
        channel.stderr().read_to_string(&mut result).unwrap();
        write!(std::io::stdout(), "{}", result).unwrap();
        channel.send_eof().unwrap();
        channel.wait_eof().unwrap();
        channel.wait_close().unwrap();
        channel.exit_status().unwrap()
    }

    pub fn upload_file(&mut self, file_path: &Path, remote_path: &Path) {
        writeln!(std::io::stdout(), "开始文件上传！").unwrap();
        let mut fs = match File::open(file_path) {
            Ok(file) => file,
            Err(e) => panic!("{}", e.to_string())
        };
        let len = fs.metadata().unwrap().len();
        let remote_file = self.session.scp_send(remote_path, 0o644, len.clone(), None);
        match remote_file {
            Err(e) => (),
            _ => {
                let mut buf = vec![0; (match len <= 1000 {
                    true => len,
                    false => len / 1000
                }) as usize];
                let mut remote_file = remote_file.unwrap();

                let pb = ProgressBar::new(fs.metadata().unwrap().len());
                pb.set_style(ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})\n{msg}")
                    .progress_chars("#>-"));
                let mut pos = 0;
                while pos < len {
                    pos = pos + fs.read(&mut buf.as_mut_slice()).unwrap() as u64;
                    remote_file.write_all(&buf.as_slice()).unwrap();
                    pb.set_position(pos);
                }
                pb.finish_with_message("文件上传完成!");
                remote_file.send_eof().unwrap();
                remote_file.wait_eof().unwrap();
                remote_file.close().unwrap();
                remote_file.wait_close().unwrap();
            }
        }
    }

    pub fn check_dir(&mut self, path: &Path) {
        let sftp = self.session.sftp().unwrap();
        match sftp.stat(path) {
            Err(e) => sftp.mkdir(path, 0o644).unwrap(),
            Ok(stat) => ()
        };
    }
}

#[derive(Clone)]
pub struct CmdUtil {
    pub current_dir: String
}

impl CmdUtil {
    pub fn new() -> CmdUtil {
        CmdUtil { current_dir: String::from("") }
    }

    pub fn change_path(&mut self, path: String) {
        self.current_dir = path;
    }

    pub fn exec(&self, cmd: String) -> i32 {
        writeln!(std::io::stdout(), "执行命令：{}", cmd).unwrap();
        let mut out;
        if cfg!(target_os = "windows") {
            out = match self.current_dir.len() {
                0 => Command::new("cmd").stdin(Stdio::piped()).stdout(Stdio::piped()).arg("/c").arg(cmd).spawn().unwrap(),
                _ => {
                    Command::new("cmd").stdin(Stdio::piped()).stdout(Stdio::piped()).arg("/c").arg(cmd).spawn().unwrap()
                }
            };
        } else {
            out = match self.current_dir.len() {
                0 => Command::new("sh").stdin(Stdio::piped()).stdout(Stdio::piped()).arg("-c").arg(cmd).spawn().unwrap(),
                _ => Command::new("sh").current_dir(&self.current_dir).stdin(Stdio::piped()).stdout(Stdio::piped()).arg("-c").arg(cmd).spawn().unwrap()
            };
        }
        let mut buf_reader = BufReader::new(out.stdout.take().unwrap());
        let mut line = String::new();
        loop {
            match buf_reader.read_line(&mut line) {
                Ok(0) => break,
                _ => write!(std::io::stdout(), "{}", &line).unwrap()
            };
        };
        out.wait().unwrap().code().unwrap()
    }
}