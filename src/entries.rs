use rand::prelude::SliceRandom;
use std::io::Result;
use std::path::{Path, PathBuf};

use crate::config;

#[derive(Debug, PartialEq)]
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
                    let fname =
                        pathdiff::diff_paths(path.join("pics").join(&fname), config::JOURNAL_PATH)
                            .unwrap();
                    Some(fname)
                })
                .collect(),
            Err(_) => vec![],
        };
        Self {
            text_path: (path.join("entry.md")).to_owned(),
            pics,
        }
    }

    pub(crate) fn content(&self) -> Result<String> {
        std::fs::read_to_string(&self.text_path)
    }

    pub(crate) fn date_str(&self) -> String {
        let filename = self
            .text_path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        filename.replace(".md", "")
    }

    pub fn pics(&self) -> &[PathBuf] {
        self.pics.as_ref()
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
        let mut entries: Vec<_> = read_dir
            .filter_map(move |e| {
                let fname = e.unwrap().file_name().to_str().unwrap().to_owned();
                Some(Entry::read(&path.join(fname)))
            })
            .collect();
        entries.sort_by_key(|entry| entry.date_str());
        Ok(Self { entries })
    }

    pub(crate) fn random(&self) -> Option<&Entry> {
        let mut thread_rng = rand::thread_rng();
        self.entries.choose(&mut thread_rng)
    }

    pub fn get_by_date(&self, date: String) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.date_str() == date)
    }

    pub fn prev(&self, entry: &Entry) -> Option<&Entry> {
        let index = self
            .entries
            .iter()
            .enumerate()
            .find(|(_, e)| *e == entry)
            .map(|(i, _)| i);
        index.and_then(|index| {
            index
                .checked_sub(1)
                .and_then(|index| self.entries.get(index))
        })
    }

    pub fn next(&self, entry: &Entry) -> Option<&Entry> {
        let index = self
            .entries
            .iter()
            .enumerate()
            .find(|(_, e)| *e == entry)
            .map(|(i, _)| i);
        index.and_then(|index| {
            index
                .checked_add(1)
                .and_then(|index| self.entries.get(index))
        })
    }
}
