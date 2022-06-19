use mio::{Poll, net::TcpStream};

use crate::log::Log;

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
        let mut n:u8 = 0;
        for (_key, v) in self.m() {
            if v.kind() == LineType::Caller {
                let age = Log::now() - v.born_time();
                if age > 300000 {
                    n += 1;
                }
            }
        }
        println!("old:{}",n);
    }

    pub fn health_check(&mut self,p:&Poll) {
        let mut info = [0;4];
        self.count_caller(&mut info);
        let need = self.healthy_size();
        let have:u8 = self.idle_caller_count();
        
        println!("{:?} have:{}",info,have);

        if have > need { self.set_spawning(false); }
        if self.spawning() { return; }
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

    fn count_caller(&mut self,info:&mut [u8]) {
        for (_key, v) in self.m() {
            if v.kind() == LineType::Caller {
                let i:usize = v.status() as usize;
                info[i] += 1;
            }
        }
    }
   
}