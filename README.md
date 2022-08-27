# Register machine for alife experiments

This is a highly experimental 16-bit register machine for Artificial Life
experiments vaguely based on RISC-V.

Aleven is a low-level assembly language that should compile down to safe code
no whatever what bytes are given it; any bytes are considered a valid program.
By safe I mean:

- no crashes

- no out of bounds memory writes

- no infinite loops

This package implements both an interpreter in Rust as well as a compiler for
this language with LLVM. Because it can interpret any string of bytes it can
be used in an environment that features mutations.

In itself Aleven does not include any experiments with evolution; for that it
needs to be integrated into an engine like
[Apilar](https://github.com/faassen/apilar), which I intend to do eventually.
