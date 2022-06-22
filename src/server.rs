use mio::{Poll, Events};
use omg_cool::{header::{Status::{self,Working,Dead},LineType}, log::Log, config::Config};
use crate::gate::Gate;

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

    pub fn status() -> Status {
        if Log::panic_file_size() > 0 {
            return Dead;
        }
        
        let n = Config::working_caller_count();
        if n > 0 {
            return Working
        }

        Dead
    }

}