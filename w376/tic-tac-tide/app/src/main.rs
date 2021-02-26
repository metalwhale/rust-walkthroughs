mod state;

use futures_util::StreamExt;
use petname::Petnames;
use serde_json::json;
use tide::Body;
use tide_websockets::{Message, WebSocket};

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut server = tide::new();
    // public route for assets
    server.at("/public").serve_dir("./public/")?;
    // index route
    server
        .at("/")
        .get(|_| async { Ok(Body::from_file("./public/index.html").await?) });
    server
        .at("/:id")
        .with(WebSocket::new(|_, mut connection| async move {
            while let Some(Ok(Message::Text(message))) = connection.next().await {
                println!("{}", message);
            }
            Ok(())
        }))
        .get(|_| async { Ok(Body::from_file("./public/board.html").await?) });
    server.at("/new").post(|_| async {
        let board_name = Petnames::default().generate_one(4, "-");
        Ok(json!({ "board_name": board_name }))
    });
    server.listen("0.0.0.0:8080").await?;
    Ok(())
}
