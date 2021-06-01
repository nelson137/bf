use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use itertools::Itertools;
use sha1::{Digest, Sha1};

#[cfg(windows)]
pub const EOL: &str = "\r\n";
#[cfg(not(windows))]
pub const EOL: &str = "\n";

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

pub fn mutex_safe_do<T, Ret, Func>(data: &Mutex<T>, func: Func) -> Ret
where
    Func: FnOnce(MutexGuard<T>) -> Ret,
{
    if let Ok(queue) = data.lock() {
        func(queue)
    } else {
        panic!("EventQueue: failed because of poisoned mutex");
    }
}

pub type Sha1Digest = [u8; 20];

pub fn sha1_digest<D: AsRef<[u8]>>(data: D) -> Sha1Digest {
    Sha1::new().chain(data).finalize().into()
}

pub trait StringExt {
    fn wrapped(&self, width: usize) -> Vec<String>;
}

impl StringExt for String {
    fn wrapped(&self, width: usize) -> Vec<String> {
        if self.len() > 0 {
            self.chars()
                .chunks(width)
                .into_iter()
                .map(|chunk| chunk.collect::<String>())
                .collect_vec()
        } else {
            vec![String::new()]
        }
    }
}

pub trait USizeExt {
    fn count_digits(&self) -> usize;
}

impl USizeExt for usize {
    fn count_digits(&self) -> usize {
        match *self {
            _ if *self < 10 => 1,
            _ if *self < 100 => 2,
            _ if *self < 1000 => 3,
            _ if *self < 10000 => 4,
            _ if *self < 100000 => 5,
            _ if *self < 1000000 => 6,
            _ if *self < 10000000 => 7,
            _ if *self < 100000000 => 8,
            _ if *self < 1000000000 => 9,
            _ if *self < 10000000000 => 10,
            _ if *self < 100000000000 => 11,
            _ if *self < 1000000000000 => 12,
            _ if *self < 10000000000000 => 13,
            _ if *self < 100000000000000 => 14,
            _ if *self < 1000000000000000 => 15,
            _ if *self < 10000000000000000 => 16,
            _ if *self < 100000000000000000 => 17,
            _ if *self < 1000000000000000000 => 18,
            _ if *self < 10000000000000000000 => 19,
            _ => 20,
        }
    }
}
