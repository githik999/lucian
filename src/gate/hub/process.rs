use omg_cool::{log::Log, header::{Status::Working,LogTag}};

use super::line_header::Line;

impl Line {
    pub fn on_error(&mut self) {
        let err = self.stream().take_error().unwrap().unwrap();
        Log::add(format!("{}|{}",self.id(),err), self.kind(), &LogTag::Unexpected);
        self.log(format!("{err}"));
    }


    pub fn on_writable(&mut self) {
        self.set_status(Working);
        self.send();
    }

}