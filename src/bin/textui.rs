use std::{
    io::{self, Stdout},
    time::Duration, error::Error, rc::Rc,
};

use anyhow::{Context, Result};
use blescan::{discover_btleplug::Scanner, state::State, signature::Signature, snapshot::{Snapshot, RssiComparison, Comparison}};
use chrono::{Utc, DateTime};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use humantime::FormattedDuration;
use ratatui::{prelude::*, widgets::{Paragraph, Row, Table, Cell}};
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
                = snapshot_to_table_rows(&current_snapshot, &previous_snapshot, now);
            let named_table = table(named_items, "Named");
            let anon_table = table(anon_items, "Anonymous");
            let (main_layout, snapshot_layout) = layout(f);
            let runtime = format_duration((now - start).truncate_to_seconds().to_std().unwrap());
            let footer = Paragraph::new(
                    format!("Now: {now}\nRun time: {runtime}\n(press 'q' to quit)"))
                .block(Block::default().title("Context").borders(Borders::ALL))
                .style(Style::default().fg(Color::Black));
            f.render_widget(named_table, snapshot_layout[0]);
            f.render_widget(anon_table, snapshot_layout[1]);
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

fn snapshot_to_table_rows<'a>(current: &Snapshot, previous: &Snapshot, now: DateTime<Utc>) -> (Vec<Row<'a>>, Vec<Row<'a>>) {
    let ordered = current.order_by_age_and_volume();
    let compared_to_previous = ordered.compared_to(now, previous);
    let (named_items, anon_items)   
        = compared_to_previous.iter().fold((Vec::new(), Vec::new()), 
            |
                (named, anon), 
                (state, comparison)
            | {
            let default_style = match comparison.rssi {
                RssiComparison::New => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::Black)
            };
            let shared_cells = vec![
                Cell::from(age_summary(comparison).to_string()).style(default_style), 
                Cell::from(format!("{}",state.rssi)).style(default_style), 
                Cell::from(rssi_summary(comparison)).style(default_style)
            ];
            match &state.signature {
                Signature::Named(n) => {
                    let name_cell = Cell::from(n.to_string()).style(default_style);
                    let row 
                        = Row::new([vec![name_cell], shared_cells].concat());
                    ([named, vec![row]].concat(), anon)
                },
                Signature::Anonymous(d) => {
                    let name = format!("{d:x}");
                    let style = match comparison.rssi {
                        RssiComparison::New => Style::default().fg(Color::Red),
                        _ => match u8::from_str_radix(&name[0..2], 16) {
                            Ok(index) => Style::default().fg(Color::Indexed(index)),
                            _ => Style::default().fg(Color::Black)
                        }
                    };
                    let name_cell = Cell::from(name).style(style);
                    let row 
                        = Row::new([vec![name_cell], shared_cells].concat())
                            .style(style);
                    (named, [anon, vec![row]].concat())
                }
            }
        });
    (named_items, anon_items)   
}

fn age_summary(comparison: &Comparison) -> FormattedDuration {
    use humantime::format_duration;
    use blescan::chrono_extra::Truncate;

    format_duration(comparison.relative_age.truncate_to_seconds().to_std().unwrap())
}

fn rssi_summary(comparison: &Comparison) -> String {
    match comparison.rssi {
        RssiComparison::Louder => "↑",
        RssiComparison::Quieter => "⌄",
        RssiComparison::Same => "=",
        RssiComparison::New => "*"
    }.to_string()
} 

fn table<'a>(rows: Vec<Row<'a>>, title: &'a str) -> Table<'a> {
    Table::new(rows)
        .style(Style::default().fg(Color::Black))
        .block(Block::default().title(title).borders(Borders::ALL))
        .widths(&[Constraint::Length(32), Constraint::Length(4), Constraint::Length(4), Constraint::Length(6)])
        .header(
            Row::new(vec!["\nName", "Last\nSeen", "\nRssi", "\nChange"])
                .height(2)
                .style(Style::default().fg(Color::Yellow))
        )
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