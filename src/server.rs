use std::{time::SystemTime, fs::File, io::Write};

use backtrace::Backtrace;
use mio::{Poll, Events};

use crate::{gate::{Gate, hub::line_header::LineType}, log::Log};
use self::Status::{Dead,Working};

static mut WORKING_CALLER: u8 = 0;

#[derive(Debug,Clone,Copy,PartialEq,PartialOrd)]
pub enum Status {
    Baby,
    Working,
    Dead,
}

pub struct Server {
    p:Poll,
    events:Events,
    gate:Gate
}

impl Server {
    pub fn new(addr:String,kind:LineType) -> Server {
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
            self.gate.hub.update_working_count();
        }
    }

    pub fn init(&mut self,n:u8,addr:String) {
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
        
        if Log::panic_file_size() > 0 {
            return Dead;
        }
        
        let n = Server::get_working_caller_count();
        if n > 0 {
            return Working
        }

        Dead
        
    }

    pub fn set_panic_hook() {
        std::panic::set_hook(Box::new(|_| {
            let bt = Backtrace::new();
            let mut f = File::options().append(true).open(Log::panic_file()).unwrap();
            f.write(format!("{:?}",bt).as_bytes()).unwrap();
        }));
    }

    pub fn get_working_caller_count() -> u8 {
        unsafe{ WORKING_CALLER }
    }

    pub fn set_working_caller_count(n:u8) {
        unsafe{ WORKING_CALLER = n; }
    }

}
