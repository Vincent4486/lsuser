use std::ffi::CStr;
use std::os::raw::c_char;

pub fn safe_string(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

pub fn parse_range(s: &str) -> (Option<u32>, Option<u32>) {
    let err = |msg: &str| -> ! {
        eprintln!("error: {msg}");
        std::process::exit(1);
    };
    if let Some(pos) = s.find('-') {
        let left = s[..pos].trim();
        let right = s[pos + 1..].trim();
        let min = if left.is_empty() {
            None
        } else {
            Some(left.parse().unwrap_or_else(|_| err(&format!("invalid number '{left}'"))))
        };
        let max = if right.is_empty() {
            None
        } else {
            Some(right.parse().unwrap_or_else(|_| err(&format!("invalid number '{right}'"))))
        };
        (min, max)
    } else {
        let val = s.trim().parse().unwrap_or_else(|_| err(&format!("invalid value '{s}'")));
        (Some(val), Some(val))
    }
}
