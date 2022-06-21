use configparser::ini::Ini;

pub struct Config {

}

impl Config {
    pub fn get_all() -> (String,String,u8,String,bool) {
        let mut config = Ini::new();
        config.load(Config::path()).unwrap();
        let app = config.get("listen","app").unwrap();
        let http = config.get("listen","http").unwrap();
        let worker:u8 = config.getuint("other","worker").unwrap().unwrap() as u8;
        let proxy_server_addr = config.get("server","addr").unwrap();
        let write_log = config.getbool("other", "write_log").unwrap().unwrap();
        (app,http,worker,proxy_server_addr,write_log)
    }

    pub fn get_listen_addr() -> (String,String) {
        let mut config = Ini::new();
        config.load(Config::path()).unwrap();
        let app = config.get("listen","app").unwrap();
        let http = config.get("listen","http").unwrap();
        (app,http)
    }

    pub fn get_proxy_server_addr() -> String {
        let mut config = Ini::new();
        config.load(Config::path()).unwrap();
        config.get("server","addr").unwrap()
    }
    
    fn path() -> String {
        "theshy.ini".to_string()
    }
}