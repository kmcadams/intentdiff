mod cli;

use clap::Parser;

use intentdiff_core::{BasicAnalyzer, SemanticAnalyzer, Snapshot, diff_signals};

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    let left_content = std::fs::read_to_string(args.left())?;
    let right_content = std::fs::read_to_string(args.right())?;

    let left_snapshot = Snapshot::new(args.left().clone(), left_content);
    let right_snapshot = Snapshot::new(args.right().clone(), right_content);

    let analyzer = BasicAnalyzer;

    let left_signals = analyzer.analyze(&left_snapshot);
    let right_signals = analyzer.analyze(&right_snapshot);

    let diff = diff_signals(&left_signals, &right_signals);

    println!("Only in left: {}", diff.only_in_left.len());
    println!("Only in right: {}", diff.only_in_right.len());
    println!("Diff: {:?}", diff);

    Ok(())
}
