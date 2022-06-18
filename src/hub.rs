use std::{collections::HashMap, net::{ToSocketAddrs, SocketAddr, Shutdown}, io::Write, fs};
use mio::{Token, net::TcpStream, Poll, Interest, event::Event};
use crate::log::Log;

use self::line::{Line, LineType};

pub mod line;

pub struct Hub {
    id:usize,
    m:HashMap<Token,Line>,
    proxy_server:Option<SocketAddr>,
    healthy_size:u8,
    spawning:bool,
}

impl Hub {
    pub fn process(&mut self,v:&Event,p:&Poll) {
        let k = &v.token();
        if v.is_error() {
            self.get_line(k).on_error();
            self.remove_pair(k, p);
            return;
        }

        if self.get_line(k).is_dead() {
            self.get_line(k).go_die();
        } else {
            if v.is_writable() {
                self.get_line(k).on_writable();
            }

            if v.is_readable() {
                self.process_read(k,p);
            }
            
        }

        if v.is_read_closed() {
            self.get_line(k).send();
            self.remove_pair(k, p);
        }
    }

    fn remove_pair(&mut self,k:&Token,p:&Poll) {
        let pid = self.get_line(k).partner_id();
        self.get_line(k).go_die();
        self.remove_line(k,p);
        if pid > 0 {
            self.get_line_by_id(pid).go_die();
        }
    }

    fn process_read(&mut self,k:&Token,p:&Poll) {
        let line = self.get_line(k);
        let pid = line.partner_id();
        let buf =  line.recv();
        
        if buf.len() == 0 {
            return;
        }

        match line.kind {
            LineType::Fox => {self.process_fox(k,buf);}
            LineType::Log => {self.process_log(k,buf);}
            LineType::Operator => {self.process_operator(k,buf,p);}
            _ => { self.tunnel(pid, buf); }
        }
    }

    fn process_fox(&mut self,k:&Token,buf:Vec<u8>) {
        let line = self.get_line(k);
        let fox_id = line.id();
        let mut caller_id = line.partner_id();
        
        match line.fox_data(buf) {
            Some(data) => {
                if caller_id == 0 {
                    caller_id = self.get_idle_caller();
                    self.get_line_by_id(caller_id).set_partner_id(fox_id);
                    self.get_line(k).set_partner_id(caller_id);
                }
                self.tunnel(caller_id, data);
            }
            _ => { }
        }
    }

    fn process_log(&mut self,k:&Token,buf:Vec<u8>) {
        self.get_line(k).http(buf);
    }

    fn process_operator(&mut self,k:&Token,buf:Vec<u8>,p:&Poll) {
        let line = self.get_line(k);
        let operator_id = line.id();
        let spider_id = line.partner_id();
        println!("process_operator spider_id:{} len:{}",spider_id,buf.len());
        if spider_id > 0 {
            self.tunnel(spider_id,buf);
            return;
        }

        match line.decrypt_sni(&buf) {
            Some(data) => {
                let host = line.host().clone();
                let id = self.create_spider(host,operator_id,p);
                if id > 0 {
                    self.get_line(k).set_partner_id(id);
                    self.get_line_by_id(id).add_queue(data);
                } else {
                    //println!("[{}]create_spider fail",App::now())
                }
            }
            _ => {}
        }
    }
}



impl Hub {
    pub fn new(id:usize) -> Hub {
        Hub { id, m:HashMap::new(),proxy_server:None,healthy_size:0,spawning:false }
    }

    fn tunnel(&mut self,pid:usize,data:Vec<u8>) {
        let line = self.get_line_by_id(pid);
        line.add_queue(data);
        line.send();
    }

