# Human-in-the-Loop MCP Server

An MCP (Model Context Protocol) server that allows AI assistants to ask questions to humans via Discord.

<img width="845" alt="Screenshot 2025-06-23 at 18 25 43" src="https://github.com/user-attachments/assets/dcdbb1a7-cb71-446e-b44d-bfe637059acb" />

## Overview

This MCP server is used when AI assistants need human input or judgment during their work. For example:

- When having an LLM create documentation, the AI designs the structure while humans provide specific content
- When the AI needs confirmation on uncertain decisions
- When specialized knowledge or personal information is required

## Requirements

- Rust (1.70 or higher)
- Discord account and bot
- MCP-compatible AI client (Claude Desktop, Copilot Edits, etc.)

## Setup

### 1. Create Discord Bot

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application
3. Create a bot in the Bot section and obtain the token
4. Set required permissions:
   - Send Messages
   - Create Public Threads
   - Read Message History
5. Enable privileged gateway intents in Bot section:
   - Message Content Intent

### 2. Install

```bash
cargo install --git https://github.com/KOBA789/human-in-the-loop.git
```

## Connecting with MCP Clients

### Claude Desktop Configuration

Add the following to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "human-in-the-loop": {
      "command": "human-in-the-loop",
      "args": [
        "--discord-channel-id", "channel-id",
        "--discord-user-id", "user-id"
      ],
      "env": {
        "DISCORD_TOKEN": "your-discord-bot-token"
      }
    }
  }
}
```

### Claude Code Configuration

For Claude Code (claude.ai/code), add to your MCP settings:

```json
{
  "mcpServers": {
    "human-in-the-loop": {
      "command": "human-in-the-loop",
      "args": [
        "--discord-channel-id", "channel-id",
        "--discord-user-id", "user-id"
      ]
    }
  }
}
```

Set the Discord token as an environment variable before running Claude Code:

```bash
export DISCORD_TOKEN="your-discord-bot-token"
claude
```

Note: The server automatically reads the Discord token from the `DISCORD_TOKEN` environment variable. You can also pass it via `--discord-token` argument if needed.

### Usage

AI assistants can ask questions to humans using the `ask_human` tool:

```
Human: Please create a documentation outline. You can ask the human as you need.
Assistant: I'll create a documentation outline. Let me ask you some questions first.
[Uses ask_human tool]
```

The AI posts questions in Discord and mentions the specified user. When the user replies in Discord, the response is returned to the AI.

#### Conversation Logging

The server now supports logging conversations to Discord for review and history tracking. To enable this feature:

1. Add the following arguments to your configuration:
   - `--enable-conversation-log` - Enable conversation logging
   - `--log-channel-id` - Discord channel ID for logs (can be same or different from main channel)
   - `--log-thread-name` - Thread name for logs (default: "Conversation Log")

Example configuration with logging enabled:

```json
{
  "mcpServers": {
    "human-in-the-loop": {
      "command": "human-in-the-loop",
      "args": [
        "--discord-channel-id", "channel-id",
        "--discord-user-id", "user-id",
        "--enable-conversation-log",
        "--log-channel-id", "log-channel-id",
        "--log-thread-name", "AI Assistant Conversation Log"
      ],
      "env": {
        "DISCORD_TOKEN": "your-discord-bot-token"
      }
    }
  }
}
```

AI assistants can use the `log_conversation` tool to record important interactions:

```
[AI uses log_conversation tool with role="assistant", message="Starting documentation process"]
[AI uses log_conversation tool with role="human", message="User's response about requirements"]
```

Logged messages appear in Discord with:
- Color-coded embeds (blue for human, green for assistant, gray for system)
- Timestamps
- Optional context information
- Organized in a dedicated thread for easy review

## How It Works

1. AI assistant calls the `ask_human` tool
2. MCP server creates a thread in the specified Discord channel (or uses existing thread)
3. Posts the question and mentions the specified user
4. Waits for user's reply
5. Returns the reply content to the AI assistant

## Finding Discord IDs

### Getting Channel ID
1. Enable Developer Mode in Discord (Settings → Advanced → Developer Mode)
2. Right-click on channel → "Copy ID"

### Getting User ID
1. Right-click on user → "Copy ID"

## Roadmap

- **Future Migration to MCP Elicitation**: Once MCP's Elicitation implementation becomes more widespread and standardized, we plan to migrate the UI from Discord to native MCP Elicitation. This will provide a more integrated experience directly within MCP-compatible clients.
