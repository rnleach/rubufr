#!/bin/sh
clear; RUST_BACKTRACE=1 cargo run --bin sonde-summarize -- ./test-data/2017083115.bufr
