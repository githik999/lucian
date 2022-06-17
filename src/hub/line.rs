use std::{io::{Write, ErrorKind, Read}, net::Shutdown};

use mio::net::TcpStream;

use crate::app::App;

#[derive(Debug)]
pub enum LineStatus {
    Born,
    Connected,
    Dead,
}

#[derive(Debug,Clone,Copy)]
pub enum LineType {
    Fox,
    Caller,
    Operator,
    Spider,
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

impl Line {
    pub fn on_error(&self) {
        let error = self.stream.take_error().unwrap();
        println!("[{}]({})[{}][{:?}][on_error]{:?} {:?},",App::now(),self.id,self.partner_id,self.kind,error,self.host);
    }

    pub fn on_read_closed(&mut self) {
        println!("[{}]({})[{}][on_read_closed]read_close:{},write_close:{},",App::now(),self.id,self.partner_id,self.read_close,self.write_close);
        self.read_close = true;
    }

    pub fn on_write_closed(&mut self) {
        println!("[{}]({})[{}][on_write_closed]read_close:{},write_close:{},",App::now(),self.id,self.partner_id,self.read_close,self.write_close);
        self.write_close = true;
    }
    
    pub fn go_die(&mut self) {
        self.queue.clear();
        match self.stream.shutdown(Shutdown::Both) {
            Ok(_) => {}
            Err(err) => { println!("shutdown fail {}",err); }
        }
        self.set_partner_id(0);
        self.set_status(LineStatus::Dead);
        println!("[{}]({})[die][{}][{:?}]host({})",App::now(),self.id,self.partner_id,self.kind,self.host);
    }

    pub fn on_writable(&mut self) {
        self.set_status(LineStatus::Connected);
        self.send();
    }

}


impl Line {
    pub fn new(id:usize,stream:TcpStream,kind:LineType) -> Line {
        println!("[{}]({})[{:?}]new line {:?}",App::now(),id,kind,stream);
        Line{ id,stream,kind,partner_id:0,status:LineStatus::Born,queue:Vec::new(),stage:0,host:String::from(""),read_close:false,write_close:false,_born:App::now() }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn stream(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    pub fn partner_id(&self) -> usize {
        self.partner_id
    }

    pub fn set_partner_id(&mut self,id:usize) {
        println!("[{}]({})[{:?}]partner_id from {} to {} status:{:?}",App::now(),self.id,self.kind,self.partner_id,id,self.status);
        self.partner_id = id
    }

    fn set_status(&mut self,v:LineStatus) {
        println!("[{}]({})[{:?}]status from {:?} to {:?}",App::now(),self.id,self.kind,self.status,v);
        self.status = v
    }

    pub fn set_host(&mut self,str:String) {
        self.host = str;
    }

    pub fn add_queue(&mut self,v:Vec<u8>) {
        self.queue.extend(v.iter());
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

    pub fn recv(&mut self) -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();
        let mut buf = [0;8192];
        loop {
            match self.stream.read(&mut buf) {
                Ok(n) => {
                    println!("({})read {} bytes",self.id,n);
                    if n > 0 {
                        data.extend_from_slice(&buf[..n]);
                    } else {
                        break;
                    }
                }
                Err(err) => {
                    println!("({})[read_error]{:?}",self.id,err);
                    break;
                }
            }
        }
        println!("[{}]({})[{}][{:?}]recv {} bytes data from {:?}",App::now(),self.id,self.partner_id(),self.kind,data.len(),self.stream);
        data
    }

    pub fn send(&mut self) {
        match self.status {
            LineStatus::Connected => {}
            _ => { println!("[{}]({}) send not allow at status {:?}.queue:{}",App::now(),self.id,self.status,self.queue.len()); return; }
        }

        loop {
            if self.queue.len() > 0 {
                match self.stream.write(&self.queue) {
                    Ok(n) => { 
                        println!("[{}]({})[{}][{:?}]write {} bytes to {}",App::now(),self.id,self.partner_id(),self.kind,n,self.stream.peer_addr().unwrap());
                        self.queue = self.queue[n..].to_vec();
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => { break; }
                    Err(e) => { println!("[{}]({})line write error {} {:?}",App::now(),self.id,e,self.kind); break; }
                }
            }
            break;
        }
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
        let str = String::from_utf8_lossy(&data);
        if str.contains("com") {
            print!("({})[{}][unexpected.com]{}",self.id,self.host(),str);
        }
        data
    }
    

}
    
//Operator

impl Line {
    pub fn host(&self) -> &String {
        &self.host
    }

    pub fn decrypt_sni(&mut self,buf:&Vec<u8>) -> Option<Vec<u8>> {
        let data = sni_shuffle::decode(&buf);
        let len:usize = data[0].try_into().unwrap();
        self.host = String::from_utf8_lossy(&data[1..len+1]).to_string();
        Some(data[len+1..].to_vec())
    }

    
}

impl Drop for Line {
    fn drop(&mut self) {
        println!("[{}]({})[{:?}][drop]with {} bytes data", App::now(),self.id(),self.kind,self.queue.len());
    }
}