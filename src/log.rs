use core::fmt::Debug;
use backtrace::Backtrace;
use std::{time::SystemTime, fs::{File, self}, io::Write};

use enum_iterator::{all, Sequence};

use crate::gate::hub::line_header::LineType;

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
    pub fn init() {
        Log::create_log_dir();
        Log::set_panic_hook();
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
        let _msg = path.clone();
        let s = format!("{}|{}\n",Log::now(),str);
        match File::options().append(true).open(path) {
            Ok(mut f) => { f.write(s.as_bytes()).unwrap(); }
            Err(error) => { panic!("Problem opening the file: {:?}", error); }
        }
    }

    pub fn now() -> u128 {
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()
    }

    fn set_panic_hook() {
        File::create(Log::panic_file()).unwrap();
        
        std::panic::set_hook(Box::new(|_| {
            let bt = Backtrace::new();
            let mut f = File::options().append(true).open(Log::panic_file()).unwrap();
            f.write(format!("{:?}",bt).as_bytes()).unwrap();
        }));
    }

    fn get_path<T: Debug>(kind:LineType,name:&T) -> String {
        format!("log/{:?}/{:?}.log",kind,name)
    }
    

    fn panic_file() -> String {
        String::from("log/panic.log")
    }

    fn create_log_dir() {
        let path = "log";
        fs::remove_dir_all(path).unwrap();
        fs::create_dir(path).unwrap(); 
    }

    

}