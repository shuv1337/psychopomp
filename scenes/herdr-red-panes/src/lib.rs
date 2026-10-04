//! Caption-only explanation of https://github.com/shuv1337/herdr/pull/11.
//! Explicit plan-clock beats; no narration manifest, media, or live Herdr access.
use anyhow::Result;
use psychopomp::{
    author::{PlanBuilder, seconds},
    caption::{CaptionActor, CaptionAlign, CaptionPlan},
    editor::{
        EditorInlineRevealPlan, EditorPartPlan, EditorRecipePlan, EditorSemanticRangePlan,
        LineMarkPlan,
        diff::{Diff, add, keep, remove},
    },
    highlight,
    math::easing::Ease,
    plan::{ContinuousChannelPlan, ReelPlan, ScenePlan, SpringPlan},
    sequence::{
        SequenceActor, SequenceParticipantPlan as Participant, SequencePlan, SequenceRowPlan as Row,
    },
    tone::Tone,
    value::ValueTokenPlan,
};
use psychopomp_pr_walkthrough::film::{self, Pr, TRANSITION, span};

pub const SOURCE: &str = "https://github.com/shuv1337/herdr/pull/11";
const FILE: &str = "herdr-tui-session.js";
// Herdr's semantic statuses: blocked is red, working yellow, done teal.
// Literal art colors are preserved by the OpenCode theme, unlike Tone::Success.
const RED: [u8; 3] = [224, 108, 117];
const YELLOW: [u8; 3] = [229, 192, 123];
const TEAL: [u8; 3] = [94, 203, 198];

const GHOST: Pr = Pr {
    number: "herdr",
    title: "1/3 · ghost permissions",
    slug: "ghost",
};
const FAILED: Pr = Pr {
    number: "herdr",
    title: "2/3 · failed turns",
    slug: "failed",
};
const FAMILY: Pr = Pr {
    number: "herdr",
    title: "3/3 · background subagents",
    slug: "family",
};

pub fn build_reel() -> Result<ReelPlan> {
    ReelPlan::dipped(
        "herdr-red-panes",
        vec![
            bookend(false)?,
            behavior(&GHOST, ghost_flow())?,
            ghost_code()?,
            behavior(&FAILED, failed_flow())?,
            failed_code()?,
            behavior(&FAMILY, family_flow())?,
            family_code()?,
            bookend(true)?,
        ],
        TRANSITION,
    )
}

fn bookend(outro: bool) -> Result<ScenePlan> {
    let mut scene = PlanBuilder::new(if outro { "outro" } else { "intro" }, seconds(6.0));
    film::header(
        &mut scene,
        &Pr {
            number: "herdr",
            title: "shuvcode pane status · integration v12 → v13",
            slug: "intro",
        },
        Some(seconds(0.25)),
    )?;
    let title = if outro {
        "red means a real request needs you."
    } else {
        "the turn ended. why is the pane red?"
    };
    let mut heading = CaptionActor::declare(
        &mut scene,
        "title",
        &CaptionPlan::line([960.0, 370.0], 48.0, vec![span(title, Tone::Plain)])
            .aligned(CaptionAlign::Center),
    )?;
    heading.show(&mut scene, seconds(0.4));
    for (index, (label, detail, accent)) in if outro {
        [
            ("blocked", "live permission or form", RED),
            ("working", "root or descendant running", YELLOW),
            ("done", "family settled · failure labeled", TEAL),
        ]
    } else {
        [
            ("ghost ask", "an interrupted permission", RED),
            ("failed turn", "a provider error", RED),
            ("early done", "children still working", YELLOW),
        ]
    }
    .into_iter()
    .enumerate()
    {
        let actor = scene.actor(
            format!("reason-{index}"),
            "value-token",
            ValueTokenPlan {
                label: label.into(),
                detail: detail.into(),
                center: [470.0 + index as f32 * 490.0, 570.0],
                size: [450.0, 122.0],
                font_size: 34.0,
                accent,
            },
        )?;
        let opacity = scene.continuous(&actor, "opacity", 0.0);
        // Value Token x/y channels are absolute centers, not offsets.
        let y = scene.continuous(&actor, "y", 586.0);
        let emphasis = scene.continuous(&actor, "emphasis", 1.0);
        scene.set(&emphasis, 0, 1.0);
        let at = seconds(0.8 + index as f64 * 0.12);
        scene.spring(&opacity, at, 1.0, 0.18, 0.0);
        scene.spring(&y, at, 570.0, 0.55, 0.12);
    }
    captions(
        &mut scene,
        &[Beat(
            1.5,
            if outro {
                "real asks still block; a finished turn no longer pretends to wait."
            } else {
                "Herdr paints what its pane-local plugin reports—not a screen guess."
            },
            Tone::Plain,
        )],
        5.7,
        film::FOOTER_Y,
    )?;
    let mut source = CaptionActor::declare(
        &mut scene,
        "source",
        &CaptionPlan::line([960.0, 875.0], 22.0, vec![span(SOURCE, Tone::Muted)])
            .aligned(CaptionAlign::Center),
    )?;
    source.show(&mut scene, seconds(1.2));
    Ok(scene.finish()?)
}

