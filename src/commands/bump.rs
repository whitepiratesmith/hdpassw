use std::io::{self, Write};
use std::path::Path;

use zeroize::Zeroizing;

use hdpassw::error::{Error, Result};
use hdpassw::session::Session;
use hdpassw::{clipboard, manager, store};

use crate::cli::BumpArgs;

/// Walks through an actual password *change*, not just a counter bump:
/// the old password is needed to log in, the new one to set on the site.
/// Both are derived before anything is written to disk, so a cancelled
/// or interrupted run leaves the metadata file untouched.
pub fn run(args: BumpArgs, db_path: &Path, session: &Session) -> Result<()> {
    let mut st = store::load(db_path)?;
    let record = st
        .sites
        .get(&args.site)
        .cloned()
        .ok_or_else(|| Error::SiteNotFound(args.site.clone()))?;

    let old = record.counter;
    let new = args
        .to
        .unwrap_or_else(|| if old < st.rotation { st.rotation } else { old + 1 });

    if new <= old {
        eprintln!(
            "'{}' is already at /{old} (target /{new}). Nothing to do.",
            args.site
        );
        return Ok(());
    }

    let charset = record.charset()?;
    let old_password = Zeroizing::new(manager::generate_password(
        session, &args.site, &record.user, old, record.length, charset,
    )?);
    let new_password = Zeroizing::new(manager::generate_password(
        session, &args.site, &record.user, new, record.length, charset,
    )?);

    if args.json {
        println!(
            r#"{{"site":{},"old_counter":{old},"new_counter":{new},"old_password":{},"new_password":{}}}"#,
            serde_json::to_string(&args.site).unwrap_or_default(),
            serde_json::to_string(old_password.as_str()).unwrap_or_default(),
            serde_json::to_string(new_password.as_str()).unwrap_or_default(),
        );
        if !args.yes {
            eprintln!(
                "Not saved. Re-run with --yes once you've changed the password on the site."
            );
            return Ok(());
        }
    } else {
        eprintln!();
        eprintln!("  Changing password for '{}':  /{old} → /{new}", args.site);
        eprintln!();

        if args.reveal {
            eprintln!("  Old password (log in with this):        {}", *old_password);
            eprintln!("  New password (set this as the new one): {}", *new_password);
            eprintln!();
        } else {
            wait_for_enter("Step 1/2 — press Enter to copy the OLD password, then log in and start the change:")?;
            clipboard::copy_and_clear(&old_password, 30)?;
            eprintln!();

            wait_for_enter("Step 2/2 — press Enter to copy the NEW password, then paste it as the new one:")?;
            clipboard::copy_and_clear(&new_password, 30)?;
            eprintln!();
        }

        if !args.yes && !confirm("  Save the updated counter to metadata now? [Y/n] ")? {
            eprintln!(
                "  Not saved. Re-run `hdpassw bump {} --to {new}` once you've finished the change.",
                args.site
            );
            return Ok(());
        }
    }

    manager::upsert_site(
        &mut st,
        session,
        &args.site,
        &record.user,
        new,
        record.length,
        charset,
        record.notes,
    )?;
    store::save(db_path, &st)?;

    eprintln!("  Saved: '{}' is now at /{new}.", args.site);
    Ok(())
}

fn wait_for_enter(prompt: &str) -> Result<()> {
    eprint!("  {prompt} ");
    io::stderr().flush().map_err(Error::Io)?;
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(Error::Io)?;
    Ok(())
}

fn confirm(prompt: &str) -> Result<bool> {
    eprint!("{prompt}");
    io::stderr().flush().map_err(Error::Io)?;
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(Error::Io)?;
    let input = input.trim();
    Ok(input.is_empty() || input.eq_ignore_ascii_case("y"))
}
