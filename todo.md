# TODO list

## Under-promotions

Brag about under-promotions in winning positions, like here: https://lichess.org/PypYTOk
Also another list of things the bot should comment on:
en-passant mate
smothered mate
When it seems to be losing but we can deliver mate aka call an ambulance...
but not for me This needs improvement

## Some friendly commenting from the engine

Get the engine to emits comments on the position and send them to the spectator room

## Adjust playstyle / level based on the opponent's level


else if game.opponent.title.is_none()
|| game.opponent.title.unwrap() != "BOT"
&& game.clock.initial < 60_000
&& game.clock.increment == 0
{
info!("Human player trying to play bullet. Setting play style to
provocative"); bot_game.engine.set_play_style(PlayStyle::Provocative);
let game_id = bot_game.id.clone();
let api_clone = self.api.clone();
tokio::spawn(async move {
api_clone
.write_in_spectator_room(
game_id.as_str(),
"Human player trying to play bullet with no increment... Will play funky
openings! :)", )
.await
});
let game_id = bot_game.id.clone();
let api_clone = self.api.clone();
tokio::spawn(async move {
api_clone
.write_in_chat(
game_id.as_str(),
"Hey! You're a human player trying to play bullet with no increment. I don't
think you stand a chance. Will play funky openings! ;)", )
.await
});
