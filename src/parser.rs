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
            let key = p
                .next()
                .ok_or(ConsulXError::MissingArgument("key"))?;
            let value = p.collect::<Vec<_>>().join(" ");
            if value.is_empty() {
                return Err(ConsulXError::MissingArgument("value"));
            }
            Ok(Command::Put {
                key: key.into(),
                value,
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
            let key = p
                .next()
                .ok_or(ConsulXError::MissingArgument("key"))?;
            let json = p.collect::<Vec<_>>().join(" ");
            if json.is_empty() {
                return Err(ConsulXError::MissingArgument("json"));
            }
            Ok(Command::PutJson {
                key: key.into(),
                json,
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
