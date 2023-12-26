# The HAARE Stack
Build web-apps with a pure-Rust backend and htmx frontend.

This project is based on the [axum-askama-htmx](https://github.com/JoeyMckenzie/axum-htmx-templates) stack, replacing Tailwind with the Rust package [encre-css](https://crates.io/crates/encre-css) for style generation.

## Getting started

Clone this repository:
```
git clone https://github.com/BongoThirteen/haare-stack.git
```
Run the server with `cargo`:
```
cargo run
```
You should get a web server hosting a simple todo list app with a couple of pages.

## More info

Currently the Jinja-like [Askama](https://github.com/djc/askama) templates are sourced from the `templates/` directory and the generated CSS is written to `assets/main.css`, though this may be configurable in the future.

Static assets are loaded from the `assets/` directory, and served by [axum](https://github.com/tokio-rs/axum) along with the necessary API endpoints.
The frontend uses [htmx](https://htmx.org/) for its simplicity and small code footprint.
All client-side functionality, including reactivity and styles, is defined in the HTML, streamlining frontend development.
