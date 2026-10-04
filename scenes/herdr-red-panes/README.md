# Herdr red panes

A **105.1-second, caption-only, silent** explanation of
https://github.com/shuv1337/herdr/pull/11 (integration v12 → v13).
No narration manifest, TTS, audio track, or live Herdr session is used.

Each chapter plays the broken report, retracts the rows visibly in reverse order,
replays the same event in the same participant slots, then animates real diff
excerpts with Line Marks:

1. **Ghost permission:** an interrupted ask receives no reply. Expire the owner's
   permission keys on terminal execution events and refresh the permission cache.
2. **Failed turn:** a provider error ends a turn; it is not a pending request.
   Report idle/Done with a `failed` label until the next root turn starts.
3. **Background subagents:** Working covers running descendants. A 1,500 ms
   Working → idle delay absorbs short completion/wake gaps instead of sending
   repeated Done transitions.

The pane badge follows Herdr's semantics: red Blocked, yellow Working, teal Done.
Plugin → pane report arrows are colored by correctness, not by the reported
state: green for a correct report (including correct reports in the broken
replay), red for a faulty one such as the premature idle pings. Execution and
permission events use the request tone; internal-state notes stay muted.
Live permission requests and forms still block. The scene does not claim to fix
the separate shuvcode child-view autoaccept bug or establish live bug frequency.

## Generate and render

```sh
cargo run -p psychopomp-herdr-red-panes
cargo run --release -- plan validate target/herdr-red-panes/reel.json
cargo run --release -- plan inspect target/herdr-red-panes/reel.json
cargo run --release -- plan render target/herdr-red-panes/reel.json output/herdr-red-panes-study.mp4 --range 13..18 --theme opencode
cargo run --release -- plan render target/herdr-red-panes/reel.json output/herdr-red-panes.mp4 --theme opencode
ffprobe -v error -show_entries stream=codec_type,width,height,r_frame_rate,nb_frames:format=duration -of json output/herdr-red-panes.mp4
```

The Scene Program writes the Reel and one Scene Plan per segment beside it.
An optional first argument overrides the generated Reel path.
All plans and review media belong under ignored `target/` or `output/` directories.
The delivered export may instead be written to `~/Videos/psychopomp/herdr-red-panes.mp4`.

## Authoring and verification

`src/lib.rs` owns every beat on the plan clock. It uses the shared
`psychopomp_pr_walkthrough::film` chrome and no-narration `code_on_clock` helper,
ordinary Sequence and Caption channels, and Value Tokens for pane status.
Messages land before downstream reactions; receivers do not scale on contact.
Participants and lifelines stay still through rewind. Captions fade sharply in
160 ms, with only one readable line in their slot.

Diffs are condensed excerpts of `herdr-tui-session.js`, not standalone executable
patches. The ghost helper omits delta cleanup, its try/catch, and asynchronous sync
continuation; the family code joins disjoint excerpts. These omissions are labeled
on screen. The failure edit retains `state = "`, `";`, and line identity while
only its value and scope indentation change. The inspector's common-text warnings
for the removed early-return guard versus new calls/root scope are different
semantic roles, not a reason to merge those lines.

Tests execute the Scene Program, validate emitted recipes, sample their actual
tracks out of order, check caption/row exclusivity every 20 ms, verify the palette
and outcomes, and inspect the emitted code delta and settled holds.

Inspect individual code steps with, for example,
`psychopomp plan steps target/herdr-red-panes/failed-code.json`.
Reel spans come from `plan inspect`; short `--range` studies preserve that clock.
