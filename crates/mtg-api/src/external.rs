//! [`ExternalAgent`]: an agent in a child process speaking newline-delimited JSON over
//! its stdin and stdout: one request per line to the process, one answer per line back.

use crate::agent::{Closed, GameOver, ProtocolAgent, ProtocolOptions, Transport};
use crate::request::{JsonAnswer, Request};
use mtg_engine::PlayerId;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Inner {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    transcript: Option<File>,
    seat: u8,
}

impl Inner {
    fn log(&mut self, dir: &str, line: &str) {
        if let Some(f) = self.transcript.as_mut() {
            let _ = writeln!(f, "{dir}{} {line}", self.seat);
        }
    }

    fn send(&mut self, line: &str) -> Result<(), Closed> {
        self.log(">", line);
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| Closed("stdin closed".into()))?;
        stdin
            .write_all(line.as_bytes())
            .and_then(|_| stdin.write_all(b"\n"))
            .and_then(|_| stdin.flush())
            .map_err(|e| Closed(format!("can't write to the agent process: {e}")))
    }

    fn receive(&mut self) -> Result<String, Closed> {
        loop {
            let mut line = String::new();
            match self.stdout.read_line(&mut line) {
                Ok(0) => return Err(Closed("the agent process closed its output".into())),
                Ok(_) => {
                    let l = line.trim();
                    if l.is_empty() {
                        continue;
                    }
                    self.log("<", l);
                    return Ok(l.to_string());
                }
                Err(e) => return Err(Closed(format!("can't read from the agent process: {e}"))),
            }
        }
    }

    /// Closes stdin and waits up to `wait` for the process to exit, then kills it.
    fn shut_down(&mut self, wait: Duration) {
        self.stdin = None;
        let start = Instant::now();
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) | Err(_) => return,
                Ok(None) if start.elapsed() >= wait => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    return;
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        self.shut_down(Duration::from_secs(2));
    }
}

/// A child process agent's connection. Cloning shares the connection, so the game's
/// agent and its owner (who sends the game-over message) can both use it.
#[derive(Clone)]
pub struct ChildTransport {
    inner: Arc<Mutex<Inner>>,
}

impl ChildTransport {
    /// Starts `command` (run by `sh -c`) as the agent for `seat`.
    pub fn spawn(command: &str, seat: PlayerId) -> std::io::Result<ChildTransport> {
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(command);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(command);
            c
        };
        Self::spawn_command(&mut cmd, seat)
    }

    /// Starts `cmd` as the agent for `seat` (its stderr goes to ours).
    pub fn spawn_command(cmd: &mut Command, seat: PlayerId) -> std::io::Result<ChildTransport> {
        let mut child = cmd
            .env("MTG_AGENT_SEAT", seat.0.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let stdin = child.stdin.take();
        let stdout = BufReader::new(child.stdout.take().expect("piped stdout"));
        Ok(ChildTransport {
            inner: Arc::new(Mutex::new(Inner {
                child,
                stdin,
                stdout,
                transcript: None,
                seat: seat.0,
            })),
        })
    }

    /// Appends every line sent (`>seat line`) and received (`<seat line`) to `file`.
    pub fn transcript(&self, file: File) {
        self.lock().transcript = Some(file);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Closes the agent's input and waits briefly for it to exit.
    pub fn shut_down(&self) {
        self.lock().shut_down(Duration::from_secs(2));
    }
}

impl Transport for ChildTransport {
    fn ask(&mut self, req: &Request) -> Result<Result<JsonAnswer, String>, Closed> {
        let line = serde_json::to_string(req).map_err(|e| Closed(e.to_string()))?;
        let mut inner = self.lock();
        inner.send(&line)?;
        let reply = inner.receive()?;
        Ok(JsonAnswer::parse(&reply))
    }

    fn game_over(&mut self, msg: &GameOver) {
        if let Ok(line) = serde_json::to_string(msg) {
            let mut inner = self.lock();
            let _ = inner.send(&line);
        }
    }
}

/// An agent in a child process.
pub type ExternalAgent = ProtocolAgent<ChildTransport>;

/// Starts `command` as the agent for `seat`. Returns the agent (to give to the game) and
/// a handle on its connection (to send the game-over message and shut it down).
pub fn spawn_external(
    command: &str,
    seat: PlayerId,
    options: ProtocolOptions,
) -> std::io::Result<(ExternalAgent, ChildTransport)> {
    let t = ChildTransport::spawn(command, seat)?;
    let agent = ProtocolAgent::new(seat, t.clone(), options).with_name(format!("cmd:{command}"));
    Ok((agent, t))
}
