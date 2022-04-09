# YOTABAITO 2.0

**This is the source code to build the portfolio site.**  

**👉[The built portfolio site is here](https://www.yotabaito.net)👈**

YOTABAITO is [ytanimura](https://github.com/ytanimura)'s portfolio site.

## Technical overview

- Clean architecture with Rust + Yew + Trunk.
- All background effects are rendered in real time by WebGL.

## Build

To build this page, the following tools must be prepared in advance.

- `cargo`: Build system for Rust.
- `trunk`: Build system for web pages based on rust-wasm.
- `wasm-bindgen-cli`: wasm-bindgen command line tool.

For more details, please see [Dockerfile for CI container](./docker/ci-container.dockerfile).

## Files and directories

Since a folder contain only markdowns, we will not put README.md in each folder, but will explain the entire contents here.

### `docker`

Dockerfile for CI container.

### `Cargo.*` + `src` + `build.rs`

Rust source code for building wasm.

### `texts`

All descriptive part. It is described by markdown, and translated into HTML by `build.rs` and embedded in wasm.

### `shaders`

Shaders for Background effects. All background effects are rendered in real time by WebGL.
All are written in code for Shadertoy and can be debugged in VSCode's Shader Toy extension.

### `styles`

SCSS style sheets.

### `resources`

Resource files. If the files already exist on the Internet, they are downloaded to this folder
by a trunk pre-script instead of being placed in the repository.

### `index.html + Trunk.toml`

These are required to build with `trunk`.

## Shader debug status

| OS | Browser | Status |  
|----|----|----|  
| Windows | Chrome, Firefox | OK |  
| WSL | Firefox | Very slow because using CPU |  
| MacOS | Chrome, Safari | OK, perhaps a little slow |  
| FireHD | Silk | OK |  
| Android | Chrome | a few bugs |  
| iPhone | - | not tested |

## License

Any use of markdown texts, logo, favicon, and original selfie by others is prohibited.
The background effects are distributed under Attribution 4.0 International or MIT License as indicated at the beginning of each shader.
Another parts, source code and SCSS, are destributed by MIT Lisence.
We are creators, not lawyers, and human beings before that.
