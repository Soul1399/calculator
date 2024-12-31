use std::{ffi::OsString, fs, io};

pub fn get_file_names(path: &str) -> io::Result<Vec<String>> {
    let entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.file_name().into_string().unwrap()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    Ok(entries)
}

pub fn get_os_file_names(path: &str) -> io::Result<Vec<OsString>> {
    let entries = fs::read_dir(path)?
        .map(|res| res.map(|e| e.file_name()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    Ok(entries)
}