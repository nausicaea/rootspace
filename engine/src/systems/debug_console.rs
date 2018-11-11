#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(test, allow(unused_mut))]
#![cfg_attr(test, allow(dead_code))]

use ecs::{EventManagerTrait, LoopStage, SystemTrait};
use event::{Event, EventFlag};
use text_manipulation::split_arguments;
use failure::Error;
use std::marker::PhantomData;
use std::io::{self, Read};
use std::string;
use std::sync::mpsc::{self, channel, Receiver};
use std::thread::spawn;
use std::time::Duration;

pub struct DebugConsole<Ctx> {
    escape_char: char,
    quote_char: char,
    worker_rx: Receiver<Result<String, DebugConsoleError>>,
    _ctx: PhantomData<Ctx>,
}

impl<Ctx> DebugConsole<Ctx> {
    pub fn new<S>(mut input_stream: S, escape_char: Option<char>, quote_char: Option<char>) -> Self
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
                            }).expect("Unable to send input from stdin via mpsc channel");
                            buf.clear()
                        } else {
                            buf.push(byte[0])
                        }
                    }
                    Err(e) => tx.send(Err(DebugConsoleError::IoError(e)))
                        .expect("Unable to send error information via mpsc channel"),
                }
            }
        });

        DebugConsole {
            escape_char: escape_char.unwrap_or('\\'),
            quote_char: quote_char.unwrap_or('"'),
            worker_rx: rx,
            _ctx: PhantomData::default(),
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

impl<Ctx> SystemTrait<Ctx, Event> for DebugConsole<Ctx>
where
    Ctx: EventManagerTrait<Event>,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::UPDATE | LoopStage::HANDLE_EVENTS
    }

    fn get_event_filter(&self) -> EventFlag {
        EventFlag::STARTUP
    }

    fn handle_event(&mut self, _ctx: &mut Ctx, event: &Event) -> Result<bool, Error> {
        if let EventFlag::STARTUP = event.flag() {
            println!("Debug console is ready");
        }

        Ok(true)
    }

    fn update(&mut self, ctx: &mut Ctx, _: &Duration, _: &Duration) -> Result<(), Error> {
        self.try_read_line()
            .map(|l| split_arguments(l, self.escape_char, self.quote_char))
            .map(|a| ctx.dispatch_later(Event::command(a)));

        Ok(())
    }
}

#[derive(Debug, Fail)]
enum DebugConsoleError {
    #[fail(display = "{}", _0)]
    IoError(#[cause] io::Error),
    #[fail(display = "{}", _0)]
    Utf8Error(#[cause] string::FromUtf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use components::model::Model;
    use mock::MockCtx;

    #[test]
    fn new() {
        let _: DebugConsole<MockCtx<Event, Model>> = DebugConsole::new(io::stdin(), None, None);
    }

    #[test]
    fn get_stage_filter() {
        let c: DebugConsole<MockCtx<Event, Model>> = DebugConsole::new(io::stdin(), None, None);

        assert_eq!(c.get_stage_filter(), LoopStage::UPDATE | LoopStage::HANDLE_EVENTS);
    }
}
