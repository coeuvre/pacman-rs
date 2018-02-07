#!/bin/sh

export PATH=$PATH:$HOME/.cargo/bin

cd ${SRCROOT}/../rust

cargo build
