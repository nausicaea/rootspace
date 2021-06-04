#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(unused_mut))]
#![cfg_attr(test, allow(dead_code))]

use std::{
    io::Read,
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use log::error;
use serde::{Deserialize, Serialize};

use ecs::{EventQueue, Resources, SerializationName, System};

use crate::{event::EngineEvent, resources::settings::Settings, text_manipulation::tokenize};

#[derive(Serialize, Deserialize)]
pub struct DebugConsole {
    #[serde(skip, default = "default_worker_rx")]
    worker_rx: Receiver<String>,
}

impl DebugConsole {
    pub fn send_command(&self, cmd_line: &str, res: &Resources) {
        let settings = res.borrow::<Settings>();

        let tokens = tokenize(
            cmd_line,
            settings.command_escape,
            settings.command_quote,
            settings.command_punctuation,
        );

        res.borrow_mut::<EventQueue<EngineEvent>>()
            .send(EngineEvent::Command(tokens));
    }
}

impl SerializationName for DebugConsole {}

impl DebugConsole {
    pub fn builder() -> DebugConsoleBuilder<std::io::Stdin> {
        DebugConsoleBuilder::default()
    }

    fn try_read_line(&self) -> Option<String> {
        match self.worker_rx.try_recv() {
            Ok(s) => return Some(s),
            Err(mpsc::TryRecvError::Empty) => (),
            Err(e) => error!("Cannot receive data from the mpsc channel: {}", e),
        };
        None
    }
}

impl std::fmt::Debug for DebugConsole {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "DebugConsole({:?})", self.worker_rx)
    }
}

impl Default for DebugConsole {
    fn default() -> Self {
        DebugConsole::builder().build()
    }
}

impl<I> From<DebugConsoleBuilder<I>> for DebugConsole
where
    I: Read + Send + 'static,
{
    fn from(value: DebugConsoleBuilder<I>) -> Self {
        DebugConsole {
            worker_rx: spawn_worker(value.input_stream),
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

pub struct DebugConsoleBuilder<I> {
    input_stream: I,
}

impl<I> DebugConsoleBuilder<I> {
    pub fn with_input<T>(self, stream: T) -> DebugConsoleBuilder<T>
    where
        T: Read + Send + 'static,
    {
        DebugConsoleBuilder { input_stream: stream }
    }
}

impl<I> DebugConsoleBuilder<I>
where
    I: Read + Send + 'static,
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

fn default_worker_rx() -> Receiver<String> {
    spawn_worker(std::io::stdin())
}

fn spawn_worker<I: Read + Send + 'static>(mut input_stream: I) -> Receiver<String> {
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    #[cfg(not(test))]
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut byte = [0u8];

        loop {
            // Read a single byte from the input stream
            match input_stream.read(&mut byte) {
                Ok(0) => (),
                Ok(_) => {
                    // Listen for a line feed byte and send the entire buffer to the main thread.
                    // Otherwise, append the byte to the buffer.
                    if byte[0] == b'\n' {
                        let line = String::from_utf8_lossy(&buf).to_string();
                        if let Err(e) = tx.send(line) {
                            error!("Cannot send data over the mpsc channel (this will happen once after you load a new world state; just try again): {}", e);
                            return;
                        }
                        buf.clear()
                    } else {
                        buf.push(byte[0])
                    }
                }
                Err(e) => {
                    error!("Cannot read data from the input stream {}: {}", std::any::type_name::<I>(), e);
                    return;
                },
            }
        }
    });

    rx
}
