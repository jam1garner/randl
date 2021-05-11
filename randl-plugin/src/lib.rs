#![feature(proc_macro_hygiene)]

use arcropolis_api::ext_callback;
use randl_core::{RandlFile, RandlEntry, Set, Value};

use randl_core::prc::{self, hash40::{self, Hash40}};

use std::fs;
use std::io::Cursor;
use std::collections::HashMap;

#[ext_callback]
fn prc_callback(hash: u64, mut data: &mut [u8]) -> Option<usize> {
    let (entry, sets) = RANDL_LOOKUP.get(&Hash40(hash))?;

    let len = data.len();

    arcropolis_api::load_original_file(hash, &mut data)?;

    let mut prc_file = prc::read_stream(&mut Cursor::new(&mut data)).map_err(|err| dbg!(err)).ok()?;

    entry.apply(&mut prc_file, sets).map_err(|err| dbg!(err)).ok()?;

    prc::write_stream(&mut Cursor::new(data), &prc_file).map_err(|err| dbg!(err)).ok()?;

    Some(len)
}

const CONFIG_FOLDER: &str = "sd:/atmosphere/contents/01006A800016E000/romfs";

type EntryAndSets = (&'static RandlEntry, &'static HashMap<String, Set>);

lazy_static::lazy_static! {
    static ref RANDL_FILES: Vec<RandlFile> = fs::read_dir(CONFIG_FOLDER).unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let is_file = entry.file_type().ok()?.is_file();
            let path = entry.path();
            let is_kdl = path.extension().map(|ext| ext == "kdl").unwrap_or(false);

            if is_file && is_kdl {
                RandlFile::open(path).ok()
            } else {
                None
            }
        })
        .collect();

    static ref RANDL_LOOKUP: HashMap<Hash40, EntryAndSets> = RANDL_FILES.iter()
        .map(|file| {
            file.entries.iter()
                .map(move |entry| {
                    PathIter::new(&entry.prc_name, &file.sets)
                        .map(move |hash| (hash, (entry, &file.sets)))
                })
                .flatten()
        })
        .flatten()
        .collect();
}

enum PathIter<'a> {
    Once(&'a str),
    Templated {
        prefix: &'a str,
        suffix: &'a str,
        templates: std::slice::Iter<'a, Value>,
    },
    None,
}

impl<'a> Iterator for PathIter<'a> {
    type Item = Hash40;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PathIter::Once(path) => {
                let hash = hash40::to_hash40(path);
                *self = PathIter::None;
                Some(hash)
            }
            PathIter::Templated { prefix, suffix, templates } => {
                templates.next()
                    .map(|middle| {
                        hash40::to_hash40(&match middle {
                            Value::Int(i) => format!(
                                "{}{}{}",
                                prefix,
                                i,
                                suffix,
                            ),
                            Value::String(s) => format!(
                                "{}{}{}",
                                prefix,
                                s,
                                suffix,
                            ),
                            _ => panic!(
                                "Invalid type in templated set. only Int and String are allowed"
                            )
                        })
                    })
            }
            PathIter::None => None,
        }
    }
}

impl<'a> PathIter<'a> {
    fn new(path_pattern: &'a str, sets: &'a HashMap<String, Set>) -> Self {
        match path_pattern.matches('{').count() {
            0 => Self::Once(path_pattern),
            1 => {
                let (prefix, rest) = path_pattern.split_once('{').unwrap();
                let (set, suffix) = rest.split_once('}').unwrap();

                let set = sets.get(set).unwrap_or_else(|| panic!("No set {:?} found.", set));

                PathIter::Templated { prefix, suffix, templates: set.0.iter() }
            },
            _ => panic!("Cannot ")
        }
    }
}

#[skyline::main(name = "randl")]
pub fn main() {
    prc_callback::install("prc");

    lazy_static::initialize(&RANDL_FILES);
    lazy_static::initialize(&RANDL_LOOKUP);
}

trait SplitOnceCompat {
    fn split_once(&self, pattern: char) -> Option<(&str, &str)>;
}

impl SplitOnceCompat for str {
    fn split_once(&self, pattern: char) -> Option<(&str, &str)> {
        let pos: usize = self.find(pattern)?;

        let (before, after) = self.split_at(pos);

        Some((before, &after[1..]))
    }
}
