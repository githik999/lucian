use mio::{Poll, net::TcpStream};

use crate::gate::hub::line_header::LineStatus;

use super::hub_header::Hub;
use super::hub::line_header::LineType;

///Caller Hub

impl Hub {
    pub fn get_idle_caller(&mut self) -> u64 {
        for (key, v) in self.m() {
            if v.available() {
                return key.0.try_into().unwrap();
            }
        }
        let mut info = [0;4];
        self.count_caller(&mut info);
        println!("{:?}",info);
        panic!("must guarantee always have idle caller");
    }

    pub fn init_callers(&mut self,n:u8,addr:&str,p:&Poll) {
        assert!(n < 64);
        self.set_healthy_size(n);
        self.set_proxy_server(addr);
        self.add_caller(n,p);
    }

    pub fn health_check(&mut self,p:&Poll) {
        let need = self.healthy_size();
        let have:u8 = self.count_idle_caller();
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
        self.new_line(stream,p,LineType::Caller);
    }

    fn count_caller(&mut self,info:&mut [u8]) {
        for (_key, v) in self.m() {
            if v.kind() == LineType::Caller {
                let i:usize = v.status() as usize;
                println!("{:?}:{}",v.status(),i);
                info[i] += 1;
            }
        }
    }

    fn count_idle_caller(&mut self) -> u8 {
        let mut ret = 0;
        for (_key, v) in self.m() {
            if v.available() {
                ret += 1;
            }
        }
        ret
    }
    

   
}