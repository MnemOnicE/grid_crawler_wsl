import re

with open("src/serial_daemon.rs", "r") as f:
    content = f.read()

# collapse nested ifs
#                         if rx_port.read_exact(&mut sync_byte).is_ok() {
#                             if sync_byte[0] == 0xAA {
#                                                 if rx_port.read_exact(payload.as_mut_slice()).is_ok() {

content = content.replace(
"""                        if rx_port.read_exact(&mut sync_byte).is_ok() {
                            if sync_byte[0] == 0xAA {
                                                if rx_port.read_exact(payload.as_mut_slice()).is_ok() {""",
"""                        if rx_port.read_exact(&mut sync_byte).is_ok()
                            && sync_byte[0] == 0xAA
                            && rx_port.read_exact(payload.as_mut_slice()).is_ok() {""")
content = content.replace(
"""                                    if lock.stats.health == 0 && lock.phase == AppPhase::Playing {
                                        lock.phase = AppPhase::GameOver;
                                    }
                                }
                            }
                        }""",
"""                                    if lock.stats.health == 0 && lock.phase == AppPhase::Playing {
                                        lock.phase = AppPhase::GameOver;
                                    }
                        }""")

with open("src/serial_daemon.rs", "w") as f:
    f.write(content)
