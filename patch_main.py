import re

with open("src/main.rs", "r") as f:
    content = f.read()

content = content.replace(
"""        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc { break; }""",
"""        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc { break; }""")

content = content.replace(
"""                    }
                }
            }
        }""",
"""                    }
                }
        }""")

content = content.replace(
"""    let mut start_y = if py >= view_h/2 { py - view_h/2 } else { 0 };
    let mut start_x = if px >= view_w/2 { px - view_w/2 } else { 0 };""",
"""    let mut start_y = py.saturating_sub(view_h / 2);
    let mut start_x = px.saturating_sub(view_w / 2);""")

with open("src/main.rs", "w") as f:
    f.write(content)
