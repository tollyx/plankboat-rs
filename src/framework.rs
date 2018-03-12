use regex::Regex;

use serenity::framework::Framework;
use serenity::client::Context;
use serenity::model::channel::Message;
use threadpool::ThreadPool;

use std::sync::Arc;
use std::str::FromStr;

use std::collections::HashMap;

use commands;
use commands::Command;

pub struct PlankFramework {
    command_prefix: &'static str,
    command_re: Regex,
    commands: HashMap<String, Arc<Command>>,
}

impl PlankFramework {
    pub fn new() -> PlankFramework {
        let mut fw = PlankFramework {
            command_prefix: "^",
            command_re: Regex::new(r#"'.*?'|".*?"|\S+"#).unwrap(),
            commands: HashMap::new(),
        };
        fw.commands.insert("roll".to_string(), Arc::new(commands::games::DiceRoll::new()));
        fw.commands.insert("roulette".to_string(), Arc::new(commands::games::Roulette::new()));

        fw
    }

    fn parse_command(&self, msg: &str) -> Option<Vec<String>> {
        if msg.starts_with(self.command_prefix) {
            let cmd = &msg[self.command_prefix.len()..];
            if cmd.len() > 0 {
                let args: Vec<String> = self.command_re.captures_iter(cmd)
                    .map(|c| String::from_str(c.get(0).unwrap().as_str()).unwrap()).collect();
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
        if let Some(cmd) = self.parse_command(&msg.content) {
            if let Some(command) = self.commands.get(&cmd[0]) {
                info!("Dispatching command '{}'", &cmd[0]);
                let c = Arc::clone(command);
                pool.execute(move || {
                    if let Err(e) = c.execute(&mut ctx, &msg, &cmd){
                        error!("{}", e);
                    };
                });
            }
            else {
                info!("Command not found: {}", cmd[0]);
            }
        }
    }
}