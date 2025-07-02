use std::sync::{Arc, OnceLock};

use serenity::{
    all::{
        AutoArchiveDuration, ChannelId, ChannelType, Context, CreateEmbed, CreateMessage,
        CreateThread, EventHandler, GatewayIntents, Ready, UserId,
    },
    Client,
};
use tokio::sync::OnceCell;

use crate::tools::Human;

pub async fn start(discord_token: &str, handler: Handler) -> anyhow::Result<()> {
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(discord_token, intents)
        .event_handler(handler)
        .await?;
    Ok(client.start().await?)
}

#[derive(Clone)]
pub struct Handler {
    ctx: Arc<OnceLock<Context>>,
}

impl Default for Handler {
    fn default() -> Self {
        Self {
            ctx: Arc::new(OnceLock::new()),
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        self.ctx.set(ctx).ok();
    }
}

pub struct HumanInDiscord {
    user_id: UserId,
    channel_id: ChannelId,
    handler: Handler,
    thread: OnceCell<ChannelId>,
    enable_conversation_log: bool,
    log_channel_id: Option<ChannelId>,
    log_thread_name: String,
    log_thread: OnceCell<ChannelId>,
}

impl HumanInDiscord {
    pub fn new(
        user_id: UserId,
        channel_id: ChannelId,
        enable_conversation_log: bool,
        log_channel_id: Option<ChannelId>,
        log_thread_name: String,
    ) -> Self {
        Self {
            user_id,
            channel_id,
            handler: Handler::default(),
            thread: OnceCell::new(),
            enable_conversation_log,
            log_channel_id,
            log_thread_name,
            log_thread: OnceCell::new(),
        }
    }

    pub fn handler(&self) -> &Handler {
        &self.handler
    }
}

#[async_trait::async_trait]
impl Human for HumanInDiscord {
    async fn ask(&self, question: &str) -> anyhow::Result<String> {
        let ctx = self
            .handler
            .ctx
            .get()
            .ok_or_else(|| anyhow::anyhow!("The connection with Discord is not ready"))?;
        let thread = self
            .thread
            .get_or_try_init(|| async {
                let thread_title = question.chars().take(100).collect::<String>();
                let channel = self
                    .channel_id
                    .create_thread(
                        &ctx.http,
                        CreateThread::new(thread_title)
                            .auto_archive_duration(AutoArchiveDuration::OneDay)
                            .kind(ChannelType::PublicThread),
                    )
                    .await?;
                anyhow::Ok(channel.id)
            })
            .await?;
        let message_text = format!("<@{}> {question}", self.user_id.get());
        thread
            .send_message(&ctx.http, CreateMessage::new().content(message_text))
            .await?;
        let message = thread
            .await_reply(ctx)
            .author_id(self.user_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to await message from the human in Discord"))?;
        Ok(message.content)
    }

    async fn log_conversation(
        &self,
        role: &str,
        message: &str,
        context: Option<&str>,
    ) -> anyhow::Result<()> {
        if !self.enable_conversation_log {
            return Ok(());
        }

        let log_channel_id = self
            .log_channel_id
            .ok_or_else(|| anyhow::anyhow!("Log channel ID not configured"))?;

        let ctx = self
            .handler
            .ctx
            .get()
            .ok_or_else(|| anyhow::anyhow!("The connection with Discord is not ready"))?;

        let log_thread = self
            .log_thread
            .get_or_try_init(|| async {
                let thread = log_channel_id
                    .create_thread(
                        &ctx.http,
                        CreateThread::new(&self.log_thread_name)
                            .auto_archive_duration(AutoArchiveDuration::OneWeek)
                            .kind(ChannelType::PublicThread),
                    )
                    .await?;
                anyhow::Ok(thread.id)
            })
            .await?;

        let color = match role {
            "human" => 0x3498db,     // Blue
            "assistant" => 0x2ecc71, // Green
            "system" => 0x95a5a6,    // Gray
            _ => 0x7f8c8d,           // Default gray
        };

        let mut embed = CreateEmbed::new()
            .title(format!("💬 {}", role.to_uppercase()))
            .description(message)
            .color(color)
            .timestamp(serenity::all::Timestamp::now());

        if let Some(ctx_info) = context {
            embed = embed.footer(serenity::all::CreateEmbedFooter::new(ctx_info));
        }

        log_thread
            .send_message(&ctx.http, CreateMessage::new().embed(embed))
            .await?;

        Ok(())
    }
}
