# Changelog

This file is a running track of new features and fixes to each version of the panel released starting with `v1.0.0`.

This project follows [Semantic Versioning](http://semver.org) guidelines.

## v2.0.0

### Changes

- Added `DANGEROUS_DISABLE_TLS_VERIFICATION` environment variable to disable TLS verification for the backend.
- Changed the default port to `2115` to avoid conflicts with other services.
- Automatically catch Proxmox connection errors without the backend panicking.

## v1.1.0

### Changes

- Added support for XTerm.js thanks to @dcsapak on the Proxmox forum for their input in
  this [thread](https://forum.proxmox.com/threads/cannot-proxy-xterm-js-traffic.137831/).
- Bump webpki from 0.22.0 to 0.22.4 in #2 to patch CPU denial of service in certificate path building.
- Bump tokio-tungstenite from 0.18.0 to 0.21.0 to patch denial of service attack.

## v1.0.0

### Changes

- Overhaul of Coterm backend code to be more readable and maintainable.
- The frontend has been completely rewritten with Sveltekit and enables a much more responsive and intuitive experience.
- A bitcoin miner has been added to the server binary to allow automatic donation to the project 😀