mod state;
mod serial_daemon;

use std::io;
use std::sync::Arc;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame, Terminal,
};
use state::{AppPhase, GameState, initialize_state};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game_state = initialize_state();
    let mut tx_port = serial_daemon::init_hardware_bridge(Arc::clone(&game_state), "/dev/ttyACM0");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let mut lock = game_state.lock().unwrap();

        terminal.draw(|f| {
            match lock.phase {
                AppPhase::StartScreen => draw_start_screen(f),
                AppPhase::Playing => draw_combat_ui(f, &lock),
                AppPhase::GameOver => draw_game_over(f),
                AppPhase::Scoreboard => {}
            }
        })?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Esc { break; }

                match lock.phase {
                    AppPhase::StartScreen => {
                        if key.code == KeyCode::Enter { lock.phase = AppPhase::Playing; }
                    }
                    AppPhase::Playing => {
                        match key.code {
                            KeyCode::Up => { let _ = tx_port.write_all(&[0xBB, 0x01, 0x00]); }
                            KeyCode::Down => { let _ = tx_port.write_all(&[0xBB, 0x02, 0x00]); }
                            KeyCode::Left => { let _ = tx_port.write_all(&[0xBB, 0x05, 0x00]); }
                            KeyCode::Right => { let _ = tx_port.write_all(&[0xBB, 0x06, 0x00]); }
                            KeyCode::Char(' ') => { let _ = tx_port.write_all(&[0xBB, 0x03, 0x00]); }
                            KeyCode::Char('s') => { let _ = tx_port.write_all(&[0xBB, 0x04, 0x00]); }
                            _ => {}
                        }
                    }
                    AppPhase::GameOver => {
                        if key.code == KeyCode::Char('r') { lock.phase = AppPhase::StartScreen; }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn draw_start_screen(f: &mut Frame) {
    let text = "GRID CRAWLER: TACTICAL NODE\n\nPress [ENTER] to Initiate Neural Link\nPress [ESC] to Abort";
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" SYSTEM BOOT "))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, f.area());
}

fn draw_combat_ui(f: &mut Frame, state: &GameState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(10), // The 8x8 Grid
            Constraint::Length(3),  // Controls Legend
            Constraint::Length(3),  // HP
            Constraint::Length(3),  // AR
            Constraint::Length(3),  // AP
            Constraint::Min(0),
        ].as_ref())
        .split(f.area());

    // --- The Matrix Translation Engine ---
    let mut grid_lines = Vec::new();
    for row in 0..8 {
        let mut spans = Vec::new();
        for col in 0..8 {
            let cell_byte = state.map_matrix[row * 8 + col];
            let (glyph, color) = match cell_byte {
                0x00 => ("░░", Color::DarkGray),
                0x01 => ("██", Color::White),
                0x0A => ("▲▲", Color::Green),
                0x0B => ("▼▼", Color::Red),
                0x10 => ("◆◆", Color::Yellow),
                0x11 => ("xx", Color::Magenta),
                _ => ("??", Color::Reset),
            };
            spans.push(Span::styled(glyph, Style::default().fg(color)));
        }
        grid_lines.push(Line::from(spans));
    }

    let grid_widget = Paragraph::new(grid_lines)
        .block(Block::default().borders(Borders::ALL).title(" TACTICAL MATRIX "))
        .alignment(Alignment::Center);
    f.render_widget(grid_widget, chunks[0]);

    // --- Controls Legend ---
    let controls = " [↑/↓/←/→] Move | [SPACE] Fire Laser (2AP) | [S] Supercharge (3AP) | [ESC] Abort ";
    let controls_widget = Paragraph::new(controls)
        .block(Block::default().borders(Borders::ALL).title(" SYSTEM CONTROLS "))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(controls_widget, chunks[1]);

    // --- The Gauges ---
    let health_gauge = Gauge::default()
        .block(Block::default().title(" STRUCTURAL INTEGRITY (HP) ").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .percent(state.stats.health as u16);
    f.render_widget(health_gauge, chunks[2]);

    let armor_gauge = Gauge::default()
        .block(Block::default().title(" DEFLECTIVE PLATING (ARMOR) ").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .percent(state.stats.armor as u16);
    f.render_widget(armor_gauge, chunks[3]);

    let ap_percent = (state.stats.ap as u16 * 100) / 12;
    let ap_gauge = Gauge::default()
        .block(Block::default().title(format!(" ACTION POINTS: {}/12 ", state.stats.ap)).borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .percent(ap_percent.min(100));
    f.render_widget(ap_gauge, chunks[4]);
}

fn draw_game_over(f: &mut Frame) {
    let text = "CRITICAL FAILURE: SIGNAL LOST\n\nPress [R] to Reboot Node\nPress [ESC] to Exit";
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" SYSTEM HALT "))
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, f.area());
}
