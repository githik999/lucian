use std::{io::{Write, ErrorKind, Read}, net::Shutdown, fs};

use mio::net::TcpStream;

use crate::log::Log;

#[derive(Debug,PartialEq)]
pub enum LineStatus {
    Born,
    Connected,
    Dead,
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum LineType {
    Fox,
    Caller,
    Operator,
    Spider,
    Log,
}

#[derive(Debug)]
pub struct  Line {
    id:usize,
    partner_id:usize,
    stream:TcpStream,
    status:LineStatus,
    pub kind:LineType,
    queue:Vec<u8>,
    stage:u8,
    host:String,
    read_close:bool,
    write_close:bool,
    _born:u128,
}

//process event
impl Line {
    pub fn on_error(&self) {
        let error = self.stream.take_error().unwrap();
        let str = format!("on_error {:?}",error);
        self.log(str);
    }

    pub fn on_read_closed(&mut self) {
        //println!("[{}]({})[{}][on_read_closed]read_close:{},write_close:{},",App::now(),self.id,self.partner_id,self.read_close,self.write_close);
        self.read_close = true;
    }

    pub fn on_write_closed(&mut self) {
        //println!("[{}]({})[{}][on_write_closed]read_close:{},write_close:{},",App::now(),self.id,self.partner_id,self.read_close,self.write_close);
        self.write_close = true;
    }

    pub fn on_writable(&mut self) {
        self.set_status(LineStatus::Connected);
        self.send();
    }

}

impl Line {
    pub fn new(id:usize,stream:TcpStream,kind:LineType) -> Line {
        let str = format!("{:?}",stream);
        Log::new(str,kind,id);
        Line{ id,stream,kind,partner_id:0,status:LineStatus::Born,queue:Vec::new(),stage:0,host:String::from(""),read_close:false,write_close:false,_born:Log::now() }
    }
    
    pub fn go_die(&mut self) {
        if self.status == LineStatus::Dead { return; }
        self.queue.clear();
        match self.status {
            LineStatus::Connected => {
                match self.stream.shutdown(Shutdown::Both) {
                    Ok(_) => {}
                    Err(err) => { println!("shutdown fail {}",err); }
                }
            }
            _ => {}
        }
        
        self.set_partner_id(0);
        self.set_status(LineStatus::Dead);
        let str = format!("die:{:?}",self.host);
        self.log(str);
    }

    pub fn recv(&mut self) -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();
        let mut buf = [0;8192];
        loop {
            match self.stream.read(&mut buf) {
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
        if self.status != LineStatus::Connected { return; } 

        loop {
            if self.queue.len() > 0 {
                match self.stream.write(&self.queue) {
                    Ok(n) => { 
                        self.log(format!("w|{}",n));
                        self.queue = self.queue[n..].to_vec();
                    }
                    Err(err) if err.kind() == ErrorKind::WouldBlock => { break; }
                    Err(err) => {
                        let str = format!("write error {:?}",err);
                        self.log(str);
                        break; 
                    }
                }
            }
            break;
        }
    }

}

//getter and setter
impl Line {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn kind(&mut self) -> LineType {
        self.kind
    }

