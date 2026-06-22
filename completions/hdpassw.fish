set -l commands init gen add ls rm seed export rotate bump recover pwned help

# Top-level commands
complete -c hdpassw -f -n "not __fish_seen_subcommand_from $commands" \
    -a init    -d 'Create a new seed phrase or restore an existing one'
complete -c hdpassw -f -n "not __fish_seen_subcommand_from $commands" \
    -a gen     -d 'Generate and copy the password for a site'
complete -c hdpassw -f -n "not __fish_seen_subcommand_from $commands" \
    -a add     -d 'Add or update a site record'
complete -c hdpassw -f -n "not __fish_seen_subcommand_from $commands" \
    -a ls      -d 'List all known sites'
complete -c hdpassw -f -n "not __fish_seen_subcommand_from $commands" \
    -a rm      -d 'Remove a site record'
complete -c hdpassw -f -n "not __fish_seen_subcommand_from $commands" \
    -a seed    -d 'Seed phrase management'
complete -c hdpassw -f -n "not __fish_seen_subcommand_from $commands" \
    -a export  -d 'Export metadata to stdout'
complete -c hdpassw -f -n "not __fish_seen_subcommand_from $commands" \
    -a rotate  -d 'Bump the global rotation counter'
complete -c hdpassw -f -n "not __fish_seen_subcommand_from $commands" \
    -a recover -d 'Recover counter from seed + verifier'
complete -c hdpassw -f -n "not __fish_seen_subcommand_from $commands" \
    -a pwned   -d 'Check site password(s) against Have I Been Pwned'
complete -c hdpassw -f -n "not __fish_seen_subcommand_from $commands" \
    -a bump    -d 'Bump the rotation counter for a single site'

# Global flags
complete -c hdpassw -l db         -d 'Metadata file path' -r -F
complete -c hdpassw -l passphrase -d 'Passphrase (prefer env var)' -r

# gen
set -l charset_vals alpha alphanumeric full pin hex
complete -c hdpassw -n "__fish_seen_subcommand_from gen" -s u -l user        -d 'Username or email' -r
complete -c hdpassw -n "__fish_seen_subcommand_from gen" -s n -l counter      -d 'Rotation counter' -r
complete -c hdpassw -n "__fish_seen_subcommand_from gen" -s l -l length       -d 'Password length' -r
complete -c hdpassw -n "__fish_seen_subcommand_from gen"      -l charset      -d 'Character set' -r -a "$charset_vals"
complete -c hdpassw -n "__fish_seen_subcommand_from gen"      -l reveal       -d 'Print to stdout'
complete -c hdpassw -n "__fish_seen_subcommand_from gen"      -l clip-timeout -d 'Clipboard clear timeout (s)' -r
complete -c hdpassw -n "__fish_seen_subcommand_from gen"      -l json         -d 'JSON output'

# add
complete -c hdpassw -n "__fish_seen_subcommand_from add" -s u -l user    -d 'Username or email' -r
complete -c hdpassw -n "__fish_seen_subcommand_from add" -s n -l counter -d 'Rotation counter' -r
complete -c hdpassw -n "__fish_seen_subcommand_from add" -s l -l length  -d 'Password length' -r
complete -c hdpassw -n "__fish_seen_subcommand_from add"      -l charset -d 'Character set' -r -a "$charset_vals"
complete -c hdpassw -n "__fish_seen_subcommand_from add"      -l notes   -d 'Optional notes' -r

# ls
complete -c hdpassw -n "__fish_seen_subcommand_from ls" -l json -d 'JSON output'

# rm
complete -c hdpassw -n "__fish_seen_subcommand_from rm" -s y -l yes -d 'Skip confirmation'

# seed subcommands
complete -c hdpassw -f -n "__fish_seen_subcommand_from seed" -a new     -d 'Generate a new mnemonic'
complete -c hdpassw -f -n "__fish_seen_subcommand_from seed" -a check   -d 'Validate a mnemonic'
complete -c hdpassw -f -n "__fish_seen_subcommand_from seed" -a restore -d 'Restore from an existing mnemonic'
complete -c hdpassw -f -n "__fish_seen_subcommand_from seed" -a remove  -d 'Remove the encrypted seed vault'

# recover
complete -c hdpassw -n "__fish_seen_subcommand_from recover" -s u -l user        -d 'Username' -r
complete -c hdpassw -n "__fish_seen_subcommand_from recover"      -l verifier     -d '4-char hex verifier' -r
complete -c hdpassw -n "__fish_seen_subcommand_from recover"      -l max-counter  -d 'Maximum counter to try' -r

# pwned
complete -c hdpassw -n "__fish_seen_subcommand_from pwned" -l json -d 'JSON output'

# bump
complete -c hdpassw -n "__fish_seen_subcommand_from bump" -l to     -d 'Set counter to this exact value' -r
complete -c hdpassw -n "__fish_seen_subcommand_from bump" -l reveal -d 'Print passwords instead of using the clipboard'
complete -c hdpassw -n "__fish_seen_subcommand_from bump" -s y -l yes -d 'Skip the save confirmation'
complete -c hdpassw -n "__fish_seen_subcommand_from bump" -l json   -d 'Output as JSON'
