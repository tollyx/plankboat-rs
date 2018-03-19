use regex::Regex;

use serenity::framework::Framework;
use serenity::client::Context;
use serenity::model::channel::Message;
use threadpool::ThreadPool;

use std::sync::Arc;
use std::str::FromStr;

use std::collections::HashMap;

use commands::Command;

pub struct PlankFramework {
    command_prefix: &'static str,
    commands: HashMap<String, Arc<Command>>,
}

impl PlankFramework {
    pub fn new() -> PlankFramework {
        let mut fw = PlankFramework {
            command_prefix: "^",
            commands: HashMap::new(),
        };

        fw
    }

    pub fn add_command<T: Command>(&mut self, name: &str, command: T) {
        self.commands.insert(name.to_string(), Arc::new(command));
    }

    fn parse_command(prefix: &str, msg: &str) -> Option<Vec<String>> {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r#"'.*?'|".*?"|\S+"#).unwrap();
        }

        if msg.starts_with(prefix) {
            let cmd = &msg[prefix.len()..];
            if cmd.len() > 0 && !cmd.chars().next().unwrap().is_whitespace() {
                let args: Vec<String> = REGEX.captures_iter(cmd)
                    .map(|c| {
                        let s = String::from_str(c.get(0).unwrap().as_str()).unwrap();
                        
                        if s.starts_with("'") {
                            s.replace("'", "")
                        }
                        else if s.starts_with("\"") {
                            s.replace("\"", "")
                        }
                        else {
                            s
                        }
                    }).collect();
                if args.len() > 0 {
                    return Some(args);
                }
            }
        }
        None
    }
}

impl Framework for PlankFramework {
    fn dispatch(&mut self, mut ctx: Context, msg: Message, pool: &ThreadPool) {
        if let Some(cmd) = PlankFramework::parse_command(self.command_prefix, &msg.content) {
            if let Some(command) = self.commands.get(&cmd[0]) {
                if cmd.len() > 1 {
                    info!("Dispatching command '{}' with args: {:?}", &cmd[0], &cmd[1..]);
                }
                else {
                    info!("Dispatching command: {}", &cmd[0]);
                }
                let c = Arc::clone(command);
                pool.execute(move || {
                    if let Err(e) = c.execute(&mut ctx, &msg, &cmd){
                        error!("{}", e);
                    };
                });
            }
            else {
                info!("Command not found: {:?}", &cmd);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_command() {
        let cmd = PlankFramework::parse_command("!", "!foobar").unwrap();
        assert_eq!(cmd[0], "foobar");
    }

    #[test]
    fn parse_command_multiple_args() {
        let cmd = PlankFramework::parse_command("!", "!foo bar test").unwrap();
        assert_eq!(cmd, vec!["foo", "bar", "test"]);
    }

    #[test]
    fn parse_command_quoted_args() {
        let cmd = PlankFramework::parse_command("!", "!test 'foo bar' test \"toot toot\"").unwrap();
        assert_eq!(cmd, vec!["test", "foo bar", "test", "toot toot"]);
    }

    #[test]
    fn parse_command_incorrect_prefix() {
        let cmd = PlankFramework::parse_command("!", "?test foobar");
        assert_eq!(cmd, None);
    }

    #[test]
    fn parse_command_space_after_prefix() {
        let cmd = PlankFramework::parse_command("!", "! test foobar");
        assert_eq!(cmd, None);
    }
}