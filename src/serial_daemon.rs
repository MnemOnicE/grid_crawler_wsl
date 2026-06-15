use serialport::SerialPort;
use std::time::Duration;
use crate::state::{SharedState, AppPhase};

pub fn init_hardware_bridge(state: SharedState, port_name: &str) -> Box<dyn SerialPort> {
    let port_name_owned = port_name.to_string();

    let tx_port = serialport::new(&port_name_owned, 115_200)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Hardware bridge offline. Check usbipd attach.");

    let mut rx_port = tx_port.try_clone().expect("Failed to clone RX port");

    std::thread::spawn(move || {
        let mut sync_byte = [0u8; 1];
        let mut payload = vec![0u8; 71]; 
        
        loop {
            // 1. Hunt for the Sync Sentinel (0xAA) ONE byte at a time
            if rx_port.read_exact(&mut sync_byte).is_ok() {
                if sync_byte[0] == 0xAA {
                    
                    // 2. Sentinel found. The stream is aligned. Grab the remaining 71 bytes.
                    if rx_port.read_exact(payload.as_mut_slice()).is_ok() {
                        let mut lock = state.lock().unwrap();
                        
                        // Shift all indices down by 1 because we already consumed the 0xAA byte
                        lock.stats.health = payload[0];
                        lock.stats.armor = payload[1];
                        lock.stats.ap = payload[2];
                        lock.stats.is_supercharging = (payload[3] & 0b0000_0001) != 0;
                        lock.stats.has_shield       = (payload[3] & 0b0000_0100) != 0;
                        lock.stats.active_item = payload[5];
                        lock.stats.item_charges = payload[6];

                        // Blast the 64-byte matrix into the UI
                        lock.map_matrix.copy_from_slice(&payload[7..71]);

                        if lock.stats.health == 0 && lock.phase == AppPhase::Playing {
                            lock.phase = AppPhase::GameOver;
                        }
                    }
                }
            }
        }
    });

    tx_port
}
