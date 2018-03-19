extern crate serenity;
extern crate clap;
extern crate regex;
extern crate threadpool;
extern crate rand;
#[macro_use] extern crate log;
extern crate fern;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate chrono;
extern crate rusqlite;
extern crate typemap;
extern crate reqwest;
extern crate url;
extern crate quick_xml;
extern crate toml;
#[macro_use] extern crate lazy_static;



use clap::{Arg, App};
use serenity::prelude::*;
use std::io::{Read, Write};

mod framework;
mod handler;
mod commands;

use std::sync::{Arc, Mutex};

struct DatabaseContainer;

impl typemap::Key for DatabaseContainer {
    type Value = Arc<Mutex<rusqlite::Connection>>;
}

#[derive(Deserialize)]
struct Config {
    bot_token: Option<String>,
    shards: Option<u64>,
    myanimelist: Option<MALcfg>,
}

#[derive(Deserialize)]
struct MALcfg {
    username: String,
    password: String,
}

fn main() {
    let matches = App::new("plankboat").version("0.1")
        .author("Adrian H. <adrian@tollyx.net>")
        .about("Crappy discord bot made by a madman")
        .arg(Arg::with_name("token")
            .value_name("TOKEN")
            .long("token")
            .short("t")
            .help("Sets the Discord API token to use"))
        .arg(Arg::with_name("shards")
            .value_name("SHARDS")
            .long("shards")
            .short("s")
            .help("Sets the amount of shards to start. Will use autosharding if no value is set"))
        .get_matches();

    setup_logger().unwrap();
    let mut cfgstr = String::new();
    
    if !std::path::Path::new("config.toml").exists() {
        warn!("Could not find a config file! Creating an example config...");
        std::fs::File::create("config.toml").expect("Could not create config file").write_all(
br#"# Bot token, required.
# bot_token = "Bot AAAAAAAAAAAAAAAAAAA"

# The number of shards to use. 
# Leave undefined to enable autosharding.
# shards = 4

# MyAnimeList login, used for the anime and manga commands. 
# Leaving it undefined will disable the commands.
# [myanimelist]
# username = "user"
# password = "pass"
"#
        ).unwrap();
        return;
    }

    std::fs::File::open("config.toml").expect("Could not open config file").read_to_string(&mut cfgstr).unwrap();
    
    let mut cfg: Config = toml::from_str(&cfgstr).expect("Could not parse config file");

    if let Some(s) = matches.value_of("token") {
        cfg.bot_token = Some(s.to_string());
    }

    if let Some(s) = matches.value_of("shards") {
        cfg.shards = Some(s.parse().expect("Invalid shards value."));
    }

    if cfg.bot_token == None {
        error!("Bot token not defined. Quitting...");
        return;
    }

    let mut client = Client::new(&cfg.bot_token.unwrap(), handler::PlankHandler::new())
        .expect("Error creating client");
    
    let mut fw = framework::PlankFramework::new();
    fw.add_command("roll", commands::games::DiceRoll::new());
    fw.add_command("roulette", commands::games::Roulette::new());
    if let Some(mal) = cfg.myanimelist {
        fw.add_command("anime", commands::myanimelist::AnimeCommand::new(&mal.username, &mal.password));
        fw.add_command("manga", commands::myanimelist::MangaCommand::new(&mal.username, &mal.password));
    }

    client.with_framework(fw);
    
    {
        let mut data = client.data.lock();
        data.insert::<DatabaseContainer>(Arc::new(Mutex::new(rusqlite::Connection::open("plankboat.sqlite").unwrap())));
    }
    
    if let Some(s) = cfg.shards {
        info!("Starting plankboat with {} shard(s)...", s);
        client.start_shards(s)
    }
    else {
        info!("Starting plankboat with autosharding...");
        client.start_autosharded()
    }.expect("Fatal error");
    info!("Exiting...");
}

fn setup_logger() -> Result<(), fern::InitError> {
    use fern::colors::{Color, ColoredLevelConfig};
    let log_colors = ColoredLevelConfig::new()
        .trace(Color::White)
        .debug(Color::Blue)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    fern::Dispatch::new()
        .level(log::LevelFilter::Warn)
        .level_for("plankboat", log::LevelFilter::Info)
        .chain(fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{} [{}] <{}> {}",
                    chrono::Local::now().format("%H:%M:%S"),
                    log_colors.color(record.level()),
                    record.target(),
                    message
                ))
            })
            .chain(std::io::stderr())
        )
        .chain(fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{} [{}] <{}> {}",
                    chrono::Local::now().format("%H:%M:%S"),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .chain(fern::log_file("output.log")?)
        )
        .apply()?;
    Ok(())
}
