use core::fmt::Debug;
use std::{fs::{File, self}, io::Write};

use enum_iterator::{all, Sequence};

use crate::{gate::hub::line_header::LineType, server::Server};

static mut WRITE_LOG: bool = false;

#[derive(Debug,Sequence)]
pub enum LogTag {
    Unique,
    Event,
    Establish,
    GoodBye,
    Unexpected,
    Default,
}

pub struct Log {}

impl Log {
    pub fn create_log_dir() {
        let path = "log";
        match fs::remove_dir_all(path) {
            _ => { fs::create_dir(path).unwrap(); }
        }
        File::create(Log::panic_file()).unwrap();
    }
    
    pub fn create_dir(kind:LineType) {
        unsafe{ if !WRITE_LOG { return; } }
        let path = format!("log/{:?}",kind);
        fs::create_dir(path).unwrap();
        for x in all::<LogTag>() {
            Log::new(kind, &x);
        }
    }

    pub fn create_file(path:String) {
        unsafe{ if !WRITE_LOG { return; } }
        let msg = path.clone();
        File::create(path).expect(msg.as_str());
    }

    pub fn add<T:Debug>(str:String,kind:LineType,name:&T) {
        unsafe{ if !WRITE_LOG { return; } }
        let path = Log::get_path(kind,name);
        let s = format!("{}|{}\n",Server::now(),str);
        let mut f = File::options().append(true).open(path).unwrap();
        f.write(s.as_bytes()).unwrap();
    }

    pub fn new<T:Debug>(kind:LineType,name:&T) {
        let path = Log::get_path(kind,name);
        Log::create_file(path);
    }

    pub fn panic_file() -> String {
        String::from("log/panic.log")
    }

    pub fn panic_file_size() -> u64 {
        match File::open(Log::panic_file()) {
            Ok(f) => { f.metadata().unwrap().len() }
            _ => { 1 }
        }
    }

    pub fn turn_on() {
        unsafe{ WRITE_LOG = true }
    }
   

    fn get_path<T: Debug>(kind:LineType,name:&T) -> String {
        format!("log/{:?}/{:?}.log",kind,name)
    }
    

    

    

    

}