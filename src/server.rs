use std::{time::SystemTime, fs::File, io::Write};

use backtrace::Backtrace;
use mio::{Poll, Events};

use crate::{gate::{Gate, hub::line_header::LineType}, log::Log};


#[derive(Debug,Clone,Copy,PartialEq,PartialOrd)]
pub enum Status {
    Born,
    Connected,
    Dead,
}

pub struct Server {
    p:Poll,
    events:Events,
    gate:Gate
}

impl Server {
    pub fn new(addr:&str,kind:LineType) -> Server {
        let p = Poll::new().unwrap();
        let events = Events::with_capacity(u8::MAX.into());
        Log::create_dir(kind);
        Log::create_dir(enum_iterator::next(&kind).unwrap());
        let gate = Gate::new(addr,kind,&p);
        Server { p, events , gate }
    }

    pub fn start(&mut self) {
        loop {
            self.p.poll(&mut self.events, None).unwrap();
            for event in self.events.iter() {
                self.gate.process(event,&self.p);
            }
        }
    }

    pub fn init(&mut self,n:u8,addr:&str) {
        self.gate.hub.init_callers(n,addr,&self.p);
    }

}

//
impl Server {
    pub fn time() -> u64 {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
    }
    
    pub fn now() -> u128 {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
    }

    pub fn status() -> Status {
        Status::Dead
    }

    fn set_panic_hook() {
        File::create(Log::panic_file()).unwrap();
        
        std::panic::set_hook(Box::new(|_| {
            let bt = Backtrace::new();
            let mut f = File::options().append(true).open(Log::panic_file()).unwrap();
            f.write(format!("{:?}",bt).as_bytes()).unwrap();
        }));
    }
}
