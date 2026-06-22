use std::path::Path;

use chrono::Local;

use hdpassw::error::Result;
use hdpassw::store;

use crate::cli::LsArgs;

pub fn run(args: LsArgs, db_path: &Path) -> Result<()> {
    let st = store::load(db_path)?;

    let sites: Vec<_> = st
        .sites
        .iter()
        .filter(|(name, _)| {
            args.filter
                .as_deref()
                .map(|f| name.contains(f))
                .unwrap_or(true)
        })
        .collect();

    if sites.is_empty() {
        eprintln!("No sites found.");
        return Ok(());
    }

    if args.json {
        let entries: Vec<_> = sites
            .iter()
            .map(|(name, r)| {
                format!(
                    r#"{{"site":{},"user":{},"counter":{},"length":{},"charset":{},"verifier":{},"modified":{}}}"#,
                    serde_json::to_string(name).unwrap_or_default(),
                    serde_json::to_string(&r.user).unwrap_or_default(),
                    r.counter,
                    r.length,
                    serde_json::to_string(&r.charset).unwrap_or_default(),
                    serde_json::to_string(&r.verifier).unwrap_or_default(),
                    serde_json::to_string(&r.modified.to_string()).unwrap_or_default(),
                )
            })
            .collect();
        println!("[{}]", entries.join(","));
        return Ok(());
    }

    let rotation = st.rotation;
    let days = (Local::now().date_naive() - st.rotation_since).num_days();
    let stale_count = sites.iter().filter(|(_, r)| r.counter < rotation).count();

    // Header line
    eprint!("  Global rotation: /{rotation}  ·  since {}", st.rotation_since);
    if days >= 90 {
        eprintln!("  ⚠  {days} days — run: hdpassw rotate");
    } else {
        eprintln!("  ({days} days)");
    }
    if stale_count > 0 {
        eprintln!(
            "  {stale_count} site(s) behind — run hdpassw bump <site> to catch up"
        );
    }
    eprintln!();

    // Width: site name + optional /N suffix + stale flag
    let name_w = sites
        .iter()
        .map(|(n, r)| if r.counter > 1 { n.len() + 1 + r.counter.to_string().len() } else { n.len() })
        .max()
        .unwrap_or(4)
        .max(4);

    println!(
        "{:<name_w$}    {:>4}  {:<12}  {}",
        "SITE", "LEN", "CHARSET", "MODIFIED"
    );
    println!("{}", "─".repeat(name_w + 34));

    for (name, r) in &sites {
        // Show /N suffix only when not at the default /1
        let display = if r.counter > 1 {
            format!("{}/{}", name, r.counter)
        } else {
            name.to_string()
        };
        let stale = r.counter < rotation;
        let advisory = if stale {
            format!("  ⚠ bump to /{rotation}")
        } else {
            String::new()
        };
        println!(
            "{:<name_w$}    {:>4}  {:<12}  {}{advisory}",
            display, r.length, r.charset, r.modified
        );
    }

    Ok(())
}
