use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};
use dialoguer::console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use ssh2::*;

fn status(code: i32) -> Result<()> {
    if code == 1 || code == 2 || code == 126 || code == 127 || code == 128 {
        Err(anyhow!("命令执行错误！"))
    } else {
        Ok(())
    }
}

#[derive(Clone)]
pub struct SshUtil {
    pub session: Session,
}

impl SshUtil {
    pub fn new(host: String, port: i64) -> Result<SshUtil> {
        let mut session = Session::new()?;
        let mut server = String::from(host);
        server.push(':');
        server.push_str(&port.to_string());
        match TcpStream::connect(server) {
            Ok(tcp) => {
                session.set_tcp_stream(tcp);
                session.set_compress(true);
                session.set_timeout(30000);
                session.handshake()?;
                Ok(SshUtil { session })
            }
            Err(err) => Err(anyhow!(err.to_string()))
        }
    }

    pub fn login_with_pwd(&mut self, name: String, password: String) -> Result<()> {
        Ok(self.session.userauth_password(&name, &password)?)
    }

    pub fn login_with_pubkey(&mut self, name: String, private_key: &Path) -> Result<()> {
        Ok(self.session.userauth_pubkey_file(&name, None, private_key, None)?)
    }

    pub fn exec(&mut self, cmd: String) -> Result<()> {
        let term = Term::stdout();
        term.write_line(&format!("执行命令：{}", cmd))?;
        let mut channel = self.session.channel_session()?;
        channel.exec(&cmd)?;
        let mut result = String::new();
        channel.read_to_string(&mut result)?;
        term.write_line(&format!("{}", result))?;
        result.clear();
        channel.stderr().read_to_string(&mut result)?;
        term.write_line(&format!("{}", result))?;
        channel.send_eof()?;
        channel.wait_eof()?;
        channel.wait_close()?;

        let status_code = channel.exit_status()?;
        Ok(status(status_code)?)
    }

    pub fn upload_file(&mut self, file_path: &Path, remote_path: &Path) -> Result<()> {
        let term = Term::stdout();
        term.write_line("开始文件上传！")?;
        let mut fs = File::open(file_path)?;
        let len = fs.metadata()?.len();
        let remote_file = self.session.scp_send(remote_path, 0o644, len.clone(), None);
        match remote_file {
            Err(e) => Err(anyhow!(e.to_string())),
            _ => {
                let mut buf = vec![0; (match len <= 1000 {
                    true => len,
                    false => len / 1000
                }) as usize];
                let mut remote_file = remote_file?;

                let pb = ProgressBar::new(len.clone());
                pb.set_style(ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})\n{msg}")
                    .progress_chars("#>-"));
                let mut pos = 0;
                while pos < len {
                    pos = pos + fs.read(&mut buf.as_mut_slice())? as u64;
                    remote_file.write_all(&buf.as_slice())?;
                    pb.set_position(pos);
                }
                pb.finish_with_message("文件上传完成!");
                remote_file.send_eof()?;
                remote_file.wait_eof()?;
                remote_file.close()?;
                remote_file.wait_close()?;
                Ok(())
            }
        }
    }

    pub fn check_dir(&mut self, path: &Path) -> Result<()> {
        match self.session.sftp() {
            Ok(sftp) => {
                match sftp.stat(path) {
                    Err(_e) => {
                        Ok(sftp.mkdir(path, 0o644)?)
                    }
                    Ok(_stat) => Ok(())
                }
            }
            Err(err) => Err(anyhow!(err.to_string()))
        }
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

    pub fn exec(&self, cmd: String) -> Result<()> {
        let term = Term::stdout();
        term.write_line(&format!("执行命令：{}", cmd))?;
        let mut out;
        if cfg!(target_os = "windows") {
            out = match self.current_dir.len() {
                0 => Command::new("cmd").stdin(Stdio::piped()).stdout(Stdio::piped()).arg("/c").arg(cmd).spawn()?,
                _ => {
                    //无法先切换到指定目录在执行命令
                    Command::new("cmd").stdin(Stdio::piped()).stdout(Stdio::piped()).arg("/c").arg(cmd).spawn()?
                }
            };
        } else {
            out = match self.current_dir.len() {
                0 => Command::new("sh").stdin(Stdio::piped()).stdout(Stdio::piped()).arg("-c").arg(cmd).spawn()?,
                _ => Command::new("sh").current_dir(&self.current_dir).stdin(Stdio::piped()).stdout(Stdio::piped()).arg("-c").arg(cmd).spawn()?
            };
        }
        let mut buf_reader = BufReader::new(out.stdout.take().unwrap());
        let mut line = String::new();
        let get_last_line = |lines: String| -> String {
            let array: Vec<&str> = lines.split("\n").collect();
            match array.len() {
                0 => lines,
                _ => array.get(array.len() - 2).unwrap().to_string()
            }
        };
        loop {
            match buf_reader.read_line(&mut line) {
                Ok(0) => break,
                _ => term.write_line(&format!("{}", get_last_line(line.clone())))?
            };
        };
        let status_code = out.wait().unwrap().code().unwrap();
        Ok(status(status_code)?)
    }
}