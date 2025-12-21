# Contributing

## Developing

### Dependencies

The following tools are required for local development:

- **[Rust](https://rustup.rs)** – required to build and run the service
- **[Git](https://git-scm.com/)** – used for version control
- **A local SMTP relay or testing tool** (for example, [Mailpit](https://mailpit.axllent.org/)) – used to test outgoing emails locally without sending real emails
- **[pre-commit](https://pre-commit.com/#install)** – enforces formatting and commit message rules before commits
- **[cargo-deny](https://github.com/EmbarkStudios/cargo-deny)** – checks dependency licenses and security advisories  
  *(Tip: it may be faster to install using [`cargo binstall`](https://github.com/cargo-bins/cargo-binstall) `cargo-deny`)*

### Setting up

- Clone the repository:
  
  ```shell
  git clone https://github.com/metabrainz/mb-mail-service.git
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
Where:
- `SMTP_PORT` is the port on which your local SMTP testing tool is running (for example, Mailpit defaults to port 1025).

#### Automatic restarts (live reload)

- Install [systemfd](https://github.com/mitsuhiko/systemfd) to provide socket activation
- Install [cargo-watch](https://github.com/watchexec/cargo-watch) to automatically rebuild and restart the service on code changes
- Run `systemfd --no-pid -s http::3000 -- cargo watch -x run`

#### Controlling logging

Logs are controlled through the `RUST_LOG` environment variable using [Tracing’s directives syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html). This allows you to control log verbosity without changing code.


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
