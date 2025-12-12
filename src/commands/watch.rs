use crate::client::ConsulXClient;
use crate::errors::Result;

pub async fn cmd_watch_key(client: &ConsulXClient, key: &str) -> Result<()> {
    println!("Watching key '{key}' (Ctrl+C to stop)...");

    let mut index = None;

    loop {
        let (new_index, value) = client.kv_watch(key, index).await?;
        index = Some(new_index);

        match value {
            Some(v) => println!("UPDATED [{key}] -> {v}"),
            None => println!("UPDATED [{key}] -> <nil>"),
        }
    }
}

pub async fn cmd_watch_prefix(client: &ConsulXClient, prefix: &str) -> Result<()> {
    println!("Watching prefix '{prefix}' (Ctrl+C to stop)...");

    let mut index = None;

    loop {
        let (new_index, keys) = client.kv_watch_prefix(prefix, index).await?;
        index = Some(new_index);

        println!("UPDATED PREFIX [{prefix}]:");
        if keys.is_empty() {
            println!("  <empty>");
        } else {
            for k in keys {
                println!("  {k}");
            }
        }
    }
}
