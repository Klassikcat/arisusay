//! Embedded ASCII (Braille) frames of Aris teabagging.
//!
//! Each motion is one `frames/<name>.txt` whose frames are joined by a form-feed
//! (`\x0c`). Regenerate with `python3 tools/gen_frames.py`.
//! (Unlike momoisay we load lines verbatim — no last-char slicing — and the
//! iterator yields *every* frame.)

use lazy_static::lazy_static;
use std::sync::Arc;

/// Frame separator used by `tools/gen_frames.py`.
const SEP: char = '\u{000c}';

const STATIC_STR: &str = include_str!("../frames/static.txt");
const ARIS_STR: &str = include_str!("../frames/aris.txt");
const MOTION_A_STR: &str = include_str!("../frames/motion_a.txt");
const MOTION_B_STR: &str = include_str!("../frames/motion_b.txt");
const MOTION_C_STR: &str = include_str!("../frames/motion_c.txt");

#[derive(Debug, Clone)]
pub struct Frame {
    pub lines: Arc<[&'static str]>,
}

impl Frame {
    fn parse(block: &'static str) -> Frame {
        let lines: Vec<&'static str> = block.lines().collect();
        Frame {
            lines: lines.into(),
        }
    }

    pub fn width(&self) -> u16 {
        self.lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0) as u16
    }

    pub fn height(&self) -> u16 {
        self.lines.len() as u16
    }
}

#[derive(Debug, Clone)]
pub struct AnimatedFrames {
    pub frames: Arc<[Frame]>,
    pub interval_ms: u64,
}

impl AnimatedFrames {
    fn parse(s: &'static str, interval_ms: u64) -> AnimatedFrames {
        let frames: Vec<Frame> = s.split(SEP).map(Frame::parse).collect();
        AnimatedFrames {
            frames: frames.into(),
            interval_ms,
        }
    }

    pub fn max_dims(&self) -> (u16, u16) {
        let w = self.frames.iter().map(Frame::width).max().unwrap_or(0);
        let h = self.frames.iter().map(Frame::height).max().unwrap_or(0);
        (w, h)
    }
}

lazy_static! {
    pub static ref STATIC_FRAME: Frame = Frame::parse(STATIC_STR);
    // The transparent-source teabag, at two speeds: base (default) and x2.
    pub static ref BASE_FRAMES: AnimatedFrames = AnimatedFrames::parse(ARIS_STR, 46);
    pub static ref X2_FRAMES: AnimatedFrames = AnimatedFrames {
        frames: BASE_FRAMES.frames.clone(),
        interval_ms: 23,
    };
    // Bobs cut from the YouTube clip (green-screen source).
    pub static ref A_FRAMES: AnimatedFrames = AnimatedFrames::parse(MOTION_A_STR, 75);
    pub static ref B_FRAMES: AnimatedFrames = AnimatedFrames::parse(MOTION_B_STR, 75);
    pub static ref C_FRAMES: AnimatedFrames = AnimatedFrames::parse(MOTION_C_STR, 75);
}

/// The canvas size that fits every motion and the static still.
pub fn canvas_dims() -> (u16, u16) {
    let mut w = STATIC_FRAME.width();
    let mut h = STATIC_FRAME.height();
    for m in [&*BASE_FRAMES, &*X2_FRAMES, &*A_FRAMES, &*B_FRAMES, &*C_FRAMES] {
        let (mw, mh) = m.max_dims();
        w = w.max(mw);
        h = h.max(mh);
    }
    (w, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_frame_is_yielded() {
        // form-feed count + 1 == frame count (nothing dropped)
        assert_eq!(BASE_FRAMES.frames.len(), ARIS_STR.matches(SEP).count() + 1);
        assert_eq!(A_FRAMES.frames.len(), MOTION_A_STR.matches(SEP).count() + 1);
        assert_eq!(B_FRAMES.frames.len(), MOTION_B_STR.matches(SEP).count() + 1);
        assert_eq!(C_FRAMES.frames.len(), MOTION_C_STR.matches(SEP).count() + 1);
    }

    #[test]
    fn x2_is_twice_base_speed() {
        assert_eq!(BASE_FRAMES.interval_ms, X2_FRAMES.interval_ms * 2);
        assert_eq!(BASE_FRAMES.frames.len(), X2_FRAMES.frames.len());
    }

    #[test]
    fn frames_are_nonempty_and_rectangular() {
        for m in [&*BASE_FRAMES, &*X2_FRAMES, &*A_FRAMES, &*B_FRAMES, &*C_FRAMES] {
            assert!(!m.frames.is_empty());
            let (w, h) = m.max_dims();
            assert!(w > 0 && h > 0);
        }
    }
}
