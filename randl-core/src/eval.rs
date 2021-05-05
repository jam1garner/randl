use rand::prelude::*;
use std::convert::TryInto;

use super::*;
use prc::{ParamKind, ParamStruct};

type Result<T> = std::result::Result<T, EvalError>;

fn struct_lookup<'a>(param: &'a mut ParamStruct, field: &str) -> Result<&'a mut ParamKind> {
    let hash = prc::hash40::to_hash40(field);
    let index = param
        .0
        .iter()
        .position(|x| x.0 == hash)
        .ok_or_else(|| EvalError::MissingField(format!("Field `{}` is missing", field)))?;

    Ok(param.0.get_mut(index).map(|x| &mut x.1).unwrap())
}

fn get_path_param_kind<'a>(
    param: &'a mut ParamKind,
    mut path: PrcPathSlice,
) -> Result<Vec<&'a mut ParamKind>> {
    let old_path = path.clone();
    match path.pop_front() {
        Some(PrcPathComponent::Field(field)) => match param {
            ParamKind::Struct(param) => get_path_struct(param, old_path),

            _ => Err(EvalError::MissingField(format!(
                "Cannot get field {:?} of non-struct param",
                field,
            ))),
        },
        Some(&PrcPathComponent::Index(index)) => match param {
            ParamKind::List(param) => get_path_param_kind(
                param
                    .0
                    .get_mut(index)
                    .ok_or_else(|| EvalError::IndexOutOfBounds(index))?,
                path,
            ),
            _ => Err(EvalError::InvalidField(
                "Cannot index into a non-list param",
            )),
        },
        Some(PrcPathComponent::Wildcard) => match param {
            ParamKind::Struct(param) => {
                let next = path;
                param
                    .0
                    .iter_mut()
                    .map(|x| get_path_param_kind(&mut x.1, next.clone()))
                    .fold(Ok(Vec::new()), |mut vec, mut new| {
                        let vec_ref = vec.as_mut().map_err(|x| x.clone())?;
                        let new = new.as_mut().map_err(|x| x.clone())?;
                        vec_ref.append(new);
                        vec
                    })
            }
            ParamKind::List(list) => {
                let next = path;
                list.0
                    .iter_mut()
                    .map(|x| get_path_param_kind(x, next.clone()))
                    .fold(Ok(Vec::new()), |mut vec, mut new| {
                        let vec_ref = vec.as_mut().map_err(|x| x.clone())?;
                        let new = new.as_mut().map_err(|x| x.clone())?;
                        vec_ref.append(new);
                        vec
                    })
            }
            _ => Err(EvalError::InvalidField(
                "A wildcard can only be applied to a struct or list",
            )),
        },
        None => Ok(Vec::from([param])),
    }
}

fn get_path_struct<'a>(
    param: &'a mut ParamStruct,
    mut path: PrcPathSlice,
) -> Result<Vec<&'a mut ParamKind>> {
    match path.pop_front() {
        Some(PrcPathComponent::Field(field)) => {
            let field = struct_lookup(param, field)?;
            get_path_param_kind(field, path)
        }
        Some(PrcPathComponent::Index(index)) => Err(EvalError::MissingField(
            "Cannot index into a `ParamStruct`".into(),
        )),
        Some(PrcPathComponent::Wildcard) => {
            let next = path;
            param
                .0
                .iter_mut()
                .map(|x| get_path_param_kind(&mut x.1, next.clone()))
                .fold(Ok(Vec::new()), |mut vec, mut new| {
                    let vec_ref = vec.as_mut().map_err(|x| x.clone())?;
                    let new = new.as_mut().map_err(|x| x.clone())?;
                    vec_ref.append(new);
                    vec
                })
        }
        None => Err(EvalError::InvalidField(
            "Path pointed to a struct, edit one field at a time.",
        )),
    }
}

fn e(a: &'static str, b: &'static str) -> Result<()> {
    Err(EvalError::InvalidAssignment(a, b))
}

