use crate::errors::{Result, ConsulXError};

#[derive(Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_whitespace_are_empty() {
        assert_eq!(parse("").unwrap(), Command::Empty);
        assert_eq!(parse("    ").unwrap(), Command::Empty);
        assert_eq!(parse("\t \n").unwrap(), Command::Empty);
    }

    #[test]
    fn get_parses_key() {
        assert_eq!(
            parse("get app/db").unwrap(),
            Command::Get { key: "app/db".into() }
        );
    }

    #[test]
    fn command_word_is_case_insensitive() {
        assert_eq!(
            parse("GET app/db").unwrap(),
            Command::Get { key: "app/db".into() }
        );
        assert_eq!(
            parse("Put k v").unwrap(),
            Command::Put { key: "k".into(), value: "v".into() }
        );
    }

    #[test]
    fn del_and_delete_are_aliases() {
        let expected = Command::Delete { key: "k".into() };
        assert_eq!(parse("del k").unwrap(), expected);
        assert_eq!(parse("delete k").unwrap(), expected);
    }

    #[test]
    fn put_preserves_internal_whitespace() {
        // Regression: split_whitespace().join(" ") used to collapse the
        // double space into a single one.
        assert_eq!(
            parse("put k hello   world").unwrap(),
            Command::Put { key: "k".into(), value: "hello   world".into() }
        );
    }

    #[test]
    fn put_strips_one_pair_of_surrounding_quotes() {
        assert_eq!(
            parse(r#"put k "a  b""#).unwrap(),
            Command::Put { key: "k".into(), value: "a  b".into() }
        );
        assert_eq!(
            parse("put k 'single'").unwrap(),
            Command::Put { key: "k".into(), value: "single".into() }
        );
    }

    #[test]
    fn put_keeps_unbalanced_or_inner_quotes() {
        assert_eq!(
            parse(r#"put k "unclosed"#).unwrap(),
            Command::Put { key: "k".into(), value: r#""unclosed"#.into() }
        );
        assert_eq!(
            parse(r#"put k say "hi""#).unwrap(),
            Command::Put { key: "k".into(), value: r#"say "hi""#.into() }
        );
    }

    #[test]
    fn put_requires_key_and_value() {
        assert!(matches!(
            parse("put"),
            Err(ConsulXError::MissingArgument("key"))
        ));
        assert!(matches!(
            parse("put onlykey"),
            Err(ConsulXError::MissingArgument("value"))
        ));
    }

    #[test]
    fn put_json_is_kept_verbatim() {
        assert_eq!(
            parse(r#"put-json k {"a": 1,  "b": 2}"#).unwrap(),
            Command::PutJson { key: "k".into(), json: r#"{"a": 1,  "b": 2}"#.into() }
        );
    }

    #[test]
    fn list_and_tree_default_to_empty_prefix() {
        assert_eq!(parse("list").unwrap(), Command::List { prefix: "".into() });
        assert_eq!(parse("tree").unwrap(), Command::Tree { prefix: "".into() });
    }

    #[test]
    fn watch_variants() {
        assert_eq!(parse("watch k").unwrap(), Command::Watch { key: "k".into() });
        assert_eq!(
            parse("watch-prefix app/").unwrap(),
            Command::WatchPrefix { prefix: "app/".into() }
        );
        assert!(matches!(
            parse("watch-prefix"),
            Err(ConsulXError::MissingArgument("prefix"))
        ));
    }

    #[test]
    fn help_aliases() {
        assert_eq!(parse("help").unwrap(), Command::Help);
        assert_eq!(parse("?").unwrap(), Command::Help);
    }

    #[test]
    fn exit_and_quit_map_to_empty() {
        assert_eq!(parse("exit").unwrap(), Command::Empty);
        assert_eq!(parse("quit").unwrap(), Command::Empty);
    }

    #[test]
    fn unknown_command_reports_name() {
        match parse("frobnicate x") {
            Err(ConsulXError::UnknownCommand(c)) => assert_eq!(c, "frobnicate"),
            other => panic!("expected UnknownCommand, got {other:?}"),
        }
    }

    #[test]
    fn unquote_helper() {
        assert_eq!(unquote(r#""abc""#), "abc");
        assert_eq!(unquote("'abc'"), "abc");
        assert_eq!(unquote("abc"), "abc");
        assert_eq!(unquote(r#""mismatch'"#), r#""mismatch'"#);
        assert_eq!(unquote("\""), "\""); // single char, not a pair
        assert_eq!(unquote(""), "");
    }
}
