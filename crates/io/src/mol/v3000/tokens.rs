//! Iterative finite tokenizer: quoted strings and flat counted lists, no recursive DOM.
use super::{Attribute, Error, MAX_TOKENS, Value};

#[derive(Debug)]
pub(super) enum Token {
    Value(Value),
    Attribute(Attribute),
}
fn syntax(line: usize) -> Error {
    Error::Syntax {
        line,
        field: "entry tokens",
    }
}
fn whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t')
}
fn skip(bytes: &[u8], offset: &mut usize) {
    while bytes.get(*offset).copied().is_some_and(whitespace) {
        *offset += 1;
    }
}

fn text(source: &str, offset: &mut usize, line: usize) -> Result<String, Error> {
    let bytes = source.as_bytes();
    if bytes.get(*offset) == Some(&b'"') {
        *offset += 1;
        let mut output = String::new();
        while let Some(&byte) = bytes.get(*offset) {
            *offset += 1;
            if byte == b'"' {
                if bytes.get(*offset) == Some(&b'"') {
                    output.push('"');
                    *offset += 1;
                } else {
                    return Ok(output);
                }
            } else {
                output.push(char::from(byte));
            }
        }
        return Err(syntax(line));
    }
    let start = *offset;
    while let Some(&byte) = bytes.get(*offset) {
        if whitespace(byte) || matches!(byte, b'=' | b')') {
            break;
        }
        if matches!(byte, b'"' | b'\'' | b'(') {
            return Err(syntax(line));
        }
        *offset += 1;
    }
    if start == *offset {
        return Err(syntax(line));
    }
    Ok(source
        .get(start..*offset)
        .ok_or_else(|| syntax(line))?
        .to_owned())
}

fn value(
    source: &str,
    offset: &mut usize,
    line: usize,
    budget: &mut usize,
) -> Result<Value, Error> {
    let bytes = source.as_bytes();
    if bytes.get(*offset) != Some(&b'(') {
        return Ok(Value::Text(text(source, offset, line)?));
    }
    *offset += 1;
    skip(bytes, offset);
    let count = text(source, offset, line)?;
    if !count.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(syntax(line));
    }
    let count = count
        .parse::<usize>()
        .map_err(|_| Error::Resource("counted list"))?;
    if count > MAX_TOKENS {
        return Err(Error::Resource("counted list exceeds 8192 values"));
    }
    let mut values = Vec::new();
    for _ in 0..count {
        if !bytes.get(*offset).copied().is_some_and(whitespace) {
            return Err(syntax(line));
        }
        skip(bytes, offset);
        if bytes.get(*offset) == Some(&b'(') {
            return Err(Error::Unsupported {
                line,
                feature: "nested counted list",
            });
        }
        values.push(text(source, offset, line)?);
        *budget = budget
            .checked_add(1)
            .filter(|count| *count <= MAX_TOKENS)
            .ok_or(Error::Resource("entry exceeds 8192 tokens/list values"))?;
    }
    skip(bytes, offset);
    if bytes.get(*offset) != Some(&b')') {
        return Err(syntax(line));
    }
    *offset += 1;
    Ok(Value::List(values))
}

pub(super) fn tokenize(source: &str, line: usize) -> Result<Vec<Token>, Error> {
    let bytes = source.as_bytes();
    let mut offset = 0;
    let mut tokens = Vec::new();
    let mut budget = 0_usize;
    loop {
        skip(bytes, &mut offset);
        if offset == bytes.len() {
            break;
        }
        budget = budget
            .checked_add(1)
            .filter(|count| *count <= MAX_TOKENS)
            .ok_or(Error::Resource("entry exceeds 8192 tokens/list values"))?;
        let first = value(source, &mut offset, line, &mut budget)?;
        if bytes.get(offset) == Some(&b'=') {
            let Value::Text(key) = first else {
                return Err(syntax(line));
            };
            if key.is_empty()
                || key.bytes().any(|byte| {
                    whitespace(byte) || matches!(byte, b'"' | b'\'' | b'(' | b')' | b'=')
                })
            {
                return Err(syntax(line));
            }
            offset += 1;
            let value = value(source, &mut offset, line, &mut budget)?;
            tokens.push(Token::Attribute(Attribute { key, value }));
        } else {
            tokens.push(Token::Value(first));
        }
        if offset < bytes.len() && !bytes.get(offset).copied().is_some_and(whitespace) {
            return Err(syntax(line));
        }
    }
    if tokens.is_empty() {
        return Err(syntax(line));
    }
    Ok(tokens)
}
pub(super) fn positional(tokens: &[Token], index: usize, line: usize) -> Result<&str, Error> {
    match tokens.get(index) {
        Some(Token::Value(Value::Text(value))) => Ok(value),
        _ => Err(syntax(line)),
    }
}
pub(super) fn parts(
    tokens: Vec<Token>,
    count: usize,
    line: usize,
) -> Result<(Vec<String>, Vec<Attribute>), Error> {
    let mut positions = Vec::new();
    let mut attributes = Vec::new();
    for (index, token) in tokens.into_iter().enumerate() {
        match token {
            Token::Value(Value::Text(value)) if index < count => positions.push(value),
            Token::Attribute(attribute) if index >= count => {
                if attributes
                    .iter()
                    .any(|prior: &Attribute| prior.key == attribute.key)
                {
                    return Err(Error::Unsupported {
                        line,
                        feature: "duplicate attribute key",
                    });
                }
                attributes.push(attribute);
            }
            _ => return Err(syntax(line)),
        }
    }
    if positions.len() != count {
        return Err(syntax(line));
    }
    Ok((positions, attributes))
}
