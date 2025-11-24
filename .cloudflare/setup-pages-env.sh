#!/bin/sh
set -eu
npm install --global yarn || true

curl -o rustup.sh https://sh.rustup.rs
chmod +x rustup.sh
./rustup.sh -y # --default-toolchain 1.81.0

. $HOME/.cargo/env
curl -o wasm-init.sh https://rustwasm.github.io/wasm-pack/installer/init.sh
chmod +x wasm-init.sh
./wasm-init.sh

cd web/
yarn install
yarn build
