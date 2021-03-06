use commands::Command;
use commands::CommandError;
use commands::CommandResult;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::channel::Channel;

use serenity;
use serenity::model::misc::Mentionable;

use regex::Regex;

use rand;
use rand::Rng;
use rand::distributions::{IndependentSample, Range};

pub struct DiceRoll {
    dice_re: Regex,
}

impl DiceRoll {
    pub fn new() -> DiceRoll {
        DiceRoll {
            dice_re: Regex::new(r#"^(\d+?)?d(\d+?)([\+-]\d+?)?$"#).unwrap(),
        }
    }
}

const MAX_DICE: i32 = 12;

impl Command for DiceRoll {
    fn execute(&self, _ctx: &mut Context, msg: &Message, args: &Vec<String>) -> CommandResult {
        if args.len() < 2 { 
            return Err(CommandError::Argument(format!("Invalid number of arguments (expected 2, got {})", args.len())));
        }

        if let Some(cap) = self.dice_re.captures(&args[1]) {
            let mut rng = rand::thread_rng();

            let num = match cap.get(1) {
                Some(m) => m.as_str().parse::<i32>(),
                None => Ok(1),
            }.unwrap_or(1);

            let sides = match cap.get(2) {
                Some(m) => m.as_str().parse::<i32>(),
                None => Ok(6),
            }.unwrap_or(6);
            
            let add = match cap.get(3) {
                Some(m) => m.as_str().parse::<i32>(),
                None => Ok(0),
            }.unwrap_or(0);

            let mut throws = String::new();
            let sum = add + if num > MAX_DICE {
                throws = "too many dice to list".to_string();
                Range::new(num, sides*num).ind_sample(&mut rng)
            }
            else {
                let range = Range::new(1, 1+sides);
                let mut sum = 0;
                for i in 0..num {
                    if i != 0 {
                        throws.push_str(", ");
                    }
                    let throw = range.ind_sample(&mut rng);
                    throws.push_str(&throw.to_string());
                    sum += throw;
                }
                sum
            };
            
            if num > 1 {
                msg.reply(&format!("{} \n[ {} ]", sum, throws))?;
            }
            else {
                msg.reply(&format!("{}", sum))?;
            }
            Ok(())
        }
        else {
            Err(CommandError::Argument(format!("Invalid dice syntax: {}", args[1])))
        }
    }
}


pub struct Roulette;

impl Roulette {
    pub fn new() -> Roulette {
        Roulette
    }
}

impl Command for Roulette {
    fn execute(&self, _ctx: &mut Context, msg: &Message, _args: &Vec<String>) -> Result<(), CommandError> {
        if let Some(ch) = msg.channel() {
            // TODO: Clean up this mess
            match ch { 
                Channel::Guild(guild_lock) => {
                    let channel = guild_lock.read();
                    let winner = if let Some(guild) = msg.guild() { 
                        let g = guild.read();
                        let members: Vec<&serenity::model::guild::Member> = g.members_with_status(serenity::model::user::OnlineStatus::Online)
                            .into_iter().filter(|m| {
                                if let Ok(perm) = channel.permissions_for(m.user.read().id) {
                                    perm.read_messages()
                                }
                                else {
                                    false
                                }
                            }).collect();
                        members[rand::thread_rng().gen_range::<usize>(0, members.len())].mention()
                    }
                    else {
                        "Nobody because of an error!".to_string()
                    };
                    channel.say(&format!("And the winner is: {}", winner))?;
                },

                // Invalid channels
                Channel::Private(private_lock) => {
                    private_lock.read().say("It wouldn't make much sense to have a roulette in here would it?")?;
                    return Ok(());
                }
                Channel::Group(group_lock) => {
                    group_lock.read().say("Oh shit, bots inside groups are supported now?!?!")?;
                    return Ok(());
                },
                _ => {
                    msg.reply("???")?;
                    return Err(CommandError::Other(format!("Somehow got a command from somewhere unexpected (by: {})", msg.author.tag())));
                }
            };


        }
        else {
            return Err(CommandError::Other(format!("Couldn't get message channel (id: {})", msg.channel_id)));
        }
        

        Ok(())
    }
}