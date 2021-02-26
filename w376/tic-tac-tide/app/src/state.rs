use std::collections::{hash_map::Entry, HashMap};
use tide_websockets::WebSocketConnection;

// TODO: Dynamically count the number of labels based on enum variants instead of hard-fix
const LABELS_COUNT: u8 = 2;

#[derive(Copy, Clone)]
enum Label {
    X,
    O,
}

enum Error {
    FullyOccupied,
    SomethingWrong,
}

struct Player {
    id: Option<String>,
    connection: WebSocketConnection,
    label: Label,
}

#[derive(Default)]
struct Game {
    id: String,
    board: [String; 9],
    players: Vec<Player>,
}

#[derive(Default)]
struct State {
    games: HashMap<String, Game>,
}

impl State {
    fn add_player(&mut self, game_id: &str, mut player: Player) -> Result<Label, Error> {
        match self.games.entry(game_id.to_string()) {
            Entry::Occupied(mut entry) => {
                let players = &mut entry.get_mut().players;
                if players.iter().any(|p| p.id == player.id) {
                    let label = player.label;
                    return Ok(label);
                }
                match players.len() as u8 {
                    l if l < LABELS_COUNT => {
                        // TODO: Choose from remaining labels
                        let label = match players[0].label {
                            Label::X => Label::O,
                            _ => Label::X,
                        };
                        player.label = label;
                        players.push(player);
                        Ok(label)
                    }
                    LABELS_COUNT => Err(Error::FullyOccupied),
                    _ => Err(Error::SomethingWrong),
                }
            }
            Entry::Vacant(entry) => {
                let label = Label::X;
                player.label = label;
                entry.insert(Game {
                    id: game_id.to_string(),
                    players: vec![player],
                    ..Default::default()
                });
                Ok(label)
            }
        }
    }
}
