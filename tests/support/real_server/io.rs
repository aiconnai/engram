use std::io::{BufRead, BufReader};
use std::process::{ChildStderr, ChildStdout};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

pub(crate) fn spawn_stdout_reader(stdout: ChildStdout) -> mpsc::Receiver<Result<String, String>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if sender.send(Ok(line)).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let _ = sender.send(Err(error.to_string()));
                    break;
                }
            }
        }
    });
    receiver
}

pub(crate) fn spawn_stderr_reader(
    stderr: ChildStderr,
    stderr_log: Arc<Mutex<String>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if let Ok(mut log) = stderr_log.lock() {
                        log.push_str(&line);
                    }
                }
                Err(error) => {
                    if let Ok(mut log) = stderr_log.lock() {
                        log.push_str(&error.to_string());
                    }
                    break;
                }
            }
        }
    })
}
