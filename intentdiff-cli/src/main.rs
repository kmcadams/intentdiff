mod cli;

use clap::Parser;

use intentdiff_core::{BasicAnalyzer, Engine, Profile, Snapshot};

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    let left_content = std::fs::read_to_string(args.left())?;
    let right_content = std::fs::read_to_string(args.right())?;

    let left_snapshot = Snapshot::new(args.left().clone(), left_content);
    let right_snapshot = Snapshot::new(args.right().clone(), right_content);

    let profile = Profile::k8s_web(); //hardcoded for now; TODO: tie to input argument/option

    let analyzer = Box::new(BasicAnalyzer::new(profile.rules));
    let engine = Engine::new(analyzer);

    let result = engine.run(left_snapshot, right_snapshot);

    println!("Only in left: {}", result.only_in_left.len());
    println!("Only in right: {}", result.only_in_right.len());
    println!("Diff: {:?}", result);

    Ok(())
}
