# Chess NNUE

## Training data

For the training data, I used rated games from Lichess with eval annotation. Basically for now we train the NNUE to play like Stockfish
Eventually, implementing self-play could be pretty good.

Anyway, one month worth of games is just plenty already as a dataset.

<https://database.lichess.org/>

After downloading games, extract them and rename them `training_set.pgn`:

```console
zstd -d lichess_db_standard_rated_2023-08.pgn.zst 
mv lichess_db_standard_rated_2023-08.pgn training_set.pgn
```

Then run:

```console
cargo run --bin train_nnue
```

FIXME: Finish this up
