mod discord;
mod mcp_handler;
mod tools;

use clap::Parser;
use discord::HumanInDiscord;
use rmcp::serve_server;
use tokio::io::{stdin, stdout};
use serenity::all::{ChannelId, UserId};

#[derive(Debug, Parser)]
struct Args {
    #[clap(long, env = "DISCORD_TOKEN")]
    discord_token: String,
    #[clap(long, env = "DISCORD_CHANNEL_ID")]
    discord_channel_id: ChannelId,
    #[clap(long, env = "DISCORD_USER_ID")]
    discord_user_id: UserId,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args {
        discord_token,
        discord_channel_id,
        discord_user_id,
    } = Args::parse();

    let human = HumanInDiscord::new(discord_user_id, discord_channel_id);
    let discord = discord::start(&discord_token, human.handler().clone());

    let handler = mcp_handler::Handler::new(human);
    let transport = (stdin(), stdout());
    let mcp = serve_server(handler, transport);

    tokio::select! {
        res = mcp => {
            res?;
        },
        res = discord => {
            res?;
        },
    }
    Ok(())
}
