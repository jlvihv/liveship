default: run

run:
    cargo run&
    cd web && bun dev

build-release:
    cd web && bun run build
    cargo build --release

up:
    cargo update
    cd web && bun update