use serenity::all::{CreateCommandOption, ResolvedOption, ResolvedValue, UserId, Color};
use serenity::builder::CreateCommand;
use serenity::model::application::CommandInteraction;
use serenity::prelude::Context;
use serenity::builder::{CreateEmbed, CreateInteractionResponseMessage};
use supabase_rs::SupabaseClient;
use dotenv::dotenv;
use std::env::var;

struct UserStatus {
    id: i32,
    discord_id: String,
    username: String,
    team_id: i32,
    created_at: String,
    status: String,
}

impl UserStatus {
    fn new(id: i32, discord_id: String, username: String, team_id: i32, created_at: String, status: String) -> Self {
        UserStatus { id, discord_id, username, team_id, created_at, status }
    }
}

pub async fn run<'a>(options: &'a [ResolvedOption<'a>]) -> CreateInteractionResponseMessage {
    dotenv().ok();

    if let Some(ResolvedOption {
        value: ResolvedValue::User(user, _member), ..
    }) = options.first()
    {
        let supabase_client: SupabaseClient = SupabaseClient::new(
            var("SUPABASE_URL").unwrap(),
            var("SUPABASE_KEY").unwrap()
            ).unwrap();
        let data = supabase_client.select("users").columns(vec!["id", "discord_id", "username", "team_id", "created_at", "status"]).eq("discord_id", &user.id.get().to_string()).execute().await;
        println!("{:?}", data);

        let user_data = match data {
            Ok(users) if !users.is_empty() => users[0].clone(),
            _ => {
                let embed = CreateEmbed::new()
                    .title("User Status")
                    .description("Not Signed Up")
                    .color(Color::RED);

                return CreateInteractionResponseMessage::new().add_embed(embed);
            }
        };
        let user_id = user_data.get("id").unwrap().to_string().parse::<i32>().unwrap_or_else(|_| {
            println!("Failed to parse user_id as i32");
            0
        });
        let discord_id = user_data.get("discord_id").unwrap().to_string();
        let username = user_data.get("username").unwrap().to_string();
        let team_id = user_data.get("team_id").unwrap().to_string().parse::<i32>().unwrap_or_else(|_| {
            println!("Failed to parse team_id as i32");
            0
        });
        let created_at = user_data.get("created_at").unwrap().to_string();
        let status = user_data.get("status").unwrap().to_string();
        let user_status = UserStatus::new(user_id, discord_id, username, team_id, created_at, status);

        let embed = CreateEmbed::new()
            .title("User Status")
            .field("ID", user_status.id.to_string(), true)
            .field("Discord ID", user_status.discord_id.strip_prefix("\"").unwrap_or(&user_status.status).strip_suffix("\"").unwrap_or(&user_status.status).to_string(), true)
            .field("Username", user_status.username.strip_prefix("\"").unwrap_or(&user_status.status).strip_suffix("\"").unwrap_or(&user_status.status).to_string(), true)
            .field("Team ID", user_status.team_id.to_string(), true)
            .field("Created At", user_status.created_at.strip_prefix("\"").unwrap_or(&user_status.status).strip_suffix("\"").unwrap_or(&user_status.status).to_string(), true)
            .field("Status", user_status.status.strip_prefix("\"").unwrap_or(&user_status.status).strip_suffix("\"").unwrap_or(&user_status.status).to_string(), true);

        CreateInteractionResponseMessage::new().add_embed(embed)
    } else {
        let embed = CreateEmbed::new()
            .title("Error")
            .description("Failed to retrieve user status. Please ensure the user exists and try again.")
            .color(Color::RED);

        CreateInteractionResponseMessage::new().add_embed(embed)
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("status").description("Grabs the league status for the current user").add_option(CreateCommandOption::new(serenity::all::CommandOptionType::User, "user", "User to check status on").required(true))
}
