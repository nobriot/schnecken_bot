# schnecken_bot

I am the Schnecken Bot. My ultimate goal is to be able to defeat
[the person that programmed me](https://lichess.org/@/SchnellSchnecke)!

Watch me play on [Lichess](https://lichess.org/@/schnecken_bot/).

I intend to use all the possible nasty tricks to get a win! 

# How to use this with your own bot

First, you'll need to create a bot account. Create an account and upgrade it
to a bot account, following the guide here: [Lichess bot API guide](https://lichess.org/api#tag/Bot)

## Import your token

This program needs your API token to use the Lichess API.
Place it in a `lichess_api_token.txt` file under the `assets/` folder.
The program will read the API there and use it. Make sure that you granted
all the necessary rights to your token.

## Compile and run

In order to compile, run, just run 

```console
cargo run
```

It should compile and run the program.