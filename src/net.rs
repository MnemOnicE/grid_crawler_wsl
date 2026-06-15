use crate::state::{SharedState, fire_at_direction, move_player};
use std::sync::Arc;
use warp::Filter;

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang=\"en\">
<head>
<meta charset=\"utf-8\">
<title>Grid Crawler Mobile</title>
<style>
body { background:#121212; color:#eee; font-family:system-ui, sans-serif; text-align:center; margin:0; padding:1rem; }
button { margin:.25rem; padding:.75rem 1.1rem; font-size:1rem; border:none; border-radius:.5rem; background:#007acc; color:#fff; }
#log { margin-top:1rem; font-family:monospace; white-space:pre-wrap; text-align:left; max-width:100%; }
</style>
</head>
<body>
<h1>Grid Crawler Remote</h1>
<div>
<button onclick=\"moveDir('up')\">Move ↑</button>
<button onclick=\"moveDir('left')\">Move ←</button>
<button onclick=\"moveDir('right')\">Move →</button>
<button onclick=\"moveDir('down')\">Move ↓</button>
</div>
<div>
<button onclick=\"fireDir('up')\">Fire ↑</button>
<button onclick=\"fireDir('left')\">Fire ←</button>
<button onclick=\"fireDir('right')\">Fire →</button>
<button onclick=\"fireDir('down')\">Fire ↓</button>
</div>
<pre id=\"log\">Connecting…</pre>
<script>
const log = document.getElementById('log');
const ws = new WebSocket(`ws://${location.host}/ws`);
ws.onopen = () => log.textContent = 'Connected.';
ws.onmessage = event => {
  const state = JSON.parse(event.data);
  log.textContent = `Map ${state.width}x${state.height}\nHP:${state.health} AP:${state.ap} ARMOR:${state.armor}\n\n` + state.map.map(v => v.toString(16).padStart(2,'0')).reduce((acc,val,i)=> {
    const row = Math.floor(i/state.width);
    return acc + val + ((i+1)%state.width===0 ? '\n' : ' ');
  }, '');
};
ws.onclose = () => log.textContent += '\nDisconnected.';
function sendCommand(action, direction) { ws.send(JSON.stringify({ action, direction })); }
function moveDir(dir) { sendCommand('move', dir); }
function fireDir(dir) { sendCommand('fire', dir); }
</script>
</body>
</html>"#;

#[derive(serde::Serialize)]
struct PublicState {
    width: usize,
    height: usize,
    map: Vec<u8>,
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
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let state_filter = warp::any().map(move || state_clone.clone());

            let ws_route = warp::path("ws")
                .and(warp::ws())
                .and(state_filter)
                .map(|ws: warp::ws::Ws, state: SharedState| {
                    ws.on_upgrade(move |socket| client_connection(socket, state))
                });

            let index_route = warp::path::end().map(|| warp::reply::html(INDEX_HTML));

            let routes = index_route.or(ws_route)
                .with(warp::cors().allow_any_origin());

            warp::serve(routes).run(addr.parse::<std::net::SocketAddr>().unwrap()).await;
        });
    });
}

async fn client_connection(ws: warp::ws::WebSocket, state: SharedState) {
    use futures::{StreamExt, SinkExt};

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
            // prepare snapshot
            let snap = {
                let lock = state_for_send.lock().unwrap();
                PublicState {
                    width: lock.width,
                    height: lock.height,
                    map: lock.map_matrix.clone(),
                    health: lock.stats.health,
                    armor: lock.stats.armor,
                    ap: lock.stats.ap,
                }
            };
            if let Ok(s) = serde_json::to_string(&snap) {
                if tx.send(warp::ws::Message::text(s)).await.is_err() { break; }
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = rx.next().await {
            if let Ok(text) = msg.to_str() {
                if let Ok(cmd) = serde_json::from_str::<ClientCommand>(text) {
                    let mut lock = state_for_recv.lock().unwrap();
                    if cmd.action == "move" {
                        if let Some(direction) = cmd.direction.as_deref() {
                            match direction {
                                "up" => { let _ = move_player(&mut lock, 0, -1); }
                                "down" => { let _ = move_player(&mut lock, 0, 1); }
                                "left" => { let _ = move_player(&mut lock, -1, 0); }
                                "right" => { let _ = move_player(&mut lock, 1, 0); }
                                _ => {}
                            }
                        }
                    } else if cmd.action == "fire" {
                        if let Some(direction) = cmd.direction.as_deref() {
                            match direction {
                                "up" => { let _ = fire_at_direction(&mut lock, 0, -1); }
                                "down" => { let _ = fire_at_direction(&mut lock, 0, 1); }
                                "left" => { let _ = fire_at_direction(&mut lock, -1, 0); }
                                "right" => { let _ = fire_at_direction(&mut lock, 1, 0); }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}
