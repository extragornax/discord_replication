use crate::handler::owner::OWNER_GROUP;
use crate::handler::hooks::delay_action;
use crate::handler::hooks::{dispatch_error, normal_message};
use std::collections::HashSet;
use serenity::framework::standard::buckets::{LimitedFor, RevertBucket};
use std::sync::Arc;
use serenity::framework::standard::macros::{check, command, group, help, hook};
use serenity::all::{Context, Message, UserId};
use serenity::framework::standard::{
    help_commands,
    Args,
    BucketBuilder,
    CommandGroup,
    CommandOptions,
    CommandResult,
    Configuration,
    HelpOptions,
    Reason,
    StandardFramework,
};
use serenity::model::permissions::Permissions;
use serenity::gateway::ShardManager;
use serenity::prelude::TypeMapKey;
use crate::{DbHandler, handle_database_init};
use crate::database::DBAccessManager;
use crate::handler::db_access::ReplicationPairData;
use crate::handler::hooks::{after, before, unknown_command};
use crate::log::write_info_log;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}


#[group]
#[commands(about, am_i_admin, ping, latency, link)]
pub struct Commands;

// The framework provides two built-in help commands for you to use. But you can also make your own
// customized help command that forwards to the behaviour of either of them.
#[help]
// This replaces the information that a user can pass a command-name as argument to gain specific
// information about it.
#[individual_command_tip = "Hello! こんにちは！Hola! Bonjour! 您好! 안녕하세요~\n\n\
If you want more information about a specific command, just pass the command as argument."]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Could not find: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name and commands. If the
// distance is lower than or equal the set distance, it will be displayed as a suggestion.
// Setting the distance to 0 will disable suggestions.
#[max_levenshtein_distance(3)]
// When you use sub-groups, Serenity will use the `indention_prefix` to indicate how deeply an item
// is indented. The default value is "-", it will be changed to "+".
#[indention_prefix = "+"]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it.
#[lacking_role = "Nothing"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible cases of
// ~~strikethrough-commands~~, but only if `strikethrough_commands_tip_in_{dm, guild}` aren't
// specified. If you pass in a value, it will be displayed instead.
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[command]
async fn latency(ctx: &Context, msg: &Message) -> CommandResult {
    // The shard manager is an interface for mutating, stopping, restarting, and retrieving
    // information about shards.
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager").await?;

            return Ok(());
        }
    };

    let runners = shard_manager.runners.lock().await;

    // Shards are backed by a "shard runner" responsible for processing events over the shard, so
    // we'll get the information about the shard runner for the shard this command was sent over.
    let runner = match runners.get(&ctx.shard_id) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    msg.reply(ctx, &format!("The shard latency is {:?}", runner.latency)).await?;

    Ok(())
}


#[command]
async fn am_i_admin(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let is_admin = if let (Some(member), Some(guild)) = (&msg.member, msg.guild(&ctx.cache)) {
        member.roles.iter().any(|role| {
            guild.roles.get(role).is_some_and(|r| r.has_permission(Permissions::ADMINISTRATOR))
        })
    } else {
        false
    };

    if is_admin {
        msg.channel_id.say(&ctx.http, "Yes, you are.").await?;
    } else {
        msg.channel_id.say(&ctx.http, "No, you are not.").await?;
    }

    Ok(())
}

#[command]
// Limit command usage to guilds.
#[only_in(guilds)]
#[checks(Owner)]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong! : )").await?;

    Ok(())
}

// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message in order for the
// command to be executed. If the check fails, the command is not called.
#[check]
#[name = "Owner"]
#[rustfmt::skip]
async fn owner_check(
    _: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    // Replace 7 with your ID to make this check pass.
    //
    // 1. If you want to pass a reason alongside failure you can do:
    //    `Reason::User("Lacked admin permission.".to_string())`,
    //
    // 2. If you want to mark it as something you want to log only:
    //    `Reason::Log("User lacked admin permission.".to_string())`,
    //
    // 3. If the check's failure origin is unknown you can mark it as such:
    //    `Reason::Unknown`
    //
    // 4. If you want log for your system and for the user, use:
    //    `Reason::UserAndLog { user, log }`
    if msg.author.id != 7 {
        return Err(Reason::User("Lacked owner permission".to_string()));
    }

    Ok(())
}


