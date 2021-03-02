use async_std::sync::Mutex;
use serde::Serialize;
use serde_json::json;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use tide_websockets::WebSocketConnection;

// TODO: Dynamically count the number of labels based on enum variants instead of hard-fix
const LABELS_COUNT: u8 = 2;

#[derive(Copy, Clone, Serialize)]
pub enum Label {
    X,
    O,
}

pub enum Error {
    FullyOccupied,
    SomethingWrong,
}

#[derive(Clone)]
pub struct Player {
    pub id: String,
    pub connection: WebSocketConnection,
    pub label: Option<Label>,
}

#[derive(Default, Clone)]
pub struct Game {
    id: String,
    pub board: [String; 9],
    players: Vec<Player>,
}

#[derive(Default, Clone)]
pub struct State {
    pub games: Arc<Mutex<HashMap<String, Game>>>, // interior mutability
}

impl State {
    // TODO: Consider using async
    pub async fn add_player(&self, game_id: &str, mut player: Player) -> Result<Label, Error> {
        let mut games = self.games.lock().await;
        match games.entry(game_id.to_string()) {
            Entry::Occupied(mut entry) => {
                let players = &mut entry.get_mut().players;
                if players.iter().any(|p| p.id == player.id) {
                    return match player.label {
                        Some(l) => Ok(l),
                        _ => Err(Error::SomethingWrong),
                    };
                }
                match players.len() as u8 {
                    c if c < LABELS_COUNT => {
                        // TODO: Choose from remaining labels
                        let label = match players[0].label {
                            Some(Label::X) => Label::O,
                            _ => Label::X,
                        };
                        player.label = Some(label);
                        players.push(player);
                        Ok(label)
                    }
                    LABELS_COUNT => Err(Error::FullyOccupied),
                    _ => Err(Error::SomethingWrong),
                }
            }
            Entry::Vacant(entry) => {
                let label = Label::X;
                player.label = Some(label);
                entry.insert(Game {
                    id: game_id.to_string(),
                    players: vec![player],
                    ..Default::default()
                });
                Ok(label)
            }
        }
    }

    pub async fn play(&self, game_id: &str, label: String, index: usize) -> tide::Result<()> {
        match self.games.lock().await.get_mut(game_id) {
            Some(game) => {
                game.board[index] = label;
            }
            None => {}
        }
        Ok(())
    }

    pub async fn send(
        &self,
        game_id: &str,
        command: String,
        board: [String; 9],
    ) -> tide::Result<()> {
        match self.games.lock().await.entry(game_id.to_string()) {
            Entry::Occupied(entry) => {
                for player in &entry.get().players {
                    player
                        .connection
                        .send_json(&json!({
                            "command": command,
                            "board": board,
                        }))
                        .await?;
                }
            }
            Entry::Vacant(_) => {}
        }
        Ok(())
    }
}
