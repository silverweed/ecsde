#!/bin/bash

mkdir -p target/{debug,release}/{cfg,assets/{textures,fonts,shaders}} && \
cp assets/textures/* target/debug/assets/textures/ && \
cp assets/textures/* target/release/assets/textures/ && \
cp assets/fonts/* target/debug/assets/fonts/ && \
cp assets/fonts/* target/release/assets/fonts/ && \
cp cfg/* target/debug/cfg/ && \
cp cfg/* target/release/cfg/ && \
cp 
cp assets/shaders/* target/debug/assets/shaders/ && \
cp assets/shaders/* target/release/assets/shaders/

rsync -rav test_resources target/debug/deps/
rsync -rav cfg assets target/debug/deps/