#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "This is a small test-bot! : )").await?;

    Ok(())
}

#[command]
async fn link(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let all_args = args.rest();
    let args: Vec<&str> = all_args.split(" ").collect();
    if args.len() == 4 {
        let from_guild = args[0].parse::<i64>();
        let from_channel = args[1].parse::<i64>();
        let to_guild = args[2].parse::<i64>();
        let to_channel = args[3].parse::<i64>();

        let data = ctx.data.read().await;

        let db_access_pool = match data.get::<DbHandler>() {
            Some(v) => {
                msg.reply(ctx, "Db access manager OK").await?;
                v
            }
            None => {
                msg.reply(ctx, "There was a problem getting the db access manager").await?;

                return Ok(());
            }
        };

        let _db_access: DBAccessManager = db_access_pool.mut_as_db_access();

        let to_insert = ReplicationPairData {
            from_guild: from_guild.clone().unwrap(),
            from_channel: from_channel.clone().unwrap(),
            to_guild: to_guild.clone().unwrap(),
            to_channel: to_channel.clone().unwrap(),
        };
        match _db_access.create_replication_pair(to_insert) {
            Ok(created) => {
                msg.reply(ctx, "Replication pair created").await?;
                write_info_log(format!("Replication pair created {:?}", created));
            }
            Err(e) => {
                msg.reply(ctx, &format!("Error creating replication pair: {:?}", e)).await?;
            }
        }

        drop(data);

        msg.channel_id.say(&ctx.http, &format!("from_guild: {}, from_channel: {}, to_guild: {}, to_channel: {}", from_guild.unwrap(), from_channel.unwrap(), to_guild.unwrap(), to_channel.unwrap())).await?;
    } else {
        msg.channel_id.say(&ctx.http, "Invalid arguments from_guild_id from_channel_id to_guild_id to_channel_id").await?;
    }

    Ok(())
}


pub(crate) async fn create_framework(owners: HashSet<UserId>, bot_id: UserId) -> StandardFramework {
    let framework = StandardFramework::new()
        // Set a function to be called prior to each command execution. This provides the context
        // of the command, the message that was received, and the full name of the command that
        // will be called.
        //
        // Avoid using this to determine whether a specific command should be executed. Instead,
        // prefer using the `#[check]` macro which gives you this functionality.
        //
        // **Note**: Async closures are unstable, you may use them in your application if you are
        // fine using nightly Rust. If not, we need to provide the function identifiers to the
        // hook-functions (before, after, normal, ...).
        .before(before)
        // Similar to `before`, except will be called directly _after_ command execution.
        .after(after)
        // Set a function that's called whenever an attempted command-call's command could not be
        // found.
        .unrecognised_command(unknown_command)
        // Set a function that's called whenever a message is not a command.
        .normal_message(normal_message)
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command can
        // only be performed by the bot owner.
        .on_dispatch_error(dispatch_error)
        // Can't be used more than once per 5 seconds:
        // .bucket("emoji", BucketBuilder::default().delay(5)).await
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay applying per
        // channel. Optionally `await_ratelimits` will delay until the command can be executed
        // instead of cancelling the command invocation.
        .bucket("complicated",
                BucketBuilder::default().limit(2).time_span(30).delay(5)
                    // The target each bucket will apply to.
                    .limit_for(LimitedFor::Channel)
                    // The maximum amount of command invocations that can be delayed per target.
                    // Setting this to 0 (default) will never await/delay commands and cancel the invocation.
                    .await_ratelimits(1)
                    // A function to call when a rate limit leads to a delay.
                    .delay_action(delay_action),
        ).await
        // The `#[group]` macro generates `static` instances of the options set for the group.
        // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
        // #name is turned all uppercase
        .help(&MY_HELP)
        .group(&COMMANDS_GROUP)
        .group(&OWNER_GROUP);

    framework.configure(
        Configuration::new().with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("!")
            // In this case, if "," would be first, a message would never be delimited at ", ",
            // forcing you to trim your arguments if you want to avoid whitespaces at the start of
            // each.
            .delimiters(vec![", ", ","])
            // Sets the bot's owners. These will be used for commands that are owners only.
            .owners(owners)
        ,
    );

    framework
}
