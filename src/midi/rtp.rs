use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::process::{Command, Output};
use std::str;

pub struct Rtp {
    pub stream: UnixStream,
}
impl Rtp {
    pub fn new(socket: String) -> Rtp {
        Rtp {
            stream: UnixStream::connect(socket).unwrap(),
        }
    }
    pub fn check_running() -> bool {
        let mut cmd = Command::new("ps");
        cmd.arg("-A");
        cmd.arg("-o");
        cmd.arg("comm=");
        str::from_utf8(&cmd.output().unwrap().stdout)
            .unwrap()
            .contains("rtpmidid")
    }
    pub fn start(&self) -> Output {
        let mut cmd = Command::new("systemctl");
        cmd.arg("start");
        cmd.arg("rtpmidid.service");
        let ouput = cmd.output().unwrap();
        self.init_connections();
        ouput
    }
    pub fn init_connections(&self) -> Output {
        let mut cmd = Command::new("");
        cmd.output().unwrap()
    }
    pub fn stop() -> Output {
        let mut cmd = Command::new("systemctl");
        cmd.arg("stop");
        cmd.arg("rtpmidid.service");
        cmd.output().unwrap()
    }
    pub fn restart() -> Output {
        let mut cmd = Command::new("systemctl");
        cmd.arg("restart");
        cmd.arg("rtpmidid.service");
        cmd.output().unwrap()
    }
    pub fn status() -> Output {
        let mut cmd = Command::new("systemctl");
        cmd.arg("status");
        cmd.arg("rtpmidid.service");
        cmd.output().unwrap()
    }
    pub fn send_command(&mut self, command: String) -> std::io::Result<String> {
        self.stream.write_all(command.as_bytes())?;
        let mut response = String::new();
        self.stream.read_to_string(&mut response)?;
        Ok(response)
    }
}
