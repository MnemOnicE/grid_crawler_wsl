import re

with open("src/state.rs", "r") as f:
    content = f.read()

# remove apply_pickup
apply_pickup_re = re.compile(r'/// Apply a pickup/drop effect at the given index in the map for the provided state\.\n/// Returns true if a pickup was consumed and applied\.\npub fn apply_pickup.*?^}\n', re.MULTILINE | re.DOTALL)
content = apply_pickup_re.sub('', content)

with open("src/state.rs", "w") as f:
    f.write(content)
