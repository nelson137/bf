use sha1::{Digest, Sha1};

#[cfg(windows)]
pub const EOL: &str = "\r\n";
#[cfg(not(windows))]
pub const EOL: &str = "\n";

fn get_terminal_width() -> Option<usize> {
    term_size::dimensions().map(|(w, _)| w).filter(|&w| w > 5)
}

pub fn get_width(width: Option<usize>) -> i32 {
    width.or_else(get_terminal_width).unwrap_or(65) as i32
}

pub type Sha1Digest = [u8; 20];

pub fn sha1_digest<D: AsRef<[u8]>>(data: D) -> Sha1Digest {
    Sha1::new().chain(data).finalize().into()
}

pub trait StringExt {
    fn wrapped(&self, width: usize) -> WrappedString;
}

impl StringExt for String {
    fn wrapped(&self, width: usize) -> WrappedString {
        WrappedString::new(self, width)
    }
}

pub struct WrappedString<'s> {
    string: &'s str,
    width: usize,
    chunk_begin: usize,
    chunk_end: usize,
}

impl<'s> WrappedString<'s> {
    fn new(string: &'s str, width: usize) -> Self {
        let chunk_end = width.min(string.len());
        Self {
            string,
            width,
            chunk_begin: 0,
            chunk_end,
        }
    }
}

impl<'s> Iterator for WrappedString<'s> {
    type Item = &'s str;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let unchunked_len = self.string.len().saturating_sub(self.chunk_begin);
        let remaining_chunks =
            (unchunked_len as f32 / self.width as f32).ceil() as usize;
        (0, Some(remaining_chunks))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.chunk_begin >= self.string.len() {
            if self.chunk_begin == 0 {
                self.chunk_begin = usize::MAX;
                return Some("");
            } else {
                return None;
            }
        }
        let chunk = &self.string[self.chunk_begin..self.chunk_end];
        self.chunk_begin = self.chunk_end;
        self.chunk_end = self.string.len().min(self.chunk_begin + self.width);
        Some(chunk)
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
