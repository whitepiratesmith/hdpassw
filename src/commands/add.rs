use std::path::Path;

use hdpassw::crypto::encode::Charset;
use hdpassw::error::Result;
use hdpassw::session::Session;
use hdpassw::{manager, store};

use crate::cli::AddArgs;

pub fn run(args: AddArgs, db_path: &Path, session: &Session) -> Result<()> {
    let charset = Charset::from_str(&args.charset)?;

    let mut store = store::load(db_path)?;
    let existing = store.sites.contains_key(&args.site);

    manager::upsert_site(
        &mut store,
        session,
        &args.site,
        &args.user,
        args.counter,
        args.length,
        charset,
        args.notes,
    )?;
    store::save(db_path, &store)?;

    let verb = if existing { "Updated" } else { "Added" };
    eprintln!("{verb} '{}'  →  {}", args.site, db_path.display());
    Ok(())
}
