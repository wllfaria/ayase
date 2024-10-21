@run $prog:
    cargo r -- $prog

@dbg:
    cargo r --bin ayadbg

@test:
    cargo test --workspace

@review:
    cargo insta review
