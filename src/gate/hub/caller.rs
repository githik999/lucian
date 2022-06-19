//Caller

use super::line_header::{Line, LineStatus};
use super::line_header::LineType::Caller;

impl Line {
    pub fn call_status(&self) -> u8 {
        if self.kind() != Caller { return u8::MAX }
        match self.status() {
            LineStatus::Born => {0}
            LineStatus::Connected => {if self.partner_id() > 0 { 1 } else { 2 }}
            LineStatus::Dead => {3}
        }
    }

    pub fn call_available(&self) -> bool {
        if self.kind() != Caller { return false; }
        if self.partner_id() > 0 { return false };
        if self.status() == LineStatus::Connected { return true; }
        false
    }
}