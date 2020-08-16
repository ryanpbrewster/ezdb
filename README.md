# EZDB

## Logging

Uses the `env_logger` crate to configure logging. You can set the visibility level like this:

```
RUST_LOG=debug ./target/debug/ezdb-server
```

If you only want to see logs from the application itself (and not from the
library dependencies) you can configure it like this:

```
RUST_LOG=ezdb=debug ./target/debug/ezdb-server
```
