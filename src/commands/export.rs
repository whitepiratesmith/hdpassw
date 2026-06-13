use std::path::Path;

use hdpassw::error::{Error, Result};
use hdpassw::store;

pub fn run(db_path: &Path) -> Result<()> {
    let store = store::load(db_path)?;
    let out = toml::to_string_pretty(&store).map_err(|e| Error::Store(e.to_string()))?;
    print!("{out}");
    Ok(())
}
