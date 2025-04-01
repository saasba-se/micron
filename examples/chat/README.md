# `chat` example

<!-- cargo-rdme start -->

This example shows how one can use `micron` to quickly build a tiny live chat
web application.

<!-- cargo-rdme end -->

## Configuration

Configure the application by modifying the `micron.toml` or through environment
variables.

### Secrets

If you want features like `oauth2` authentication or `stripe` payments to work,
you will also need to provide relevant secrets.

Recommended way of dealing with secrets is to create a special
`secret.micron.toml` config file. It can be then put in `.gitignore` so that
it's never exposed to source control by mistake. Alternatively you can also
put your secrets into the regular config file or provide them through
environment variables.

### Portability

You can define a non-default configuration struct in the application code
itself. This is useful in case you will want to move the resulting binary
artifact around. If you rely on external config files(s), you will need to
bring them with you.


## Running

`cargo run --release` will build the application and serve on the port
specified in the config.

Running `cargo build --release` outputs a portable binary containing all the
application logic and assets.

Running an unoptimized artifact enables additional debugging information to be
returned with http responses.







