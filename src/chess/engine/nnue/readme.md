# Chess NNUE

Disclaimer: This is not really an NNUE, but rather a small neural net that I trained on
chess positions.

## Training data

For the training data, I used rated games from Lichess with eval annotation. Basically for now we train the neural net to play like Stockfish
Eventually, implementing self-play could be pretty good.

Anyway, one month worth of games is just plenty already as a dataset.

<https://database.lichess.org/>

After downloading games, extract them and rename them `training_set.pgn`:

```console
zstd -d lichess_db_standard_rated_2023-08.pgn.zst 
mv lichess_db_standard_rated_2023-08.pgn training_set_full.pgn
```

The training set is kinda huge, so it makes sense to train on a fraction at a time.
For example, increment the offset to start from in the tail command, then use head to get only 10 million lines.

```console
tail -n +10000000 training_set_full.pgn | head -n 10000000 > training_set.pgn
```

Then run:

```console
cargo run --bin train_nnue
```

Once the neural net is trained, it is tested with the last batch that has been kept out of the training set.
We can get an idea of how well it performs in `predictions.csv`

FIXME: Finish this up
