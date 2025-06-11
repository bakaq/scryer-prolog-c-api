# C embedding API for Scryer Prolog

## How to use

First, you need to compile to get the shared library:

```
# Debug build
cargo build
# Release build
cargo build --release
```

The `.so` will be in the `target/debug` or in the `target/release`
directory, depending on how you built it. You can then use it to
dynamically link with a C project, load it dynamically at runtime with
[`dlopen` ](https://www.man7.org/linux/man-pages/man3/dlopen.3.html)
(and things that use it under the hood, like Python's
[`ctypes`](https://docs.python.org/3/library/ctypes.html) library), etc...

You can also generate the C header with `cbindgen`:

```
cbindgen -o scryer_prolog.h
```

All the API functions are documented in the source and in the generated header.
There are also examples of usage from C in the `c_examples` directory.
