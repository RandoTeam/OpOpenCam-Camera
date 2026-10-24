//! Render one of the app's DNG files to a neutral JPEG.
//!
//! Usage:
//! `cargo run --release --example render_dng -- <input.dng> <output.jpg>`

use std::process::ExitCode;
use std::time::Instant;

use opopencam::raw_render::render_dng_file;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 3 {
        eprintln!("usage: cargo run --release --example render_dng -- <input.dng> <output.jpg>");
        return ExitCode::from(2);
    }

    let started = Instant::now();
    match render_dng_file(&arguments[1], &arguments[2]) {
        Ok(summary) => {
            println!(
                "input:  {}x{} (orientation {})",
                summary.input_width, summary.input_height, summary.orientation
            );
            println!("output: {}x{}", summary.output_width, summary.output_height);
            println!(
                "wb gains (R,G,B): {:.3}, {:.3}, {:.3}",
                summary.gains[0], summary.gains[1], summary.gains[2]
            );
            println!("bytes:  {}", summary.jpeg_bytes);
            println!("elapsed: {:.2} s", started.elapsed().as_secs_f64());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
