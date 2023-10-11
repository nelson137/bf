use sha1::{digest::Update, Digest, Sha1};

#[cfg(windows)]
pub const EOL: &str = "\r\n";
#[cfg(not(windows))]
pub const EOL: &str = "\n";

pub type Sha1Digest = [u8; 20];

pub fn sha1_digest<D: AsRef<[u8]>>(data: D) -> Sha1Digest {
    Sha1::new().chain(data).finalize().into()
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
