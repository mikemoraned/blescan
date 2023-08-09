use std::{
    io::{self, Stdout},
    time::Duration, error::Error,
};

use anyhow::{Context, Result};
use blescan::{discover_btleplug::Scanner, state::State, signature::Signature};
use chrono::{Utc, DurationRound};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders}
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal().context("setup failed")?;
    run(&mut terminal).await?;
    restore_terminal(&mut terminal).context("restore terminal failed")?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode().context("failed to enable raw mode")?;
    execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stdout)).context("creating terminal failed")
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("unable to switch to main screen")?;
    terminal.show_cursor().context("unable to show cursor")
}

async fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    use humantime::format_duration;

    let mut scanner = Scanner::new().await?;
    let mut state = State::default();
    let start = Utc::now().duration_round(chrono::Duration::seconds(1)).unwrap();
    loop {
        terminal.draw(|f| {
            let ordered_by_age = state.snapshot().order_by_age_oldest_last();
            let named_items : Vec<ListItem> 
                = ordered_by_age.0.iter().flat_map(|state| {
                    let age = (state.date_time.duration_round(chrono::Duration::seconds(1)).unwrap() 
                            - start).to_std().unwrap();
                    if let Signature::Named(n) = &state.signature {
                        Some(ListItem::new(format!(
                            "{:<32}[{}]:{:>4}\n", n, format_duration(age), state.rssi)))
                    }
                    else {
                        None
                    }
                }).collect();
            let named_list = List::new(named_items)
                .block(Block::default().title("Named").borders(Borders::ALL))
                .style(Style::default().fg(Color::Black));
            let anon_items : Vec<ListItem> 
                = ordered_by_age.0.iter().flat_map(|state| {
                    if let Signature::Anonymous(d) = &state.signature {
                        let age = (state.date_time.duration_round(chrono::Duration::seconds(1)).unwrap() 
                            - start).to_std().unwrap();
                        Some(ListItem::new(format!(
                            "{:x}[{}]:{:>4}\n", d, format_duration(age), state.rssi)))
                    }
                    else {
                        None
                    }
                }).collect();
            let anon_list = List::new(anon_items)
                .block(Block::default().title("Anonymous").borders(Borders::ALL))
                .style(Style::default().fg(Color::Black));
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Percentage(50)
                    ].as_ref()
                )
                .split(f.size());
            f.render_widget(named_list, chunks[0]);
            f.render_widget(anon_list, chunks[1]);
        })?;
        if should_quit()? {
            break;
        }
        let events = scanner.scan().await?;
        state.discover(events);
    }
    Ok(())
}

// /// Render the application. This is where you would draw the application UI. This example just
// /// draws a greeting.
// fn render_app(f: &mut ratatui::Frame<CrosstermBackend<Stdout>>) {
//     // let greeting = Paragraph::new("Hello World! (press 'q' to quit)");
//     // frame.render_widget(greeting, frame.size());
//     let chunks = Layout::default()
//         .direction(Direction::Horizontal)
//         .margin(1)
//         .constraints(
//             [
//                 Constraint::Percentage(50),
//                 Constraint::Percentage(50)
//             ].as_ref()
//         )
//         .split(f.size());
//     let block = Block::default()
//          .title("Named")
//          .borders(Borders::ALL);
//     f.render_widget(block, chunks[0]);
//     let block = Block::default()
//          .title("Anonymous")
//          .borders(Borders::ALL);
//     f.render_widget(block, chunks[1]);
// }

fn should_quit() -> Result<bool> {
    if event::poll(Duration::from_millis(250)).context("event poll failed")? {
        if let Event::Key(key) = event::read().context("event read failed")? {
            return Ok(KeyCode::Char('q') == key.code);
        }
    }
    Ok(false)
}