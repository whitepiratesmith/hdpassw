use std::path::Path;

use chrono::Local;

use crate::cli::AddArgs;
use crate::crypto::encode::Charset;
use crate::error::Result;
use crate::session::Session;
use crate::store::{self, site::SiteRecord};

pub fn run(args: AddArgs, db_path: &Path, session: &Session) -> Result<()> {
    // Validate charset early
    Charset::from_str(&args.charset)?;

    let mut store = store::load(db_path)?;
    let today = Local::now().date_naive();

    let verifier = session.verifier(&args.site, &args.user, args.counter)?;

    let existing = store.sites.get(&args.site);
    let created = existing.map(|r| r.created).unwrap_or(today);

    let record = SiteRecord {
        user: args.user,
        counter: args.counter,
        length: args.length,
        charset: args.charset,
        verifier,
        created,
        modified: today,
        notes: args.notes,
    };

    let verb = if existing.is_some() { "Updated" } else { "Added" };
    store.sites.insert(args.site.clone(), record);
    store::save(db_path, &store)?;

    eprintln!("{verb} '{}'  →  {}", args.site, db_path.display());
    Ok(())
}
