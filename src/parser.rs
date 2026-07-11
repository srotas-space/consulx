use crate::errors::{Result, ConsulXError};

#[derive(Debug)]
pub enum Command {
    Get { key: String },
    Put { key: String, value: String },
    Delete { key: String },
    List { prefix: String },
    Watch { key: String },
    WatchPrefix { prefix: String },
    Tree { prefix: String },
    GetJson { key: String },
    PutJson { key: String, json: String },
    Edit { key: String },
    Help,
    Empty,
}

/// Strip one pair of matching surrounding quotes, if present.
fn unquote(s: &str) -> &str {
    let bytes = s.as_bytes();
    if s.len() >= 2 {
        let (first, last) = (bytes[0], bytes[bytes.len() - 1]);
        if (first == b'"' || first == b'\'') && first == last {
            return &s[1..s.len() - 1];
        }
    }
    s
}

/// Split `input` into (command, first-arg, verbatim-rest). The rest preserves
/// the original spacing so values with embedded whitespace survive intact.
fn split3(input: &str) -> (String, Option<&str>, Option<&str>) {
    let mut it = input.splitn(3, char::is_whitespace);
    let cmd = it.next().unwrap_or("").to_lowercase();
    let arg = it.next().filter(|s| !s.is_empty());
    let rest = it.next().map(str::trim_start).filter(|s| !s.is_empty());
    (cmd, arg, rest)
}

pub fn parse(input: &str) -> Result<Command> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Ok(Command::Empty);
    }

    let mut p = trimmed.split_whitespace();
    let cmd = p.next().unwrap().to_lowercase();

    match cmd.as_str() {
        "get" => Ok(Command::Get {
            key: p
                .next()
                .ok_or(ConsulXError::MissingArgument("key"))?
                .into(),
        }),
        "put" => {
            let (_, key, rest) = split3(trimmed);
            let key = key.ok_or(ConsulXError::MissingArgument("key"))?;
            let value = rest.ok_or(ConsulXError::MissingArgument("value"))?;
            Ok(Command::Put {
                key: key.into(),
                value: unquote(value).to_string(),
            })
        }
        "del" | "delete" => Ok(Command::Delete {
            key: p
                .next()
                .ok_or(ConsulXError::MissingArgument("key"))?
                .into(),
        }),
        "list" => Ok(Command::List {
            prefix: p.next().unwrap_or("").into(),
        }),
        "watch" => Ok(Command::Watch {
            key: p
                .next()
                .ok_or(ConsulXError::MissingArgument("key"))?
                .into(),
        }),
        "watch-prefix" => Ok(Command::WatchPrefix {
            prefix: p
                .next()
                .ok_or(ConsulXError::MissingArgument("prefix"))?
                .into(),
        }),
        "tree" => Ok(Command::Tree {
            prefix: p.next().unwrap_or("").into(),
        }),
        "get-json" => Ok(Command::GetJson {
            key: p
                .next()
                .ok_or(ConsulXError::MissingArgument("key"))?
                .into(),
        }),
        "put-json" => {
            let (_, key, rest) = split3(trimmed);
            let key = key.ok_or(ConsulXError::MissingArgument("key"))?;
            let json = rest.ok_or(ConsulXError::MissingArgument("json"))?;
            Ok(Command::PutJson {
                key: key.into(),
                json: json.to_string(),
            })
        }
        "edit" => Ok(Command::Edit {
            key: p
                .next()
                .ok_or(ConsulXError::MissingArgument("key"))?
                .into(),
        }),
        "help" | "?" => Ok(Command::Help),
        "exit" | "quit" => Ok(Command::Empty),
        other => Err(ConsulXError::UnknownCommand(other.to_string())),
    }
}
