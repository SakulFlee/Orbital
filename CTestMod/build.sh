#!/usr/bin/env bash

clang --target=wasm32 -nostdlib -Wl,--no-entry -Wl,--export-all -o test.wasm test.c

