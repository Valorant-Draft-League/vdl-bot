use serenity::all::{Color, CreateAutocompleteResponse, CreateCommandOption, ResolvedOption, ResolvedValue, UserId};
use serenity::builder::CreateCommand;
use serenity::builder::{CreateEmbed, CreateInteractionResponseMessage};
use supabase_rs::SupabaseClient;
use dotenv::dotenv;
use std::env::var;
use shuttle_runtime::SecretStore;

pub async fn run<'a>(
    options: &'a [ResolvedOption<'a>],
    secrets: &SecretStore,
) -> CreateInteractionResponseMessage {
    dotenv().ok();

    if let Some(ResolvedOption {
        value: ResolvedValue::String(value), ..
    }) = options.first()
    {
        let supabase_client: SupabaseClient = SupabaseClient::new(
            secrets.get("SUPABASE_URL").expect("SUPABASE_URL not found"),
            secrets.get("SUPABASE_KEY").expect("SUPABASE_KEY not found"),
        )
        .unwrap();

        let image_url = format!("https://owgxafpmhpneozularvz.supabase.co/storage/v1/object/public/vdlcdn//{}.png", value.replace(" ", "_").to_lowercase());
        println!("{}", image_url);
        let franchise_id_result = supabase_client.select("franchises").columns(vec!["id", "gm", "agms", "description"]).eq("name", value).execute().await;
        let franchise_data = match franchise_id_result {
            Ok(data) => data[0].clone(),
            Err(_) => return CreateInteractionResponseMessage::new().add_embed(CreateEmbed::new().title("Error").description("Failed to fetch franchise data.").color(Color::RED)),
        };
        let franchise_id = franchise_data.get("id").unwrap().to_string();
        println!("{}", franchise_id);
        let tier_names_result = supabase_client.select("teams").columns(vec!["tier", "name", "franchise_id"]).execute().await;
        let tier_names = match tier_names_result {
            Ok(data) => data.into_iter().filter(|team| team.get("franchise_id").unwrap().to_string() == franchise_id).collect::<Vec<_>>(),
            Err(_) => return CreateInteractionResponseMessage::new().add_embed(CreateEmbed::new().title("Error").description("Failed to fetch tier names.").color(Color::RED)),
        };

        let franchise_name = value.to_lowercase().replace(" ", "_");
        let tier_team_names = tier_names.iter().map(|team| {
            let tier_string = team.get("tier").unwrap().to_string();
            let tier = tier_string
                .strip_prefix('\"').unwrap_or(&tier_string)
                .strip_suffix('\"').unwrap_or(&tier_string);
            let team_name_string = team.get("name").unwrap().to_string();
            let team_name = team_name_string
                .strip_prefix('\"').unwrap_or(&team_name_string)
                .strip_suffix('\"').unwrap_or(&team_name_string);
            format!("**{}**: {}", tier, team_name)
        }).collect::<Vec<_>>();

        let gm = franchise_data.get("gm").unwrap().to_string();
        let agms = franchise_data.get("agms").unwrap().to_string();
        let description = franchise_data.get("description").unwrap().to_string().replace("\\n", "").replace('"', "");

        let gm_user_result = supabase_client.select("users").columns(vec!["username"]).eq("id", &gm.strip_prefix('\"').unwrap_or(&gm).strip_suffix('\"').unwrap_or(&gm)).execute().await;
        let gm_username = match gm_user_result {
            Ok(data) => {
                let username_string = data[0].get("username").unwrap().to_string();
                username_string
                    .strip_prefix('\"').unwrap_or(&username_string)
                    .strip_suffix('\"').unwrap_or(&username_string)
                    .to_string()
            },
            Err(_) => "Unknown".to_string(),
        };

        let agms_user_result = supabase_client.select("users").columns(vec!["username"]).eq("id", &agms.strip_prefix('\"').unwrap_or(&agms).strip_suffix('\"').unwrap_or(&agms)).execute().await;
        let agms_username = match agms_user_result {
            Ok(data) => {
                let username_string = data[0].get("username").unwrap().to_string();
                username_string
                    .strip_prefix('\"').unwrap_or(&username_string)
                    .strip_suffix('\"').unwrap_or(&username_string)
                    .to_string()
            },
            Err(_) => "Unknown".to_string(),
        };

        let embed = CreateEmbed::new()
            .title(format!("{} Franchise", value))
            .description(description)
            .thumbnail(image_url)
            .color(Color::BLUE)
            .field("Tier Team Names", tier_team_names.join("\n"), true)
            .field("GM", gm_username, true)
            .field("AGMs", agms_username, true);

        CreateInteractionResponseMessage::new().add_embed(embed)
    } else {
        let embed = CreateEmbed::new()
            .title("Error")
            .description("No franchise name provided. Please provide a franchise name to pull info on.")
            .color(Color::RED);

        CreateInteractionResponseMessage::new().add_embed(embed)
    }
}

pub fn register() -> CreateCommand {
    let franchise_arg = CreateCommandOption::new(serenity::all::CommandOptionType::String, "franchise", "Franchise to pull info on")
        .add_string_choice("Apex Pulse", "Apex Pulse")
        .add_string_choice("Crimson Circuit", "Crimson Circuit")
        .add_string_choice("Eclipse Syndicate", "Eclipse Syndicate")
        .add_string_choice("Neon Strikers", "Neon Strikers")
        .add_string_choice("Nova Sector", "Nova Sector")
        .add_string_choice("Quantum Rift", "Quantum Rift")
        .add_string_choice("Solar Vortex", "Solar Vortex")
        .add_string_choice("Vanguard Core", "Vanguard Core");

    CreateCommand::new("franchise")
        .description("A profile command")
        .add_option(franchise_arg)
}