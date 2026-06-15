mod net;
mod serial_daemon;
mod state;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::random;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};
use state::{
    AppPhase, GameState, fire_at_direction, initialize_state, move_player, regenerate_map,
    spawn_drops,
};
use std::env;
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--version" || arg == "-v") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let game_state = initialize_state();
    let _tx_port = serial_daemon::init_hardware_bridge(Arc::clone(&game_state), "/dev/ttyACM0");
    net::start_ws_server(Arc::clone(&game_state), "127.0.0.1:9001");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut last_spawn = Instant::now();
    let mut aiming = false;

    loop {
        let mut lock = game_state.lock().unwrap();

        terminal.draw(|f| match lock.phase {
            AppPhase::StartScreen => draw_start_screen(f, &lock),
            AppPhase::Playing => draw_combat_ui(f, &lock, aiming),
            AppPhase::GameOver => draw_game_over(f),
        })?;

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
        {
            if key.code == KeyCode::Esc {
                break;
            }

            match lock.phase {
                AppPhase::StartScreen => {
                    if key.code == KeyCode::Enter {
                        lock.phase = AppPhase::Playing;
                    }
                    if key.code == KeyCode::Char('g') {
                        // randomize seed
                        let new_seed = random::<u64>();
                        let cur_w = lock.width;
                        regenerate_map(&mut lock, new_seed, cur_w);
                        lock.seed = new_seed;
                    }
                    if key.code == KeyCode::Char('s') {
                        // change map size cycle
                        let sizes = [8usize, 10, 12, 16];
                        let cur = lock.width;
                        let mut next = sizes[0];
                        for &sz in &sizes {
                            if sz > cur {
                                next = sz;
                                break;
                            }
                        }
                        if next == cur {
                            next = sizes[0];
                        }
                        let new_seed = lock.seed.wrapping_add(1);
                        regenerate_map(&mut lock, new_seed, next);
                    }
                }
                AppPhase::Playing => {
                    let _action_taken = match key.code {
                        KeyCode::Up | KeyCode::Char('w') => {
                            if aiming {
                                aiming = false;
                                fire_at_direction(&mut lock, 0, -1)
                            } else {
                                move_player(&mut lock, 0, -1)
                            }
                        }
                        KeyCode::Down | KeyCode::Char('s') => {
                            if aiming {
                                aiming = false;
                                fire_at_direction(&mut lock, 0, 1)
                            } else {
                                move_player(&mut lock, 0, 1)
                            }
                        }
                        KeyCode::Left | KeyCode::Char('a') => {
                            if aiming {
                                aiming = false;
                                fire_at_direction(&mut lock, -1, 0)
                            } else {
                                move_player(&mut lock, -1, 0)
                            }
                        }
                        KeyCode::Right | KeyCode::Char('d') => {
                            if aiming {
                                aiming = false;
                                fire_at_direction(&mut lock, 1, 0)
                            } else {
                                move_player(&mut lock, 1, 0)
                            }
                        }
                        KeyCode::Char(' ') => {
                            aiming = !aiming;
                            false
                        }
                        KeyCode::Char('O') => {
                            /* Overdrive */
                            false
                        }
                        _ => false,
                    };
                }
                AppPhase::GameOver => {
                    if key.code == KeyCode::Char('r') {
                        lock.phase = AppPhase::StartScreen;
                    }
                }
            }
        }

        if lock.phase == AppPhase::Playing && last_spawn.elapsed() >= Duration::from_secs(5) {
            spawn_drops(&mut lock, random::<u64>());
            last_spawn = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn print_help() {
    println!("Grid Crawler WSL {}\n", env!("CARGO_PKG_VERSION"));
    println!("Usage: cargo run --release -- [OPTIONS]\n");
    println!("Options:");
    println!("  -h, --help       Print this help message");
    println!("  -v, --version    Print the current version");
    println!("\nRun the binary and open the mobile client at http://127.0.0.1:9001/");
}

fn draw_start_screen(f: &mut Frame, state: &GameState) {
    let text = format!(
        "GRID CRAWLER: TACTICAL NODE\n\nSeed: {}  Size: {}x{}\n\nPress [ENTER] to Initiate Neural Link\nPress [G] Randomize Seed\nPress [S] Change Map Size\nPress [ESC] to Abort",
        state.seed, state.width, state.height,
    );
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" SYSTEM BOOT "),
        )
        .alignment(Alignment::Center);
    f.render_widget(paragraph, f.area());
}