struct Beat(f64, &'static str, Tone);

/// One fully readable line per beat; the previous line is gone before the next
/// appears. Prose is sharp, not typed or smeared into competing silhouettes.
fn captions(scene: &mut PlanBuilder, beats: &[Beat], end: f64, y: f32) -> Result<()> {
    for (index, Beat(at, text, tone)) in beats.iter().enumerate() {
        let mut caption = CaptionActor::declare(
            scene,
            format!("caption-{index}"),
            &CaptionPlan::line([film::LEFT, y], 28.0, vec![span(text, *tone)]),
        )?;
        let opacity = caption.channel(scene, "opacity", 0.0);
        scene.ease(&opacity, seconds(*at), 1.0, 0.16, Ease::Smootherstep);
        let until = beats.get(index + 1).map_or(end, |beat| beat.0);
        scene.ease(
            &opacity,
            seconds(until - 0.18),
            0.0,
            0.16,
            Ease::Smootherstep,
        );
        scene.cue(format!("caption-{index}"), seconds(*at), seconds(until));
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Pane {
    Working,
    Blocked,
    Done,
    Failed,
}

impl Pane {
    fn label(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Blocked => "blocked",
            Self::Done => "done",
            Self::Failed => "done · failed",
        }
    }

    fn color(self) -> [u8; 3] {
        match self {
            Self::Working => YELLOW,
            Self::Blocked => RED,
            Self::Done | Self::Failed => TEAL,
        }
    }
}

fn pane_status(scene: &mut PlanBuilder, beats: &[(f64, Pane)], end: f64) -> Result<()> {
    for (index, &(at, status)) in beats.iter().enumerate() {
        let actor = scene.actor(
            format!("pane-{index}"),
            "value-token",
            ValueTokenPlan {
                label: status.label().into(),
                detail: String::new(),
                center: [1460.0, 912.0],
                size: [330.0, 74.0],
                font_size: 30.0,
                accent: status.color(),
            },
        )?;
        let emphasis = scene.continuous(&actor, "emphasis", 1.0);
        scene.set(&emphasis, 0, 1.0);
        let opacity = scene.continuous(&actor, "opacity", 0.0);
        scene.ease(&opacity, seconds(at), 1.0, 0.12, Ease::Smootherstep);
        let until = beats.get(index + 1).map_or(end, |beat| beat.0);
        scene.ease(
            &opacity,
            seconds(until - 0.14),
            0.0,
            0.12,
            Ease::Smootherstep,
        );
    }
    Ok(())
}

struct Flow {
    rows: Vec<Row>,
    slots: u32,
    before: Vec<(&'static str, f64)>,
    after: Vec<(&'static str, f64)>,
    captions: Vec<Beat>,
    statuses: Vec<(f64, Pane)>,
    rewind: f64,
    duration: f64,
    /// A temporary row whose slot is reused later in the fixed story.
    fade: Option<(&'static str, f64)>,
}

fn behavior(pr: &Pr, flow: Flow) -> Result<ScenePlan> {
    let mut scene = PlanBuilder::new(format!("{}-behavior", pr.slug), seconds(flow.duration));
    film::header(&mut scene, pr, Some(seconds(0.25)))?;
    let mut before = film::chip(&mut scene, "chip-before", Tone::Error, "before")?;
    before.show(&mut scene, seconds(0.35));
    before.hide(&mut scene, seconds(flow.rewind));
    let mut rewind = film::chip(&mut scene, "chip-rewind", Tone::Muted, "◀◀ rewind")?;
    rewind.show(&mut scene, seconds(flow.rewind + 0.25));
    rewind.hide(&mut scene, seconds(flow.rewind + 1.45));
    let mut fixed = film::chip(&mut scene, "chip-fixed", Tone::Success, "after the fix")?;
    fixed.show(&mut scene, seconds(flow.rewind + 1.8));
    let recipe = SequencePlan {
        origin: [210.0, 196.0],
        width: 1500.0,
        row_height: if flow.slots == 6 { 92.0 } else { 78.0 },
        slots: Some(flow.slots),
        participants: vec![
            Participant::new("tui", "shuvcode", "TUI / service events"),
            Participant::new("plugin", "Herdr plugin", "pane-local state machine"),
            Participant::new("pane", "Herdr pane", "reports → color"),
        ],
        rows: flow.rows,
    };
    let mut sequence = SequenceActor::declare(&mut scene, "flow", &recipe)?;
    sequence.animate(&mut scene, "opacity", 0.0, seconds(0.3), 1.0, 0.5);
    sequence.animate(&mut scene, "lifelines", 0.0, seconds(0.5), 1.0, 0.9);
    for &(row, at) in &flow.before {
        sequence.reveal(&mut scene, row, seconds(at));
    }
    // The actual rows retract last-to-first. Shared participants, their lifelines,
    // and their slots never disappear; this is a visible rewind, not a new diagram.
    for (index, &(row, _)) in flow.before.iter().rev().enumerate() {
        let reveal = sequence.row_channel(&mut scene, row, "reveal", 0.0);
        scene.ease(
            &reveal,
            seconds(flow.rewind + index as f64 * 0.07),
            0.0,
            1.25,
            Ease::CubicBezier([0.65, 0.0, 0.25, 1.0]),
        );
    }
    for &(row, at) in &flow.after {
        sequence.reveal(&mut scene, row, seconds(at));
    }
    if let Some((row, at)) = flow.fade {
        sequence.fade(&mut scene, row, seconds(at), 0.0);
    }
    captions(
        &mut scene,
        &flow.captions,
        flow.duration - 0.2,
        film::FOOTER_Y,
    )?;
    pane_status(&mut scene, &flow.statuses, flow.duration)?;
    scene.cue("before", 0, seconds(flow.rewind));
    scene.cue("rewind", seconds(flow.rewind), seconds(flow.rewind + 1.8));
    scene.cue("fixed", seconds(flow.rewind + 1.8), seconds(flow.duration));
    Ok(scene.finish()?)
}

fn report(id: &str, label: &str, correct: bool) -> Row {
    let tone = if correct { Tone::Success } else { Tone::Error };
    Row::message(id, "plugin", "pane", label, tone)
}

fn ghost_flow() -> Flow {
    Flow {
        slots: 6,
        rows: vec![
            Row::message("ask", "tui", "plugin", "permission.asked", Tone::Request).in_slot(0),
            report("block", "blocked", true).in_slot(1),
            Row::message(
                "terminal",
                "tui",
                "plugin",
                "execution.interrupted",
                Tone::Request,
            )
            .in_slot(2),
            Row::note("stale", &["plugin"], "cached ask survives", Tone::Muted).in_slot(3),
            report("stuck", "blocked, again", false).in_slot(4),
            Row::note("ended", &["tui"], "execution ended", Tone::Muted).in_slot(5),
            Row::note(
                "expire",
                &["plugin"],
                "expire this member's asks",
                Tone::Muted,
            )
            .in_slot(3),
            Row::reply(
                "sync",
                "plugin",
                "tui",
                "invalidate + sync permissions",
                Tone::Request,
            )
            .in_slot(4),
            report("idle", "idle", true).in_slot(5),
        ],
        before: vec![
            ("ask", 1.2),
            ("block", 2.1),
            ("terminal", 3.8),
            ("stale", 5.2),
            ("stuck", 6.2),
            ("ended", 6.9),
        ],
        after: vec![
            ("ask", 10.2),
            ("block", 11.1),
            ("terminal", 12.6),
            ("expire", 13.5),
            ("sync", 14.8),
            ("idle", 16.0),
        ],
        captions: vec![
            Beat(
                0.8,
                "a permission ask correctly blocks the pane.",
                Tone::Plain,
            ),
            Beat(
                3.5,
                "interrupt ends the run; no permission.replied follows.",
                Tone::Plain,
            ),
            Beat(
                6.2,
                "the stale cache keeps a blocker that no longer exists.",
                Tone::Error,
            ),
            Beat(8.2, "same execution. replay with the fix.", Tone::Muted),
            Beat(
                10.2,
                "the live ask still blocks—until its execution ends.",
                Tone::Plain,
            ),
            Beat(
                13.2,
                "terminal events expire that member's permission blockers.",
                Tone::Plain,
            ),
            Beat(
                14.8,
                "invalidate and sync refresh the stale permission cache.",
                Tone::Plain,
            ),
            Beat(
                16.7,
                "the dead prompt is gone; the pane settles to done.",
                Tone::Plain,
            ),
        ],
        statuses: vec![
            (0.8, Pane::Working),
            (2.7, Pane::Blocked),
            (9.9, Pane::Working),
            (11.7, Pane::Blocked),
            (16.7, Pane::Done),
        ],
        rewind: 8.2,
        duration: 20.0,
        fade: None,
    }
}

fn failed_flow() -> Flow {
    Flow {
        slots: 7,
        rows: vec![
            Row::message("start", "tui", "plugin", "execution.started", Tone::Request).in_slot(0),
            report("work", "working", true).in_slot(1),
            Row::message(
                "error",
                "tui",
                "plugin",
                "execution.failed · 429",
                Tone::Request,
            )
            .in_slot(2),
            Row::note("old-map", &["plugin"], "failed → blocked", Tone::Muted).in_slot(3),
            report("blocked", "blocked", false).in_slot(4),
            Row::note(
                "new-map",
                &["plugin"],
                "failed → idle + metadata",
                Tone::Muted,
            )
            .in_slot(3),
            report("failed-idle", "idle · label: failed", true).in_slot(4),
            Row::message(
                "next",
                "tui",
                "plugin",
                "next root execution.started",
                Tone::Request,
            )
            .in_slot(5),
            report("clear", "working · clear failed label", true).in_slot(6),
        ],
        before: vec![
            ("start", 1.1),
            ("work", 2.1),
            ("error", 3.8),
            ("old-map", 4.8),
            ("blocked", 6.0),
        ],
        after: vec![
            ("start", 10.2),
            ("work", 11.2),
            ("error", 12.7),
            ("new-map", 13.7),
            ("failed-idle", 14.9),
            ("next", 18.5),
            ("clear", 19.5),
        ],
        captions: vec![
            Beat(
                0.8,
                "provider errors end turns: rate limits, auth failures, 499s.",
                Tone::Plain,
            ),
            Beat(
                3.8,
                "the old plugin maps that finished turn to blocked.",
                Tone::Plain,
            ),
            Beat(6.1, "an error is not an unanswered request.", Tone::Error),
            Beat(8.2, "same error. replay with the fix.", Tone::Muted),
            Beat(
                10.2,
                "failure becomes a result, not a permanent blocker.",
                Tone::Plain,
            ),
            Beat(
                15.6,
                "done stays teal; a failed label preserves the outcome.",
                Tone::Plain,
            ),
            Beat(18.5, "the next root turn clears that label.", Tone::Plain),
        ],
        statuses: vec![
            (0.8, Pane::Working),
            (6.7, Pane::Blocked),
            (9.9, Pane::Working),
            (15.6, Pane::Failed),
            (20.2, Pane::Working),
        ],
        rewind: 8.2,
        duration: 22.0,
        fade: None,
    }
}

fn family_flow() -> Flow {
    Flow {
        slots: 7,
        rows: vec![
            Row::message(
                "root",
                "tui",
                "plugin",
                "root execution.started",
                Tone::Request,
            )
            .in_slot(0),
            Row::message(
                "child",
                "tui",
                "plugin",
                "child execution.started",
                Tone::Request,
            )
            .in_slot(1),
            Row::message(
                "root-done",
                "tui",
                "plugin",
                "root execution.succeeded",
                Tone::Request,
            )
            .in_slot(2),
            Row::note(
                "ignored",
                &["plugin"],
                "child execution ignored",
                Tone::Muted,
            )
            .in_slot(3),
            report("early", "idle", false).in_slot(4),
            Row::message(
                "wake",
                "tui",
                "plugin",
                "child done → parent wakes",
                Tone::Request,
            )
            .in_slot(5),
            report("again", "idle, again", false).in_slot(6),
            Row::note(
                "family",
                &["plugin"],
                "running descendant → working",
                Tone::Muted,
            )
            .in_slot(3),
            report("working", "working", true).in_slot(4),
            Row::message(
                "last",
                "tui",
                "plugin",
                "last child completes",
                Tone::Request,
            )
            .in_slot(5),
            Row::note(
                "delay",
                &["plugin", "pane"],
                "idle deadline: +1.5 s",
                Tone::Muted,
            )
            .in_slot(6),
            report("idle", "idle · once settled", true).in_slot(6),
        ],
        before: vec![
            ("root", 1.0),
            ("child", 2.4),
            ("root-done", 3.8),
            ("ignored", 4.7),
            ("early", 5.8),
            ("wake", 7.0),
            ("again", 8.1),
        ],
        after: vec![
            ("root", 12.0),
            ("child", 13.1),
            ("root-done", 14.3),
            ("family", 15.2),
            ("working", 16.3),
            ("last", 17.5),
            ("delay", 18.2),
            ("idle", 19.9),
        ],
        captions: vec![
            Beat(
                0.8,
                "background children outlive their parent's turn.",
                Tone::Plain,
            ),
            Beat(
                3.8,
                "the old plugin tracks the root, not its children.",
                Tone::Plain,
            ),
            Beat(
                6.7,
                "child wakeups can trigger repeated done pings.",
                Tone::Plain,
            ),
            Beat(10.0, "same family. replay with the fix.", Tone::Muted),
            Beat(
                12.0,
                "working now covers the root and all running descendants.",
                Tone::Plain,
            ),
            Beat(
                15.0,
                "a root completion no longer hides background work.",
                Tone::Plain,
            ),
            Beat(
                18.2,
                "wait 1.5 s: absorb the child → parent wake gap.",
                Tone::Plain,
            ),
            Beat(
                20.5,
                "one done transition, after the whole family settles.",
                Tone::Plain,
            ),
        ],
        statuses: vec![
            (0.8, Pane::Working),
            (6.5, Pane::Done),
            (7.5, Pane::Working),
            (8.8, Pane::Done),
            (11.7, Pane::Working),
            (20.6, Pane::Done),
        ],
        rewind: 10.0,
        duration: 24.0,
        fade: Some(("delay", 19.3)),
    }
}

fn code_scene(
    pr: &Pr,
    diff: &Diff,
    times: &[f64],
    duration: f64,
    beats: &[Beat],
    note: &str,
) -> Result<ScenePlan> {
    let mut scene = PlanBuilder::new(format!("{}-code", pr.slug), seconds(duration));
    film::code_on_clock(
        &mut scene,
        pr,
        diff,
        &times.iter().copied().map(seconds).collect::<Vec<_>>(),
        note,
        true,
    )?;
    captions(&mut scene, beats, duration - 0.2, 960.0)?;
    // Held steps also expose the diff's identity and placement to `plan steps`.
    scene.presentation_step("before", "Before", seconds(0.8), seconds(1.8));
    for (index, &at) in times.iter().enumerate() {
        scene.presentation_step(
            format!("edit-{index}"),
            "Change",
            seconds(at),
            seconds(at + 2.0),
        );
    }
    Ok(scene.finish()?)
}

fn ghost_code() -> Result<ScenePlan> {
    code_scene(
        &GHOST,
        &Diff {
            file_name: FILE,
            lines: vec![
                add(1, "function settleMember(member) {"),
                add(1, "  for (const [key, owner] of blockers) {"),
                add(
                    1,
                    "    if (owner === member && key.startsWith(\"permission:\"))",
                ),
                add(1, "      expired.set(key, owner);"),
                add(1, "  }"),
                add(2, "  const permission = api.data.session.permission;"),
                add(2, "  if (typeof permission.invalidate === \"function\")"),
                add(2, "    permission.invalidate(member);"),
                add(2, "  if (typeof permission.sync === \"function\")"),
                add(2, "    void Promise.resolve(permission.sync(member));"),
                add(1, "}"),
            ],
        },
        &[3.0, 6.0],
        10.0,
        &[
            Beat(0.8, "terminal events call settleMember(id).", Tone::Plain),
            Beat(
                2.6,
                "expired keys cannot be revived by the stale cache.",
                Tone::Plain,
            ),
            Beat(
                5.8,
                "invalidate and sync also remove the TUI's ghost prompt.",
                Tone::Plain,
            ),
        ],
        "condensed excerpt · delta cleanup, try/catch and sync continuation omitted",
    )
}

fn failed_code() -> Result<ScenePlan> {
    let mut plan = code_scene(
        &FAILED,
        &Diff {
            file_name: FILE,
            lines: vec![
                keep("case \"session.execution.failed\":"),
                remove(1, "  if (id !== selected) return;"),
                add(1, "  executionChanges.set(id, false);"),
                add(1, "  settleMember(id);"),
                add(1, "  if (id === selected) {"),
                keep("  state = \"blocked\";"),
                add(2, "    if (event.type === \"session.execution.failed\") {"),
                add(2, "      failed = true;"),
                add(2, "      enqueue(undefined, true);"),
                add(2, "    }"),
                add(1, "  }"),
                keep("  break;"),
            ],
        },
        &[3.0, 6.0],
        10.0,
        &[
            Beat(
                0.8,
                "the old failure branch left state = blocked.",
                Tone::Plain,
            ),
            Beat(
                2.6,
                "settle the execution and change only the state value.",
                Tone::Plain,
            ),
            Beat(
                5.8,
                "send failure metadata; the next root start clears it.",
                Tone::Plain,
            ),
        ],
        "excerpt · succeeded and interrupted share this terminal branch",
    )?;
    // Preserve `state = "` and `";` as the value changes and its scope indents.
    // The template's ordinary Line Marks still animate the surrounding real diff.
    let actor = plan
        .actors
        .iter_mut()
        .find(|actor| actor.id == "editor")
        .unwrap();
    let mut recipe: EditorRecipePlan = serde_json::from_value(actor.data.clone())?;
    let line = recipe
        .lines
        .iter_mut()
        .find(|line| line.id == "line-5")
        .unwrap();
    line.parts = [
        ("indent", "  "),
        ("scope-indent", "  "),
        ("prefix", "state = \""),
        ("blocked", "blocked"),
        ("idle", "idle"),
        ("suffix", "\";"),
    ]
    .into_iter()
    .map(|(id, text)| EditorPartPlan {
        id: id.into(),
        spans: highlight::typescript(text),
    })
    .collect();
    line.mark = Some(LineMarkPlan::Added);
    for (range, part, reversed) in [
        ("scope", "scope-indent", false),
        ("old", "blocked", true),
        ("new", "idle", false),
    ] {
        line.semantic_ranges.push(EditorSemanticRangePlan {
            id: range.into(),
            first_part_id: part.into(),
            last_part_id: part.into(),
        });
        recipe
            .additional_inline_reveals
            .push(EditorInlineRevealPlan {
                line_id: "line-5".into(),
                range_id: range.into(),
                channel: Some("state-edit".into()),
                reversed,
            });
    }
    actor.data = serde_json::to_value(recipe)?;
    for property in ["state-edit", "mark.line-5"] {
        plan.continuous_channels.push(ContinuousChannelPlan {
            id: format!("editor.{property}"),
            actor_id: "editor".into(),
            property: property.into(),
            initial: 0.0.into(),
            events: vec![SpringPlan::visual(0.35, 0.0).event(seconds(3.0), 1.0)],
        });
    }
    plan.validate()?;
    Ok(plan)
}

fn family_code() -> Result<ScenePlan> {
    code_scene(
        &FAMILY,
        &Diff {
            file_name: FILE,
            lines: vec![
                add(2, "const IDLE_DELAY_MS = 1_500;"),
                add(1, "function effective() {"),
                add(1, "  if (blockers.size) return \"blocked\";"),
                add(
                    1,
                    "  return state === \"working\" || familyActive() ? \"working\" : \"idle\";",
                ),
                add(1, "}"),
                keep(""),
                add(2, "if (raw === \"idle\" && published === \"working\") {"),
                add(2, "  idleAt ??= Date.now() + IDLE_DELAY_MS;"),
                add(2, "  if (Date.now() < idleAt) value = \"working\";"),
                add(2, "} else {"),
                add(2, "  idleAt = undefined;"),
                add(2, "}"),
            ],
        },
        &[3.0, 6.0],
        12.0,
        &[
            Beat(
                0.8,
                "familyActive() checks running descendants, not just the root.",
                Tone::Plain,
            ),
            Beat(
                2.6,
                "real blockers take priority; family activity keeps working.",
                Tone::Plain,
            ),
            Beat(
                5.8,
                "only working → idle waits 1,500 ms; renewed work resets it.",
                Tone::Plain,
            ),
        ],
        "condensed excerpts · familyActive() and publish() context omitted",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use psychopomp::{
        editor::inspect_steps,
        plan::{ScalarPlan, compile_channels},
        timeline::{PropertyId, Timeline},
    };

    fn timeline(plan: &ScenePlan) -> Timeline {
        compile_channels(
            plan.continuous_channels
                .iter()
                .map(|channel| (channel, PropertyId::new(&channel.id))),
            plan.duration_nanos,
            |value| match value {
                ScalarPlan::Literal(value) => Ok(*value),
                _ => anyhow::bail!("this film has no renderer-resolved scalars"),
            },
        )
        .unwrap()
    }

    fn sample(timeline: &Timeline, property: &str, at: f64, default: f32) -> f32 {
        timeline
            .sample_at(&PropertyId::new(property), at)
            .map_or(default, |s| s.position)
    }

    #[test]
    fn reel_is_caption_only_valid_and_reproducible() {
        let reel = build_reel().unwrap();
        reel.validate().unwrap();
        assert_eq!(reel.segments.len(), 8);
        assert_eq!(reel.duration_nanos(), seconds(105.1));
        assert_eq!(
            serde_json::to_value(&reel).unwrap(),
            serde_json::to_value(build_reel().unwrap()).unwrap()
        );
        let round_trip: ReelPlan =
            serde_json::from_value(serde_json::to_value(&reel).unwrap()).unwrap();
        round_trip.validate().unwrap();
        for segment in round_trip.segments {
            assert!(
                segment.plan.media.is_empty(),
                "no real or fake audio assets"
            );
            assert!(segment.plan.state_channels.is_empty());
            for actor in &segment.plan.actors {
                match actor.recipe.as_str() {
                    "sequence" => serde_json::from_value::<SequencePlan>(actor.data.clone())
                        .unwrap()
                        .validate()
                        .unwrap(),
                    "caption" => serde_json::from_value::<CaptionPlan>(actor.data.clone())
                        .unwrap()
                        .validate()
                        .unwrap(),
                    "value-token" => serde_json::from_value::<ValueTokenPlan>(actor.data.clone())
                        .unwrap()
                        .validate()
                        .unwrap(),
                    "editor" => {
                        serde_json::from_value::<EditorRecipePlan>(actor.data.clone())
                            .unwrap()
                            .compile()
                            .unwrap();
                    }
                    _ => panic!("unexpected recipe {}", actor.recipe),
                }
            }
        }
    }

    #[test]
    fn rewind_retracts_shared_rows_then_replays_in_place() {
        for (pr, flow) in [
            (&GHOST, ghost_flow()),
            (&FAILED, failed_flow()),
            (&FAMILY, family_flow()),
        ] {
            let shared = flow.before[0].0;
            let rewind = flow.rewind;
            let fixed = flow.after[0].1;
            let plan = behavior(pr, flow).unwrap();
            let timeline = timeline(&plan);
            let key = format!("flow.row.{shared}.reveal");
            assert!(sample(&timeline, &key, rewind - 0.1, 0.0) > 0.99);
            assert_eq!(sample(&timeline, &key, fixed - 0.01, 0.0), 0.0);
            assert!(sample(&timeline, &key, fixed + 1.0, 0.0) > 0.99);
            // Sample backward and out of order: no previous frame owns the pose.
            let mid = sample(&timeline, &key, rewind + 0.6, 0.0);
            sample(&timeline, &key, fixed + 1.0, 0.0);
            assert_eq!(sample(&timeline, &key, rewind + 0.6, 0.0), mid);
            assert_eq!(sample(&timeline, "flow.lifelines", fixed, 0.0), 1.0);
        }
    }

    #[test]
    fn captions_statuses_and_shared_slots_do_not_compete() {
        for segment in build_reel().unwrap().segments {
            let plan = segment.plan;
            let timeline = timeline(&plan);
            let sequence = plan
                .actors
                .iter()
                .find(|actor| actor.recipe == "sequence")
                .map(|actor| serde_json::from_value::<SequencePlan>(actor.data.clone()).unwrap());
            for tick in 0..(plan.duration_nanos / 20_000_000) {
                let at = tick as f64 * 0.02;
                for prefix in ["caption-", "pane-"] {
                    let visible = plan
                        .actors
                        .iter()
                        .filter(|actor| actor.id.starts_with(prefix))
                        .filter(|actor| {
                            sample(&timeline, &format!("{}.opacity", actor.id), at, 0.0) > 0.01
                        })
                        .count();
                    assert!(visible <= 1, "{} at {at}: {prefix} overlap", plan.id);
                }
                if let Some(sequence) = &sequence {
                    let mut slots = std::collections::HashSet::new();
                    for (row, slot) in sequence.rows.iter().zip(sequence.row_slots()) {
                        let alpha =
                            sample(&timeline, &format!("flow.row.{}.reveal", row.id()), at, 0.0)
                                * sample(
                                    &timeline,
                                    &format!("flow.row.{}.opacity", row.id()),
                                    at,
                                    1.0,
                                );
                        assert!(
                            alpha <= 0.02 || slots.insert(slot),
                            "{} at {at}: slot {slot} overlap",
                            plan.id
                        );
                    }
                }
            }
        }
    }

    fn status_at(plan: &ScenePlan, at: f64) -> ValueTokenPlan {
        let timeline = timeline(plan);
        let actor = plan
            .actors
            .iter()
            .find(|actor| {
                actor.id.starts_with("pane-")
                    && sample(&timeline, &format!("{}.opacity", actor.id), at, 0.0) > 0.99
            })
            .unwrap();
        serde_json::from_value(actor.data.clone()).unwrap()
    }

    #[test]
    fn pane_outcomes_match_the_three_stories_and_real_palette() {
        let ghost = behavior(&GHOST, ghost_flow()).unwrap();
        assert_eq!(status_at(&ghost, 7.5).accent, RED);
        assert_eq!(status_at(&ghost, 18.0).label, "done");
        assert_eq!(status_at(&ghost, 18.0).accent, TEAL);
        let failed = behavior(&FAILED, failed_flow()).unwrap();
        assert_eq!(status_at(&failed, 16.5).label, "done · failed");
        assert_eq!(status_at(&failed, 21.0).label, "working");
        let family = behavior(&FAMILY, family_flow()).unwrap();
        assert_eq!(status_at(&family, 6.8).label, "done");
        assert_eq!(status_at(&family, 16.8).label, "working");
        assert_eq!(status_at(&family, 19.5).accent, YELLOW);
        assert_eq!(status_at(&family, 22.0).accent, TEAL);
        let track = timeline(&family);
        assert_eq!(sample(&track, "flow.row.idle.reveal", 19.69, 0.0), 0.0);
    }

    #[test]
    fn arrow_tone_marks_report_correctness_not_pane_state() {
        let faulty = ["stuck", "blocked", "early", "again"];
        for (pr, flow) in [
            (&GHOST, ghost_flow()),
            (&FAILED, failed_flow()),
            (&FAMILY, family_flow()),
        ] {
            let plan = behavior(pr, flow).unwrap();
            let actor = plan
                .actors
                .iter()
                .find(|actor| actor.recipe == "sequence")
                .unwrap();
            let sequence: SequencePlan = serde_json::from_value(actor.data.clone()).unwrap();
            for row in &sequence.rows {
                let reports = matches!(row, Row::Message { to, .. } if to == "pane");
                let expected = match (reports, faulty.contains(&row.id())) {
                    (true, true) => vec![Tone::Error],
                    (true, false) => vec![Tone::Success],
                    (false, _) => vec![Tone::Request, Tone::Muted],
                };
                assert!(
                    expected.contains(&row.tone()),
                    "{} row {}: {:?}",
                    plan.id,
                    row.id(),
                    row.tone()
                );
            }
        }
    }

    #[test]
    fn state_value_edit_retains_common_parts_and_all_code_holds_settle() {
        for plan in [
            ghost_code().unwrap(),
            failed_code().unwrap(),
            family_code().unwrap(),
        ] {
            let steps = serde_json::to_value(inspect_steps(&plan).unwrap()).unwrap();
            assert!(
                steps["warnings"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|warning| warning["code"] != "unsettled-code-step")
            );
            if plan.id == "failed-code" {
                let state = steps["steps"][1]["editors"][0]["lines"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|line| line["id"] == "line-5")
                    .unwrap();
                assert_eq!(state["before"], "  state = \"blocked\";");
                assert_eq!(state["after"], "    state = \"idle\";");
                assert_eq!(state["beforeDelta"], "  state = \"«blocked»\";");
                assert_eq!(
                    state["changedPartIds"],
                    serde_json::json!(["scope-indent", "blocked", "idle"])
                );
            }
        }
    }

    #[test]
    fn bookend_tiles_enter_at_their_authored_centers() {
        let plan = bookend(false).unwrap();
        let track = timeline(&plan);
        for index in 0..3 {
            assert_eq!(
                sample(&track, &format!("reason-{index}.y"), 3.5, 0.0),
                570.0
            );
        }
    }
}
