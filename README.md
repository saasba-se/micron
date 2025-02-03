## <img src="assets/micron.png" width="300">

[![Static Badge](https://img.shields.io/badge/discord-server-blue)](https://discord.gg/Q3CzGTEHaC)

<!-- cargo-rdme start -->

*Build web apps fast. Repeat.*

`micron` provides a range of functionality useful for building *micro* web
applications *n times*, where *n is greater than a dozen*.

The library covers a wide range of functionality useful in different kinds
of applications, e.g. user management, auth, mailings, posts, comments,
payments, invoice generation and more.

It is quite opinionated and channels decisions made based on core
motivations: simplicity, extendability and development speed.

It focuses on the
[hypermedia-based approach](https://htmx.org/essays/hateoas/) with
server-side generated html, champions embedded key-value databases for data
storage and targets a deployment model of single all-in-one binary per
application.


### Data model

At the most basic level, `micron` library defines a concrete data model
that can be used directly in the context of small applications.

If you were building a saas application, for example, you could make
immediate use of ready-made types to handle your `User`s, `Product`s,
`Order`s, `Payment`s, subscription `Plan`s and more.

You can implement application-specific types *on top*, leading to
relatively consise codebases with as little repetition across
applications as possible.


### Common logic

`micron` provides an extensive set of operations for working with
application state as defined with the data model.

This logic is referred to as *common* as it remains fully
web-framework-agnostic. It can be found in the respective top-level
modules.


### Framework-specific logic

`micron` takes it upon itself to provide building blocks for use with
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

The fastest way to get started is to build and run the provided examples.
Clone the repo, navigate to any of the examples in the top level `examples`
directory and simply do `cargo run`.

The chosen example should now be accessible at `localhost:8000`. If that
port is unavailable on your machine simply modify the `address` field of
the `./examples/{example}/micron.toml` file.

For a more involved application using `micron` see the
[`ruda` project](https://github.com/adamsky/ruda).

Examples showing off particular aspects of the library are also provided
inside `lib/examples`.

While you already have the repository cloned you can try running the
`micron-cli` tool. Navigate to `./cli` and do `cargo run`. With
`micron-cli` you'll be able to inspect and mutate application state,
either on-line or off-line.


#### Pulling the library

Pull in `micron` dependency into your project, putting the following into
your `Cargo.toml` `[dependencies]`:

```toml
micron = "0.1.0"
```

As `micron` tends to be opinionated and encourages employing certain
solutions for the sake of efficiency, the default feature set is meant to
be sensible for most use-cases.

That said, crate features can provide additional and/or alternative
solutions in some cases (for example different storage engines). Feel free
to make use of them as needed.

```toml
micron = { version = "0.1.0", default-features = false, features = ["axum", "fjall"] }
```


#### Using the library

`micron` is organized into modules based on type of functionality they
provide.

Common usage pattern involves defining a configuration (or loading it from
file) and using it to generate a `micron` router to be merged with the
main application router.

Assets and templates should be provided alongside Rust code. Assets can
either be read from the filesystem at runtime or embedded into the
application binary to enhance portability.


#### Deploying

Deployment of a `micron`-based application can be as easy as compiling
down to a single binary and exposing the web server listener to the outside
world. They only requirement on the system is a Rust installation and
a network connection.

<!-- cargo-rdme end -->