    pub fn stream(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    pub fn partner_id(&self) -> usize {
        self.partner_id
    }

    pub fn set_partner_id(&mut self,id:usize) {
        if self.partner_id == id { return; }
        let str = format!("p|{}",id);
        self.log(str);
        self.partner_id = id
    }

    fn set_status(&mut self,v:LineStatus) {
        if self.status == v { return; }
        self.log(format!("s|{:?}",v));
        self.status = v;
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn set_host(&mut self,str:String) {
        self.host = str;
    }

    pub fn add_queue(&mut self,v:Vec<u8>) {
        if v.len() > 0 {
            self.queue.extend(v.iter());
            self.log(format!("q|{}",v.len()));
        }
    }

    pub fn is_dead(&self) -> bool {
        match self.status {
            LineStatus::Dead => { true }
            _ => { false }
        }
    }

    pub fn just_born(&self) -> bool {
        match self.status {
            LineStatus::Born => { true }
            _ => { false }
        }
    }

    pub fn is_read_closed(&self) -> bool {
        self.read_close
    }

    pub fn is_write_closed(&self) -> bool {
        self.write_close
    }

    pub fn available(&self) -> bool {
        if self.partner_id > 0 { return false };
        match self.status {
            LineStatus::Born => { true }
            LineStatus::Connected => { true }
            _ => { false }
        }
    }

    fn log(&self,str:String) {
        Log::add(str,self.kind,self.id);
    }

}
    
//Fox
impl Line {

    pub fn fox_data(&mut self,buf:Vec<u8>) -> Option<Vec<u8>> {
        match self.stage {
            0 => { self.hand_shake(); }
            1 => { self.decode_socks5_cmd(buf); }
            2 => { return Some(self.sni_encrypt(buf)); }
            _ => { return Some(buf); }
        }
        None
    }

    fn hand_shake(&mut self) {
        self.stage = self.stage + 1;
        self.add_queue([0x05,0x00].to_vec()); 
        self.send();
    }

    fn decode_socks5_cmd(&mut self,buf:Vec<u8>) {
        self.stage = self.stage + 1;
        let n = buf.len();
        let domain = String::from_utf8_lossy(&buf[5..n-2]);
        let port = u16::from_be_bytes([buf[n-2],buf[n-1]]);
        self.host = domain.to_string()+":"+&port.to_string();
        self.add_queue([0x05,0x00,0x00,0x01,0x00,0x00,0x00,0x00,0x00,0x00].to_vec());
        self.send();
    }

    fn sni_encrypt(&mut self,buf:Vec<u8>) -> Vec<u8> {
        self.stage = self.stage + 1;
        let len:u8 = self.host.len().try_into().unwrap();
        let input = [ [len].to_vec() , self.host.as_bytes().to_vec() , buf ].concat();
        let data = sni_shuffle::encode(&input);
        data
    }
    

}
    
//Operator

impl Line {
    pub fn decrypt_sni(&mut self,buf:&Vec<u8>) -> Option<Vec<u8>> {
        let data = sni_shuffle::decode(&buf);
        let len:usize = data[0].try_into().unwrap();
        self.host = String::from_utf8_lossy(&data[1..len+1]).to_string();
        Some(data[len+1..].to_vec())
    }
}

//Log
impl Line {
    pub fn http(&mut self,buf:Vec<u8>) {
        let str = String::from_utf8_lossy(&buf);
        let req:Vec<&str> = str.split(" ").collect();
        let (content,code,mime) = self.get_content(req[1]);
        let head = format!("HTTP/1.1 {} OK\r\nContent-Type: {}; charset=utf-8\r\n\r\n",code,mime);
        self.add_queue(head.as_bytes().to_vec());
        self.add_queue(content);
        self.send();
        self.go_die();
    }

    fn get_content(&self,mut path:&str) -> (Vec<u8>,u16,&str) {
        println!("{}",path);
        let empty = Vec::new();
        let mut mime = "text/html";
        if path.len() > 256 { return (empty,403,mime) }
        if path == "/" { path = "index.html"; }
        if path.contains(".js") { 
            mime = "text/javascript";
        } else if path.contains(".css") {
            mime = "text/css";
        } else if path.contains(".png") {
            mime = "image/png";
        }
        
        match fs::read(format!("dist/{}",path)) {
            Ok(data) => { (data,200,mime) }
            _ => { (empty,404,mime) }
        }
    }
   

}

impl Drop for Line {
    fn drop(&mut self) {
        //println!("[{}]({})[{:?}][drop]with {} bytes data", App::now(),self.id(),self.kind,self.queue.len());
    }
}
