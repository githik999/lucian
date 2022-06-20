use core::fmt::Debug;
use std::{time::SystemTime, fs::{File, self}, io::Write};

use enum_iterator::{all, Sequence};

use crate::gate::hub::line_header::LineType;

#[derive(Debug,Sequence)]
pub enum LogTag {
    Default,
    Unique,
    Event,
    Establish,
    GoodBye,
    Unexpected,
    FirstLineID,
}

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
        for x in all::<LogTag>() {
            Log::new(kind, &x);
        }

    }

    pub fn new<T:Debug>(kind:LineType,name:&T) {
        let path = Log::get_path(kind,name);
        Log::create_file(path);
    }

    pub fn create_file(path:String) {
        let msg = path.clone();
        File::create(path).expect(msg.as_str());
    }

    pub fn add<T:Debug>(str:String,kind:LineType,name:&T) {
        let path = Log::get_path(kind,name);
        let msg = path.clone();
        let mut f = File::options().append(true).open(path).expect(msg.as_str());
        let s = format!("{}|{}\n",Log::now(),str);
        f.write(s.as_bytes()).unwrap();
    }

    pub fn now() -> u128 {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
    }

    fn get_path<T: Debug>(kind:LineType,name:&T) -> String {
        format!("log/{:?}/{:?}.log",kind,name)
    }

    

}