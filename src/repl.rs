use reedline::{DefaultPrompt, DefaultCompleter, Reedline, Signal};
use colored::Colorize;

use crate::client::ConsulXClient;
use crate::commands::{kv, watch};
use crate::errors::{ConsulXError, Result};
use crate::parser::{parse, Command};

pub async fn start_repl(client: ConsulXClient) -> Result<()> {
    let client = std::sync::Arc::new(client);

    // Basic command completion
    let commands = vec![
        "get".into(),
        "put".into(),
        "del".into(),
        "delete".into(),
        "list".into(),
        "watch".into(),
        "watch-prefix".into(),
        "tree".into(),
        "get-json".into(),
        "put-json".into(),
        "edit".into(),
        "help".into(),
        "exit".into(),
        "quit".into(),
    ];
    let completer = Box::new(DefaultCompleter::new_with_wordlen(commands, 2));

    // ❗ FIX: use default prompt instead of constructing from &str
    let mut line_editor = Reedline::create().with_completer(completer);
    let prompt = DefaultPrompt::default();

    println!("{}", "Welcome to consulx REPL".bold());
    println!("Type 'help' for commands. 'exit' or 'quit' to leave.");

    loop {
        match line_editor.read_line(&prompt) {
            Ok(Signal::Success(input)) => {
                let trimmed = input.trim();

                if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
                    println!("bye 👋");
                    break;
                }

                match parse(trimmed) {
                    Ok(Command::Empty) => {}
                    Ok(Command::Help) => print_help(),
                    Ok(Command::Get { key }) => kv::cmd_get(&client, &key).await?,
                    Ok(Command::Put { key, value }) => kv::cmd_put(&client, &key, &value).await?,
                    Ok(Command::Delete { key }) => kv::cmd_delete(&client, &key).await?,
                    Ok(Command::List { prefix }) => kv::cmd_list(&client, &prefix).await?,
                    Ok(Command::Watch { key }) => watch::cmd_watch_key(&client, &key).await?,
                    Ok(Command::WatchPrefix { prefix }) => {
                        watch::cmd_watch_prefix(&client, &prefix).await?
                    }
                    Ok(Command::Tree { prefix }) => kv::cmd_tree(&client, &prefix).await?,
                    Ok(Command::GetJson { key }) => kv::cmd_get_json(&client, &key).await?,
                    Ok(Command::PutJson { key, json }) => {
                        kv::cmd_put_json(&client, &key, &json).await?
                    }
                    Ok(Command::Edit { key }) => kv::cmd_edit(&client, &key).await?,
                    Err(ConsulXError::UnknownCommand(cmd)) => {
                        eprintln!("{} {}", "Unknown command:".red(), cmd);
                    }
                    Err(e) => {
                        eprintln!("{} {}", "Error:".red(), e);
                    }
                }
            }
            Ok(Signal::CtrlC) | Ok(Signal::CtrlD) => {
                println!("\nbye 👋");
                break;
            }
            Err(e) => {
                eprintln!("Readline error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("{}", "Commands:".bold());
    println!("  get <key>");
    println!("  put <key> <value>");
    println!("  del|delete <key>");
    println!("  list <prefix>");
    println!("  watch <key>");
    println!("  watch-prefix <prefix>");
    println!("  tree <prefix>             # ASCII tree view");
    println!("  get-json <key>           # pretty-print JSON");
    println!("  put-json <key> <json>    # validate & store JSON");
    println!("  edit <key>               # open value in $EDITOR");
    println!("  help");
    println!("  exit | quit");
}
