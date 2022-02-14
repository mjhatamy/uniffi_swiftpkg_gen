use std::ffi::OsStr;
use std::io;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::io::{BufReader, BufRead};
use super::ext::*;
use colored::Colorize;

#[derive(Debug)]
pub(crate) struct CommandBuilder {
    default_shell: String
}

impl CommandBuilder {
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        let default_shell = Command::new("sh")
            .arg("-c")
            .args(["echo $SHELL"])
            .output()
            .expect("Unable to get default shell. Check your system if any shell is installed")
            .utf8_string().first()
            .unwrap().clone();
        CommandBuilder {
            default_shell
        }
    }

    #[allow(unused)]
    pub(crate) fn args<I, S>(&self, params: I) -> io::Result<Output>
        where
            I: IntoIterator<Item = S>,
            S: AsRef<OsStr> {
        let mut params = params;

        Command::new(&self.default_shell)
            .arg("-c")
            .args(params).output()
    }

    #[allow(unused)]
    pub(crate) fn args_stream<I, S>(&self, params: I) -> ExitStatus
        where
            I: IntoIterator<Item = S>,
            S: AsRef<OsStr> {
        let mut params = params;

        let mut cmd = Command::new(&self.default_shell)
            .arg("-c")
            .args(params)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            //.stderr(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let stdout = cmd.stdout.as_mut().unwrap();
            let stdout_reader = BufReader::new(stdout);
            let stdout_lines = stdout_reader.lines();

            for line in stdout_lines.filter_map(|line| line.ok()) {
                println!("{:?}", line);
            }
        }


        let status = cmd.wait().unwrap();

        if let Some(err) = cmd.stderr {
            let err_stream = BufReader::new(err).lines();
            for line in err_stream.filter_map(|line| line.ok()) {
                println!("ERR:::: {:?}", line.red());
            }
        }

        status
    }
}