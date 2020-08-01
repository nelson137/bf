use std::path::PathBuf;
use std::process::exit;

#[cfg(windows)]
pub const EOL: &str = "\r\n";
#[cfg(not(windows))]
pub const EOL: &str = "\n";

pub fn die(msg: String) -> ! {
    eprintln!("bf: {}", msg);
    exit(1);
}

pub fn get_width(width: Option<usize>) -> i32 {
    (match width {
        Some(w) => w,
        None => match term_size::dimensions() {
            Some((w, _h)) if w > 5 => w,
            _ => 65, // Wide enough for 16 cells
        },
    }) as i32
}

pub fn is_valid_infile(value: String) -> Result<(), String> {
    if value == "-" {
        return Ok(());
    }

    let path = PathBuf::from(&value);
    if path.exists() {
        if path.is_dir() {
            Err(format!("file is a directory: {}", value))
        } else {
            Ok(())
        }
    } else {
        Err(format!("no such file exists: {}", value))
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
