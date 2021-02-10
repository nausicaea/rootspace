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

#[derive(Debug)]
pub struct DebugConsole {
    escape_char: char,
    quote_char: char,
    punctuation_char: char,
    worker_rx: Receiver<Result<String, DebugConsoleError>>,
}

impl DebugConsole {
    pub fn builder() -> DebugConsoleBuilder<std::io::Stdin> {
        DebugConsoleBuilder::default()
    }

    pub fn send_command(&self, cmd_line: &str, res: &Resources) {
        let tokens = tokenize(
            cmd_line,
            self.escape_char,
            self.quote_char,
            self.punctuation_char
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

impl<S> From<DebugConsoleBuilder<S>> for DebugConsole
where
    S: Read + Send + 'static,
{
    fn from(value: DebugConsoleBuilder<S>) -> Self {
        let (tx, rx) = channel();

        let mut input_stream = value.input_stream;

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

        DebugConsole {
            escape_char: value.escape_char,
            quote_char: value.quote_char,
            punctuation_char: value.punctuation_char,
            worker_rx: rx,
        }
    }
}

impl System for DebugConsole {
    fn name(&self) -> &'static str {
        stringify!(DebugConsole)
    }

    fn run(&mut self, res: &Resources, _: &Duration, _: &Duration) {
        if let Some(line) = self.try_read_line() {
            self.send_command(&line, res);
        }
    }
}

pub struct DebugConsoleBuilder<S> {
    input_stream: S,
    escape_char: char,
    quote_char: char,
    punctuation_char: char,
}

impl<S> DebugConsoleBuilder<S> {
    pub fn with_input<T>(self, stream: T) -> DebugConsoleBuilder<T>
        where
            T: Read + Send + 'static,
    {
        DebugConsoleBuilder {
            input_stream: stream,
            escape_char: self.escape_char,
            quote_char: self.quote_char,
            punctuation_char: self.punctuation_char,
        }
    }
}

impl<S> DebugConsoleBuilder<S>
where
    S: Read + Send + 'static,
{

    pub fn with_escape_char(mut self, chr: char) -> Self {
        self.escape_char = chr;
        self
    }

    pub fn with_quote_char(mut self, chr: char) -> Self {
        self.quote_char = chr;
        self
    }

    pub fn with_punctuation_char(mut self, chr: char) -> Self {
        self.punctuation_char = chr;
        self
    }

    pub fn build(self) -> DebugConsole {
        DebugConsole::from(self)
    }
}

impl Default for DebugConsoleBuilder<std::io::Stdin> {
    fn default() -> Self {
        DebugConsoleBuilder {
            input_stream: std::io::stdin(),
            escape_char: '\\',
            quote_char: '"',
            punctuation_char: ';',
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