use std::io::ErrorKind;

use mio::{net::TcpListener, Poll, Interest, Token, event::Event};

use crate::{hub::{Hub, line::LineType}, app::App};
const LISTENER: Token = Token(0);
pub struct Gate {
    listener:TcpListener,
    front_line_type:LineType,
    pub hub:Hub
}

impl Gate {
    pub fn new(addr:&str,front_line_type:LineType,p:&Poll) -> Gate {
        let addr = addr.parse().unwrap();
        let mut listener = TcpListener::bind(addr).unwrap();
        p.registry().register(&mut listener, LISTENER, Interest::READABLE).unwrap();
        println!("[{}]gate listening on {} waiting for connections...",App::now(),addr);
        Gate{ listener,front_line_type,hub:Hub::new(0) }
    }

    pub fn process(&mut self, event:&Event,p:&Poll) {
        match  self.front_line_type {
            LineType::Fox | LineType::Operator => { println!("[{}]{:?}", App::now(), event); }
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
                    match self.front_line_type {
                        LineType::Fox => { self.hub.add_one_caller(p); }
                        _ => {}
                    }
                    self.hub.new_line(socket,p,self.front_line_type);
                }
                    
                Err(err) if err.kind() == ErrorKind::WouldBlock => { break; }
                
                _ => { panic!("accept error"); }
            }
        }

       
    }

}