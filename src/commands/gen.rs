use std::path::Path;

use chrono::Local;

use hdpassw::crypto::encode::Charset;
use hdpassw::error::Result;
use hdpassw::session::Session;
use hdpassw::store::{self, site::SiteRecord};
use hdpassw::{clipboard, manager};

use crate::cli::GenArgs;

/// Parse an optional `/N` generation suffix from the site argument.
/// `"github.com/2"` → `("github.com", Some(2))`
/// `"github.com"`   → `("github.com", None)`
fn split_site_gen(raw: &str) -> (String, Option<u32>) {
    if let Some(pos) = raw.rfind('/') {
        if let Ok(n) = raw[pos + 1..].parse::<u32>() {
            if n > 0 {
                return (raw[..pos].to_string(), Some(n));
            }
        }
    }
    (raw.to_string(), None)
}

pub fn run(args: GenArgs, db_path: &Path, session: &Session) -> Result<()> {
    let mut st = store::load(db_path)?;
    let rotation = st.rotation;

    // Warn if 90+ days without a rotation
    let days = (Local::now().date_naive() - st.rotation_since).num_days();
    if days >= 90 {
        eprintln!(
            "  ⚠  {days} days since last rotation — consider running: hdpassw rotate"
        );
    }

    // Strip trailing /N from site name — treat it as a counter override
    let (site, gen_from_name) = split_site_gen(&args.site);
    let counter_override = args.counter.or(gen_from_name);

    let already_stored = st.sites.contains_key(&site);

    // Resolve settings: explicit args > stored record > defaults
    let (user, mut counter, length, charset_str) = match st.sites.get(&site) {
        Some(rec) => (
            args.user.as_deref().unwrap_or(&rec.user).to_owned(),
            counter_override.unwrap_or(rec.counter),
            args.length.unwrap_or(rec.length),
            args.charset.as_deref().unwrap_or(&rec.charset).to_owned(),
        ),
        None => (
            args.user.unwrap_or_default(),
            counter_override.unwrap_or(rotation), // new sites start at current rotation
            args.length.unwrap_or(32),
            args.charset.unwrap_or_else(|| "alphanumeric".into()),
        ),
    };

    // Auto-bump to current rotation if behind (and no explicit counter given)
    let old_counter = counter;
    if counter_override.is_none() && counter < rotation {
        counter = rotation;
    }

    let charset = Charset::from_str(&charset_str)?;
    let password = manager::generate_password(session, &site, &user, counter, length, charset)?;

    // Save / update site record
    let today = Local::now().date_naive();
    let verifier = session.verifier(&site, &user, counter)?;

    let created = st.sites.get(&site).map(|r| r.created).unwrap_or(today);
    let record = SiteRecord {
        user: user.clone(),
        counter,
        length,
        charset: charset_str.clone(),
        verifier,
        created,
        modified: today,
        notes: st.sites.get(&site).and_then(|r| r.notes.clone()),
    };
    st.sites.insert(site.clone(), record);
    store::save(db_path, &st)?;

    // Notify if this gen rotated the site
    if already_stored && old_counter < counter {
        eprintln!("  /{old_counter} → /{counter}  {site}  (update your account credentials)");
    }

    if args.json {
        println!(
            r#"{{"site":{},"user":{},"counter":{},"length":{},"charset":{}}}"#,
            serde_json::to_string(&site).unwrap_or_default(),
            serde_json::to_string(&user).unwrap_or_default(),
            counter,
            length,
            serde_json::to_string(&charset_str).unwrap_or_default(),
        );
    }

    if args.reveal {
        println!("{password}");
    } else {
        clipboard::copy_and_clear(&password, args.clip_timeout)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    Ok(())
}
