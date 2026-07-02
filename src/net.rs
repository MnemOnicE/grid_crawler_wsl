use crate::state::{SharedState, fire_at_direction, move_player};
use std::sync::Arc;
use warp::Filter;

const INDEX_HTML: &str = include_str!("index.html");

#[derive(serde::Serialize)]
struct PublicState<'a> {
    width: usize,
    height: usize,
    map: &'a [u8],
    health: u8,
    armor: u8,
    ap: u8,
}

/// Start a lightweight WebSocket server for the mobile client in a background thread.
pub fn start_ws_server(state: SharedState, addr: &str) {
    // run tokio runtime in background thread
    let state_clone = state.clone();
    let addr = addr.to_string();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let state_filter = warp::any().map(move || state_clone.clone());

            let ws_route = warp::path("ws").and(warp::ws()).and(state_filter).map(
                |ws: warp::ws::Ws, state: SharedState| {
                    ws.on_upgrade(move |socket| client_connection(socket, state))
                },
            );
            let index_route = warp::path::end().map(|| warp::reply::html(INDEX_HTML));

            let allowed_origins_env = std::env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://127.0.0.1:9001,http://localhost:9001".to_string());

            let routes = index_route
                .or(ws_route)
                .with(warp::cors().allow_origins(allowed_origins_env.split(',')));

            warp::serve(routes)
                .run(addr.parse::<std::net::SocketAddr>().unwrap())
                .await;
        });
    });
}

async fn client_connection(ws: warp::ws::WebSocket, state: SharedState) {
    use futures::{SinkExt, StreamExt};

    #[derive(serde::Deserialize)]
    struct ClientCommand {
        action: String,
        direction: Option<String>,
    }

    let (mut tx, mut rx) = ws.split();
    let state_for_send = Arc::clone(&state);
    let state_for_recv = Arc::clone(&state);

    let send_task = tokio::spawn(async move {
        loop {
            // Serialize while holding the lock to avoid cloning the map matrix
            let json_res = {
                let lock = state_for_send.lock().unwrap();
                let snap = PublicState {
                    width: lock.width,
                    height: lock.height,
                    map: &lock.map_matrix,
                    health: lock.stats.health,
                    armor: lock.stats.armor,
                    ap: lock.stats.ap,
                };
                serde_json::to_string(&snap)
            };
            if let Ok(s) = json_res
                && tx.send(warp::ws::Message::text(s)).await.is_err()
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = rx.next().await {
            if let Ok(text) = msg.to_str()
                && let Ok(cmd) = serde_json::from_str::<ClientCommand>(text)
            {
                let mut lock = state_for_recv.lock().unwrap();
                if let Some(direction) = cmd.direction.as_deref() {
                    let offsets = match direction {
                        "up" => Some((0, -1)),
                        "down" => Some((0, 1)),
                        "left" => Some((-1, 0)),
                        "right" => Some((1, 0)),
                        _ => None,
                    };

                    if let Some((dx, dy)) = offsets {
                        if cmd.action == "move" {
                            let _ = move_player(&mut lock, dx, dy);
                        } else if cmd.action == "fire" {
                            let _ = fire_at_direction(&mut lock, dx, dy);
                        }
                    }
                }
            }
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}
