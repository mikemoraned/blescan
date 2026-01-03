use std::{
    error::Error,
    io::{self, Stdout},
    path::Path,
    rc::Rc,
    time::Duration,
};

use anyhow::{Context, Result};
use blescan_discovery::ScanMode;
use blescan_domain::{
    signature::Signature,
    snapshot::{Comparison, RssiComparison, Snapshot},
    state::State,
};
use blescan_sinks::history::{EventSink, noop::NoopEventSink};
use chrono::{DateTime, Utc};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use humantime::FormattedDuration;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
};
use ratatui::{
    prelude::*,
    widgets::{Cell, Paragraph, Row, Table},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to SQLite db file to record events to
    #[arg(short, long)]
    db: Option<String>,

    /// scan mode: local or mote
    #[arg(short, long, default_value = "local")]
    mode: ScanMode,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut terminal = setup_terminal().context("setup failed")?;
    let mut sink: Box<dyn EventSink> = sink(&args).await?;
    run(&mut sink, &mut terminal, args.mode).await?;
    sink.close().await?;
    restore_terminal(&mut terminal).context("restore terminal failed")?;
    Ok(())
}

async fn sink(args: &Args) -> Result<Box<dyn EventSink>, Box<dyn Error>> {
    use blescan_sinks::history::sqllite::SQLLiteEventSink;

    match &args.db {
        Some(name) => {
            let path = Path::new(&name);
            SQLLiteEventSink::create_from_file(path).await
        }
        None => Ok(Box::<NoopEventSink>::default()),
    }
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

async fn run(
    sink: &mut Box<dyn EventSink>,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mode: ScanMode,
) -> Result<(), Box<dyn Error>> {
    use blescan_domain::chrono_extra::Truncate;
    use humantime::format_duration;

    let mut scanner = mode.create_scanner().await?;
    let mut state = State::default();
    let start = Utc::now();
    let mut previous_snapshot = Snapshot::default();
    loop {
        let current_snapshot = state.snapshot();
        terminal.draw(|f| {
            let now = Utc::now();
            let (named_items, anon_items) =
                snapshot_to_table_rows(&current_snapshot, &previous_snapshot, now);
            let named_table = table(named_items, "Named");
            let anon_table = table(anon_items, "Anonymous");
            let (main_layout, snapshot_layout) = layout(f);
            let runtime = format_duration((now - start).truncate_to_seconds().to_std().unwrap());
            let footer = Paragraph::new(format!(
                "Now: {now}, Total Run time: {runtime}\n(press 'q' to quit)"
            ))
            .block(Block::default().title("Context").borders(Borders::ALL))
            .style(Style::default().fg(Color::Black));
            f.render_widget(named_table, snapshot_layout[0]);
            f.render_widget(anon_table, snapshot_layout[1]);
            f.render_widget(footer, main_layout[0]);
        })?;
        if should_quit()? {
            break;
        }
        let events = scanner.scan().await?;
        sink.save(&events).await?;
        state.discover(&events);
        previous_snapshot = current_snapshot;
    }
    Ok(())
}

fn snapshot_to_table_rows<'a>(
    current: &Snapshot,
    previous: &Snapshot,
    now: DateTime<Utc>,
) -> (Vec<Row<'a>>, Vec<Row<'a>>) {
    let ordered = current.order_by_age_and_volume();
    let compared_to_previous = ordered.compared_to(now, previous);
    let (named_items, anon_items) = compared_to_previous.iter().fold(
        (Vec::new(), Vec::new()),
        |(named, anon), (state, comparison)| {
            let default_style = match comparison.rssi {
                RssiComparison::New => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::Black),
            };
            let shared_cells = vec![
                Cell::from(age_summary(comparison).to_string()).style(default_style),
                Cell::from(format!("{}", state.rssi)).style(default_style),
                Cell::from(rssi_summary(comparison)).style(default_style),
            ];
            match &state.signature {
                Signature::Named(n) => {
                    let name_cell = Cell::from(n.to_string()).style(default_style);
                    let row = Row::new([vec![name_cell], shared_cells].concat());
                    ([named, vec![row]].concat(), anon)
                }
                Signature::Anonymous(d) => {
                    let name = d.clone();
                    let style = match comparison.rssi {
                        RssiComparison::New => Style::default().fg(Color::Red),
                        _ => match u8::from_str_radix(&name[0..2], 16) {
                            Ok(index) => Style::default().fg(Color::Indexed(index)),
                            _ => Style::default().fg(Color::Black),
                        },
                    };
                    let name_cell = Cell::from(name).style(style);
                    let row = Row::new([vec![name_cell], shared_cells].concat()).style(style);
                    (named, [anon, vec![row]].concat())
                }
            }
        },
    );
    (named_items, anon_items)
}

fn age_summary(comparison: &Comparison) -> FormattedDuration {
    use blescan_domain::chrono_extra::Truncate;
    use humantime::format_duration;

    format_duration(
        comparison
            .relative_age
            .truncate_to_seconds()
            .to_std()
            .unwrap(),
    )
}

fn rssi_summary(comparison: &Comparison) -> String {
    match comparison.rssi {
        RssiComparison::Louder => "↑",
        RssiComparison::Quieter => "⌄",
        RssiComparison::Same => "=",
        RssiComparison::New => "*",
    }
    .to_string()
}

fn table<'a>(rows: Vec<Row<'a>>, title: &'a str) -> Table<'a> {
    Table::new(
        rows,
        &[
            Constraint::Length(32),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(6),
        ],
    )
    .style(Style::default().fg(Color::Black))
    .block(Block::default().title(title).borders(Borders::ALL))
    .header(
        Row::new(vec!["\nName", "Last\nSeen", "\nRssi", "\nChange"])
            .height(2)
            .style(Style::default().fg(Color::Yellow)),
    )
}

fn layout(frame: &mut Frame) -> (Rc<[Rect]>, Rc<[Rect]>) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(frame.area());
    let snapshot_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(main_layout[1]);
    (main_layout, snapshot_layout)
}

fn should_quit() -> Result<bool> {
    if event::poll(Duration::from_millis(250)).context("event poll failed")?
        && let Event::Key(key) = event::read().context("event read failed")?
    {
        return Ok(KeyCode::Char('q') == key.code);
    }
    Ok(false)
}
