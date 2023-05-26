#!/bin/bash

# Caddy webserver process
caddy run --config /etc/caddy/Caddyfile --adapter caddyfile &

# Coterm backend process
/var/www/coterm/coterm &

# Wait for any process to exit
wait -n

# Exit with status of process that exited first
exit $?