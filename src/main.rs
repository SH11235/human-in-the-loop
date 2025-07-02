mod discord;
mod tools;

use clap::Parser;
use discord::HumanInDiscord;
use rmcp::serve_server;
use serenity::all::{ChannelId, UserId};
use tokio::io::{stdin, stdout};

#[derive(Debug, Parser)]
struct Args {
    #[clap(long, env = "DISCORD_TOKEN")]
    discord_token: String,
    #[clap(long, env = "DISCORD_CHANNEL_ID")]
    discord_channel_id: ChannelId,
    #[clap(long, env = "DISCORD_USER_ID")]
    discord_user_id: UserId,
    #[clap(long, env = "ENABLE_CONVERSATION_LOG")]
    enable_conversation_log: bool,
    #[clap(long, env = "LOG_CHANNEL_ID")]
    log_channel_id: Option<ChannelId>,
    #[clap(long, env = "LOG_THREAD_NAME", default_value = "Conversation Log")]
    log_thread_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args {
        discord_token,
        discord_channel_id,
        discord_user_id,
        enable_conversation_log,
        log_channel_id,
        log_thread_name,
    } = Args::parse();

    let human = HumanInDiscord::new(
        discord_user_id,
        discord_channel_id,
        enable_conversation_log,
        log_channel_id,
        log_thread_name,
    );
    let discord = discord::start(&discord_token, human.handler().clone());

    let handler = tools::HumanInTheLoop::new(human);
    let transport = (stdin(), stdout());
    let mcp = serve_server(handler, transport).await?;

    tokio::select! {
        res = mcp.waiting() => {
            res?;
        },
        res = discord => {
            res?;
        },
    }
    Ok(())
}
