# mgdocker

**[Download the latest release here.](https://github.com/mgerb/mgdocker/releases)**

A web application for managing docker containers. Built for my personal needs.

- [rust](https://www.rust-lang.org/)
- [leptos](https://github.com/leptos-rs/leptos)
- [htmx](https://htmx.org/)
- [axum](https://github.com/tokio-rs/axum)

## Features
- docker compose pull
- docker compose down && docker compose up -d
- view docker_compose.yml
- prune images

![mgdocker](./screenshots/mgdocker_animation.gif)

## Usage

```
mgdocker - A simple web interface for managing docker containers and images

Usage: mgdocker [OPTIONS]

Options:
  -p, --port <PORT>  Port that the server will run on [default: 8080]
      --host <HOST>  Host that the server will run on [default: localhost]
  -h, --help         Print help
  -V, --version      Print version
```
