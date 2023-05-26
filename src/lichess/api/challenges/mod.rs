use crate::lichess::api::*;

pub async fn accept_challenge(challenge_id: &str) -> Result<(), ()> {
  let api_endpoint: String = String::from(format!("challenge/{}/accept", challenge_id));
  //let json_response: JsonValue;
  if let Err(_) = lichess_post(&api_endpoint, "").await {
    return Err(());
  }

  Ok(())
}

pub async fn decline_challenge(challenge_id: &str, reason: &str) -> Result<(), ()> {
  let api_endpoint: String = String::from(format!("challenge/{}/decline", challenge_id));
  let body: String = String::from(format!("reason={}", encode(reason)));

  if let Ok(_) = lichess_post(&api_endpoint, &body).await {
    Ok(())
  } else {
    Err(())
  }
}

pub async fn send_challenge(player: &str) -> Result<(), ()> {
  let api_endpoint: String = String::from(format!("challenge/{}", player));
  // Let's hardcode this to 3+0 for now.
  let body_parameters =
    "rated=true&clock.limit=180&clock.increment=0&color=random&variant=standard";
  if let Ok(_) = lichess_post(&api_endpoint, body_parameters).await {
    Ok(())
  } else {
    Err(())
  }
}
