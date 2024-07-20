# Operations

## Installation

### Building from source

- Install dependencies:
  - [Rust](https://rustup.rs)
  - [git](https://git-scm.com/)
- Clone the repository:
  
  ```shell
  git clone https://github.com/JadedBlueEyes/mb-mail-service.git
  cd mb-mail-service
  ```
  
- Build in release mode:
  
  ```shell
  cargo build --release
  ```

- The executable will be in `target/release/mb-mail-service`.

## Configuration

This service is primarily configured through environment variables.

### Listening

By default the server will use the `automatic_selection`
mode, which will use a passed file descriptor if available,
but otherwise will listen on the TCP port configured
(by default 127.0.0.1:3000)

| Setting name | Value                                                     | Default value                                                                             |
| ------------ | --------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| APP_LISTEN_MODE         | `file_descriptor` \| `automatic_selection` \| `tcp_listener` | `automatic_selection`                                                                      |
| APP_LISTEN_PORT         | unsigned integer                                          | `file_descriptor`: Ignored<br>`automatic_selection`: `3000`<br>`tcp_listener`: required      |
| APP_LISTEN_HOST         | IP address                                                | `file_descriptor`: Ignored<br>`automatic_selection`: `127.0.0.1`<br>`tcp_listener`: required |

### Mailing

> ⚠️ `APP_SMTP_MODE` defaults to `plaintext`, which is not safe to use over the network.

| Setting name      | Value                                             | Default value |
| ----------------- | ------------------------------------------------- | ------------- |
| APP_SMTP_MODE     | `plaintext` \| `startls` \| `tls`                 | `plaintext`   |
| APP_SMTP_PORT         | The port of the SMTP relay to connect to          | `25`          |
| APP_SMTP_HOST         | The hostname of the SMTP relay to connect to      | `localhost`   |
| APP_SMTP_TIMEOUT      | The timeout duration                              | 5 seconds     |

### Sentry

- `SENTRY_DSN`: Where to send Sentry events. If unset, no sentry events will be sent.
- More options are available in the Sentry docs: <https://docs.sentry.io/platforms/rust/configuration/options/>
