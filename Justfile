@run $prog:
    cargo r -- $prog

@dbg:
    cargo r --bin ayadbg

@test:
    cargo test --workspace -- --nocapture

@review:
    cargo insta review
