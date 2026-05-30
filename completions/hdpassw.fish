set -l commands gen add ls rm seed export recover help

# Top-level commands
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
    -a recover -d 'Recover counter from seed + verifier'

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
complete -c hdpassw -f -n "__fish_seen_subcommand_from seed" -a new   -d 'Generate a new mnemonic'
complete -c hdpassw -f -n "__fish_seen_subcommand_from seed" -a check -d 'Validate a mnemonic'

# recover
complete -c hdpassw -n "__fish_seen_subcommand_from recover" -s u -l user        -d 'Username' -r
complete -c hdpassw -n "__fish_seen_subcommand_from recover"      -l verifier     -d '4-char hex verifier' -r
complete -c hdpassw -n "__fish_seen_subcommand_from recover"      -l max-counter  -d 'Maximum counter to try' -r
