use serenity::builder::CreateCommand;
use serenity::model::application::CommandInteraction;
use serenity::prelude::Context;
use serenity::builder::{CreateEmbed, CreateInteractionResponseMessage};

pub async fn run(ctx: Context, int: CommandInteraction) -> CreateInteractionResponseMessage {
    let user = &int.user;
    let guild_id = int.guild_id.unwrap();
    let member = guild_id.member(&ctx.http, user.id).await.unwrap();

    let embed = CreateEmbed::new()
        .title(format!("Profile: {}", user.name))
        .description(format!("ID: {}", user.id))
        .field("Discriminator".to_string(), format!("{:?}", user.discriminator), true)
        .field("Global Name".to_string(), user.global_name.as_ref().unwrap_or(&"N/A".to_string()).to_string(), true)
        .field("Username".to_string(), user.name.to_string(), true)
        .field("User Tag".to_string(), format!("@{}", user.name), true)
        .field("Roles".to_string(), member.roles.iter().map(|role| role.get().to_string()).collect::<Vec<String>>().join(", "), true)
        .field("Flags".to_string(), format!("{:?}", user.public_flags), true)
        .field("Server Roles".to_string(), member.roles.iter().map(|role| format!("<@&{}>", role.get())).collect::<Vec<String>>().join(", "), true)
        .thumbnail(user.avatar_url().unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_string()));
    
    let builder = CreateInteractionResponseMessage::new().tts(false).embed(embed);
    builder
    // let _ = int.channel_id.send_message(&ctx, builder).await;
}

pub fn register() -> CreateCommand {
    CreateCommand::new("profile").description("A profile command")
}