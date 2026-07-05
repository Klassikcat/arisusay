//! Command-line interface — mirrors `momoisay` (say / animate / freestyle).

use crate::frames::{AnimatedFrames, A_FRAMES, BASE_FRAMES, B_FRAMES, C_FRAMES, X2_FRAMES};
use clap::{Parser, Subcommand, ValueEnum};

/// Aris teabagging. Pick a variant by number (`1`..`5`) or name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Motion {
    /// Aris teabagging (default speed).
    #[value(name = "teabag", alias = "1")]
    Base,
    /// Same, but twice as fast.
    #[value(name = "x2", alias = "2")]
    X2,
    /// Mid-teabag bob from the clip (1:09-1:18, minus the clone intro) — formerly `dance`.
    #[value(name = "bagging", alias = "3", alias = "a")]
    A,
    /// Hands-up teabag bob from the clip (1:20-1:24).
    #[value(name = "handsup", alias = "4", alias = "b")]
    B,
    /// Fist-pump dance bob from the clip's opening (0:01-0:06), turning around halfway.
    #[value(name = "dance", alias = "5", alias = "c")]
    C,
}

impl Motion {
    pub const ALL: [Motion; 5] = [Motion::Base, Motion::X2, Motion::A, Motion::B, Motion::C];

    pub fn frames(self) -> &'static AnimatedFrames {
        match self {
            Motion::Base => &BASE_FRAMES,
            Motion::X2 => &X2_FRAMES,
            Motion::A => &A_FRAMES,
            Motion::B => &B_FRAMES,
            Motion::C => &C_FRAMES,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Motion::Base => "teabag",
            Motion::X2 => "teabag x2",
            Motion::A => "bagging",
            Motion::B => "handsup",
            Motion::C => "dance",
        }
    }
}

#[derive(Parser)]
#[command(name = "arisusay")]
#[command(version)]
#[command(
    about = "Like momoisay, but it's Tendou Aris from Blue Archive — and she teabags. \
             Homage to the 청세치/세치혀 animation."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Print a still Aris saying the provided text (no animation)
    Say {
        /// The text for Aris to say
        text: String,
    },

    /// Loop Aris teabagging until you quit (q / Esc / Ctrl-C)
    Animate {
        /// Variant: 1|2|3|4|5 or teabag|x2|bagging|handsup|dance
        #[arg(value_enum, default_value = "teabag")]
        variant: Motion,

        /// Optional text for Aris to say alongside the animation
        #[arg(short, long)]
        text: Option<String>,
    },

    /// Loop random teabag motions forever. Pretty cool for ricing btw.
    Freestyle {
        /// Optional text for Aris to say alongside the animation
        #[arg(short, long)]
        text: Option<String>,
    },
}
