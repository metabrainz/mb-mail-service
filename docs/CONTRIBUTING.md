# Contributing

## Developing

### Dependencies

- [Rust](https://rustup.rs)
- [git](https://git-scm.com/)
- A local SMTP relay or testing tool, like [Mailpit](https://mailpit.axllent.org/)
- [pre-commit](https://pre-commit.com/#install)
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) (It may be faster to install with [`cargo binstall`](https://github.com/cargo-bins/cargo-binstall) `cargo-deny`)

### Setting up

- Clone the repository:
  
  ```shell
  git clone https://github.com/JadedBlueEyes/mb-mail-service.git
  cd mb-mail-service
  ```

- Install pre-commit hooks:
  
  ```shell
  pre-commit install
  ```

### Running

```shell
cargo run
```

To send mail to your local SMTP relay, first start it, and then tell the service which port it is running on with an environment variable:

```shell
SMTP_PORT="1025" cargo run
```

#### Automatic restarts (live reload)

- Install [systemfd](https://github.com/mitsuhiko/systemfd)
- Install [cargo-watch](https://github.com/watchexec/cargo-watch)
- Run `systemfd --no-pid -s http::3000 -- cargo watch -x run`

#### Controlling logging

Logs are controlled through the `RUST_LOG` environment variable using [Tracing's Directives syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html).

For example:

```shell
APP_SMTP_PORT="1025" RUST_LOG="trace,html5ever=warn,lettre::transport::smtp::client::async_connection=warn,runtime=warn,tokio::task=warn" systemfd --no-pid -s http::3000 -- cargo watch -x run
```

### Testing

```shell
cargo test
```

## Committing

Commits in this repository follow [Conventional Commits](https://daily-dev-tips.com/posts/git-basics-conventional-commits/) on a best-effort basis. This is automatically enforced by pre-commit hooks. For examples, please look at the commit history of the repository.
