use std::{io::{ErrorKind, Read}, net::Shutdown};

use mio::event::Event;

use super::line_header::{LineStatus::{Connected,Occupied,Dead},Line};
use crate::log::{Log, LogTag};

impl Line {
    
    pub fn go_die(&mut self) {
        self.clear_queue();
        self.shutdown_stream();
        self.set_partner_id(0);
        self.set_status(Dead);
        let t = Log::now() - self.born_time();
        self.log(format!("die|{}",t));
        Log::add(format!("{}|{}|{}",self.id(),self.host(),t), self.kind(), &LogTag::GoodBye);
    }

    pub fn event_after_die(&self,e:&Event) {
        self.log(format!("e|{:?}",e));
    }

    pub fn recv(&mut self) -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();
        let mut buf = [0;8192];
        loop {
            match self.stream().read(&mut buf) {
                Ok(n) => {
                    if n > 0 {
                        data.extend_from_slice(&buf[..n]);
                    } else {
                        break;
                    }
                }
                
                Err(e) if e.kind() == ErrorKind::WouldBlock => { break; }

                Err(err) => {
                    let str = format!("read_error {:?}",err);
                    self.log(str);
                    break;
                }
            }
        }

        let str = format!("r|{}",data.len());
        self.log(str);
        data
    }

    pub fn send(&mut self) {
        let s = self.status();
        if s != Connected && s != Occupied { return; } 

        loop {
            if self.queue().len() > 0 {
                self.pour_queue();
            } else {
                break;
            }
        }
    }

    pub fn log(&self,str:String) {
        Log::add(str,self.kind(),&self.id());
    }


    fn shutdown_stream(&mut self) {
        if self.status() != Connected { return; }
        match self.stream().shutdown(Shutdown::Both) {
            Ok(_) => {}
            Err(err) => { println!("shutdown fail {}",err); }
        }
    }

}


impl Drop for Line {
    fn drop(&mut self) {
        //println!("[{}]({})[{:?}][drop]with {} bytes data", App::now(),self.id(),self.kind,self.queue.len());
    }
}
