use std::net::ToSocketAddrs;
use mio::{Token, net::TcpStream, Poll, event::Event};
use crate::log::{Log, LogTag};
use self::line_header::LineType;
use super::hub_header::Hub;

pub mod line_header;
mod fox;
mod line;
mod http;
mod process;
mod operator;


impl Hub {
    
    pub fn process(&mut self,event:&Event,p:&Poll) {
        let k = &event.token();
        Log::add(format!("{:?}",event), self.get_line(k).kind(), &LogTag::Event);        
        if self.get_line(k).is_dead() {
            self.get_line(k).event_after_die(event);
            return;
        }

        if event.is_error() {
            self.get_mut_line(k).on_error();
            self.dead_pair(k);
            return;
        }
        
        if event.is_writable() {
            self.get_mut_line(k).on_writable();
        }

        if event.is_readable() {
            self.process_read(k,p);
        }

        if event.is_write_closed() {
            self.get_mut_line(k).write_closed();
        }

        if event.is_read_closed() {
            self.get_mut_line(k).read_closed();
            self.dead_pair(k);
        }

    }


    fn process_read(&mut self,k:&Token,p:&Poll) {
        let line = self.get_mut_line(k);
        let pid = line.partner_id();
        let buf =  line.recv();
        
        if buf.len() == 0 {
            return;
        }

        match line.kind() {
            LineType::Fox => {self.process_fox(k,buf,p);}
            LineType::Http => {self.process_http(k,buf);}
            LineType::Operator => {self.process_operator(k,buf,p);}
            _ => { self.tunnel(pid, buf); }
        }
    }

    fn process_fox(&mut self,k:&Token,buf:Vec<u8>,p:&Poll) {
        let line = self.get_mut_line(k);
        let fox_id = line.id();
        let mut caller_id = line.partner_id();
        
        match line.fox_data(buf) {
            Some(data) => {
                if caller_id == 0 {
                    self.check(p);
                    caller_id = self.idle_caller();
                    self.get_mut_line_by_id(caller_id).set_partner_id(fox_id);
                    self.get_mut_line(k).set_partner_id(caller_id);
                }
                self.tunnel(caller_id, data);
            }
            _ => { }
        }
    }

    fn process_http(&mut self,k:&Token,buf:Vec<u8>) {
        self.get_mut_line(k).http_data(buf);
    }

    fn process_operator(&mut self,k:&Token,buf:Vec<u8>,p:&Poll) {
        let line = self.get_mut_line(k);
        let operator_id = line.id();
        let spider_id = line.partner_id();
        if spider_id > 0 {
            self.tunnel(spider_id,buf);
            return;
        }

        match line.decrypt_sni(&buf) {
            Some(data) => {
                let host = line.host().clone();
                let id = self.create_spider(host,operator_id,p);
                if id > 0 {
                    self.get_mut_line(k).set_partner_id(id);
                    self.get_mut_line_by_id(id).add_queue(data);
                } else {
                    //println!("[{}]create_spider fail",App::now())
                }
            }
            _ => {}
        }
    }

    fn tunnel(&mut self,pid:u64,data:Vec<u8>) {
        assert!(pid > 0);
        let line = self.get_mut_line_by_id(pid);
        line.add_queue(data);
        line.send();
    }

    fn create_spider(&mut self,host:String,operator_id:u64,p:&Poll) -> u64 {
        match host.to_socket_addrs() {
            Ok(mut it) => {
                let addr = it.next().unwrap();
                let stream = TcpStream::connect(addr).unwrap();
                let id = self.new_line(stream,p,LineType::Spider);
                let spider = self.get_mut_line_by_id(id);
                spider.set_partner_id(operator_id);
                spider.set_host(host,0);
                return id
            }
            Err(e) => {
                Log::add(format!("dns lookup fail|{}|{}",host,e), LineType::Spider, &LogTag::Unexpected);
            }
        }
        0
    }

    fn check(&mut self,p:&Poll) {
        self.old_check();
        self.dead_check();
        self.health_check(p);
    }


}