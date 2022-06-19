use super::line_header::Line;
use super::line_header::LineStatus::Connected;

//process event
impl Line {
    pub fn on_error(&mut self) {
        let error = self.stream().take_error().unwrap();
        let str = format!("on_error {:?}",error);
        self.log(str);
    }

    pub fn on_read_closed(&mut self) {
        //println!("[{}]({})[{}][on_read_closed]read_close:{},write_close:{},",App::now(),self.id,self.partner_id,self.read_close,self.write_close);
        //self.read_close = true;
    }

    pub fn on_write_closed(&mut self) {
        //println!("[{}]({})[{}][on_write_closed]read_close:{},write_close:{},",App::now(),self.id,self.partner_id,self.read_close,self.write_close);
        //self.write_close = true;
    }

    pub fn on_writable(&mut self) {
        self.set_status(Connected);
        self.send();
    }

}