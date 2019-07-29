workflow "Run tests" {
  on = "push"
  resolves = ["test"]
}

action "test" {
  uses = "icepuma/rust-action@master"
  args = "cargo clippy -- -Dwarnings && cargo test"
}
