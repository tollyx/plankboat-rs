extern crate serenity;
extern crate clap;
extern crate regex;
extern crate threadpool;
extern crate rand;

#[macro_use]
extern crate log;
extern crate fern;

extern crate serde_json;

extern crate chrono;

use clap::{Arg, App};

use serenity::client::Client;

mod framework;
mod handler;
mod commands;

fn main() {
    let matches = App::new("plankboat").version("0.1")
        .author("Adrian H. <adrian@tollyx.net>")
        .about("Crappy discord bot made by a madman")
        .arg(Arg::with_name("token")
            .value_name("TOKEN")
            .help("Sets the Discord API token to use")
            .required(true)
            .index(1))
        .arg(Arg::with_name("shards")
            .value_name("SHARDS")
            .long("shards")
            .short("s")
            .help("Sets the amount of shards to start. Will use autosharding if no value is set"))
        .get_matches();

    setup_logger().unwrap();

    let token = matches.value_of("token").unwrap();

    let mut client = Client::new(token, handler::PlankHandler::new())
        .expect("Error creating client");
    client.with_framework(framework::PlankFramework::new());

    info!("Starting plankboat...");
    if let Some(s) = matches.value_of("shards") {
        client.start_shards(s.parse::<u64>().expect("Invalid shards value"))
    }
    else {
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
        .level_for("plankboat", log::LevelFilter::Trace)
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
