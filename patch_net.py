import re

with open("src/net.rs", "r") as f:
    content = f.read()

content = content.replace(
"""            if let Ok(s) = serde_json::to_string(&snap) {
                if tx.send(warp::ws::Message::text(s)).await.is_err() { break; }
            }""",
"""            if let Ok(s) = serde_json::to_string(&snap)
                && tx.send(warp::ws::Message::text(s)).await.is_err() { break; }""")

content = content.replace(
"""            if let Ok(text) = msg.to_str() {
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
            }""",
"""            if let Ok(text) = msg.to_str()
                && let Ok(cmd) = serde_json::from_str::<ClientCommand>(text) {
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
                    } else if cmd.action == "fire"
                        && let Some(direction) = cmd.direction.as_deref() {
                            match direction {
                                "up" => { let _ = fire_at_direction(&mut lock, 0, -1); }
                                "down" => { let _ = fire_at_direction(&mut lock, 0, 1); }
                                "left" => { let _ = fire_at_direction(&mut lock, -1, 0); }
                                "right" => { let _ = fire_at_direction(&mut lock, 1, 0); }
                                _ => {}
                            }
                        }
            }""")


with open("src/net.rs", "w") as f:
    f.write(content)
