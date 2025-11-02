#!/bin/bash
set -e

echo "Installing just..."
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin
echo "just installed."
