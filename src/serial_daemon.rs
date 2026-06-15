use crate::state::{AppPhase, SharedState};
use std::io::Write;
use std::time::Duration;

pub fn init_hardware_bridge(state: SharedState, port_name: &str) -> Box<dyn Write + Send> {
    let port_name_owned = port_name.to_string();

    match serialport::new(&port_name_owned, 115_200)
        .timeout(Duration::from_millis(10))
        .open()
    {
        Ok(tx_port) => {
            // try clone for RX monitoring if possible
            if let Ok(mut rx_port) = tx_port.try_clone() {
                std::thread::spawn(move || {
                    let mut sync_byte = [0u8; 1];
                    let mut payload = vec![0u8; 71];

                    loop {
                        if rx_port.read_exact(&mut sync_byte).is_ok()
                            && sync_byte[0] == 0xAA
                            && rx_port.read_exact(payload.as_mut_slice()).is_ok()
                        {
                            let mut lock = state.lock().unwrap();
                            lock.stats.health = payload[0];
                            lock.stats.armor = payload[1];
                            lock.stats.ap = payload[2];
                            lock.stats.is_supercharging = (payload[3] & 0b0000_0001) != 0;
                            lock.stats.has_shield = (payload[3] & 0b0000_0100) != 0;
                            lock.stats.active_item = payload[5];
                            lock.stats.item_charges = payload[6];
                            let map_len = lock.map_matrix.len();
                            let end = (7 + map_len).min(payload.len());
                            let count = end - 7;
                            if count > 0 {
                                lock.map_matrix[..count].copy_from_slice(&payload[7..end]);
                            }
                            if lock.stats.health == 0 && lock.phase == AppPhase::Playing {
                                lock.phase = AppPhase::GameOver;
                            }
                        }
                    }
                });
            }
            Box::new(tx_port)
        }
        Err(_) => {
            // Fallback: no hardware available. Use sink writer and spawn a simulated RX thread.
            let sink = std::io::sink();
            let sim_state = state.clone();
            std::thread::spawn(move || {
                // Simple simulator: every second, reduce cooldowns or spawn small updates
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    let mut lock = sim_state.lock().unwrap();
                    // regen small AP over time
                    lock.stats.ap = (lock.stats.ap + 1).min(12);
                }
            });
            Box::new(sink)
        }
    }
}
