#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

if [ -z $CI_PROJECT_NAME ] ; then
    rustup update nightly-2021-02-12
fi

rustup default nightly-2021-02-12-x86_64-unknown-linux-gnu
rustup target add wasm32-unknown-unknown --toolchain nightly-2021-02-12
