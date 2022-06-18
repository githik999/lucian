use mio::{Poll, Events};

use crate::{gate::Gate,log::Log, hub::header::LineType};

pub struct Server {
    p:Poll,
    events:Events,
    gate:Gate
}

impl Server {
    pub fn new(port:usize,kind:LineType) -> Server {
        let p = Poll::new().unwrap();
        let events = Events::with_capacity(128);
        
        Log::create_dir(kind);
        match kind {
            LineType::Fox => { Log::create_dir(LineType::Caller); }
            LineType::Operator => { Log::create_dir(LineType::Spider); }
            _ => {}
        }

        let gate = Gate::new(port,kind,&p);

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