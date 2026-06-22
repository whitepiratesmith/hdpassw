#compdef hdpassw

_hdpassw() {
    local state

    _arguments \
        '--db[metadata file path]:file:_files' \
        '--passphrase[passphrase (prefer env var)]:passphrase:' \
        '(-h --help)'{-h,--help}'[show help]' \
        '(-V --version)'{-V,--version}'[show version]' \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            local commands=(
                'init:Create a new seed phrase or restore an existing one'
                'gen:Generate and copy the password for a site'
                'add:Add or update a site record'
                'ls:List all known sites'
                'rm:Remove a site record'
                'seed:Seed phrase management'
                'export:Export metadata to stdout'
                'rotate:Bump the global rotation counter'
                'bump:Bump the rotation counter for a single site'
                'recover:Recover counter from seed + verifier'
                'pwned:Check site password(s) against Have I Been Pwned'
                'help:Print help'
            )
            _describe 'command' commands
            ;;
        args)
            local charsets='(alpha alphanumeric full pin hex)'
            case ${words[1]} in
                gen)
                    _arguments \
                        '1:site:' \
                        {-u,--user}'[username or email]:user:' \
                        {-n,--counter}'[rotation counter]:counter:' \
                        {-l,--length}'[password length]:length:' \
                        '--charset[character set]:charset:'"$charsets" \
                        '--reveal[print to stdout]' \
                        '--clip-timeout[clipboard clear timeout]:seconds:' \
                        '--json[JSON output]'
                    ;;
                add)
                    _arguments \
                        '1:site:' \
                        {-u,--user}'[username or email]:user:' \
                        {-n,--counter}'[rotation counter]:counter:' \
                        {-l,--length}'[password length]:length:' \
                        '--charset[character set]:charset:'"$charsets" \
                        '--notes[optional notes]:notes:'
                    ;;
                ls)
                    _arguments \
                        '1:filter:' \
                        '--json[JSON output]'
                    ;;
                rm)
                    _arguments \
                        '1:site:' \
                        {-y,--yes}'[skip confirmation]'
                    ;;
                seed)
                    local seed_cmds=(
                        'new:Generate a new mnemonic'
                        'check:Validate a mnemonic'
                        'restore:Restore from an existing mnemonic'
                        'remove:Remove the encrypted seed vault'
                    )
                    _describe 'seed command' seed_cmds
                    ;;
                recover)
                    _arguments \
                        '1:site:' \
                        {-u,--user}'[username]:user:' \
                        '--verifier[4-char hex verifier]:verifier:' \
                        '--max-counter[maximum counter to try]:n:'
                    ;;
                pwned)
                    _arguments \
                        '1:site:' \
                        '--json[JSON output]'
                    ;;
                bump)
                    _arguments \
                        '1:site:' \
                        '--to[set counter to this exact value]:counter:' \
                        '--reveal[print passwords instead of using the clipboard]' \
                        {-y,--yes}'[skip the save confirmation]' \
                        '--json[output as JSON]'
                    ;;
            esac
            ;;
    esac
}

_hdpassw "$@"
