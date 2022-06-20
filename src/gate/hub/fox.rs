use super::line_header::Line;

//Fox
impl Line {

    pub fn fox_data(&mut self,buf:Vec<u8>) -> Option<Vec<u8>> {
        match self.stage() {
            0 => { self.hand_shake(); }
            1 => { self.decode_socks5_cmd(buf); }
            2 => { return Some(self.sni_encrypt(buf)); }
            _ => { return Some(buf); }
        }
        None
    }

    fn hand_shake(&mut self) {
        self.next_stage();
        self.add_queue([0x05,0x00].to_vec()); 
        self.send();
    }

    fn decode_socks5_cmd(&mut self,buf:Vec<u8>) {
        self.next_stage();
        let n = buf.len();
        let atyp = buf[3];
        let port = u16::from_be_bytes([buf[n-2],buf[n-1]]);
        let host = self.decode_host(atyp, &buf[4..n-2],port);
        self.set_host(host,0);
        self.add_queue([0x05,0x00,0x00,0x01,0x00,0x00,0x00,0x00,0x00,0x00].to_vec());
        self.send();
    }

    fn decode_host(&self,atyp:u8,buf:&[u8],port:u16) -> String {
        match atyp {
            1 => { format!("{}.{}.{}.{}:{}",buf[0],buf[1],buf[2],buf[3],port) }
            2 => { format!("ipv6:{}",port) }
            3 => { 
                let domain = String::from_utf8_lossy(&buf[1..]).to_string();
                format!("{}:{}",domain,port)
            }
            _ => { format!("unexpected:{}",atyp) }
        }
    }

    fn sni_encrypt(&mut self,buf:Vec<u8>) -> Vec<u8> {
        self.next_stage();
        let len:u8 = self.host().len().try_into().unwrap();
        let input = [ [len].to_vec() , self.host().as_bytes().to_vec() , self.id().to_be_bytes().to_vec(), buf ].concat();
        let data = sni_shuffle::encode(&input);
        data
    }
    

}