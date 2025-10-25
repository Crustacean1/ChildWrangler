#!/bin/bash

echo "Target: ${TARGETPLATFORM}"
		
case $TARGETPLATFORM in
	linux/amd64)
		echo "x86 linux detected"
	;;
	linux/arm64)
		echo "AARCH 64 linux detected"
		export LEPTOS_BIN_TARGET_TRIPLE="aarch64-unknown-linux-gnu" 
	;;
	linux/arm/v7)
		echo "ARMv7 linux detected"
		export LEPTOS_BIN_TARGET_TRIPLE="armv7-unknown-linux-gnueabihf"
	;;
	*)
		echo "Target not recognized, quitting"
		exit 1
	;;
esac

cargo test
cargo leptos build --release	

ls -la /output
ls -la /output/release
ls -la
ls -la /output/*
find -name child_wrangler /
