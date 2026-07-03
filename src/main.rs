mod cli;
mod display;
mod frames;

use clap::Parser;
use cli::{Cli, Commands, Motion};
use display::{play_once, required_size, spawn_exit_listener, terminal_fits, TerminalGuard};
use rand::Rng;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Say { text } => {
            display::display_say(&frames::STATIC_FRAME, &text);
        }
        Commands::Animate { variant, text } => {
            run_loop(Some(variant), text).await;
        }
        Commands::Freestyle { text } => {
            run_loop(None, text).await;
        }
    }
}

/// Drive the animation loop. `fixed = None` means freestyle (random each pass).
async fn run_loop(fixed: Option<Motion>, text: Option<String>) {
    let (canvas_w, canvas_h) = frames::canvas_dims();
    let (need_w, need_h) = required_size(canvas_w, canvas_h, text.as_deref());

    match terminal_fits(need_w, need_h) {
        Ok(true) => {}
        Ok(false) => {
            println!("your terminal is too small for aris (need at least {need_w}x{need_h})");
            return;
        }
        Err(e) => {
            eprintln!("could not read terminal size: {e}");
            return;
        }
    }

    let _guard = match TerminalGuard::enter() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("failed to set up terminal: {e}");
            return;
        }
    };

    let (exit_tx, _) = broadcast::channel::<()>(1);
    spawn_exit_listener(exit_tx.clone());

    loop {
        let motion = fixed.unwrap_or_else(|| {
            // Fresh ThreadRng per pick so nothing !Send is held across .await.
            let i = rand::rng().random_range(0..Motion::ALL.len());
            Motion::ALL[i]
        });

        let exit_rx = exit_tx.subscribe();
        match play_once(
            motion.frames(),
            canvas_w,
            canvas_h,
            motion.label(),
            text.as_deref(),
            exit_rx,
        )
        .await
        {
            Ok(true) => break,
            Ok(false) => {}
            Err(e) => {
                eprintln!("animation error: {e}");
                break;
            }
        }
    }
    // `_guard` drops here → terminal restored.
}
