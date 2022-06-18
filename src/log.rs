use std::{time::SystemTime, fs::{File, self}, io::Write};

use crate::hub::line::LineType;

pub struct Log {

}

impl Log {
    pub fn init() {
        match fs::remove_dir_all("log") {
            _ => {}
        }
        fs::create_dir("log").unwrap();
    }

    pub fn create_dir(kind:LineType) {
        let path = format!("log/{:?}",kind);
        fs::create_dir(path).unwrap();
        Log::new(format!("[init]"),kind,0);
    }

    pub fn new(str:String,kind:LineType,id:usize) { 
        let path = Log::get_path(kind,id);
        let s = format!("{}|{}\r",Log::now(),str);
        let msg = path.clone();
        fs::write(path, s).expect(msg.as_str());
    }

    pub fn add(str:String,kind:LineType,id:usize) {
        let path = Log::get_path(kind,id);
        let msg = path.clone();
        let mut f = File::options().append(true).open(path).expect(msg.as_str());
        let s = format!("{}|{}\r",Log::now(),str);
        f.write(s.as_bytes()).unwrap();
    }

    pub fn now() -> u128 {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
    }

    fn get_path(kind:LineType,id:usize) -> String {
        format!("log/{:?}/{}.log",kind,id)
    }

}