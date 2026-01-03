use std::{error::Error, path::Path};

use blescan_discovery::ScanMode;
use blescan_domain::{
    signature::Signature,
    snapshot::{Comparison, RssiComparison, Snapshot},
    state::State,
};
use blescan_sinks::history::{EventSink, noop::NoopEventSink};
use chrono::{DateTime, Utc};
use clap::Parser;
use humantime::FormattedDuration;

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
    // Initialize tracing subscriber with env filter support (RUST_LOG)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    let args = Args::parse();
    let mut sink: Box<dyn EventSink> = sink(&args).await?;
    run(&mut sink, args.mode).await?;
    sink.close().await?;
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

async fn run(sink: &mut Box<dyn EventSink>, mode: ScanMode) -> Result<(), Box<dyn Error>> {
    let mut scanner = mode.create_scanner().await?;
    let mut state = State::default();
    let start = Utc::now();
    let mut previous_snapshot = Snapshot::default();

    loop {
        let current_snapshot = state.snapshot();
        let now = Utc::now();

        // Print scan cycle results
        print_scan_results(&current_snapshot, &previous_snapshot, now, start);

        let events = scanner.scan().await?;
        sink.save(&events).await?;
        state.discover(&events);
        previous_snapshot = current_snapshot;
    }
}

fn print_scan_results(
    current: &Snapshot,
    previous: &Snapshot,
    now: DateTime<Utc>,
    start: DateTime<Utc>,
) {
    use blescan_domain::chrono_extra::Truncate;
    use humantime::format_duration;

    let runtime = format_duration((now - start).truncate_to_seconds().to_std().unwrap());
    println!("\n=== Scan Results at {} (Runtime: {}) ===", now, runtime);

    let ordered = current.order_by_age_and_volume();
    let compared_to_previous = ordered.compared_to(now, previous);

    let (named_items, anon_items): (Vec<_>, Vec<_>) = compared_to_previous
        .iter()
        .partition(|(state, _)| matches!(state.signature, Signature::Named(_)));

    if !named_items.is_empty() {
        println!("\nNamed Devices:");
        println!("{:<32} {:>6} {:>4} {:>6}", "Name", "Age", "RSSI", "Change");
        println!("{}", "-".repeat(52));
        for (state, comparison) in &named_items {
            if let Signature::Named(name) = &state.signature {
                println!(
                    "{:<32} {:>6} {:>4} {:>6}",
                    name,
                    age_summary(comparison),
                    state.rssi,
                    rssi_summary(comparison)
                );
            }
        }
    }

    if !anon_items.is_empty() {
        println!("\nAnonymous Devices:");
        println!("{:<32} {:>6} {:>4} {:>6}", "Address", "Age", "RSSI", "Change");
        println!("{}", "-".repeat(52));
        for (state, comparison) in &anon_items {
            if let Signature::Anonymous(addr) = &state.signature {
                println!(
                    "{:<32} {:>6} {:>4} {:>6}",
                    addr,
                    age_summary(comparison),
                    state.rssi,
                    rssi_summary(comparison)
                );
            }
        }
    }

    let total = named_items.len() + anon_items.len();
    println!("\nTotal devices: {} (Named: {}, Anonymous: {})",
             total, named_items.len(), anon_items.len());
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

fn rssi_summary(comparison: &Comparison) -> &'static str {
    match comparison.rssi {
        RssiComparison::Louder => "↑",
        RssiComparison::Quieter => "⌄",
        RssiComparison::Same => "=",
        RssiComparison::New => "*",
    }
}
