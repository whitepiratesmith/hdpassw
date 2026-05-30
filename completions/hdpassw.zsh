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
                'gen:Generate and copy the password for a site'
                'add:Add or update a site record'
                'ls:List all known sites'
                'rm:Remove a site record'
                'seed:Seed phrase management'
                'export:Export metadata to stdout'
                'recover:Recover counter from seed + verifier'
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
                    local seed_cmds=('new:Generate a new mnemonic' 'check:Validate a mnemonic')
                    _describe 'seed command' seed_cmds
                    ;;
                recover)
                    _arguments \
                        '1:site:' \
                        {-u,--user}'[username]:user:' \
                        '--verifier[4-char hex verifier]:verifier:' \
                        '--max-counter[maximum counter to try]:n:'
                    ;;
            esac
            ;;
    esac
}

_hdpassw "$@"
