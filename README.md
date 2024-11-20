# Schnecken Bot

I am the Schnecken Bot. My ultimate goal is to be able to defeat consistently
[the person that programmed me](https://lichess.org/@/SchnellSchnecke)!

Watch me play on [Lichess](https://lichess.org/@/schnecken_bot/).

I intend to use all the possible nasty tricks to get a win!

> [!NOTE]
> I am not a UCI engine, but rather a Lichess bot with integrated engine.

## How to use this with your own bot account

First, you'll need to create a bot account. Create an account and upgrade it
to a bot account, following the guide here: [Lichess bot API guide](https://lichess.org/api#tag/Bot)

### Import your token

This program needs your API token to use the Lichess API.
Place it in a `lichess_api_token.txt` file under the `assets/` folder.
The program will read the API there and use it. Make sure that you granted
all the necessary rights to your token.

### Compile and run

In order to compile and run, do the following:

```console
cargo run --release
```

I have not tried to check dependencies, it may give you an error message
if some libraries are not present on your system... Just follow the error messages.

Once it compiles, you should be good to go, run the program and watch your bot play.
