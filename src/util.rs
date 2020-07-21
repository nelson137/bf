use std::process::exit;

#[cfg(windows)]
pub const EOL: &str = "\r\n";
#[cfg(not(windows))]
pub const EOL: &str = "\n";

pub fn die(msg: String) -> ! {
    eprintln!("bf: {}", msg);
    exit(1);
}

pub fn get_width(width: Option<u32>) -> u32 {
    match width {
        Some(w) => w,
        None => match term_size::dimensions() {
            Some((w, _h)) if w > 5 => w as u32,
            _ => 65, // Wide enough for 16 cells
        },
    }
}

pub fn is_valid_width(value: String) -> Result<(), String> {
    match value.parse::<i64>() {
        Ok(n) => {
            if n < 5 {
                Err("value must be an integer > 5".to_string())
            } else {
                Ok(())
            }
        }
        Err(err) => Err(err.to_string()),
    }
}
