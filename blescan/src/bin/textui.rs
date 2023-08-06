use std::{
    io::{self, Stdout},
    time::Duration, error::Error,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame,
};
use blescan::scanner::Scanner;

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
    let mut scanner = Scanner::new().await?;
    loop {
        terminal.draw(|f| {
            let items = [ListItem::new("Item 1"), ListItem::new("Item 2"), ListItem::new("Item 3")];
            let list = List::new(items)
                .block(Block::default().title("List").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");
            let size = f.size();
            f.render_widget(list, size);
        })?;
        if should_quit()? {
            break;
        }
        scanner.scan().await?;
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