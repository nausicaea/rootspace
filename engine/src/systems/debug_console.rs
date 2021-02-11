#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(unused_mut))]
#![cfg_attr(test, allow(dead_code))]

use crate::{event::EngineEvent, text_manipulation::tokenize};
use ecs::{EventQueue, Resources, System};
use log::{error, warn};
#[cfg(not(test))]
use std::thread::spawn;
use std::{
    io::{self, Read},
    string,
    sync::mpsc::{self, channel, Receiver},
    time::Duration,
};
use thiserror::Error;
use crate::resources::Settings;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugConsole {
    #[serde(skip, default = "default_worker_rx")]
    worker_rx: Receiver<Result<String, DebugConsoleError>>,
}

impl DebugConsole {
    pub fn builder() -> DebugConsoleBuilder<std::io::Stdin> {
        DebugConsoleBuilder::default()
    }

    pub fn send_command(&self, cmd_line: &str, res: &Resources) {
        let settings = res.borrow::<Settings>();

        let tokens = tokenize(
            cmd_line,
            settings.command_escape,
            settings.command_quote,
            settings.command_punctuation
        );

        res.borrow_mut::<EventQueue<EngineEvent>>()
            .send(EngineEvent::Command(tokens));
    }

    fn try_read_line(&self) -> Option<String> {
        match self.worker_rx.try_recv() {
            Ok(Ok(s)) => return Some(s),
            Ok(Err(e)) => warn!("{}", e),
            Err(mpsc::TryRecvError::Empty) => (),
            Err(e) => error!("{}", e),
        };
        None
    }
}

impl Default for DebugConsole {
    fn default() -> Self {
        DebugConsole::builder().build()
    }
}

fn default_worker_rx() -> Receiver<Result<String, DebugConsoleError>> {
    spawn_worker(std::io::stdin())
}

fn spawn_worker<S: Read + Send + 'static>(mut input_stream: S) -> Receiver<Result<String, DebugConsoleError>> {
    let (tx, rx) = channel();

    #[cfg(not(test))]
    spawn(move || {
        let mut buf = Vec::new();
        let mut byte = [0u8];

        loop {
            match input_stream.read(&mut byte) {
                Ok(0) => (),
                Ok(_) => {
                    if byte[0] == 0x0A {
                        tx.send(match String::from_utf8(buf.clone()) {
                            Ok(l) => Ok(l),
                            Err(e) => Err(DebugConsoleError::Utf8Error(e)),
                        })
                            .expect("Unable to send input from stdin via mpsc channel");
                        buf.clear()
                    } else {
                        buf.push(byte[0])
                    }
                }
                Err(e) => tx
                    .send(Err(DebugConsoleError::IoError(e)))
                    .expect("Unable to send error information via mpsc channel"),
            }
        }
    });

    rx
}

impl<S> From<DebugConsoleBuilder<S>> for DebugConsole
where
    S: Read + Send + 'static,
{
    fn from(value: DebugConsoleBuilder<S>) -> Self {
        let mut input_stream = value.input_stream;

        DebugConsole {
            worker_rx: spawn_worker(input_stream),
        }
    }
}

impl System for DebugConsole {
    fn run(&mut self, res: &Resources, _: &Duration, _: &Duration) {
        if let Some(line) = self.try_read_line() {
            self.send_command(&line, res);
        }
    }
}

pub struct DebugConsoleBuilder<S> {
    input_stream: S,
}

impl<S> DebugConsoleBuilder<S> {
    pub fn with_input<T>(self, stream: T) -> DebugConsoleBuilder<T>
        where
            T: Read + Send + 'static,
    {
        DebugConsoleBuilder {
            input_stream: stream,
        }
    }
}

impl<S> DebugConsoleBuilder<S>
where
    S: Read + Send + 'static,
{
    pub fn build(self) -> DebugConsole {
        DebugConsole::from(self)
    }
}

impl Default for DebugConsoleBuilder<std::io::Stdin> {
    fn default() -> Self {
        DebugConsoleBuilder {
            input_stream: std::io::stdin(),
        }
    }
}

#[derive(Debug, Error)]
enum DebugConsoleError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    Utf8Error(#[from] string::FromUtf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdin_builder() {
        let _dc: DebugConsole = DebugConsole::builder()
            .with_escape_char('!')
            .build();
    }

    #[test]
    fn test_empty_stream_builder() {
        let _dc: DebugConsole = DebugConsole::builder()
            .with_input(std::io::empty())
            .build();
    }
}