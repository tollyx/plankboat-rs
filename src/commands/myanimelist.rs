use commands::{Command, CommandError, CommandResult};
use reqwest;
use url;
use quick_xml;

use quick_xml::events::Event;

use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::utils::Colour;

const ANIME_URL: &str = "https://myanimelist.net/api/anime/search.xml";
const MANGA_URL: &str = "https://myanimelist.net/api/manga/search.xml";

#[derive(Debug, PartialEq)]
enum ResultFields {
    None,
    Unknown,
    Id,
    Title,
    EnglishTitle,
    TitleSynonyms,
    Episodes,
    EntryType,
    Chapters,
    Volumes,
    Score,
    Status,
    StartDate,
    EndDate,
    Synopsis,
    Image,
    Entry,
}

#[derive(Debug)]
struct Entry {
    pub id: String,
    pub title: String,
    pub english_title: String,
    pub title_synonyms: String,
    pub score: String,
    pub episodes: String,
    pub chapters: String,
    pub volumes: String,
    pub entry_type: String,
    pub status: String,
    pub start_date: String,
    pub end_date: String,
    pub synopsis: String,
    pub image: String,
}

struct MyAnimeListApi {
    username: String,
    password: String,
}

impl MyAnimeListApi {
    fn query(&self, url: &str, q: &str) -> reqwest::Result<reqwest::Response> {
        let cli = reqwest::Client::new();
        cli.get(url::Url::parse_with_params(url, &[("q", q)]).unwrap())
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .send()
    }

