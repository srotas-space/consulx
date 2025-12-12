use crate::client::ConsulXClient;
use crate::errors::Result;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::process::Command as ProcCommand;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn cmd_get(client: &ConsulXClient, key: &str) -> Result<()> {
    match client.kv_get_raw(key).await? {
        Some(v) => println!("{v}"),
        None => println!("<nil>"),
    }
    Ok(())
}

pub async fn cmd_put(client: &ConsulXClient, key: &str, value: &str) -> Result<()> {
    client.kv_put(key, value).await?;
    println!("OK");
    Ok(())
}

pub async fn cmd_delete(client: &ConsulXClient, key: &str) -> Result<()> {
    client.kv_delete(key).await?;
    println!("OK");
    Ok(())
}

pub async fn cmd_list(client: &ConsulXClient, prefix: &str) -> Result<()> {
    let keys = client.kv_list(prefix).await?;
    if keys.is_empty() {
        println!("<empty>");
    } else {
        for k in keys {
            println!("{k}");
        }
    }
    Ok(())
}

/// JSON-aware GET: parses as JSON and pretty-prints
pub async fn cmd_get_json(client: &ConsulXClient, key: &str) -> Result<()> {
    match client.kv_get_raw(key).await? {
        Some(v) => {
            match serde_json::from_str::<Value>(&v) {
                Ok(json) => {
                    let pretty = serde_json::to_string_pretty(&json)?;
                    println!("{pretty}");
                }
                Err(_) => {
                    eprintln!("Value is not valid JSON, raw:");
                    println!("{v}");
                }
            }
        }
        None => println!("<nil>"),
    }
    Ok(())
}

/// JSON-aware PUT: validates JSON, then stores minified JSON
pub async fn cmd_put_json(client: &ConsulXClient, key: &str, json_str: &str) -> Result<()> {
    let json: Value = serde_json::from_str(json_str)?;
    let minified = serde_json::to_string(&json)?;
    client.kv_put(key, &minified).await?;
    println!("OK (json)");
    Ok(())
}

/// Build a tree from keys and print as ASCII tree
pub async fn cmd_tree(client: &ConsulXClient, prefix: &str) -> Result<()> {
    let keys = client.kv_list(prefix).await?;

    if keys.is_empty() {
        println!("<empty>");
        return Ok(());
    }

    #[derive(Default)]
    struct Node {
        children: BTreeMap<String, Node>,
        is_leaf: bool,
    }

    let mut root = Node::default();

    for key in keys {
        let rel = if prefix.is_empty() {
            key.clone()
        } else {
            key.strip_prefix(prefix).unwrap_or(&key).trim_start_matches('/').to_string()
        };

        let parts: Vec<&str> = rel.split('/').filter(|p| !p.is_empty()).collect();
        if parts.is_empty() {
            continue;
        }

        let mut cur = &mut root;
        for (i, part) in parts.iter().enumerate() {
            cur = cur.children.entry((*part).to_string()).or_default();
            if i == parts.len() - 1 {
                cur.is_leaf = true;
            }
        }
    }

    fn print_node(node: &Node, name: &str, prefix: &str, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };
        println!("{}{}{}", prefix, connector, name);

        let mut children_iter = node.children.iter().peekable();
        while let Some((child_name, child_node)) = children_iter.next() {
            let next_last = children_iter.peek().is_none();
            let next_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            print_node(child_node, child_name, &next_prefix, next_last);
        }
    }

    // root has no name; print children
    let mut iter = root.children.iter().peekable();
    while let Some((name, node)) = iter.next() {
        let is_last = iter.peek().is_none();
        print_node(node, name, "", is_last);
    }

    Ok(())
}

/// Edit key using $EDITOR, write back on save
pub async fn cmd_edit(client: &ConsulXClient, key: &str) -> Result<()> {
    let current = client.kv_get_raw(key).await?.unwrap_or_default();

    // temp file path
    let mut path = std::env::temp_dir();
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    path.push(format!("consulx-{}-{}.tmp", key.replace('/', "_"), ts));

    // write current value
    fs::write(&path, current)?;

    // pick editor
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

    // launch editor (blocking)
    let status = ProcCommand::new(editor)
        .arg(&path)
        .status()?;

    if !status.success() {
        eprintln!("Editor exited with non-zero status, aborting update");
        return Ok(());
    }

    // read new value
    let new_val = fs::read_to_string(&path)?;
    client.kv_put(key, &new_val).await?;
    println!("OK (edited)");

    // optional cleanup: fs::remove_file(&path).ok();
    Ok(())
}
