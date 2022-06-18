//Operator

use super::header::Line;

impl Line {
    pub fn decrypt_sni(&mut self,buf:&Vec<u8>) -> Option<Vec<u8>> {
        let data = sni_shuffle::decode(&buf);
        let len:usize = data[0].try_into().unwrap();
        let pos = len+9;
        let tag_bytes = data[len+1..pos].try_into().unwrap();
        let tag = u64::from_be_bytes(tag_bytes);
        self.set_host(String::from_utf8_lossy(&data[1..len+1]).to_string(),tag);
        Some(data[pos..].to_vec())
    }
}

