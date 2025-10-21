#!/bin/bash

curl https://sh.rustup.rs -sSf -o ./rustinstall 
chmod +x ./rustinstall
./rustinstall -y 

cargo install cargo-leptos
rustup target add wasm32-unknown-unknown

