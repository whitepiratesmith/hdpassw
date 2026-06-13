use hdpassw::crypto::verifier;
use hdpassw::error::{Error, Result};
use hdpassw::session::Session;

use crate::cli::RecoverArgs;

/// Scan counters 1..=max_counter until the derived verifier matches.
/// Useful when you have the seed but lost the rotation log.
pub fn run(args: RecoverArgs, session: &Session) -> Result<()> {
    eprintln!(
        "Scanning counters 1..={} for site='{}' user='{}'…",
        args.max_counter, args.site, args.user
    );

    for n in 1..=args.max_counter {
        let key = session.site_key(&args.site, &args.user, n)?;
        let v = verifier(&key);
        if v == args.verifier {
            eprintln!("✓  Found! counter = {n}  (verifier: {v})");
            return Ok(());
        }
        eprint!("  n={n} → {v}\r");
    }

    Err(Error::SiteNotFound(format!(
        "no counter in 1..={} matched verifier '{}'",
        args.max_counter, args.verifier
    )))
}
