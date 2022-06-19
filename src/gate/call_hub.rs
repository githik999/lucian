use mio::{Poll, net::TcpStream};

use super::hub_header::Hub;
use super::hub::line_header::LineType;

///Caller Hub

impl Hub {
    pub fn get_idle_caller(&mut self) -> u64 {
        for (key, v) in self.m() {
            if v.call_available() {
                return key.0.try_into().unwrap();
            }
        }
        
        let info = [0;4];
        self.count_caller(&info);
        println!("{:?}",info);
        panic!("must guarantee always have idle caller");
    }

    

    pub fn init_callers(&mut self,n:u8,addr:&str,p:&Poll) {
        assert!(n < 64);
        self.set_healthy_size(n);
        self.set_proxy_server(addr);
        self.add_caller(n,p);
    }
    
    fn add_caller(&mut self,n:u8,p:&Poll) {
        for _ in 0..n {
            self.add_one_caller(p);
        }
    }

    pub fn add_one_caller(&mut self,p:&Poll) {
        let stream = TcpStream::connect(self.proxy_server()).unwrap();
        self.new_line(stream,p,LineType::Caller);
    }

    fn count_caller(&mut self,_info:&[u8]) {
        for (_key, v) in self.m() {
            if v.kind() == LineType::Caller {
                
            }
        }

    }

    

   
}