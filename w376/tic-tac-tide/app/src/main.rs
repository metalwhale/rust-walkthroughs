mod state;

use futures_util::StreamExt;
use petname::Petnames;
use serde_json::json;
use state::{Error, Player, State};
use tide::{Body, Request};
use tide_websockets::{Message, WebSocket};

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut server = tide::with_state(State::default());
    // public route for assets
    server.at("/public").serve_dir("./public/")?;
    // index route
    server
        .at("/")
        .get(|_| async { Ok(Body::from_file("./public/index.html").await?) });
    // game route
    server
        .at("/:id")
        .with(WebSocket::new(
            |request: Request<State>, mut connection| async move {
                let state = request.state();
                let game_id = request.param("id")?;
                let player_id = Petnames::default().generate_one(4, "-");
                let player = Player {
                    id: player_id.clone(),
                    connection: connection.clone(),
                    label: None,
                };
                match state.add_player(game_id, player).await {
                    Ok(label) => {
                        let games = state.games.lock().await;
                        connection
                            .send_json(&json!({
                                "command": "ADD",
                                "label": label,
                                "board": games.get(game_id).unwrap().board,
                                "player_id": player_id,
                            }))
                            .await?;
                    }
                    Err(error) => {
                        connection
                            .send_json(&json!({
                                "command": match error {
                                    Error::FullyOccupied => "FULL",
                                    Error::SomethingWrong => "",
                                }
                            }))
                            .await?;
                    }
                };
                while let Some(Ok(Message::Text(message))) = connection.next().await {
                    let parts: Vec<&str> = message.split(":").collect();
                    let (command, label) = (parts[0], parts[1]);
                    match command {
                        "PLAY" => {
                            let index: usize = parts[2].parse().unwrap_or_default();
                            state
                                .play(game_id, label.to_string(), index)
                                .await
                                .unwrap_or_default();
                            let board =
                                state.games.lock().await.get(game_id).unwrap().board.clone();
                            state.send(game_id, "STATE".to_string(), board).await?;
                        }
                        _ => {}
                    }
                }
                Ok(())
            },
        ))
        .get(|_| async { Ok(Body::from_file("./public/board.html").await?) });
    server.at("/new").post(|_| async {
        let board_name = Petnames::default().generate_one(2, "-");
        Ok(json!({ "board_name": board_name }))
    });
    server.listen("0.0.0.0:8080").await?;
    Ok(())
}
