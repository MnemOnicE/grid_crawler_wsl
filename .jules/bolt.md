## 2024-06-15 - Optimize grid rendering loop
**Learning:** Found an opportunity to replace the O(N) linear search for the player index with an index stored in the GameState, saving a search on every frame.
**Action:** Add `player_idx: usize` to `GameState`, update it in `generate_map`, `move_player` and use it in `draw_combat_ui`.

## 2024-06-15 - Optimize WS Server Tokio Runtime
**Learning:** The WebSocket server running in the background thread for the mobile client was unnecessarily using a `multi_thread` runtime. Since it only processes simple JSON messages and holds little state, a `current_thread` runtime is more lightweight and performant.
**Action:** Changed `tokio::runtime::Builder::new_multi_thread()` to `tokio::runtime::Builder::new_current_thread()`.
