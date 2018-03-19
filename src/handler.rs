use serenity::model::gateway::Ready;
use serenity::model::guild::{Guild, PartialGuild};
use serenity::client::{Context, EventHandler};

use serenity::prelude::RwLock;

use serenity::model::event::ResumedEvent;
use serenity::model::id::GuildId;

use serde_json;

use std::sync::Arc;

pub struct PlankHandler;

impl PlankHandler {
    pub fn new() -> PlankHandler {
        PlankHandler
    }
}

impl EventHandler for PlankHandler {

    fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected! (shard: {})", ready.user.name, ctx.shard_id);
    }

    fn resume(&self, ctx: Context, _resume: ResumedEvent) {
        info!("Resumed! (shard: {})", ctx.shard_id);
    }

    fn guild_create(&self, ctx: Context, guild: Guild, cached: bool) {
        info!("Joined guild '{}' (shard: {}, cached: {})", guild.name, ctx.shard_id, cached);
    }

    fn guild_delete(&self, ctx: Context, part_guild: PartialGuild, _guildcache: Option<Arc<RwLock<Guild>>>) {
        info!("Left guild '{}' (shard: {})", part_guild.name, ctx.shard_id);
    }

    fn guild_unavailable(&self, ctx: Context, id: GuildId) {
        info!("Guild '{}' is unavailable! (shard: {})", id, ctx.shard_id)
    }

    fn unknown(&self, _ctx: Context, name: String, data: serde_json::Value) {
        warn!("Uknown event '{}': {}", name, data);
    }
}