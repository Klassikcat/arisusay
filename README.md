# ArisuSay

Like [`momoisay`](https://crates.io/crates/momoisay), but instead of Momoi saying
things, it's **Tendou Aris** (天童アリス — the dark-haired Blue Archive girl) doing her
famous teabag, rendered as monochrome **Braille** dots in your terminal. Aris is drawn
alone — background removed — halo and all.

```
▶ teabag   ·   q / Esc to quit
```

## Variants

Pick a variant with `animate <variant>` — by **number** or **name**:

| # | name     | what it is |
|---|----------|------------|
| 1 | `teabag` | the teabag loop (transparent source), default speed |
| 2 | `x2`     | same loop, twice as fast |
| 3 | `a`      | hands-together bob from the clip (1:09–1:18, minus the clone intro) |
| 4 | `b`      | both-hands-up bob from the clip (1:20–1:24) |

## Install

```sh
cargo build --release          # → target/release/arisusay
# or install to ~/.cargo/bin:
cargo install --path .
```

## Usage

```sh
arisusay say "정의 실현!"       # static Aris + speech bubble (prints once)

arisusay animate               # teabag, default speed
arisusay animate x2            # twice as fast        (same as: animate 2)
arisusay animate a             # hands-together bob   (same as: animate 3)
arisusay animate b             # both-hands-up bob    (same as: animate 4)

arisusay freestyle             # random variant each loop
arisusay freestyle -t "ㅋㅋㅋ"  # ...with a speech bubble
```

**Controls:** `q`, `Esc`, or `Ctrl-C` to quit. The animation runs in the alternate
screen and restores your terminal on exit (even on panic).

**Terminal size:** the canvas is ~34×22, so a window of ~36×26 is enough (wider when
using `-t`/`--text`). Too small → it prints a friendly message and exits.

## How the frames are made

Frames are pre-rendered monochrome Braille baked into the binary via `include_str!` —
no image files or codecs at runtime, exactly like `momoisay`. They live in
`frames/*.txt` (one file per variant; individual frames joined by a form-feed).

The source media is **not committed** (it's the original creator's content, and
the build doesn't need it — only `frames/*.txt` does). To regenerate the frames,
first fetch the sources into `refs/`:

```sh
mkdir -p refs
# teabag loop (transparent background) — `teabag` / `x2`
curl -L -o refs/aris2.gif "https://media.tenor.com/oA5ClfmykW8AAAAi/alice-aris.gif"
# green-screen clips from https://www.youtube.com/watch?v=T9F1Wk8DQdg
yt-dlp --download-sections "*00:01:05-00:01:27" -f "bv*[height<=720]/b" \
       -o "refs/yt_src.%(ext)s"  "https://www.youtube.com/watch?v=T9F1Wk8DQdg"
yt-dlp --download-sections "*00:01:14-00:01:30" -f "bv*[height<=720]/b" \
       -o "refs/yt_src3.%(ext)s" "https://www.youtube.com/watch?v=T9F1Wk8DQdg"
```

- `aris2.gif` — `teabag` / `x2`
- `yt_src.webm` — `a` is cut from it (8.5–13 s: the single-Aris hands-together
  bob, after the clone intro ends)
- `yt_src3.mp4` — `b` is cut from it (5.75–11.3 s: the both-hands-up bob,
  ending before a second Aris walks in)

Then regenerate:

```sh
python3 tools/gen_frames.py          # needs Python, Pillow, numpy, scipy, ffmpeg
```

Note: yt-dlp's section download seeks by keyframe, so cut windows in
`tools/gen_frames.py` may need a small nudge if you re-download.

`tools/gen_frames.py` removes each source's background (alpha for the gif, chroma-key
for the green screen), builds a union bounding box per source so Aris is never clipped
or jittery, then fills a Braille dot wherever the pixel is opaque and dark.

## Layout

```
src/main.rs        # tokio entry, clap dispatch, animation loop
src/cli.rs         # say / animate / freestyle ; Motion (teabag, x2, a, b)
src/frames.rs      # embedded frames + loader (yields every frame, verbatim lines)
src/display.rs     # speech bubble, RAII terminal guard, exit listener, render loop
frames/*.txt       # generated Braille frames (aris, motion_a, motion_b, static)
tools/gen_frames.py  # frame generator (sources go in refs/, not committed)
```

## Differences from momoisay

Same feel, three small fixes: frame lines are loaded **verbatim** (momoisay slices off
each line's last char), the iterator yields **every** frame (momoisay drops the last
one), and a RAII guard restores the terminal even if the program **panics**.

## Credits

Original animations of Aris teabagging by **청세치 / 세치혀**. This is a fan-made
terminal-art homage; all character/animation rights belong to the original creators and
Nexon (Blue Archive). Built as a companion to
[`momoisay`](https://crates.io/crates/momoisay).
