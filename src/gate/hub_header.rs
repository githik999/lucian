use std::{collections::HashMap, net::SocketAddr};

use mio::{Token, Poll, net::TcpStream, Interest};

use crate::{gate::hub::line_header::{Line,LineType},  log::Log};


pub struct Hub {
    id:u64,
    m:HashMap<Token,Line>,
    proxy_server:Option<SocketAddr>,
    healthy_size:u8,
    spawning:bool
}

//Get
impl Hub {
    pub fn m(&mut self) -> &HashMap<Token,Line> {
        &self.m
    }

    pub fn proxy_server(&self) -> SocketAddr {
        self.proxy_server.unwrap()
    }

    pub fn healthy_size(&self) -> u8 {
        self.healthy_size
    }

    pub fn spawning(&self) -> bool {
        self.spawning
    }

    pub fn get_line_by_id(&mut self,id:u64) -> &mut Line {
        assert!(id > 0);
        self.get_line(self.token(id))
    }

    pub fn get_line(&mut self,token:Token) -> &mut Line {
        self.m.get_mut(&token).expect(&token.0.to_string())
    }

    fn token(&self,id:u64) -> Token {
        Token(id.try_into().unwrap())
    }

    
}

//Set

impl Hub {
    
    pub fn new(id:u64) -> Hub {
        Hub { id,m:HashMap::new(),proxy_server:None,healthy_size:0,spawning:false }
    }

    pub fn next_id(&mut self) -> u64 {
        self.id = self.id + 1;
        self.id
    }

    pub fn set_healthy_size(&mut self,n:u8) {
        self.healthy_size = n;
    }

    pub fn set_proxy_server(&mut self,addr:&str) {
        self.proxy_server = Some(addr.parse().unwrap());
    }

    pub fn set_spawning(&mut self,v:bool) {
        self.spawning = v
    }

    pub fn dead_pair(&mut self,k:Token,p:&Poll) {
        let pid = self.get_line(k).partner_id();
        self.dead_line_id(pid,p);
        self.dead_line(k,p);
    }

    pub fn dead_line_id(&mut self,id:u64,p:&Poll) {
        if id > 0 {
            self.dead_line(self.token(id), p);
        }
        
    }

    pub fn dead_line(&mut self,k:Token,p:&Poll) {
        let kind = self.get_line(k).kind();
        let s = self.get_line(k).stream();
        p.registry().deregister(s).unwrap();
        self.get_line(k).go_die();
    }

    pub fn new_line(&mut self,mut stream:TcpStream,p:&Poll,kind:LineType) -> u64 {
        let id = self.next_id();
        let token = Token(id.try_into().unwrap());
        p.registry().register(&mut stream, token, Interest::READABLE | Interest::WRITABLE).unwrap();
        let line = Line::new(id,stream,kind);
        self.m.insert(token, line);
        id
    }
}