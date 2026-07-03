//! Command-line interface — mirrors `momoisay` (say / animate / freestyle).

use crate::frames::{AnimatedFrames, A_FRAMES, BASE_FRAMES, B_FRAMES, X2_FRAMES};
use clap::{Parser, Subcommand, ValueEnum};

/// Aris teabagging. Pick a variant by number (`1`..`4`) or name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Motion {
    /// Aris teabagging (default speed).
    #[value(name = "teabag", alias = "1")]
    Base,
    /// Same, but twice as fast.
    #[value(name = "x2", alias = "2")]
    X2,
    /// Hands-together bob from the clip (1:09-1:18, minus the clone intro).
    #[value(name = "a", alias = "3")]
    A,
    /// Both-hands-up bob from the clip (1:20-1:24).
    #[value(name = "b", alias = "4")]
    B,
}

impl Motion {
    pub const ALL: [Motion; 4] = [Motion::Base, Motion::X2, Motion::A, Motion::B];

    pub fn frames(self) -> &'static AnimatedFrames {
        match self {
            Motion::Base => &BASE_FRAMES,
            Motion::X2 => &X2_FRAMES,
            Motion::A => &A_FRAMES,
            Motion::B => &B_FRAMES,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Motion::Base => "teabag",
            Motion::X2 => "teabag x2",
            Motion::A => "motion a",
            Motion::B => "hands up",
        }
    }
}

#[derive(Parser)]
#[command(name = "arisuteabagging")]
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
        /// Variant: 1|2|3|4 or teabag|x2|a|b
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
