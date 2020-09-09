# EZDB

This repo is an attempt to make a multi-tenant CRUD application server.
Something like [PostgREST](http://postgrest.org/), but even lighter-weight.

The basic idea is:
  - the developer sets up a _project_, and at least one _database_ within that project
  - the developer can run execute arbitrary SQL statements to initialize and manage their databases
  - the developer can specify _SQL templates_ and _rules_ for each template, which allows end users to safely execute a limited set of SQL statements
  - (hopefully) tooling can generate high-quality client libraries based on those SQL templates, making it really easy to build a web or mobile frontend on top of them

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
