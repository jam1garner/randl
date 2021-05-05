use std::{collections::HashMap, fs, path::Path};

use hash40::Hash40;
use kdl::{KdlError, KdlNode, KdlValue};

#[derive(thiserror::Error, Debug, Clone)]
enum Error {
    #[error("the file could not be read")]
    FileReadFail,
    #[error("could not parse: invalid KDL")]
    ParseFail(KdlError),
    #[error("The value did not match the specified type")]
    IncorrectType,
    #[error("Type name invalid")]
    InvalidType,
    #[error("Value must follow type declaration in set")]
    NoValue,
    #[error("No percent provided for the given chance expression")]
    NoPercent,
    #[error("Chance expressions cannot be mixed with other expressions")]
    MixedExprs,
    #[error("Too many expressions were provided. Only multiple `chance` expessions may be used")]
    TooManyExprs,
    #[error("An expressions is required for every parameter entry specified")]
    ExprRequired,
    #[error("Invalid return: {0}")]
    InvalidReturn(&'static str),
    #[error("Invalid chance: {0}")]
    InvalidChance(&'static str),
    #[error("Invalid expression: {0}")]
    InvalidExpr(String),
    #[error("Invalid top-level entry: {0}")]
    InvalidRandlEntry(&'static str),
}

#[derive(Debug)]
struct RandlFile {
    entries: Vec<RandlEntry>,
    sets: HashMap<String, Set>,
}

impl RandlFile {
    fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::from_str(
            fs::read_to_string(path)
                .map_err(|_| Error::FileReadFail)?
                .as_str(),
        )
    }

    fn from_str(s: &str) -> Result<Self, Error> {
        kdl::parse_document(s)
            .map(Self::from_nodes)
            .map_err(Error::ParseFail)?
    }

    fn from_nodes(nodes: Vec<KdlNode>) -> Result<Self, Error> {
        let mut sets = HashMap::new();
        let mut entries = Vec::new();
        for node in nodes.into_iter() {
            match node.name.as_str() {
                "set" => {
                    let (name, set) = Set::from_node(node)?;
                    sets.insert(name, set);
                }
                "file" => {
                    entries.push(RandlEntry::from_node(node)?);
                }
                _ => {
                    return Err(Error::InvalidRandlEntry(
                        "entries must be of type `file` or `set`",
                    ))
                }
            }
        }

        Ok(Self { sets, entries })
    }
}

#[derive(Debug)]
enum PrcPathComponent {
    Field(String),
    Index(usize),
    Wildcard,
}

impl PrcPathComponent {
    fn from_str(s: &str) -> Result<Self, Error> {
        if s.chars().all(char::is_numeric) {
            Ok(PrcPathComponent::Index(s.parse().unwrap()))
        } else {
            if s == "*" {
                Ok(PrcPathComponent::Wildcard)
            } else {
                Ok(PrcPathComponent::Field(s.to_owned()))
            }
        }
    }
}

#[derive(Debug)]
struct PrcPath(Vec<PrcPathComponent>);

#[derive(Debug)]
struct RandlEntry {
    prc_name: String,
    prc_fields: Vec<PrcEntry>,
}

impl RandlEntry {
    fn from_node(mut node: KdlNode) -> Result<Self, Error> {
        match node.values.len() {
            0 => Err(Error::InvalidRandlEntry(
                "entries must contain a filename pattern",
            )),
            1 => match node.values.pop().unwrap() {
                KdlValue::String(prc_name) => Ok(Self {
                    prc_name,
                    prc_fields: node
                        .children
                        .into_iter()
                        .map(PrcEntry::from_node)
                        .collect::<Result<_, _>>()?,
                }),
                _ => Err(Error::InvalidRandlEntry("filename must be string")),
            },
            _ => Err(Error::InvalidRandlEntry("only one file name per entry")),
        }
    }
}

#[derive(Debug)]
struct PrcEntry {
    path: PrcPath,
    expr: Expr,
}

impl PrcEntry {
    fn from_node(node: KdlNode) -> Result<Self, Error> {
        let path = PrcPath(
            node.name
                .split('.')
                .map(PrcPathComponent::from_str)
                .collect::<Result<_, _>>()?,
        );

        Expr::from_nodes(node.children).map(move |expr| PrcEntry { path, expr })
    }
}

#[derive(Debug)]
struct ChanceStmt {
    percent: f64,
    expr: Box<Expr>,
}

impl ChanceStmt {
    fn from_nodes(nodes: Vec<KdlNode>) -> Result<Vec<Self>, Error> {
        if nodes.iter().all(|node| node.name == "chance") {
            let nodes: Vec<ChanceStmt> = nodes
                .into_iter()
                .map(|node| {
                    Ok(ChanceStmt {
                        percent: match node.properties.get("percent").ok_or(Error::NoPercent)? {
                            &KdlValue::Int(int) => int as f64,
                            &KdlValue::Float(f) => f,
                            _ => {
                                return Err(Error::InvalidChance(
                                    "percent must be an integer or a float",
                                ))
                            }
                        },
                        expr: Box::new(Expr::from_nodes(node.children)?),
                    })
                })
                .collect::<Result<_, _>>()?;

            let total: f64 = nodes.iter().map(|chance| chance.percent).sum();

            if 100.0 - total <= 0.1 {
                Ok(nodes)
            } else {
                Err(Error::InvalidChance(
                    "Chance statements must add up to >99.9%",
                ))
            }
        } else {
            Err(Error::MixedExprs)
        }
    }
}

#[derive(Debug)]
enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Hash40(Hash40),
}

impl Value {
    fn from_node(mut node: KdlNode) -> Result<Self, Error> {
        if node.values.len() == 1 {
            match &*node.name {
                "int" => match node.values[0] {
                    KdlValue::Int(int) => Ok(Value::Int(int)),
                    _ => Err(Error::IncorrectType),
                },
                "string" => match node.values.pop().unwrap() {
                    KdlValue::String(string) => Ok(Value::String(string)),
                    _ => Err(Error::IncorrectType),
                },
                "float" => match node.values[0] {
                    KdlValue::Float(float) => Ok(Value::Float(float)),
                    _ => Err(Error::IncorrectType),
                },
                "bool" => match node.values[0] {
                    KdlValue::Boolean(bool) => Ok(Value::Bool(bool)),
                    _ => Err(Error::IncorrectType),
                },
                "hash40" => {
                    todo!()
                }
                _ => Err(Error::InvalidType),
            }
        } else {
            Err(Error::NoValue)
        }
    }
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
    AnonymousSet(Vec<Value>),
}

#[derive(Debug)]
struct Set(Vec<Value>);

impl Set {
    fn from_node(mut node: KdlNode) -> Result<(String, Self), Error> {
        let set_name = match node.values.len() {
            0 => Err(Error::InvalidRandlEntry("sets must contain a name")),
            1 => match node.values.pop().unwrap() {
                KdlValue::String(name) => Ok(name),
                _ => Err(Error::InvalidRandlEntry("set name must be string")),
            },
            _ => Err(Error::InvalidRandlEntry("only one name per set")),
        }?;

        node.children
            .into_iter()
            .map(Value::from_node)
            .collect::<Result<_, _>>()
            .map(Set)
            .map(move |set| (set_name, set))
    }
}

impl Return {
    fn from_node(mut node: KdlNode) -> Result<Self, Error> {
        assert_eq!(node.name, "return");

        if node.children.is_empty() {
            if node.properties.is_empty() {
                match node.values.len() {
                    0 => {
                        return Err(Error::InvalidReturn(
                            "Returns must have either children, properties, or values",
                        ))
                    }
                    1 => Ok(Return::Constant(match node.values.pop().unwrap() {
                        KdlValue::Int(int) => Value::Int(int),
                        KdlValue::Float(f) => Value::Float(f),
                        KdlValue::String(s) => Value::String(s),
                        KdlValue::Boolean(b) => Value::Bool(b),
                        KdlValue::Null => return Err(Error::InvalidType),
                    })),
                    _ => {
                        return Err(Error::InvalidReturn(
                            "Returns may not have more than one return value",
                        ))
                    }
                }
            } else {
                if !node.values.is_empty() {
                    return Err(Error::InvalidReturn(
                        "Returns may not have both properties and values",
                    ));
                }

                // search properties
                let from = node.properties.get("from");
                let to = node.properties.get("to");
                match (from, to) {
                    (Some(_), None) => {
                        Err(Error::InvalidReturn("Return range missing `to` property"))
                    }
                    (None, Some(_)) => {
                        Err(Error::InvalidReturn("Return range missing `from` property"))
                    }
                    (Some(from), Some(to)) => Ok(Return::Range(match (from, to) {
                        (&KdlValue::Int(from), &KdlValue::Int(to)) => Range::Int(from, to),
                        (&KdlValue::Int(from), &KdlValue::Float(to)) => {
                            Range::Float(from as f64, to)
                        }
                        (&KdlValue::Float(from), &KdlValue::Int(to)) => {
                            Range::Float(from, to as f64)
                        }
                        (&KdlValue::Float(from), &KdlValue::Float(to)) => Range::Float(from, to),
                        _ => {
                            return Err(Error::InvalidReturn(
                                "Ranges must be for only integers or floats",
                            ))
                        }
                    })),
                    _ => {
                        if let Some(hash40) = node.properties.get("hash40") {
                            let hash40 = match hash40 {
                                &KdlValue::Int(hash) => Hash40(hash as u64),
                                KdlValue::String(s) => hash40::to_hash40(&s),
                                _ => {
                                    return Err(Error::InvalidReturn(
                                        "`hash40` property must be a string or an integer",
                                    ))
                                }
                            };

                            Ok(Return::Constant(Value::Hash40(hash40)))
                        } else if let Some(set_name) = node.properties.get("set") {
                            if let KdlValue::String(set_name) = set_name {
                                Ok(Return::Set(set_name.clone()))
                            } else {
                                Err(Error::InvalidReturn("Return set must be a string"))
                            }
                        } else {
                            Err(Error::InvalidReturn(
                                "Invalid property in return. Only `set`, `to`, `from`, and `hash40` are supported"
                            ))
                        }
                    }
                }
            }
        } else {
            if !node.properties.is_empty() {
                return Err(Error::InvalidReturn(
                    "Return nodes with children cannot have properties",
                ));
            }
            if node.values.is_empty() {
                // parse children
                node.children
                    .into_iter()
                    .map(Value::from_node)
                    .collect::<Result<_, _>>()
                    .map(Return::AnonymousSet)
            } else {
                return Err(Error::InvalidReturn(
                    "Return nodes with children cannot have values",
                ));
            }
        }
    }
}

#[derive(Debug)]
enum Expr {
    Random(Vec<ChanceStmt>),
    Return(Return),
    Original,
}

impl Expr {
    fn from_nodes(mut x: Vec<KdlNode>) -> Result<Self, Error> {
        match x.len() {
            0 => Err(Error::ExprRequired),
            1 => {
                // parse any type
                match &*x[0].name {
                    "chance" => ChanceStmt::from_nodes(x).map(Expr::Random),
                    "original" => Ok(Expr::Original),
                    "return" => Return::from_node(x.pop().unwrap()).map(Expr::Return),
                    name => Err(Error::InvalidExpr(format!(
                        "{} is not a valid expression",
                        name
                    ))),
                }
            }
            _ if x[0].name == "chance" => ChanceStmt::from_nodes(x).map(Expr::Random),
            _ => Err(Error::TooManyExprs),
        }
    }
}

fn main() {
    let kdl = RandlFile::open("test.kdl").unwrap();

    dbg!(kdl);
}
