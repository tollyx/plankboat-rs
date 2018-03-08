use serenity::client::Context;
use serenity::model::channel::Message;
use serenity;

use std::fmt;
use std::sync::Arc;

pub mod games;

pub type CommandResult = Result<(), CommandError>;

impl From<CommandError> for CommandResult {
    fn from(err: CommandError) -> CommandResult {
        Err(err)
    }
}

pub enum CommandError {
    Serenity(serenity::Error),
    Argument(String),
    Other(String),
}

impl From<serenity::Error> for CommandError {
    fn from(err: serenity::Error) -> CommandError {
        CommandError::Serenity(err)
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CommandError::Serenity(ref err) => {
                write!(f, "Error executing a command: {}", err)
            },
            CommandError::Argument(ref s) => {
                write!(f, "Invalid arguments to a command: {}", s)
            },
            CommandError::Other(ref s) => {
                write!(f, "CommandError: {}", s)
            }
        }
    }
}

pub trait Command {
    fn execute(&self, _ctx: &mut Context, _msg: &Message, _args: &Vec<String>) -> CommandResult;
}

impl Command for Arc<Command> {
    fn execute(&self, _ctx: &mut Context, _msg: &Message, _args: &Vec<String>) -> CommandResult {
        (**self).execute(_ctx, _msg, _args)
    }
}
