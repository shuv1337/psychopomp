use std::{env, fs, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let output = PathBuf::from(
        env::args()
            .nth(1)
            .unwrap_or_else(|| "target/herdr-red-panes/reel.json".into()),
    );
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let reel = psychopomp_herdr_red_panes::build_reel()?;
    let directory = output.parent().unwrap_or_else(|| std::path::Path::new("."));
    for segment in &reel.segments {
        fs::write(
            directory.join(format!("{}.json", segment.plan.id)),
            serde_json::to_string_pretty(&segment.plan)? + "\n",
        )?;
    }
    fs::write(&output, serde_json::to_string_pretty(&reel)? + "\n")?;
    eprintln!(
        "wrote {} ({:.1}s, {} segments, no audio)",
        output.display(),
        reel.duration_nanos() as f64 / 1e9,
        reel.segments.len()
    );
    Ok(())
}
