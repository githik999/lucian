use std::{io::ErrorKind, net::SocketAddr};

use mio::{net::TcpListener, Poll, Interest, Token, event::Event};

use crate::log::Log;

use self::hub_header::Hub;
use self::hub::line_header::LineType;

mod call_hub;
pub mod hub;
pub mod hub_header;

const LISTENER: Token = Token(0);
pub struct Gate {
    listener:TcpListener,
    front_type:LineType,
    pub hub:Hub
}

impl Gate {
    pub fn new(port:usize,front_type:LineType,p:&Poll) -> Gate {
        let addr = Gate::parse_addr(port,front_type);
        let mut listener = TcpListener::bind(addr).unwrap();
        p.registry().register(&mut listener, LISTENER, Interest::READABLE).unwrap();
        let str = format!("gate({:?}) listening on {} waiting for connections...",front_type,addr);
        Log::add(str, front_type,0);
        Gate{ listener,front_type,hub:Hub::new(0) }
    }

    pub fn process(&mut self, event:&Event,p:&Poll) {
        Log::add(format!("{:?}",event), self.front_type, 0);
        match event.token() {
            LISTENER => { self.on_listener_event(event,p); }
            _ => { self.hub.process(event,p); }
        }
    }

    pub fn check(&mut self,p:&Poll) {
        if self.front_type == LineType::Fox {
            self.hub.old_check();
            self.hub.dead_check();
            self.hub.health_check(p);
        }
        
    }


    fn on_listener_event(&mut self, event:&Event,p:&Poll) {
        
        if event.is_error() {
            panic!("unexpected listener error");
        }

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

    fn parse_addr(port:usize,front_type:LineType) -> SocketAddr {
        let mut ip= "127.0.0.1";
        if front_type == LineType::Operator {
            ip = "0.0.0.0";
        }
        format!("{}:{}",ip,port).parse().unwrap()
    }

}