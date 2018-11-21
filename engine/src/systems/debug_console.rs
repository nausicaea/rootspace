#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(test, allow(unused_mut))]
#![cfg_attr(test, allow(dead_code))]

use ecs::{EventManagerTrait, LoopStage, SystemTrait};
use event::EngineEventTrait;
use failure::Error;
use std::{
    io::{self, Read},
    marker::PhantomData,
    string,
    sync::mpsc::{self, channel, Receiver},
    thread::spawn,
    time::Duration,
};
use text_manipulation::split_arguments;

pub struct DebugConsole<Ctx, Evt> {
    escape_char: char,
    quote_char: char,
    worker_rx: Receiver<Result<String, DebugConsoleError>>,
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
}

impl<Ctx, Evt> DebugConsole<Ctx, Evt> {
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
            worker_rx: rx,
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
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

impl<Ctx, Evt> SystemTrait<Ctx, Evt> for DebugConsole<Ctx, Evt>
where
    Ctx: EventManagerTrait<Evt>,
    Evt: EngineEventTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::UPDATE | LoopStage::HANDLE_EVENTS
    }

    fn get_event_filter(&self) -> Evt::EventFlag {
        Evt::startup()
    }

    fn handle_event(&mut self, _ctx: &mut Ctx, event: &Evt) -> Result<bool, Error> {
        if event.matches_filter(Evt::startup()) {
            println!("Debug console is ready");
        }

        Ok(true)
    }

    fn update(&mut self, ctx: &mut Ctx, _: &Duration, _: &Duration) -> Result<(), Error> {
        self.try_read_line()
            .map(|l| split_arguments(l, self.escape_char, self.quote_char))
            .map(|a| ctx.dispatch_later(Evt::new_command(a)));

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
    use context::Context;
    use mock::MockEvt;

    #[test]
    fn new() {
        let _: DebugConsole<Context<MockEvt>, MockEvt> = DebugConsole::new(io::stdin(), None, None);
    }

    #[test]
    fn get_stage_filter() {
        let c: DebugConsole<Context<MockEvt>, MockEvt> = DebugConsole::new(io::stdin(), None, None);

        assert_eq!(c.get_stage_filter(), LoopStage::UPDATE | LoopStage::HANDLE_EVENTS);
    }
}
