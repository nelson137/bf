use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

pub fn read_script(infile: &Option<PathBuf>) -> Result<Vec<u8>, String> {
    match infile {
        Some(path) if *path != PathBuf::from("-") => read_script_file(path),
        _ => read_script_stdin(),
    }
}

fn read_script_stdin() -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    if let Some(err) = io::stdin().read_to_end(&mut buf).err() {
        Err(format!("Failed to read script from stdin: {}", err))
    } else {
        Ok(buf)
    }
}

fn read_script_file(path: &PathBuf) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|err| {
        format!("Failed to open infile: {}: {}", path.display(), err)
    })?;
    let mut buf = Vec::new();
    match file.read_to_end(&mut buf) {
        Ok(_) => Ok(buf),
        Err(err) => Err(format!(
            "Failed to read infile: {}: {}",
            path.display(),
            err
        )),
    }
}
