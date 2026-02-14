# Happening nu

Comming soon.

## Project setup history

```Bash
$ rustc --version
#rustc 1.91.1 (ed61e7d7e 2025-11-07)

$  rustup --version
#rustup 1.28.2 (e4f3ad6f8 2025-04-28)
#info: This is the version for the rustup toolchain manager, not the rustc compiler.
#info: The currently active `rustc` version is `rustc 1.91.1 (ed61e7d7e 2025-11-07)`
```

```Bash
cargo init --name happeningnu

cargo add axum
cargo add tokio --features full
cargo add tower
cargo add tower_http -F trace
cargo add tower_http -F fs
cargo add hyper
cargo add serde --features derive
cargo add tera
cargo add sea-orm --features sqlx-sqlite,runtime-tokio-rustls,macros,with-chrono,with-uuid
cargo install sea-orm-cli
cargo add sea-orm-migration --features sqlx-sqlite
cargo add dotenvy
cargo add validator -F derive
cargo add time
cargo add axum-messages
cargo add tower-sessions
cargo add tower-sessions-sqlx-store -F sqlite
cargo add argon2
cargo add chrono
cargo add serde_json

```

```Bash
brew install cargo-watch
cargo watch -x run
```
