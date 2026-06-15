import re

with open("src/state.rs", "r") as f:
    content = f.read()

# I also need to update the test to use consume_tile_effect instead of apply_pickup
test_re = re.compile(r'let applied = apply_pickup\(&mut gs, 0\);')
content = test_re.sub('let applied = consume_tile_effect(&mut gs, Tile::Health as u8); gs.map_matrix[0] = Tile::Empty as u8;', content)

with open("src/state.rs", "w") as f:
    f.write(content)
