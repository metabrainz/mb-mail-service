# Operations

## Configuration

This service is primarily configured through environment variables.

### Listening

If the process is passed a socket, for example by systemd or listenfd, it will listen on that. Otherwise, it will create a socket from the following options.

- `HOST`: The IP address that the service will listen on. Defaults to `127.0.0.1`. Set to `0.0.0.0` to listen on all addresses.
- `PORT`: The Port that the service will listen on. Defaults to `3000`.

### Mailing

- `SMTP_MODE`: `plaintext`, `startls` or `tls`. Defaults to `plaintext`, which is not safe to use over the network.
- `SMTP_HOST`: The SMTP relay to connect to. Defaults to `localhost`.
- `SMTP_PORT`: The port to connect to. Defaults to `25`.
