use std::collections::VecDeque;

use mio::{Poll, net::TcpStream};
use omg_cool::{time::Time, header::{Status, LineAge, LineType, LogTag}, log::Log};


use super::hub_header::Hub;

///Caller Hub

impl Hub {
    pub fn init_callers(&mut self,n:u8,addr:String,p:&Poll) {
        assert!(n < u8::MAX/4);
        self.set_healthy_size(n);
        self.set_proxy_server(addr);
        self.add_caller(n,p);
    }

    pub fn working_caller_count(&self) -> u8 {
        let mut ret = 0;
        if self.idle_caller_count() == 0 {
            return ret;
        }
        
        for id in self.idle_caller_list() {
            let v = self.get_line_by_id(*id);
            if v.status() == Status::Working {
                ret += 1;
            }
        }
        
        ret
    }

    pub fn old_check(&mut self) {
        let mut old = Vec::new();
        let mut young = VecDeque::new();

        let t = Time::now();
        
        for id in self.idle_caller_list() {
            let id = *id;
            match self.age(id, t) {
                LineAge::Young => { young.push_back(id); }
                LineAge::Old => { old.push(id); }
                _ => {}
            }
        }

        Log::add(format!("young:{}|old:{}|dead:{}",young.len(),old.len(),self.dead_count()), LineType::Caller, &LogTag::Unique);

        for id in old {
            self.kill_line_by_id(id);
        }

        self.idle_caller_list_mut().clone_from(&young);
    }

    pub fn health_check(&mut self,p:&Poll) {
        let need = self.healthy_size();
        let have:u8 = self.idle_caller_count();
        Log::add(format!("have:{}|need:{}",have,need), LineType::Caller, &LogTag::Unique);
        if have < need {
            self.add_caller(self.healthy_size(), p);
        }
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

    fn age(&self,id:u64,now:u128) -> LineAge {
        let v = self.get_line_by_id(id);
        if v.kind() != LineType::Caller { return LineAge::Defalut; }
        let s = v.status();
        if s == Status::Dead { return LineAge::Defalut; }
        let age = now - v.born_time();
        if age < 3*60*1000 { 
            return LineAge::Young;
        }
        LineAge::Old
    }
   
}