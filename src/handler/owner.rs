use serenity::all::{Context, Message};
use serenity::all::EditChannel;
use serenity::framework::standard::{
    Args,
    CommandResult,
};
use serenity::framework::standard::macros::{command, group};

#[group]
#[owners_only]
// Limit all commands to be guild-restricted.
#[only_in(guilds)]
// Summary only appears when listing multiple groups.
#[summary = "Commands for server owners"]
#[commands(slow_mode)]
struct Owner;

#[command]
async fn slow_mode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let say_content = if let Ok(slow_mode_rate_seconds) = args.single::<u16>() {
        let builder = EditChannel::new().rate_limit_per_user(slow_mode_rate_seconds);
        if let Err(why) = msg.channel_id.edit(&ctx.http, builder).await {
            println!("Error setting channel's slow mode rate: {why:?}");

            format!("Failed to set slow mode to `{slow_mode_rate_seconds}` seconds.")
        } else {
            format!("Successfully set slow mode rate to `{slow_mode_rate_seconds}` seconds.")
        }
    } else if let Some(channel) = msg.channel_id.to_channel_cached(&ctx.cache) {
        let slow_mode_rate = channel.rate_limit_per_user.unwrap_or(0);
        format!("Current slow mode rate is `{slow_mode_rate}` seconds.")
    } else {
        "Failed to find channel in cache.".to_string()
    };

    msg.channel_id.say(&ctx.http, say_content).await?;

    Ok(())
}
