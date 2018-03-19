use serenity::client::Context;
use serenity::model::channel::Message;
use serenity;
use quick_xml;
use reqwest;

use std::fmt;
use std::sync::Arc;

pub mod games;
pub mod myanimelist;

pub type CommandResult = Result<(), CommandError>;

impl From<CommandError> for CommandResult {
    fn from(err: CommandError) -> CommandResult {
        Err(err)
    }
}

pub enum CommandError {
    Serenity(serenity::Error),
    Xml(quick_xml::Error),
    Reqwest(reqwest::Error),
    Argument(String),
    Other(String),
}

impl From<reqwest::Error> for CommandError {
    fn from(err: reqwest::Error) -> CommandError {
        CommandError::Reqwest(err)
    }
}

impl From<quick_xml::Error> for CommandError {
    fn from(err: quick_xml::Error) -> CommandError {
        CommandError::Xml(err)
    }
}

impl From<serenity::Error> for CommandError {
    fn from(err: serenity::Error) -> CommandError {
        CommandError::Serenity(err)
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CommandError::Reqwest(ref err) => {
                write!(f, "Reqwest error while executing a command: {}", err)
            },
            CommandError::Xml(ref err) => {
                write!(f, "quick_xml error while executing a command: {}", err)
            },
            CommandError::Serenity(ref err) => {
                write!(f, "Serenity error while executing a command: {}", err)
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

struct ScriptedCommand {
    uses_prefix: bool,
    enabled_in_pm: bool,
}

pub trait Command: Send + Sync + 'static {
    fn execute(&self, _ctx: &mut Context, _msg: &Message, _args: &Vec<String>) -> CommandResult;
}

impl Command for Arc<Command> {
    fn execute(&self, _ctx: &mut Context, _msg: &Message, _args: &Vec<String>) -> CommandResult {
        (**self).execute(_ctx, _msg, _args)
    }
}
