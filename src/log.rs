use std::{time::SystemTime, fs::{File, self}, io::Write};

use crate::gate::hub::line_header::LineType;

pub struct Log {}

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

    pub fn new(str:String,kind:LineType,id:u64) { 
        let path = Log::get_path(kind,id);
        let s = format!("{}|{}\n",Log::now(),str);
        let msg = path.clone();
        fs::write(path, s).expect(msg.as_str());
    }

    pub fn add(str:String,kind:LineType,id:u64) {
        let path = Log::get_path(kind,id);
        let msg = path.clone();
        let mut f = File::options().append(true).open(path).expect(msg.as_str());
        let s = format!("{}|{}\n",Log::now(),str);
        f.write(s.as_bytes()).unwrap();
    }

    pub fn now() -> u128 {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
    }

    fn get_path(kind:LineType,id:u64) -> String {
        format!("log/{:?}/{}.log",kind,id)
    }

}