#!/bin/bash

echo "Target: ${TARGETPLATFORM}"

		
case $TARGETPLATFORM in
	linux/amd64)
		echo "Building for x86 linux"
		cargo leptos build --release	
		cargo leptos build --bin message_daemon	
	;;
	linux/aarch64)
		rustup target add aarch64-unknown-linux-gnu
		LEPTOS_BIN_TARGET_TRIPLE="aarch64-unknown-linux-gnu" cargo leptos build --release	
		LEPTOS_BIN_TARGET_TRIPLE="aarch64-unknown-linux-gnu" cargo leptos build --bin message_daemon	
		echo "Building for  aarch linux"
	;;
	linux/armv7)
		echo "Builidng for ARMv7 linux"

		apt-get install -yqq gcc-arm-linux-gnueabihf
		echo "[target.armv7-unknown-linux-gnueabihf]\nlinker = \"arm-linux-gnueabihf-gcc\"" >> ~/.cargo/config.toml
		rustup target add armv7-unknown-linux-gnueabihf 
		LEPTOS_BIN_TARGET_TRIPLE="armv7-unknown-linux-gnueabihf" cargo leptos build --release	
		LEPTOS_BIN_TARGET_TRIPLE="armv7-unknown-linux-gnueabihf" cargo leptos build --bin message_daemon	
	;;
	*)
		echo "Target not recognized, quitting"
	;;
esac
