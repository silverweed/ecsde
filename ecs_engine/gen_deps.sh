#!/bin/bash

for i in $(ls inle_*/Cargo.toml); do
	for line in $(grep '../inle' $i | cut -f1 -d=); do
		echo "$(dirname $i) -> $line;"
	done
done > deps.dot
