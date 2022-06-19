use super::line_header::Line;
use super::line_header::LineStatus::Connected;

//process event
impl Line {
    pub fn on_error(&mut self) {
        let error = self.stream().take_error().unwrap();
        let str = format!("on_error {:?}",error);
        self.log(str);
    }


    pub fn on_writable(&mut self) {
        self.set_status(Connected);
        self.send();
    }

}