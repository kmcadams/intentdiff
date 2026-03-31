mod cli;

use clap::Parser;

use intentdiff_core::{
    Engine, Profile, SignalStrength, Snapshot, render_markdown, render_terminal,
};

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    let left_content = std::fs::read_to_string(args.left())?;
    let right_content = std::fs::read_to_string(args.right())?;

    let left_snapshot = Snapshot::new(args.left().clone(), left_content)?;
    let right_snapshot = Snapshot::new(args.right().clone(), right_content)?;

    let profile = Profile::k8s_web(); //hardcoded for now; TODO: tie to input argument/option

    let analyzer = profile.build_analyzer();
    let engine = Engine::new(Box::new(analyzer));

    let result = engine.run(left_snapshot, right_snapshot);

    let output = match args.format() {
        cli::Format::Terminal => render_terminal(&result),
        cli::Format::Markdown => render_markdown(&result),
    };

    println!("{output}");

    if let Some(threshold) = args.fail_on() {
        if result.policy.meets_or_exceeds(threshold) {
            std::process::exit(exit_code_for_threshold(threshold));
        }
    }

    Ok(())
}

fn exit_code_for_threshold(_threshold: SignalStrength) -> i32 {
    2
}
