use dotenv::dotenv;
use rand::{thread_rng, Rng};
use serenity::all::{
    ActivityData, ChannelId, ChannelType, CreateChannel, GuildId, Interaction, Ready, UserId,
    VoiceState,
};
use serenity::builder::CreateInteractionResponse as _;
use serenity::prelude::*;
use serenity::{async_trait, Client};
use shuttle_runtime::SecretStore;
use std::{
    collections::{HashMap, HashSet},
    env,
    sync::{Arc, Mutex},
};
use serenity::all::GatewayIntents;

mod commands;

struct Handler {
    target_voice_channel_id: ChannelId,
    created_vcs: Arc<Mutex<HashSet<ChannelId>>>,
    vc_occupants: Arc<Mutex<HashMap<ChannelId, HashSet<UserId>>>>,
}

fn grab_vc_name() -> String {
    let agents = [
        "Astra", "Breach", "Brimstone", "Chamber", "Cypher", "Fade",
        "Gekko", "Harbor", "Jett", "KAY/O", "Killjoy", "Neon",
        "Omen", "Phoenix", "Raze", "Reyna", "Sage", "Skye",
        "Sova", "Viper", "Yoru",
    ];
    let mut rng = thread_rng();
    agents[rng.gen_range(0..agents.len())].to_string()
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(cmd) = interaction {
            let content = match cmd.data.name.as_str() {
                "profile"   => commands::profile::run(ctx.clone(), cmd.clone()).await,
                "status"    => commands::status::run(&cmd.data.options()).await,
                "franchise" => commands::franchise::run(&cmd.data.options()).await,
                _ => serenity::builder::CreateInteractionResponseMessage::new()
                        .content("Unknown command"),
            };
            tokio::spawn(async move {
                if let Err(why) = cmd.create_response(&ctx.http, 
                    serenity::builder::CreateInteractionResponse::Message(content)
                ).await {
                    eprintln!("Cannot respond to slash command: {}", why);
                }
            });
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        ctx.set_presence(
            Some(ActivityData::playing("preparing for S1 VDL Draft!")),
            serenity::all::OnlineStatus::DoNotDisturb,
        );

        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );
        let _ = guild_id
            .set_commands(&ctx.http, vec![
                commands::profile::register(),
                commands::status::register(),
                commands::franchise::register(),
            ])
            .await;
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        // Determine the channel they came from, falling back to our occupancy map if needed
        let old_chan = old
            .and_then(|vs| vs.channel_id)
            .or_else(|| {
                let occ_map = self.vc_occupants.lock().unwrap();
                occ_map.iter().find_map(|(chan_id, users)| {
                    if users.contains(&new.user_id) {
                        Some(*chan_id)
                    } else {
                        None
                    }
                })
            });
        let new_chan = new.channel_id;

        println!("left:  {:?}, joined: {:?}", old_chan, new_chan);

        // ─── 1) If they actually left one of our dynamic VCs, update & possibly delete ───
        if let Some(left) = old_chan {
            if new_chan != Some(left) && self.created_vcs.lock().unwrap().contains(&left) {
                let should_delete = {
                    let mut occ_map = self.vc_occupants.lock().unwrap();
                    if let Some(users) = occ_map.get_mut(&left) {
                        users.remove(&new.user_id);
                        if users.is_empty() {
                            occ_map.remove(&left);
                            self.created_vcs.lock().unwrap().remove(&left);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };

                if should_delete {
                    if let Err(e) = left.delete(&ctx.http).await {
                        eprintln!("Error deleting empty VC {}: {:?}", left, e);
                    }
                }
            }
        }

        // ─── 2) If they joined one of our dynamic VCs, track them ───────────────────
        if let Some(joined) = new_chan {
            if self.created_vcs.lock().unwrap().contains(&joined) {
                self.vc_occupants
                    .lock()
                    .unwrap()
                    .entry(joined)
                    .or_default()
                    .insert(new.user_id);
            }
        }

        // ─── 3) If they joined the lobby, spawn a fresh VC & move them ──────────────
        if old_chan != Some(self.target_voice_channel_id)
            && new_chan == Some(self.target_voice_channel_id)
        {
            let name = grab_vc_name();
            let guild_id = GuildId::new(
                env::var("GUILD_ID").expect("GUILD_ID not set")
                    .parse().expect("GUILD_ID invalid"),
            );
            let builder = CreateChannel::new(name.clone()).kind(ChannelType::Voice);

            if let Ok(channel) = guild_id.create_channel(&ctx.http, builder).await {
                self.created_vcs.lock().unwrap().insert(channel.id);
                let mut set = HashSet::new();
                set.insert(new.user_id);
                self.vc_occupants
                    .lock()
                    .unwrap()
                    .insert(channel.id, set);

                if let Err(e) = guild_id.move_member(&ctx.http, new.user_id, channel.id).await {
                    eprintln!("Failed to move into {}: {:?}", name, e);
                }
            }
        }
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    dotenv().ok();
    let token = secrets.get("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");

    let intents = 
        GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let client = Client::builder(token, intents)
        .event_handler(Handler {
            target_voice_channel_id: ChannelId::new(1365567367752716303),
            created_vcs: Arc::new(Mutex::new(HashSet::new())),
            vc_occupants: Arc::new(Mutex::new(HashMap::new())),
        })
        .await
        .expect("Error creating client");

    Ok(client.into())
}
