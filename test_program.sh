#!/bin/sh

clear;

RUST_BACKTRACE=1 cargo run --bin sonde-summarize -- ./test-data/2017083115.bufr

RUST_BACKTRACE=1 cargo run --bin sonde-dump -- ./test-data/2017083115.bufr

