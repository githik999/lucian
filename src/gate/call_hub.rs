use std::collections::VecDeque;

use mio::{Poll, net::TcpStream};

use crate::gate::hub::line_header::LineStatus;
use crate::log::{Log, LogTag};

use super::hub_header::Hub;
use super::hub::line_header::LineType;

///Caller Hub

impl Hub {
    pub fn init_callers(&mut self,n:u8,addr:&str,p:&Poll) {
        assert!(n < u8::MAX/4);
        self.set_healthy_size(n);
        self.set_proxy_server(addr);
        self.add_caller(n,p);
    }

    pub fn old_check(&mut self) {
        let mut old = Vec::new();
        let mut young = VecDeque::new();
        
        for id in self.idle_caller_list() {
            let id = *id;
            let v = self.get_line_by_id(id);
            if v.kind() == LineType::Caller && v.status() <= LineStatus::Connected {
                let age = Log::now() - v.born_time();
                if age > 180000 {
                    old.push(id)
                } else {
                    young.push_back(id)
                }
            }
        }

        for id in old {
            self.kill_line_by_id(id);
        }

        self.idle_caller_list_mut().clone_from(&young);
    }

    pub fn health_check(&mut self,p:&Poll) {
        let need = self.healthy_size();
        let have:u8 = self.idle_caller_count();
        let lock = self.spawning();
        Log::add(format!("have:{},spawning:{}",have,lock), LineType::Caller, &LogTag::Unique);
        if have >= need { self.set_spawning(false); }
        if lock { return; }
        if have < need {
            self.spawn(p);
        }
    }

    fn spawn(&mut self,p:&Poll) {
        self.add_caller(self.healthy_size(), p);
        self.set_spawning(true);
    }
    
    fn add_caller(&mut self,n:u8,p:&Poll) {
        for _ in 0..n {
            self.add_one_caller(p);
        }
    }

    fn add_one_caller(&mut self,p:&Poll) {
        let stream = TcpStream::connect(self.proxy_server()).unwrap();
        let id = self.new_line(stream,p,LineType::Caller);
        self.add_idle_caller(id);
    }
   
}