//! Terminal rendering: speech bubble, static print, and the animation loop.
//! Adapted from `momoisay`, with a RAII terminal guard so a panic can't leave
//! the terminal in raw/alternate-screen mode.

use crate::frames::{AnimatedFrames, Frame};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stdout, Write};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration as TokioDuration};

/// Wrap `text` into a bordered speech bubble with a tail pointing at Aris.
pub fn create_speech_bubble(text: &str, max_width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= max_width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }

    let width = lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0)
        .max(1);
    let mut bubble = Vec::new();
    bubble.push(format!("┌{}┐", "─".repeat(width + 2)));
    for line in &lines {
        bubble.push(format!("│ {line:<width$} │"));
    }
    bubble.push(format!("└{}┘", "─".repeat(width + 2)));
    // tail pointing left toward Aris
    bubble.push("   /".to_string());
    bubble.push("  /".to_string());
    bubble.push(" /".to_string());
    bubble
}

/// Bubble width in terminal columns (border-aware).
fn bubble_cols(bubble: &[String]) -> u16 {
    bubble.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16
}

/// Print a single still frame beside a speech bubble, once, to stdout.
pub fn display_say(frame: &Frame, text: &str) {
    let bubble = create_speech_bubble(text, 30);
    let rows = frame.lines.len().max(bubble.len());
    for i in 0..rows {
        let f = frame.lines.get(i).copied().unwrap_or("");
        let b = bubble.get(i).map(String::as_str).unwrap_or("");
        println!("{f}  {b}");
    }
}

/// Columns/rows the animation needs, given optional accompanying text.
/// Height includes one footer row (motion label + quit hint).
pub fn required_size(canvas_w: u16, canvas_h: u16, text: Option<&str>) -> (u16, u16) {
    let extra = text
        .map(|t| 2 + bubble_cols(&create_speech_bubble(t, 30)))
        .unwrap_or(0);
    (canvas_w + extra, canvas_h + 1)
}

pub fn terminal_fits(need_w: u16, need_h: u16) -> io::Result<bool> {
    let (w, h) = terminal::size()?;
    Ok(w >= need_w && h >= need_h)
}

/// RAII guard: enters raw mode + alternate screen, restores both on drop
/// (including during panic unwinding).
pub struct TerminalGuard;

impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        let mut out = stdout();
        execute!(out, EnterAlternateScreen, Hide)?;
        terminal::enable_raw_mode()?;
        execute!(out, Clear(ClearType::All))?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = terminal::disable_raw_mode();
        let _ = execute!(out, Show, LeaveAlternateScreen);
    }
}

/// Listen for q / Esc / Ctrl-C on a background task and broadcast an exit signal.
pub fn spawn_exit_listener(exit_tx: broadcast::Sender<()>) {
    tokio::spawn(async move {
        loop {
            let hit = tokio::task::spawn_blocking(|| {
                if event::poll(Duration::from_millis(10)).unwrap_or(false) {
                    if let Ok(Event::Key(k)) = event::read() {
                        return matches!(k.code, KeyCode::Char('q') | KeyCode::Esc)
                            || (k.code == KeyCode::Char('c')
                                && k.modifiers.contains(KeyModifiers::CONTROL));
                    }
                }
                false
            })
            .await;
            if let Ok(true) = hit {
                let _ = exit_tx.send(());
                break;
            }
            sleep(TokioDuration::from_millis(10)).await;
        }
    });
}

/// Play every frame of `frames` once, centered. Returns `Ok(true)` if the user
/// asked to quit. Frames are the same size, so we overwrite in place (no
/// per-frame clear) to avoid flicker.
pub async fn play_once(
    frames: &AnimatedFrames,
    canvas_w: u16,
    canvas_h: u16,
    label: &str,
    text: Option<&str>,
    mut exit_rx: broadcast::Receiver<()>,
) -> io::Result<bool> {
    let bubble = text.map(|t| create_speech_bubble(t, 30));
    let (term_w, term_h) = terminal::size()?;
    let mut out = stdout();

    let total_w = canvas_w + bubble.as_ref().map_or(0, |b| 2 + bubble_cols(b));
    let start_x = term_w.saturating_sub(total_w) / 2;
    let start_y = term_h.saturating_sub(canvas_h + 1) / 2;

    // Footer padded to the full canvas width (centered), so switching motions
    // with different label lengths leaves no stray characters behind.
    let text_line = format!("▶ {label}   ·   q / Esc to quit");
    let pad = (canvas_w as usize).saturating_sub(text_line.chars().count());
    let footer = format!(
        "{}{}{}",
        " ".repeat(pad / 2),
        text_line,
        " ".repeat(pad - pad / 2)
    );
    let footer_x = start_x;

    for frame in frames.frames.iter() {
        if exit_rx.try_recv().is_ok() {
            return Ok(true);
        }

        // Draw the frame centered on the shared canvas, padding every row to
        // the full canvas size so switching between differently-sized motions
        // (e.g. in freestyle) leaves no residue behind.
        let y_off = canvas_h.saturating_sub(frame.lines.len() as u16) / 2;
        for row in 0..canvas_h {
            let line = row
                .checked_sub(y_off)
                .and_then(|i| frame.lines.get(i as usize))
                .copied()
                .unwrap_or("");
            let padded = format!("{line:<width$}", width = canvas_w as usize);
            execute!(out, MoveTo(start_x, start_y + row), Print(padded))?;
        }

        if let Some(ref bubble) = bubble {
            let bx = start_x + canvas_w + 2;
            let by = start_y + canvas_h.saturating_sub(bubble.len() as u16) / 2;
            for (i, line) in bubble.iter().enumerate() {
                execute!(out, MoveTo(bx, by + i as u16), Print(line))?;
            }
        }

        execute!(out, MoveTo(footer_x, start_y + canvas_h), Print(&footer))?;

        out.flush()?;

        tokio::select! {
            _ = sleep(TokioDuration::from_millis(frames.interval_ms)) => {}
            _ = exit_rx.recv() => return Ok(true),
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bubble_wraps_and_borders() {
        let b = create_speech_bubble("hello world", 30);
        assert!(b.first().unwrap().starts_with('┌'));
        assert!(b.iter().any(|l| l.contains("hello")));
    }

    #[test]
    fn required_size_grows_with_text() {
        let (w0, _) = required_size(88, 31, None);
        let (w1, _) = required_size(88, 31, Some("some text here"));
        assert_eq!(w0, 88);
        assert!(w1 > w0);
    }
}
