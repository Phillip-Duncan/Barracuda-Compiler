#!/bin/bash


# Very simple build file to generate headers when building the application
cargo test --features c-headers -- generate_headers
cargo build