#!/bin/bash

cp assets/textures/* target/debug/assets/textures/ && \
cp assets/textures/* target/release/assets/textures/ && \
cp cfg/* target/debug/cfg/ && \
cp cfg/* target/release/cfg/ && \
cp assets/shaders/* target/debug/assets/shaders/ && \
cp assets/shaders/* target/release/assets/shaders/
