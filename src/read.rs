use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use crate::{err, util::BfResult};

pub fn read_script(infile: &Option<PathBuf>) -> BfResult<Vec<u8>> {
    match infile {
        Some(path) if *path != PathBuf::from("-") => read_script_file(path),
        _ => read_script_stdin(),
    }
}

fn read_script_stdin() -> BfResult<Vec<u8>> {
    let mut buf = Vec::new();
    match io::stdin().read_to_end(&mut buf) {
        Ok(_) => Ok(buf),
        Err(e) => Err(err!(FileRead, e, PathBuf::from("STDIN"))),
    }
}

pub fn read_script_file(path: &PathBuf) -> BfResult<Vec<u8>> {
    let mut file = File::open(path)
        .map_err(|e| err!(FileOpen, e, path.clone()))?;
    let mut buf = Vec::new();
    match file.read_to_end(&mut buf) {
        Ok(_) => Ok(buf),
        Err(e) => Err(err!(FileRead, e, path.clone()))
    }
}
