use std::fs;

use super::line_header::Line;

impl Line {
    pub fn http_data(&mut self,buf:Vec<u8>) {
        let str = String::from_utf8_lossy(&buf);
        let req:Vec<&str> = str.split(" ").collect();
        if req.len() < 2 {
            self.go_die(); 
            return; 
        }
        
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
        } else if path.contains(".pac") {
            mime = "application/x-ns-proxy-autoconfig";
        }

        
        
        match fs::read(format!("dist/{}",path)) {
            Ok(data) => { (data,200,mime) }
            _ => { (empty,404,mime) }
        }
    }
   

}