    fn parse_response(&self, mut response: reqwest::Response) -> Result<Entry, CommandError> {
        let text = response.text().unwrap();

        let mut xml = quick_xml::Reader::from_str(&text);

        // println!("{}", text);

        let mut buf = Vec::new();
        let mut field = ResultFields::None;
        
        let mut entry = Entry {
            id: String::new(),
            title: String::new(),
            english_title: String::new(),
            title_synonyms: String::new(),
            score: String::new(),
            episodes: String::new(),
            chapters: String::new(),
            volumes: String::new(),
            entry_type: String::new(),
            status: String::new(),
            start_date: String::new(),
            end_date: String::new(),
            synopsis: String::new(),
            image: String::new(),
        };

        loop {
            match xml.read_event(&mut buf) {
                Ok(Event::Start(e)) => {
                    match e.name() {
                        b"id" => { field = ResultFields::Id; },
                        b"title" => { field = ResultFields::Title; },
                        b"english" => { field = ResultFields::EnglishTitle; },
                        b"synonyms" => { field = ResultFields::TitleSynonyms; },
                        b"episodes" => { field = ResultFields::Episodes; },
                        b"type" => { field = ResultFields::EntryType; },
                        b"status" => { field = ResultFields::Status; },
                        b"score" => { field = ResultFields::Score; },
                        b"chapters" => { field = ResultFields::Chapters; },
                        b"volumes" => { field = ResultFields::Volumes; },
                        b"start_date" => { field = ResultFields::StartDate; },
                        b"end_date" => { field = ResultFields::EndDate; },
                        b"synopsis" => { field = ResultFields::Synopsis; },
                        b"image" => { field = ResultFields::Image; },
                        _ => { field = ResultFields::Unknown; }
                    }
                },
                Ok(Event::Text(e)) => {
                    match field {
                        ResultFields::Id => {
                            entry.id = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::Title => {
                            entry.title = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::EnglishTitle => {
                            entry.english_title = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::TitleSynonyms => {
                            entry.title_synonyms = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::Episodes => {
                            entry.episodes = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::Chapters => {
                            entry.chapters = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::Volumes => {
                            entry.volumes = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::Score => {
                            entry.score = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::EntryType => {
                            entry.entry_type = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::Status => {
                            entry.status = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::StartDate => {
                            entry.start_date = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::EndDate => {
                            entry.end_date = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::Synopsis => {
                            entry.synopsis = e.unescape_and_decode(&xml)?;
                        },
                        ResultFields::Image => {
                            entry.image = e.unescape_and_decode(&xml)?;
                        },
                        _ => {},
                    }
                }
                Ok(Event::End(e)) => {
                                            
                    if e.name() == b"entry" {
                        field = ResultFields::Entry;
                        break; // Break after the first entry
                    }
                    else {
                        field = ResultFields::None;
                    }
                },
                Ok(Event::Eof) => {
                    break;
                },
                Err(e) => {
                    return Err(CommandError::Xml(e));
                },
                _ => {}
            }
        }
        
        lazy_static! {
            static ref REPLACEMENTS: Vec<(&'static str, &'static str)> = vec![
                ("<br />", ""),
                ("&#039;", "'"),
                ("[i]", "*"),
                ("[/i]", "*"),
                ("&quot;", "\""),
                ("&mdash;", "—"),
                ("&ndash;", "–"),
            ];
        }
        
        for rep in REPLACEMENTS.iter() {
            entry.synopsis = entry.synopsis.replace(rep.0, rep.1);
        }

        if entry.synopsis.len() >= 2048 {
            entry.synopsis.truncate(2044);
            entry.synopsis.push_str("...");
        }
        if entry.english_title.is_empty() {
            entry.english_title = "—".to_string();
        }
        if entry.title_synonyms.is_empty() {
            entry.title_synonyms = "—".to_string();
        }
        if entry.score.is_empty() {
            entry.score = "—".to_string();
        }
        if entry.status.is_empty() {
            entry.status = "—".to_string();
        }
        if entry.episodes.is_empty() {
            entry.episodes = "—".to_string();
        }
        if entry.chapters.is_empty() {
            entry.chapters = "—".to_string();
        }
        if entry.volumes.is_empty() {
            entry.volumes = "—".to_string();
        }
        if entry.entry_type.is_empty() {
            entry.entry_type = "—".to_string();
        }
        if entry.start_date.is_empty() {
            entry.start_date = "—".to_string();
        }
        if entry.end_date.is_empty() {
            entry.end_date = "—".to_string();
        }

        if field == ResultFields::Entry {
            Ok(entry)
        }
        else {
            Err(CommandError::Argument("Found no entries in xml response".to_string()))
        }
    }

    pub fn search_anime(&self, query: &str) -> Result<Entry, CommandError> {
        let res = self.query(ANIME_URL, query)?;

        if res.status().is_success() {
            if let Ok(entry) = self.parse_response(res) {
                Ok(entry)
            }
            else {
                Err(CommandError::Argument(format!("Could not find anime: {}", query)))
            }
        }
        else {
            Err(CommandError::Other(format!("Failed https request: {}", res.status())))
        }
    }

    pub fn search_manga(&self, query: &str) -> Result<Entry, CommandError> {
        let res = self.query(MANGA_URL, query)?;

        if res.status().is_success() {
            if let Ok(entry) = self.parse_response(res) {
                Ok(entry)
            }
            else {
                Err(CommandError::Argument(format!("Could not find manga: {}", query)))
            }
        }
        else {
            Err(CommandError::Other(format!("Failed https request: {}", res.status())))
        }
    }
}

pub struct AnimeCommand {
    mal: MyAnimeListApi,
}

impl AnimeCommand {
    pub fn new(user: &str, pass: &str) -> AnimeCommand {
        AnimeCommand {
            mal: MyAnimeListApi {
                username: user.to_string(),
                password: pass.to_string(),
            }
        }
    }
}

impl Command for AnimeCommand {
    fn execute(&self, _ctx: &mut Context, _msg: &Message, _args: &Vec<String>) -> CommandResult {
        if _args.len() < 2 { return Err(CommandError::Argument("Missing query string".to_string())) }

        let query = _args[1..].join(&" ");

        match self.mal.search_anime(&query) {
            Ok(entry) => {
                let _ = _msg.channel_id.send_message(|m| m
                    .embed(|e| e
                        .author(|a| a
                            .name("MyAnimeList")
                            .url("https://myanimelist.net/")
                            .icon_url("https://myanimelist.cdn-dena.com/img/sp/icon/apple-touch-icon-256.png"))
                        .title(&entry.title)
                        .description(&entry.synopsis)
                        .thumbnail(&entry.image)
                        .fields(vec![
                            ("English:", entry.english_title, true),
                            ("Synonyms:", entry.title_synonyms, true),
                            ("Score:", entry.score, true),
                            ("Type:", entry.entry_type, true),
                            ("Status:", entry.status, true),
                            ("Episodes:", entry.episodes, true),
                            ("Start date:", entry.start_date, true),
                            ("End date:", entry.end_date, true)
                        ])
                        .colour(Colour::from_rgb(46, 81, 162))
                        .url(&format!("https://myanimelist.net/anime/{}/", entry.id))
                    )
                )?;
                Ok(())
            },
            Err(CommandError::Argument(s)) => {
                _msg.reply(&format!("Query failed: {}", &s))?;
                Ok(())
            },
            Err(e) => Err(e),
        }

    }
}

pub struct MangaCommand {
    mal: MyAnimeListApi,
}

impl MangaCommand {
    pub fn new(user: &str, pass: &str) -> MangaCommand {
        MangaCommand {
            mal: MyAnimeListApi {
                username: user.to_string(),
                password: pass.to_string(),
            }
        }
    }
}

impl Command for MangaCommand {
    fn execute(&self, _ctx: &mut Context, _msg: &Message, _args: &Vec<String>) -> CommandResult {
        if _args.len() < 2 { return Err(CommandError::Argument("Missing query string".to_string())) }

        let query = _args[1..].join(&" ");

        match self.mal.search_manga(&query) {
            Ok(entry) => {
                let _ = _msg.channel_id.send_message(|m| m
                    .embed(|e| e
                        .author(|a| a
                            .name("MyAnimeList")
                            .url("https://myanimelist.net/")
                            .icon_url("https://myanimelist.cdn-dena.com/img/sp/icon/apple-touch-icon-256.png"))
                        .title(&entry.title)
                        .description(&entry.synopsis)
                        .thumbnail(&entry.image)
                        .fields(vec![
                            ("English:", entry.english_title, true),
                            ("Synonyms:", entry.title_synonyms, true),
                            ("Score:", entry.score, true),
                            ("Type:", entry.entry_type, true),
                            ("Status:", entry.status, true),
                            ("Chapters:", entry.chapters, true),
                            ("Volumes:", entry.volumes, true),
                            ("Start date:", entry.start_date, true),
                            ("End date:", entry.end_date, true)
                        ])
                        .colour(Colour::from_rgb(46, 81, 162))
                        .url(&format!("https://myanimelist.net/manga/{}/", entry.id))
                    )
                )?;
                Ok(())
            },
            Err(CommandError::Argument(s)) => {
                _msg.reply(&format!("Query failed: {}", &s))?;
                Ok(())
            },
            Err(e) => Err(e),
        }

    }
}
