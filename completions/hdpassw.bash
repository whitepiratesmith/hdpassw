_hdpassw() {
    local cur prev words cword
    _init_completion || return

    local commands="init gen add ls rm seed export rotate bump recover pwned help"
    local seed_commands="new check restore remove"
    local charsets="alpha alphanumeric full pin hex"

    case "${words[1]}" in
        gen)
            case "$prev" in
                --charset)   COMPREPLY=($(compgen -W "$charsets" -- "$cur")) ; return ;;
                --user|-u)   return ;;
                --counter|-n|--length|-l|--clip-timeout) return ;;
                --db)        COMPREPLY=($(compgen -f -- "$cur")) ; return ;;
            esac
            COMPREPLY=($(compgen -W "--user --counter --length --charset --reveal --clip-timeout --json --db --passphrase" -- "$cur"))
            ;;
        add)
            case "$prev" in
                --charset)   COMPREPLY=($(compgen -W "$charsets" -- "$cur")) ; return ;;
                --user|-u|--counter|-n|--length|-l|--notes) return ;;
                --db)        COMPREPLY=($(compgen -f -- "$cur")) ; return ;;
            esac
            COMPREPLY=($(compgen -W "--user --counter --length --charset --notes --db --passphrase" -- "$cur"))
            ;;
        ls)
            COMPREPLY=($(compgen -W "--json --db" -- "$cur"))
            ;;
        rm)
            COMPREPLY=($(compgen -W "--yes --db" -- "$cur"))
            ;;
        seed)
            COMPREPLY=($(compgen -W "$seed_commands" -- "$cur"))
            ;;
        recover)
            case "$prev" in
                --user|-u|--verifier|--max-counter) return ;;
                --db) COMPREPLY=($(compgen -f -- "$cur")) ; return ;;
            esac
            COMPREPLY=($(compgen -W "--user --verifier --max-counter --db --passphrase" -- "$cur"))
            ;;
        pwned)
            case "$prev" in
                --db) COMPREPLY=($(compgen -f -- "$cur")) ; return ;;
            esac
            COMPREPLY=($(compgen -W "--json --db --passphrase" -- "$cur"))
            ;;
        bump)
            case "$prev" in
                --to) return ;;
                --db) COMPREPLY=($(compgen -f -- "$cur")) ; return ;;
            esac
            COMPREPLY=($(compgen -W "--to --reveal --yes --json --db --passphrase" -- "$cur"))
            ;;
        *)
            COMPREPLY=($(compgen -W "$commands" -- "$cur"))
            ;;
    esac
}

complete -F _hdpassw hdpassw
