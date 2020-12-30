#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(unused_mut))]
#![cfg_attr(test, allow(dead_code))]

use crate::{event::EngineEvent, text_manipulation::tokenize};
use ecs::{EventQueue, Resources, System};
use log::{error, warn};
#[cfg(not(test))]
use std::thread::spawn;
use std::{
    collections::HashSet,
    io::{self, Read},
    string,
    sync::mpsc::{self, channel, Receiver},
    time::Duration,
};
use thiserror::Error;

pub struct DebugConsole {
    escape_char: char,
    quote_char: char,
    separator_chars: HashSet<char>,
    worker_rx: Receiver<Result<String, DebugConsoleError>>,
}

impl DebugConsole {
    pub fn new<S>(
        mut input_stream: S,
        escape_char: Option<char>,
        quote_char: Option<char>,
        separator_chars: &[char],
    ) -> Self
    where
        S: Read + Send + 'static,
    {
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

        DebugConsole {
            escape_char: escape_char.unwrap_or('\\'),
            quote_char: quote_char.unwrap_or('"'),
            separator_chars: separator_chars.iter().cloned().collect(),
            worker_rx: rx,
        }
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
        DebugConsole::new(io::stdin(), Some('\\'), Some('"'), &[';'])
    }
}

impl System for DebugConsole {
    fn name(&self) -> &'static str {
        stringify!(DebugConsole)
    }

    fn run(&mut self, res: &Resources, _: &Duration, _: &Duration) {
        self.try_read_line()
            .map(|l| tokenize(l, self.escape_char, self.quote_char, &self.separator_chars))
            .map(|t| {
                res.borrow_mut::<EventQueue<EngineEvent>>()
                    .send(EngineEvent::Command(t))
            });
    }
}

#[derive(Debug, Error)]
enum DebugConsoleError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    Utf8Error(#[from] string::FromUtf8Error),
}
