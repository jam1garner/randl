use super::*;
use ParseError as Error;

use prc::hash40;

impl RandlFile {
    pub fn from_nodes(nodes: Vec<KdlNode>) -> Result<Self, Error> {
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

impl Value {
    fn from_node(mut node: KdlNode) -> Result<Self, Error> {
        if node.name == "value" {
            match node.values.len() {
                0 => {
                    if let Some(hash40) = node.properties.get("hash40") {
                        let hash40 = match hash40 {
                            &KdlValue::Int(hash) => Hash40(hash as u64),
                            KdlValue::String(s) => hash40::to_hash40(&s),
                            _ => {
                                return Err(Error::InvalidValueStmt(
                                    "`hash40` property must be a string or an integer",
                                ))
                            }
                        };

                        Ok(Value::Hash40(hash40))
                    } else {
                        return Err(Error::InvalidValueStmt(
                            "set `value` missing value. Syntax is `value [int/float/string/bool]` or `value hash40=[string/int]`",
                        ))
                    }
                }
                1 => Ok(match node.values.pop().unwrap() {
                    KdlValue::Int(int) => Value::Int(int),
                    KdlValue::Float(f) => Value::Float(f),
                    KdlValue::String(s) => Value::String(s),
                    KdlValue::Boolean(b) => Value::Bool(b),
                    KdlValue::Null => return Err(Error::InvalidType),
                }),
                _ => {
                    return Err(Error::InvalidReturn(
                        "Set value declerations may not represent more than one return value",
                    ))
                }
            }
        } else {
            Err(ParseError::NonValueInSet(node.name.clone()))
        }
        // if node.values.len() == 1 {
        //     match &*node.name {
        //         "int" => match node.values[0] {
        //             KdlValue::Int(int) => Ok(Value::Int(int)),
        //             _ => Err(Error::IncorrectType),
        //         },
        //         "string" => match node.values.pop().unwrap() {
        //             KdlValue::String(string) => Ok(Value::String(string)),
        //             _ => Err(Error::IncorrectType),
        //         },
        //         "float" => match node.values[0] {
        //             KdlValue::Float(float) => Ok(Value::Float(float)),
        //             _ => Err(Error::IncorrectType),
        //         },
        //         "bool" => match node.values[0] {
        //             KdlValue::Boolean(bool) => Ok(Value::Bool(bool)),
        //             _ => Err(Error::IncorrectType),
        //         },
        //         "hash40" => {
        //             todo!()
        //         }
        //         _ => Err(Error::InvalidType),
        //     }
        // } else {
        //     Err(Error::NoValue)
        // }
    }
}

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
                    .map(Set)
                    .map(Return::AnonymousSet)
            } else {
                return Err(Error::InvalidReturn(
                    "Return nodes with children cannot have values",
                ));
            }
        }
    }
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
