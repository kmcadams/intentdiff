mod cli;

use clap::Parser;

use intentdiff_core::{BasicAnalyzer, EmptyDirRule, Engine, Snapshot, TlsEnabledRule};

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    let left_content = std::fs::read_to_string(args.left())?;
    let right_content = std::fs::read_to_string(args.right())?;

    let left_snapshot = Snapshot::new(args.left().clone(), left_content);
    let right_snapshot = Snapshot::new(args.right().clone(), right_content);

    let analyzer = Box::new(BasicAnalyzer::new(vec![
        Box::new(EmptyDirRule),
        Box::new(TlsEnabledRule),
    ]));
    let engine = Engine::new(analyzer);

    let result = engine.run(left_snapshot, right_snapshot);

    println!("Only in left: {}", result.only_in_left.len());
    println!("Only in right: {}", result.only_in_right.len());
    println!("Diff: {:?}", result);

    Ok(())
}
