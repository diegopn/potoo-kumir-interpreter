# Security Policy

## Reporting a vulnerability

Do not publish vulnerability details in issues, pull requests, discussions, or public comments. If the private vulnerability reporting form is available on this repository's Security page, use it. Otherwise, use the [Potoo contact page](https://diegopn.github.io/#contato) only to request a private reporting channel. Do not include technical details or a proof of concept in that first message.

After arranging a private channel, include the affected commit, Rust version, operating system, reproduction steps, and impact. Remove credentials, personal information, and unrelated data.

For other project concerns, see [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Runtime scope

This crate parses and executes source using the Kumir 2.1.0-rc11 classic or high-grade profile. It does not provide an operating-system sandbox. Programs can consume CPU through loops and recursion, and can request file or executor actions through host services.

The interpreter applies a one-million-statement execution limit, a recursion-depth limit of 1,024, and a one-million-element limit for tables. These limits reduce some resource-exhaustion risks; they do not bound source size, all temporary allocations, time spent in a host callback, or memory used by an embedding application.

The interpreter has no direct filesystem or GUI access. An embedding application that supplies `FileHost` decides which paths, encodings, handles, and operations are allowed. A `ModuleHost` can perform actor, drawing, keyboard, or other environment actions and must enforce any application-specific restrictions. File and module hosts should treat source programs as untrusted input and validate every requested operation.

The bundled command-line host connects standard input and output only. It does not install a file or module host, so source programs requiring those capabilities stop with a runtime error.
