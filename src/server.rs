use mio::{Poll, Events};
use crate::{gate::{Gate, hub::line_header::LineType}, log::{Log, LogTag}};

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
        match kind {
            LineType::Fox => { Log::create_dir(LineType::Caller); }
            LineType::Operator => { Log::create_dir(LineType::Spider); }
            _ => {}
        }
        let gate = Gate::new(addr,kind,&p);
        Server { p, events , gate }
    }

    pub fn start(&mut self) {
        loop {
            self.p.poll(&mut self.events, None).unwrap();
            
            if self.gate.front_type() == &LineType::Fox {
                let n = self.events.into_iter().count();
                Log::add(format!("event num:{}",n), LineType::Caller, &LogTag::Unique);
                if n == 1 {
                    self.gate.check(&self.p);
                }
            }

            for event in self.events.iter() {
                self.gate.process(event,&self.p);
            }
            
        }
    }

    pub fn init(&mut self,n:u8,addr:&str) {
        self.gate.hub.init_callers(n,addr,&self.p);
    }

}