    fn create_spider(&mut self,host:String,operator_id:usize,p:&Poll) -> usize {
        //println!("[{}][{}]dns lookup {} start.....",App::now(),operator_id,host);
        match host.to_socket_addrs() {
            Ok(mut it) => {
                let addr = it.next().unwrap();
                //println!("[{}][{}]dns lookup finish {} {}",App::now(),operator_id,host,addr);
                let stream = TcpStream::connect(addr).unwrap();
                let id = self.new_line(stream,p,LineType::Spider);
                let spider = self.get_line_by_id(id);
                spider.set_partner_id(operator_id);
                spider.set_host(host);
                return id
            }
            Err(e) => { /*println!("[{}]{} dns lookup fail {}",App::now(),host,e);*/ }
        }
        0
    }

    pub fn new_line(&mut self,mut stream:TcpStream,p:&Poll,kind:LineType) -> usize {
        let id = self.next_id();
        let token = Token(id);
        p.registry().register(&mut stream, token, Interest::READABLE | Interest::WRITABLE).unwrap();
        let line = Line::new(id,stream,kind);
        self.m.insert(token, line);
        id
    }

    pub fn remove_line_by_id(&mut self,id:usize,p:&Poll) {
        if id > 0 {
            self.remove_line(&Token(id),p);
        }
    }

    fn get_line_by_id(&mut self,id:usize) -> &mut Line {
        assert!(id > 0);
        self.get_line(&Token(id))
    }

    fn get_line(&mut self,token:&Token) -> &mut Line {
        self.m.get_mut(token).expect(&token.0.to_string())
    }
    
    fn remove_line(&mut self,k:&Token,p:&Poll) {
        let str = format!("remove line {:?}",k);
        let kind = self.get_line(k).kind();
        let s = self.get_line(k).stream();
        p.registry().deregister(s).unwrap();
        self.m.remove(k);
        Log::add(str,kind,0);
    }

    fn next_id(&mut self) -> usize {
        self.id = self.id + 1;
        self.id
    }
}


///Caller Hub

impl Hub {
    pub fn get_idle_caller(&self) -> usize {
        for (key, v) in &self.m {
            match v.kind {
                LineType::Caller => {
                    if v.available() {
                        return key.0;
                    }
                }
                _ => { }
            }
        }
        
        let info = self.count_caller();
        println!("{:?}",info);
        panic!("must guarantee always have idle caller");
    }

    

    pub fn init_callers(&mut self,n:u8,addr:&str,p:&Poll) {
        assert!(n < 64);
        self.healthy_size = n;
        self.proxy_server = Some(addr.parse().unwrap());
        self.add_caller(n,p);
    }
    
    pub fn check_callers(&mut self,p:&Poll) {
        let (idle,born,working,dead) = self.count_caller();
        //println!("[{}]check_callers idle:{} born:{} working:{} dead:{} healthy:{} spawning:{}",App::now(),idle,born,working,dead,self.healthy_size,self.spawning);
        if idle >= self.healthy_size { 
            self.spawning = false;
            return; 
        }
        
        if self.spawning { 
            return;
        }

        self.spawning = true;
        self.add_caller(self.healthy_size,p);
    }

    fn add_caller(&mut self,n:u8,p:&Poll) {
        for _ in 0..n {
            self.add_one_caller(p);
        }
    }

    pub fn add_one_caller(&mut self,p:&Poll) {
        let stream = TcpStream::connect(self.proxy_server.unwrap()).unwrap();
        self.new_line(stream,p,LineType::Caller);
    }

    fn count_caller(&self) -> (u8,u8,u8,u8) {
        let mut idle = 0;
        let mut born = 0;
        let mut working = 0;
        let mut dead = 0;
        
        for (_key, v) in &self.m {
            match v.kind {
                LineType::Caller => {
                    if v.available() {
                        idle = idle + 1;
                    } else if v.just_born() {
                        born = born + 1;
                    } else if v.is_dead() {
                        dead = dead + 1;
                    } else if v.partner_id() > 0 {
                        working = working + 1;
                    }
                }
                _ => {}
            }
        }

        (idle,born,working,dead)
    }

    

    

   
}