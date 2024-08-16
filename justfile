run:
    #!/usr/bin/env bash
    trap 'umount test; rm -rf test' EXIT
    cargo r -- ./test -f test.json
