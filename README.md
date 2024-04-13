# Bookshelf

A little Rust and Tauri project to keep track about all my owned books.

This project is mainly used to get familiar with different Rust concepts,
but also to prevent to buy a book more than once. ;-)

## Technology Stack

- [Tauri](https://tauri.app/)
- [Svelte](https://svelte.dev/)
- [Sqlite](https://www.sqlite.org/index.html)
- [VS Codium](https://vscodium.com/)


## Known Issues

Currently, starting app on a Linux System with proprietary Nvidia drivers will show a blank app. To remedy such behaviour, add following environment variable:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 
```