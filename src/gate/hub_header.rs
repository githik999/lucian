use std::{collections::{HashMap, VecDeque}, net::SocketAddr};

use mio::{Token, Poll, net::TcpStream, Interest};
use omg_cool::{header::{LineType, LogTag}, log::Log};

use super::hub::line_header::Line;


pub struct Hub {
    id:u64,
    healthy_size:u8,
    proxy_server:Option<SocketAddr>,
    m:HashMap<Token,Line>,
    idle_caller:VecDeque<u64>,
    dead:Vec<u64>
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

    pub fn idle_caller_count(&self) -> u8 {
        self.idle_caller.len() as u8
    }

    pub fn idle_caller(&mut self) -> u64 {
        self.idle_caller.pop_front().unwrap()
    }

    pub fn idle_caller_list(&self) -> &VecDeque<u64> {
        &self.idle_caller
    }

    pub fn idle_caller_list_mut(&mut self) -> &mut VecDeque<u64> {
        &mut self.idle_caller
    }

    pub fn dead_count(&self) -> u8 {
        self.dead.len() as u8
    }

    pub fn get_line_by_id(&self,id:u64) -> &Line {
        assert!(id > 0);
        self.get_line(&self.token(id))
    }

    pub fn get_line(&self,token:&Token) -> &Line {
        self.m.get(token).unwrap()
    }

    pub fn get_mut_line_by_id(&mut self,id:u64) -> &mut Line {
        assert!(id > 0);
        self.get_mut_line(&self.token(id))
    }

    pub fn get_mut_line(&mut self,token:&Token) -> &mut Line {
        self.m.get_mut(token).unwrap()
    }
    
    pub fn dead_check(&mut self) {
        if self.dead_count() > self.healthy_size() {
            self.remove_dead();
        }
    }

    fn token(&self,id:u64) -> Token {
        Token(id.try_into().unwrap())
    }

    
}

//Set

impl Hub {
    
    pub fn new(id:u64) -> Hub {
        Hub { id,healthy_size:0,proxy_server:None,m:HashMap::new(),idle_caller:VecDeque::new(),dead:Vec::new() }
    }

    pub fn next_id(&mut self) -> u64 {
        self.id = self.id + 1;
        self.id
    }

    pub fn set_healthy_size(&mut self,n:u8) {
        self.healthy_size = n;
    }

    pub fn set_proxy_server(&mut self,addr:String) {
        self.proxy_server = Some(addr.parse().unwrap());
    }

    pub fn dead_pair(&mut self,k:&Token) {
        let pid = self.get_line(k).partner_id();
        self.kill_line_by_id(pid);
        self.kill_line(k);
    }

    pub fn kill_line_by_id(&mut self,id:u64) {
        if id > 0 {
            self.kill_line(&self.token(id));
        }
    }

    pub fn kill_line(&mut self,k:&Token) {
        self.get_mut_line(k).go_die();
        self.add_dead(k);
    }

    pub fn remove_dead(&mut self) {
        loop {
            match self.dead.pop() {
                Some(id) => {
                    let kind = self.get_line_by_id(id).kind();
                    self.m.remove(&self.token(id));
                    Log::add(format!("rm|{}",id), kind, &LogTag::Default);
                }
                None => {break;}
            }
        }
    }

    pub fn new_line(&mut self,mut stream:TcpStream,p:&Poll,kind:LineType) -> u64 {
        let id = self.next_id();
        let token = Token(id.try_into().unwrap());
        p.registry().register(&mut stream, token, Interest::READABLE | Interest::WRITABLE).unwrap();
        let line = Line::new(id,stream,kind);
        self.m.insert(token, line);
        id
    }

    pub fn add_idle_caller(&mut self,id:u64) {
        self.idle_caller.push_back(id);
    }

    fn add_dead(&mut self,k:&Token) {
        self.dead.push(k.0 as u64);
    }

}