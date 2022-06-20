use std::io::ErrorKind;

use mio::{net::TcpListener, Poll, Interest, Token, event::Event};

use crate::log::{Log, LogTag};

use self::hub_header::Hub;
use self::hub::line_header::LineType;

mod call_hub;
pub mod hub;
pub mod hub_header;

const LISTENER: Token = Token(0);
pub struct Gate {
    listener:TcpListener,
    front_type:LineType,
    last_check:u64,
    pub hub:Hub
}

impl Gate {
    pub fn new(addr:&str,front_type:LineType,p:&Poll) -> Gate {
        let addr = addr.parse().unwrap();
        let mut listener = TcpListener::bind(addr).unwrap();
        p.registry().register(&mut listener, LISTENER, Interest::READABLE).unwrap();
        let str = format!("gate({:?}) listening on {} waiting for connections...",front_type,addr);
        Log::add(str, front_type,&LogTag::Default);
        Gate{ listener,front_type,last_check:0,hub:Hub::new(LogTag::Default as u64) }
    }

    pub fn front_type(&self) -> &LineType { 
        &self.front_type
    }

    pub fn process(&mut self, event:&Event,p:&Poll) {
        match event.token() {
            LISTENER => { self.on_listener_event(event,p); }
            _ => { self.hub.process(event,p); }
        }
    }

    pub fn check(&mut self) {
        if self.front_type != LineType::Fox { return; }
        let t = Log::time();
        if t - self.last_check > 1 {
            self.hub.old_check();
            self.hub.dead_check();
            self.last_check = t;
        }
    }


    fn on_listener_event(&mut self, event:&Event,p:&Poll) {
        
        Log::add(format!("{:?}",event), LineType::Defalut, &LogTag::Event);        
        
        if event.is_error() {
            panic!("unexpected listener error");
        }
        
        self.hub.health_check(p);

        loop {
            match self.listener.accept() {
                Ok((socket, _)) => {
                    self.hub.new_line(socket,p,self.front_type);
                }
                    
                Err(err) if err.kind() == ErrorKind::WouldBlock => { break; }
                
                _ => { panic!("accept error"); }
            }
        }
       
    }
    

}