impl RandlEntry {
    pub fn apply(&self, file: &mut ParamStruct, sets: &HashMap<String, Set>) -> Result<()> {
        for field in self.prc_fields.iter() {
            let to_edit = get_path_struct(file, field.path.as_ref())?;

            for param in to_edit {
                let val = field.expr.eval(sets)?;

                match (param, val) {
                    (_, Value::Original) => {}
                    (ParamKind::Bool(b), Value::Bool(new)) => {
                        *b = new;
                    }
                    (ParamKind::I8(i), Value::Int(new)) => {
                        *i = new.try_into().map_err(|_| EvalError::IntTooBig)?;
                    }
                    (ParamKind::U8(i), Value::Int(new)) => {
                        *i = new.try_into().map_err(|_| EvalError::IntTooBig)?;
                    }
                    (ParamKind::I16(i), Value::Int(new)) => {
                        *i = new.try_into().map_err(|_| EvalError::IntTooBig)?;
                    }
                    (ParamKind::U16(i), Value::Int(new)) => {
                        *i = new.try_into().map_err(|_| EvalError::IntTooBig)?;
                    }
                    (ParamKind::I32(i), Value::Int(new)) => {
                        *i = new.try_into().map_err(|_| EvalError::IntTooBig)?;
                    }
                    (ParamKind::U32(i), Value::Int(new)) => {
                        *i = new.try_into().map_err(|_| EvalError::IntTooBig)?;
                    }
                    (ParamKind::Float(f), Value::Int(new)) => {
                        *f = new as f32;
                    }
                    (ParamKind::Float(f), Value::Float(new)) => {
                        *f = new as f32;
                    }
                    (ParamKind::Hash(h), Value::Int(new)) => {
                        *h = Hash40(new as u64);
                    }
                    (ParamKind::Hash(h), Value::String(s)) => {
                        *h = prc::hash40::to_hash40(&s);
                    }
                    (ParamKind::Hash(h), Value::Hash40(new)) => {
                        *h = new;
                    }
                    (ParamKind::Str(s), Value::String(new)) => {
                        *s = new;
                    }
                    (ParamKind::List(_), _) => {
                        return Err(EvalError::InvalidAssignment("values", "lists"))
                    }
                    (ParamKind::Struct(_), _) => {
                        return Err(EvalError::InvalidAssignment("values", "structs"))
                    }
                    // look, sometimes I write bad code too.
                    // at minimum give me credit for automating this externally
                    (ParamKind::Bool(_), Value::Int(_)) => return e("int", "bool"),
                    (ParamKind::Bool(_), Value::Float(_)) => return e("float", "bool"),
                    (ParamKind::Bool(_), Value::String(_)) => return e("string", "bool"),
                    (ParamKind::Bool(_), Value::Hash40(_)) => return e("hash40", "bool"),
                    (ParamKind::I8(_), Value::Float(_)) => return e("float", "int"),
                    (ParamKind::I8(_), Value::String(_)) => return e("string", "int"),
                    (ParamKind::I8(_), Value::Bool(_)) => return e("bool", "int"),
                    (ParamKind::I8(_), Value::Hash40(_)) => return e("hash40", "int"),
                    (ParamKind::U8(_), Value::Float(_)) => return e("float", "int"),
                    (ParamKind::U8(_), Value::String(_)) => return e("string", "int"),
                    (ParamKind::U8(_), Value::Bool(_)) => return e("bool", "int"),
                    (ParamKind::U8(_), Value::Hash40(_)) => return e("hash40", "int"),
                    (ParamKind::I16(_), Value::Float(_)) => return e("float", "int"),
                    (ParamKind::I16(_), Value::String(_)) => return e("string", "int"),
                    (ParamKind::I16(_), Value::Bool(_)) => return e("bool", "int"),
                    (ParamKind::I16(_), Value::Hash40(_)) => return e("hash40", "int"),
                    (ParamKind::U16(_), Value::Float(_)) => return e("float", "int"),
                    (ParamKind::U16(_), Value::String(_)) => return e("string", "int"),
                    (ParamKind::U16(_), Value::Bool(_)) => return e("float", "int"),
                    (ParamKind::U16(_), Value::Hash40(_)) => return e("hash40", "int"),
                    (ParamKind::I32(_), Value::Float(_)) => return e("float", "int"),
                    (ParamKind::I32(_), Value::String(_)) => return e("string", "int"),
                    (ParamKind::I32(_), Value::Bool(_)) => return e("bool", "int"),
                    (ParamKind::I32(_), Value::Hash40(_)) => return e("hash40", "int"),
                    (ParamKind::U32(_), Value::Float(_)) => return e("float", "int"),
                    (ParamKind::U32(_), Value::String(_)) => return e("string", "int"),
                    (ParamKind::U32(_), Value::Bool(_)) => return e("bool", "int"),
                    (ParamKind::U32(_), Value::Hash40(_)) => return e("hash40", "int"),
                    (ParamKind::Float(_), Value::String(_)) => return e("string", "float"),
                    (ParamKind::Float(_), Value::Bool(_)) => return e("bool", "float"),
                    (ParamKind::Float(_), Value::Hash40(_)) => return e("hash40", "float"),
                    (ParamKind::Hash(_), Value::Float(_)) => return e("float", "hash40"),
                    (ParamKind::Hash(_), Value::Bool(_)) => return e("string", "float"),
                    (ParamKind::Str(_), Value::Int(_)) => return e("int", "string"),
                    (ParamKind::Str(_), Value::Float(_)) => return e("float", "string"),
                    (ParamKind::Str(_), Value::Bool(_)) => return e("bool", "string"),
                    (ParamKind::Str(_), Value::Hash40(_)) => return e("hash40", "string"),
                }
            }
        }

        Ok(())
    }
}

impl Expr {
    fn eval(&self, sets: &HashMap<String, Set>) -> Result<Value> {
        match self {
            Expr::Random(chances) => {
                if chances.len() == 1 {
                    chances[0].expr.eval(sets)
                } else {
                    let mut percentile = rand::thread_rng().gen::<f64>() * 100.0;

                    let (last, chances) = chances.split_last().unwrap();
                    for chance in chances {
                        percentile -= chance.percent;
                        if percentile < 0.0 {
                            return chance.expr.eval(sets)
                        }
                    }

                    last.expr.eval(sets)
                }
            }
            Expr::Return(ret) => match ret {
                Return::Constant(c) => Ok(c.clone()),
                Return::Range(range) => match range {
                    &Range::Int(from, to) => Ok(Value::Int(rand::thread_rng().gen_range(from..to))),
                    &Range::Float(from, to) => {
                        Ok(Value::Float(rand::thread_rng().gen_range(from..to)))
                    }
                },
                Return::Set(set) => {
                    let set = sets.get(set).ok_or_else(|| EvalError::InvalidSet(set.clone()))?;
                    set.eval()
                }
                Return::AnonymousSet(set) => set.eval(),
            },
            Expr::Original => Ok(Value::Original),
        }
    }
}

impl Set {
    fn eval(&self) -> Result<Value> {
        let len = self.0.len();

        Ok(self.0[rand::thread_rng().gen_range(0..len)].clone())
    }
}