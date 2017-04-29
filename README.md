# asmjs-experiments

This repo is a playground for Rust <-> asm.js interop experiments.

Conversions to / from JavaScript types are done using Embind, but because we can't use high-level C++ templates, we use "untouchable" (undocumented) low-level C APIs instead.

`src` folder contains implementations of such conversions for types when they're generic enough, and in that case corresponding `tests` file uses these.


`tests` folder additionally serves as a playground for low-level APIs directly before they are safely generalized and moved to `src`.

`rustlib.js` contains JavaScript functions in format of Emscripten library which are exposed to Rust as normal C. This allows implementing custom conversions for Rust types more efficiently than using already exposed APIs, but requires building final executables with `--js-library rustlib.js` link argument.
