<p align="center">
  <img src="https://github.com/ConvoyPanel/coterm/assets/37554696/6456b357-6e18-4856-bbe6-3b026d52f7f0" alt="Coterm banner" />
</p>

**NOTICE: Xterm.js support is currently not implemented. I'm asking the community for support in helping us reverse engineer the network layer of Proxmox's Xterm.js sessions.**

# Convoy Terminal

Convoy terminal is a console proxy for Convoy that hides the Proxmox origin IP address. Written with Rust and utilizing Svelte, Coterm is built for performance, and every millisecond counts.

## Quick start

```
docker -p 3000:3000 -e CONVOY_URL="<panel url>" -e TOKEN="<coterm token>" ghcr.io/convoypanel/coterm:latest
```
The port can be modified by editing the first number to a different value. For example, if you want to broadcast on port 80, you do `...-p 80:3000...`. More information about publishing ports can be found [here](https://docs.docker.com/network/#published-ports) on the Docker documentation.

## Docker compose

While the quick start is an easy way to get up and running, you may want to enable TLS for Coterm to serve console sessions securely. For that reason, we recommend using a `docker-compose.yml` configuration. The default configuration we have below is for [Caddy](https://caddyserver.com/). You may modify the settings to use other web servers like Nginx, Apache, etc.

Download the example compose file and environment file
```sh
curl -o docker-compose.yml https://raw.githubusercontent.com/ConvoyPanel/coterm/develop/docker-compose.example.yml
curl -o .env https://raw.githubusercontent.com/ConvoyPanel/coterm/develop/.env.docker.example
```

Please open the `.env` environment file in your editor of choice and populate the variables.

If you need to modify the Caddy web server configuration, please refer to the [Caddyfile documentation](https://caddyserver.com/docs/caddyfile).

## For development

### Build the source code

```
git clone "https://github.com/ConvoyPanel/coterm.git"
npm install
npm run build
cd ./src-rust && cargo build --release
```

### On the fly changes

```
npm install
npm run dev
```

Note: you will need to re-run `cargo run` every time you make an edit to the Rust backend.
```
cd ./src-rust
cargo run
```
