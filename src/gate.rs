use std::{io::ErrorKind, net::SocketAddr};

use mio::{net::TcpListener, Poll, Interest, Token, event::Event};

use crate::{hub::{line_header::LineType}, log::Log, hub_header::Hub};

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
        match  self.front_type {
            //LineType::Fox | LineType::Operator => { println!("[{}]{:?}", App::now(), event); }
            // => { println!("{:?}",event); }
            _ => {}
        }
        
        match event.token() {
            LISTENER => { self.on_listener_event(event,p); }
            _ => { self.hub.process(event,p); }
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