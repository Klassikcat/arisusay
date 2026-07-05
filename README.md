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

| #   | name      | what it is                                                        |
| --- | --------- | ----------------------------------------------------------------- |
| 1   | `teabag`  | the teabag loop (transparent source), default speed               |
| 2   | `x2`      | same loop, twice as fast                                          |
| 3   | `bagging` | mid-teabag bob from the clip — the ex-`dance` (alias: `a`)        |
| 4   | `handsup` | hands-up teabag bob from the clip (alias: `b`)                    |
| 5   | `dance`   | fist-pump dance bob from the clip's opening (alias: `c`)          |

## Install

```sh
cargo install arisusay
```

For local development from a checkout:

```sh
cargo build --release          # → target/release/arisusay
cargo install --path .         # → ~/.cargo/bin/arisusay
```

## Usage

```sh
arisusay say "bam ba ca bam!"       # static Aris + speech bubble (prints once)

arisusay animate               # teabag, default speed
arisusay animate x2            # twice as fast        (same as: animate 2)
arisusay animate bagging       # mid-teabag bob         (same as: animate 3 / a)
arisusay animate handsup       # hands-up teabag bob    (same as: animate 4 / b)
arisusay animate dance         # opening fist-pump bob  (same as: animate 5 / c)
arisusay animate dance -t "ㅋㅋㅋ" # animation + speech bubble

arisusay freestyle             # random variant each loop
arisusay freestyle -t "ㅋㅋㅋ"  # ...with a speech bubble
```

**Controls:** `q`, `Esc`, or `Ctrl-C` to quit. The animation runs in the alternate
screen and restores your terminal on exit (even on panic).

**Terminal size:** without text, the shared canvas needs at least 34×23 columns/rows
(34×22 art plus the footer). `-t`/`--text` adds a speech bubble to the right. Too
small → it prints a friendly message and exits.

## How the frames are made

Frames are pre-rendered monochrome Braille baked into the binary via `include_str!` —
no image files or codecs at runtime, exactly like `momoisay`. They live in
`frames/*.txt` (`x2` reuses `aris.txt` at a shorter frame interval; individual frames
are joined by a form-feed).

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
yt-dlp --download-sections "*00:00:01-00:00:06" -f "bv*[height<=720]/b" \
       -o "refs/yt_src2.%(ext)s" "https://www.youtube.com/watch?v=T9F1Wk8DQdg"
yt-dlp --download-sections "*00:01:14-00:01:30" -f "bv*[height<=720]/b" \
       -o "refs/yt_src3.%(ext)s" "https://www.youtube.com/watch?v=T9F1Wk8DQdg"
```

- `aris2.gif` — `teabag` / `x2`
- `yt_src.webm` — `bagging` is cut from it (8.5–13 s: the single-Aris
  mid-teabag bob, after the clone intro ends)
- `yt_src2.mp4` — `dance` is all of it (the opening fist-pump bob that
  turns around halfway, before any clones show up)
- `yt_src3.mp4` — `handsup` is cut from it (5.75–11.3 s: the hands-up teabag bob,
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
src/cli.rs         # say / animate / freestyle ; Motion (teabag, x2, bagging, handsup, dance)
src/frames.rs      # embedded frames + loader (yields every frame, verbatim lines)
src/display.rs     # speech bubble, RAII terminal guard, exit listener, render loop
frames/*.txt       # generated Braille frames (aris, motion_a..motion_c, static)
tools/gen_frames.py  # frame generator (sources go in refs/, not committed)
```

## Differences from momoisay

Same feel, three small fixes: frame lines are loaded **verbatim** (momoisay slices off
each line's last char), the iterator yields **every** frame (momoisay drops the last
one), and a RAII guard restores the terminal even if the program **panics**.

## License

See `LICENSE`.

Short version: project code is available under MIT terms. Bundled/generated Aris
frame data is covered by a separate fan-art asset notice: it may be used with this
project, but no rights are granted to Blue Archive, Tendou Aris, or the original
animation sources.

## Credits

Original animations of Aris teabagging by **청세치 / 세치혀**. This is a fan-made
terminal-art homage; all character/animation rights belong to the original creators and
Nexon (Blue Archive). Built as a companion to
[`momoisay`](https://crates.io/crates/momoisay).
