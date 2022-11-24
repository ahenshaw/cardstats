use clap::Parser;
use spinners::{Spinner, Spinners};
use std::time::Instant;
use thousands::Separable;

use cardstats::simulate_hands;

#[derive(Parser, Debug)]
struct Args {
    /// Approximate umber of hands to simulate.  The actual number will be a multiple of 10.
    hands: usize,
}

fn main() {
    let args = Args::parse();

    println!();
    let mut spinner = Spinner::new(
        Spinners::Dots12,
        format!(
            "Simulating {} standard poker hands",
            args.hands.separate_with_commas()
        )
        .into(),
    );
    let start = Instant::now();
    let counts = simulate_hands(args.hands);
    let elapsed = Instant::now() - start;

    let reps: usize = counts.total();
    spinner.stop_with_message(format!(
        "Finished simulating {} standard poker hands",
        reps.separate_with_commas()
    ));

    println!("Elapsed time: {:?}\n", elapsed);
    let mut stats = counts.most_common();
    stats.reverse();
    for (k, v) in stats {
        println!(
            "{:14}: {:10.6}%",
            format!("{k:?}"),
            100.0 * v as f32 / reps as f32
        );
    }
}
