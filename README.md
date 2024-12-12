## <img src="assets/saasbase-logo.png" width="300">

[![Static Badge](https://img.shields.io/badge/discord-server-blue)](https://discord.gg/Q3CzGTEHaC)

<!-- cargo-rdme start -->

*Build saas fast. Repeat.*

`saasbase` provides a range of functionality useful for building saas web
applications, covering many things from user management and auth, to
payments and invoice generation.

It is quite opinionated and channels decisions made based on core
motivations: simplicity, extendability and development speed.

It focuses on the
[hypermedia-based approach](https://htmx.org/essays/hateoas/) with
server-side generated html, champions embedded key-value databases for data
storage and targets a deployment model of single all-in-one binary per
application.


### Data model

At the most basic level, `saasbase` library defines a concrete data model
specifically targeting saas applications. Here one can find notions of
`User`s, `Product`s, `Payment`s, subscription `Plan`s and more.

The provided data model aims to fit most common use-cases for simple saas
applications.


### Common logic

`saasbase` provides an extensive set of operations for working with
application state as defined with the data model.

This logic is referred to as *common* as it remains framework-agnostic.
It can be found in the respective top-level modules.


### Framework-specific logic

`saasbase` takes it upon itself to provide building blocks for use with
selected web frameworks. This includes middleware, extractors and
ready-made endpoints.

Framework-specific functionality is provided in respective feature-gated
modules. Note that currently only the `axum` web framework is supported.

In terms of provisioned endpoints, you'll find that most of the
functionality that doesn't need a custom html response is included. That's
everything from `/login` form submit to common `/api` operations. Pages as
well as html parts to be injected dynamically must be created by library
user. See the `examples` directory for example usage.


### Geting started

The fastest way to get started is to follow the provided examples. They
will set you up on the way to quickly create a fully-featured application.

Otherwise, here are some of the basics if you want to start from scratch.


#### Pulling the library

Pull in `saasbase` dependency into your project, putting the following into
your `Cargo.toml` `[dependencies]`:

```toml
saasbase = "0.1.0"
```

As `saasbase` tends to be opinionated and encourages employing certain
solutions for the sake of efficiency, the default feature set is meant to
be sensible for most use-cases.

That said, crate features can provide additional and/or alternative
solutions in some cases (for example different database storages). Feel
free to make use of them as needed.

```toml
saasbase = { version = "0.1.0", default-features = false, features = ["axum", "fjall"] }
```


#### Using the library

`saasbase` is organized into modules based on type of functionality they
provide.

Common usage pattern involves defining a configuration (or loading it from
file) and using it to generate a `saasbase` router to be merged with the
main application router.

Assets and templates should be provided alongside Rust code. Assets can
either be read from the filesystem at runtime or embedded into the
application binary to enhance portability.


#### Deploying

Deployment of a `saasbase`-based application can be as easy as compiling
down to a single binary and exposing the web server listener to the outside
world. They only requirement on the system is a Rust installation and
a network connection.

<!-- cargo-rdme end -->
