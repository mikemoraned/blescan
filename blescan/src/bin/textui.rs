use std::{
    io::{self, Stdout},
    time::Duration, error::Error, rc::Rc,
};

use anyhow::{Context, Result};
use blescan::{discover_btleplug::Scanner, state::State, signature::Signature, snapshot::{Snapshot, RssiComparison}};
use chrono::{Utc, DateTime};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::{List, ListItem, Paragraph}};
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
    use blescan::chrono_extra::Truncate;

    let mut scanner = Scanner::new().await?;
    let mut state = State::default();
    let start = Utc::now();
    let mut previous_snapshot = Snapshot::default();
    loop {
        let current_snapshot = state.snapshot();
        terminal.draw(|f| {
            let now = Utc::now();
            let (named_items, anon_items) 
                = snapshot_to_list_items(&current_snapshot, &previous_snapshot, now);
            let named_list = list(named_items, "Named");
            let anon_list = list(anon_items, "Anonymous");
            let (main_layout, snapshot_layout) = layout(f);
            let runtime = format_duration((now - start).truncate_to_seconds().to_std().unwrap());
            let footer = Paragraph::new(
                    format!("Now: {now}\nRun time: {runtime}\n(press 'q' to quit)"))
                .block(Block::default().title("Context").borders(Borders::ALL))
                .style(Style::default().fg(Color::Black));
            f.render_widget(named_list, snapshot_layout[0]);
            f.render_widget(anon_list, snapshot_layout[1]);
            f.render_widget(footer, main_layout[1]);
        })?;
        if should_quit()? {
            break;
        }
        let events = scanner.scan().await?;
        state.discover(events);
        previous_snapshot = current_snapshot;
    }
    Ok(())
}

fn snapshot_to_list_items<'a>(current: &Snapshot, previous: &Snapshot, now: DateTime<Utc>) -> (Vec<ListItem<'a>>, Vec<ListItem<'a>>) {
    use humantime::format_duration;
    use blescan::chrono_extra::Truncate;

    let ordered = current.order_by_age_and_volume();
    let compared_to_previous = ordered.compared_to(now, previous);
    let (named_items, anon_items)   
        = compared_to_previous.iter().fold((Vec::new(), Vec::new()), 
            |
                (named, anon), 
                (state, comparison)
            | {
            let age_summary 
                = format_duration(comparison.relative_age.truncate_to_seconds().to_std().unwrap());
            let rssi_summary = match comparison.rssi {
                RssiComparison::Louder => "↑",
                RssiComparison::Quieter => "⌄",
                RssiComparison::Same => "=",
                RssiComparison::New => "*"
            };
            match &state.signature {
                Signature::Named(n) => {
                    let item 
                        = ListItem::new(format!("{:<32}[{}]:{:>4}({})", 
                            n, age_summary, state.rssi, rssi_summary));
                    ([named, vec![item]].concat(), anon)
                },
                Signature::Anonymous(d) => {
                    let item 
                        = ListItem::new(format!("{:x}[{}]:{:>4}({})", 
                            d, age_summary, state.rssi, rssi_summary));
                    (named, [anon, vec![item]].concat())
                }
            }
        });
    (named_items, anon_items)   
}

fn list<'a>(items: Vec<ListItem<'a>>, title: &'a str) -> List<'a> {
    List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .style(Style::default().fg(Color::Black))
}

fn layout(frame: &mut Frame<'_, CrosstermBackend<Stdout>>) -> (Rc<[Rect]>, Rc<[Rect]>) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(90),
                Constraint::Percentage(10)
            ].as_ref()
        )
        .split(frame.size());
    let snapshot_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ].as_ref()
        )
        .split(main_layout[0]);

    (main_layout, snapshot_layout)
}

fn should_quit() -> Result<bool> {
    if event::poll(Duration::from_millis(250)).context("event poll failed")? {
        if let Event::Key(key) = event::read().context("event read failed")? {
            return Ok(KeyCode::Char('q') == key.code);
        }
    }
    Ok(false)
}