fn draw_combat_ui(f: &mut Frame, state: &GameState, aiming: bool) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(10), // The 8x8 Grid
                Constraint::Length(3),  // Controls Legend
                Constraint::Length(3),  // HP
                Constraint::Length(3),  // AR
                Constraint::Length(3),  // AP
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(f.area());

    // --- The Battlefield Renderer (supports viewport for larger maps) ---
    let mut grid_lines = Vec::new();
    // viewport dimensions
    let view_h = state.height.min(8);
    let view_w = state.width.min(16);
    // find player to center viewport
    let player_idx = state.player_idx;
    let px = player_idx % state.width;
    let py = player_idx / state.width;
    let mut start_y = py.saturating_sub(view_h / 2);
    let mut start_x = px.saturating_sub(view_w / 2);
    if start_y + view_h > state.height {
        start_y = state.height.saturating_sub(view_h);
    }
    if start_x + view_w > state.width {
        start_x = state.width.saturating_sub(view_w);
    }
    for row in start_y..(start_y + view_h) {
        let mut spans = Vec::new();
        for col in start_x..(start_x + view_w) {
            let idx = row * state.width + col;
            let cell_byte = state.map_matrix.get(idx).copied().unwrap_or(0x00);
            let (glyph, color) = match cell_byte {
                0x00 => ("··", Color::DarkGray), // empty / mud
                0x01 => ("██", Color::White),    // obstacle / cover
                0x0A => ("⊙⊙", Color::Green),    // player tank
                0x0B => ("✖✖", Color::Red),      // enemy tank
                0x10 => ("◆◆", Color::Yellow),   // pickup / powerup
                0x11 => ("░░", Color::Magenta),  // wreckage
                _ => ("??", Color::Reset),
            };
            spans.push(Span::styled(glyph, Style::default().fg(color)));
        }
        grid_lines.push(Line::from(spans));
    }

    let grid_widget = Paragraph::new(grid_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" BATTLEFIELD "),
        )
        .alignment(Alignment::Center);
    f.render_widget(grid_widget, chunks[0]);

    // --- Controls Legend ---
    let controls = if aiming {
        " [↑/↓/←/→/WASD] Choose Fire Direction | [SPACE] Cancel Aim "
    } else {
        " [↑/↓/←/→/WASD] Maneuver Tank | [SPACE] Aim Shell | [O] Overdrive | [ESC] Retreat "
    };
    let controls_widget = Paragraph::new(controls)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" SYSTEM CONTROLS "),
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(controls_widget, chunks[1]);

    // --- The Gauges ---
    let health_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" STRUCTURAL INTEGRITY (HP) ")
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .percent(state.stats.health as u16);
    f.render_widget(health_gauge, chunks[2]);

    let armor_gauge = Gauge::default()
        .block(
            Block::default()
                .title(" DEFLECTIVE PLATING (ARMOR) ")
                .borders(Borders::ALL),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .percent(state.stats.armor as u16);
    f.render_widget(armor_gauge, chunks[3]);

    let ap_percent = (state.stats.ap as u16 * 100) / 12;
    let ap_gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!(" ACTION POINTS: {}/12 ", state.stats.ap))
                .borders(Borders::ALL),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .percent(ap_percent.min(100));
    f.render_widget(ap_gauge, chunks[4]);
}

fn draw_game_over(f: &mut Frame) {
    let text = "CRITICAL FAILURE: SIGNAL LOST\n\nPress [R] to Reboot Node\nPress [ESC] to Exit";
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" SYSTEM HALT "),
        )
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, f.area());
}
