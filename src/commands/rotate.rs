use std::path::Path;

use chrono::Local;

use hdpassw::error::Result;
use hdpassw::store;

pub fn run(db_path: &Path) -> Result<()> {
    let mut st = store::load(db_path)?;
    let old = st.rotation;
    let new = old + 1;

    st.rotation = new;
    st.rotation_since = Local::now().date_naive();
    store::save(db_path, &st)?;

    eprintln!();
    eprintln!("  Global rotation: /{old} → /{new}");
    eprintln!();

    let stale: Vec<_> = st
        .sites
        .iter()
        .filter(|(_, r)| r.counter < new)
        .collect();

    if stale.is_empty() {
        eprintln!("  All sites are already at /{new}.  Nothing to do.");
    } else {
        eprintln!("  Sites that need regenerating ({}):", stale.len());
        eprintln!();
        let w = stale.iter().map(|(n, _)| n.len()).max().unwrap_or(4);
        for (name, r) in &stale {
            eprintln!("    {:<w$}  /{} → /{new}", name, r.counter);
        }
        eprintln!();
        eprintln!("  Run  hdpassw gen <site>  for each one to get the new");
        eprintln!("  password and update your account credentials.");
    }

    eprintln!();
    Ok(())
}
