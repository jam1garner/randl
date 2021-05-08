use std::{collections::HashMap, fs, path::Path};

use prc::hash40::Hash40;
use kdl::{KdlNode, KdlValue};

pub use prc;

mod parser;
mod error;
mod eval;

pub use error::{EvalError, ParseError};

#[derive(Debug)]
pub struct RandlFile {
    pub entries: Vec<RandlEntry>,
    pub sets: HashMap<String, Set>,
}

impl RandlFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        Self::from_str(
            fs::read_to_string(path)
                .map_err(|_| ParseError::FileReadFail)?
                .as_str(),
        )
    }

    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        kdl::parse_document(s)
            .map(Self::from_nodes)
            .map_err(ParseError::ParseFail)?
    }
}

#[derive(Debug)]
enum PrcPathComponent {
    Field(String),
    Index(usize),
    Wildcard,
}

#[derive(Debug)]
struct PrcPath(Vec<PrcPathComponent>);

impl PrcPath {
    fn as_ref(&self) -> PrcPathSlice {
        PrcPathSlice(&self.0[..])
    }
}

#[derive(Debug, Clone, Copy)]
struct PrcPathSlice<'a>(&'a [PrcPathComponent]);

impl<'a> PrcPathSlice<'a> {
    fn pop_front(&mut self) -> Option<&PrcPathComponent> {
        let ret = self.0.get(0)?;
        self.0 = &self.0[1..];
        Some(ret)
    }
}

#[derive(Debug)]
pub struct RandlEntry {
    pub prc_name: String,
    prc_fields: Vec<PrcEntry>,
}

#[derive(Debug)]
struct PrcEntry {
    path: PrcPath,
    expr: Expr,
}

#[derive(Debug)]
struct ChanceStmt {
    percent: f64,
    expr: Box<Expr>,
}

#[derive(Debug, Clone)]
enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Hash40(Hash40),
    Original,
}

#[derive(Debug)]
enum Range {
    Int(i64, i64),
    Float(f64, f64),
}

#[derive(Debug)]
enum Return {
    Constant(Value),
    Range(Range),
    Set(String),
    AnonymousSet(Set),
}

#[derive(Debug)]
pub struct Set(Vec<Value>);

#[derive(Debug)]
enum Expr {
    Random(Vec<ChanceStmt>),
    Return(Return),
    Original,
}
