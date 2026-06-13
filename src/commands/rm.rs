use std::io::{self, Write};
use std::path::Path;

use hdpassw::error::{Error, Result};
use hdpassw::store;

use crate::cli::RmArgs;

pub fn run(args: RmArgs, db_path: &Path) -> Result<()> {
    let mut store = store::load(db_path)?;

    if !store.sites.contains_key(&args.site) {
        return Err(Error::SiteNotFound(args.site));
    }

    if !args.yes {
        print!("Remove '{}'? [y/N] ", args.site);
        io::stdout().flush().map_err(Error::Io)?;
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(Error::Io)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            eprintln!("Aborted.");
            return Ok(());
        }
    }

    store.sites.remove(&args.site);
    store::save(db_path, &store)?;
    eprintln!("Removed '{}'.", args.site);
    Ok(())
}
