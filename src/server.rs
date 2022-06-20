use mio::{Poll, Events};
use crate::{gate::{Gate, hub::line_header::LineType}, log::{Log, LogType}};

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
            let count = self.events.into_iter().count();
            Log::add(format!("{}",count), LineType::Caller, LogType::Unique as u64);
            if count == 1 {
                self.gate.check(&self.p);
            }
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