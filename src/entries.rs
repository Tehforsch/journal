use std::io::Result;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Entry {
    text_path: PathBuf,
    pics: Vec<PathBuf>,
}

impl Entry {
    fn read(path: &Path) -> Self {
        let pics = match std::fs::read_dir(path.join("pics")) {
            Ok(entries) => entries
                .filter_map(move |e| {
                    let fname = e.unwrap().file_name().to_str().unwrap().to_owned();
                    Some(Entry::read(&path.join(fname)))
                })
                .collect(),
            Err(_) => vec![],
        };
        Self {
            text_path: path.to_owned(),
            pics: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Entries {
    entries: Vec<Entry>,
}

impl Entries {
    pub fn read(path: &Path) -> Result<Self> {
        let path = path.to_owned();
        let read_dir = std::fs::read_dir(path.clone())?;
        let entries = read_dir
            .filter_map(move |e| {
                let fname = e.unwrap().file_name().to_str().unwrap().to_owned();
                Some(Entry::read(&path.join(fname)))
            })
            .collect();
        Ok(Self { entries })
    }
}
