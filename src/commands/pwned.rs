use std::path::Path;

use hdpassw::crypto::encode::Charset;
use hdpassw::error::{Error, Result};
use hdpassw::manager;
use hdpassw::pwned::{self, PwnedStatus};
use hdpassw::session::Session;
use hdpassw::store;

use crate::cli::PwnedArgs;

/// Derive the password for `site`'s stored settings and check it against
/// the Have I Been Pwned range API.
fn check_one(site: &str, record: &store::site::SiteRecord, session: &Session) -> Result<PwnedStatus> {
    let charset = Charset::from_str(&record.charset)?;
    let password = manager::generate_password(
        session,
        site,
        &record.user,
        record.counter,
        record.length,
        charset,
    )?;
    pwned::check(&password)
}

pub fn run(args: PwnedArgs, db_path: &Path, session: &Session) -> Result<()> {
    let st = store::load(db_path)?;

    let sites: Vec<(&String, &store::site::SiteRecord)> = match &args.site {
        Some(site) => {
            let record = st
                .sites
                .get(site)
                .ok_or_else(|| Error::SiteNotFound(site.clone()))?;
            vec![(site, record)]
        }
        None => st.sites.iter().collect(),
    };

    let mut results = Vec::with_capacity(sites.len());
    for (site, record) in &sites {
        let status = check_one(site, record, session)?;
        results.push(((*site).clone(), status));
    }

    if args.json {
        let parts: Vec<String> = results
            .iter()
            .map(|(site, status)| {
                let count = match status {
                    PwnedStatus::Safe => 0,
                    PwnedStatus::Pwned(n) => *n,
                };
                format!(
                    r#"{{"site":{},"pwned":{},"count":{count}}}"#,
                    serde_json::to_string(site).unwrap_or_default(),
                    !matches!(status, PwnedStatus::Safe),
                )
            })
            .collect();
        println!("[{}]", parts.join(","));
        return Ok(());
    }

    let mut pwned_count = 0;
    for (site, status) in &results {
        match status {
            PwnedStatus::Safe => println!("✓ {site}: not found in known breaches"),
            PwnedStatus::Pwned(count) => {
                pwned_count += 1;
                println!("⚠ {site}: found in {count} known breach(es) — rotate it");
            }
        }
    }

    if results.len() > 1 {
        println!("\n{pwned_count} of {} password(s) found in known breaches", results.len());
    }

    Ok(())
}
