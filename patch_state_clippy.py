import re

with open("src/state.rs", "r") as f:
    content = f.read()

# remove unused code `let mut start_y = if py >= view_h/2 { py - view_h/2 } else { 0 };`

# in src/state.rs, change `for i in 0..size` to `for item in map.iter_mut().take(size)`
content = re.sub(r'for i in 0\.\.size \{\n\s*if rng\.gen_bool\(0\.12\) \{\n\s*map\[i\] = Tile::Wall as u8;\n\s*\}\n\s*\}', r'for item in map.iter_mut().take(size) {\n        if rng.gen_bool(0.12) {\n            *item = Tile::Wall as u8;\n        }\n    }', content)

content = re.sub(r'for i in 0\.\.size \{\n\s*if map\[i\] == Tile::Empty as u8 \{\n\s*let roll: f64 = rng\.gen_range\(0\.0\.\.1\.0\);\n\s*if roll < 0\.03 \{\n\s*map\[i\] = Tile::Resource as u8;\n\s*\} else if roll < 0\.07 \{\n\s*// health or other pickups\n\s*map\[i\] = if rng\.gen_bool\(0\.5\) \{ Tile::Health as u8 \} else \{ Tile::Smoke as u8 \};\n\s*\}\n\s*\}\n\s*\}', r'for item in map.iter_mut().take(size) {\n        if *item == Tile::Empty as u8 {\n            let roll: f64 = rng.gen_range(0.0..1.0);\n            if roll < 0.03 {\n                *item = Tile::Resource as u8;\n            } else if roll < 0.07 {\n                // health or other pickups\n                *item = if rng.gen_bool(0.5) { Tile::Health as u8 } else { Tile::Smoke as u8 };\n            }\n        }\n    }', content)

content = content.replace("let idx = rng.gen_range(0..size) as usize;", "let idx = rng.gen_range(0..size);")


with open("src/state.rs", "w") as f:
    f.write(content)
