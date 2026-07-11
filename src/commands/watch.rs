use crate::client::ConsulXClient;
use crate::errors::Result;
use std::time::Duration;

/// Consul blocking queries return the same `X-Consul-Index` when nothing has
/// changed (the query simply timed out), and a *smaller* index means the
/// table was reset — in which case we must restart from index 0. This helper
/// applies that reconciliation and reports whether the state actually changed.
fn reconcile(prev: Option<u64>, mut new_index: u64) -> (u64, bool) {
    if let Some(prev) = prev {
        if new_index < prev {
            new_index = 0; // reset per Consul blocking-query guidance
        }
        (new_index, new_index != prev)
    } else {
        (new_index, true) // first observation always prints
    }
}

pub async fn cmd_watch_key(client: &ConsulXClient, key: &str) -> Result<()> {
    println!("Watching key '{key}' (Ctrl+C to stop)...");

    let mut index: Option<u64> = None;

    loop {
        let (raw_index, value) = client.kv_watch(key, index).await?;
        let (new_index, changed) = reconcile(index, raw_index);

        if changed {
            match value {
                Some(v) => println!("UPDATED [{key}] -> {v}"),
                None => println!("UPDATED [{key}] -> <nil>"),
            }
        }

        index = Some(new_index);

        // A 0 index can't be used as a blocking cursor, so Consul would return
        // immediately — back off to avoid hammering the agent in a tight loop.
        if new_index == 0 {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}

pub async fn cmd_watch_prefix(client: &ConsulXClient, prefix: &str) -> Result<()> {
    println!("Watching prefix '{prefix}' (Ctrl+C to stop)...");

    let mut index: Option<u64> = None;

    loop {
        let (raw_index, keys) = client.kv_watch_prefix(prefix, index).await?;
        let (new_index, changed) = reconcile(index, raw_index);

        if changed {
            println!("UPDATED PREFIX [{prefix}]:");
            if keys.is_empty() {
                println!("  <empty>");
            } else {
                for k in keys {
                    println!("  {k}");
                }
            }
        }

        index = Some(new_index);

        if new_index == 0 {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}
