use serde_json::Value;
use std::collections::HashMap;
use std::hash::BuildHasher;

use super::{EvalContext, Expr};

pub(super) fn eval_bool<S: BuildHasher>(
    expr: &Expr,
    context: &EvalContext<'_, S>,
) -> Result<bool, String> {
    bool_value(eval(expr, context)?, "conditional expression")
}

fn eval<S: BuildHasher>(expr: &Expr, context: &EvalContext<'_, S>) -> Result<Value, String> {
    match expr {
        Expr::Literal(value) => Ok(value.clone()),
        Expr::Variable(name) => lookup(name, context),
        Expr::Array(values) => values
            .iter()
            .map(|value| eval(value, context))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Expr::Equal(left, right) => Ok(Value::Bool(eval(left, context)? == eval(right, context)?)),
        Expr::NotEqual(left, right) => {
            Ok(Value::Bool(eval(left, context)? != eval(right, context)?))
        }
        Expr::In(left, right) => {
            let needle = eval(left, context)?;
            match eval(right, context)? {
                Value::Array(values) => {
                    Ok(Value::Bool(values.iter().any(|value| value == &needle)))
                }
                _ => Err("right side of `in` must be an array".to_string()),
            }
        }
        Expr::And(left, right) => {
            if !bool_value(eval(left, context)?, "left side of `and`")? {
                return Ok(Value::Bool(false));
            }
            Ok(Value::Bool(bool_value(eval(right, context)?, "right side of `and`")?))
        }
        Expr::Or(left, right) => {
            if bool_value(eval(left, context)?, "left side of `or`")? {
                return Ok(Value::Bool(true));
            }
            Ok(Value::Bool(bool_value(eval(right, context)?, "right side of `or`")?))
        }
    }
}

fn lookup<S: BuildHasher>(name: &str, context: &EvalContext<'_, S>) -> Result<Value, String> {
    let parts = name.split('.').collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(format!("invalid identifier `{name}`"));
    }
    match parts.as_slice() {
        ["frontmatter"] | ["config"] => Err(format!("identifier `{name}` needs a key")),
        ["frontmatter", rest @ ..] => lookup_path(context.frontmatter, rest, name),
        ["config", rest @ ..] => lookup_path(context.config, rest, name),
        _ => lookup_path(context.frontmatter, &parts, name)
            .or_else(|_| lookup_path(context.config, &parts, name)),
    }
}

fn lookup_path<S: BuildHasher>(
    map: &HashMap<String, Value, S>,
    parts: &[&str],
    name: &str,
) -> Result<Value, String> {
    let Some((first, rest)) = parts.split_first() else {
        return Err(format!("unknown identifier `{name}`"));
    };
    let Some(mut value) = map.get(*first) else {
        return Err(format!("unknown identifier `{name}`"));
    };
    for part in rest {
        let Value::Object(object) = value else {
            return Err(format!("unknown identifier `{name}`"));
        };
        let Some(next) = object.get(*part) else {
            return Err(format!("unknown identifier `{name}`"));
        };
        value = next;
    }
    Ok(value.clone())
}

fn bool_value(value: Value, label: &str) -> Result<bool, String> {
    match value {
        Value::Bool(value) => Ok(value),
        _ => Err(format!("{label} must evaluate to a boolean")),
    }
}
