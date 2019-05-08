workflow "Build and test on push" {
  resolves = ["Rust Action"]
  on = "push"
}

action "Rust Action" {
  uses = "icepuma/rust-action@1.0.6"
  args = "cargo fmt -- --check && cargo clippy -- -Dwarnings && cargo test"
}
