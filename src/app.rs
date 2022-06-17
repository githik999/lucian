use std::time::SystemTime;

use mio::{Poll, Events};

use crate::{gate::Gate, hub::line::LineType};

pub struct App {
    p:Poll,
    events:Events,
    gate:Gate
}

impl App {
    pub fn new(addr:&str,kind:LineType) -> App {
        let p = Poll::new().unwrap();
        let events = Events::with_capacity(1024);
        let gate = Gate::new(addr,kind,&p);
        App { p, events , gate }
    }

    pub fn start(&mut self) { 
        loop {
            println!("waiting for events....");
            self.p.poll(&mut self.events, None).unwrap();
            for event in self.events.iter() {
                self.gate.process(event,&self.p);
            }
        }
    }

    pub fn init(&mut self,n:u8,addr:&str) {
        self.gate.hub.init_callers(n,addr,&self.p);
    }

    pub fn now() -> u128 {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
    }
}