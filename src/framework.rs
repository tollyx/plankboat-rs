use regex::Regex;

use serenity::framework::Framework;
use serenity::client::Context;
use serenity::model::channel::Message;
use threadpool::ThreadPool;

use std::sync::Arc;
use std::str::FromStr;

use commands;
use commands::Command;

pub struct PlankFramework {
    command_prefix: &'static str,
    command_re: Regex,
    dice: Arc<commands::games::DiceRoll>,
    roulette: Arc<commands::games::Roulette>,
}

impl PlankFramework {
    pub fn new() -> PlankFramework {
        PlankFramework {
            command_prefix: "^",
            command_re: Regex::new(r#"'.*?'|".*?"|\S+"#).unwrap(),
            dice: Arc::new(commands::games::DiceRoll::new()),
            roulette: Arc::new(commands::games::Roulette::new()),
        }
    }

    fn parse_command(&self, msg: &str) -> Option<Vec<String>> {
        if msg.starts_with(self.command_prefix) {
            let cmd = &msg[self.command_prefix.len()..];
            if cmd.len() > 0 {
                let args: Vec<String> = self.command_re.captures_iter(cmd)
                    .map(|c| String::from_str(c.get(0).unwrap().as_str()).unwrap()).collect();
                if args.len() > 0 {
                    return Some(args.to_vec());
                }
            }
        }
        None
    }
}

impl Framework for PlankFramework {
    fn dispatch(&mut self, mut ctx: Context, msg: Message, pool: &ThreadPool) {
        if let Some(cmd) = self.parse_command(&msg.content) {

            match cmd[0].as_str() {
                "roll" => {
                    info!("Dispatching command '{}'", &cmd[0]);
                    let command = Arc::clone(&self.dice);
                    pool.execute(move || {
                        if let Err(e) = command.execute(&mut ctx, &msg, &cmd){
                            error!("{}", e);
                        };
                    });
                },
                "roulette" => {
                    let command = Arc::clone(&self.roulette);
                    info!("Dispatching command '{}'", &cmd[0]);
                    pool.execute(move || {
                        if let Err(e) = command.execute(&mut ctx, &msg, &cmd){
                            error!("{}", e);
                        };
                    });
                }
                _ => {
                    info!("Command not found: {}", cmd[0]);
                }
            }
        }
    }
}