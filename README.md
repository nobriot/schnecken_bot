# Schnecken Bot

I am the Schnecken Bot. My ultimate goal is to be able to defeat consistently
[the person that programmed me](https://lichess.org/@/SchnellSchnecke)!

Watch me play on [Lichess](https://lichess.org/@/schnecken_bot/).

I intend to use all the possible nasty tricks to get a win!

> [!NOTE]
> I am a Lichess bot with an integrated engine, not a UCI engine wrapper.
> The engine is also available as a standalone UCI engine, see below.

## How to use this with your own bot account

First, you'll need a bot account. Create an account and upgrade it to a bot
account, following the [Lichess bot API guide](https://lichess.org/api#tag/Bot).

### API token

Create a [personal API token](https://lichess.org/account/oauth/token) with the
following scopes:

- `bot:play` - playing games
- `challenge:write` - sending and answering challenges
- `follow:read` - finding opponents among the users the bot follows
- `follow:write` - following/unfollowing opponents
- `tournament:write` - joining tournaments

The scopes are checked at startup. The token is looked up in this order:

1. the `--api-token` command line argument,
2. the `LICHESS_API_TOKEN` environment variable,
3. the config file `~/.config/schnecken_bot/config.toml`,
4. an interactive prompt.

A token given on the command line, in the environment or at the prompt is saved
to the config file, so you only need to provide it once.

### Compile and run

The project uses a nightly Rust toolchain (picked automatically through
`rust-toolchain.toml`). To compile and run:

```console
cargo run --release
```

Command line options:

| Option                 | Description                                                      |
| ---------------------- | ---------------------------------------------------------------- |
| `--api-token <TOKEN>`  | Lichess API token                                                |
| `--cache-size <MB>`    | Engine cache table size in MB (default: 1024)                    |
| `--play-style <STYLE>` | Engine play style: normal, conservative, aggressive, provocative |
| `--log-file <PATH>`    | Also append the logs to this file                                |
| `--log-level <LEVEL>`  | Log level: off, error, warn, info, debug, trace                  |

Pass them after `--`, e.g. `cargo run --release -- --log-file bot.log`.
Without `--log-level`, the log level comes from the `RUST_LOG` environment variable
(default: `info`).

### Configuration file

`~/.config/schnecken_bot/config.toml`:

```toml
[lichess]
api_token = "lip_..."

[engine]
cache_table_size = 1024
play_style = "normal"
```

Command line options take precedence over the config file.

### Commands

While running, the bot reads commands from the terminal:

- `play` or `p` - challenge a bot we follow (or a new bot with a similar rating)
- `exit` - exit the program, keeping ongoing games alive
- `quit` or `q` - exit the program, aborting/resigning ongoing games
- `log <level>` - change the log level (off, error, warn, info, debug, trace)
- `help` - display the help

## What the bot does on its own

- Accepts challenges, and challenges other bots: first among the bots it
  follows, otherwise a new bot with a similar rating.
- Joins the upcoming arenas of the
  [daily-bot-tournaments](https://lichess.org/team/daily-bot-tournaments) team.
- Chats a bit during games.

## Standalone UCI engine

The engine can also be used in any UCI compatible GUI:

```console
cargo run --release -p chess --bin schnecken_engine
```

## Development

```console
cargo test --workspace
cargo bench --workspace
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

Binaries are built with `-Ctarget-cpu=native`, so they are not portable to
other machines.
