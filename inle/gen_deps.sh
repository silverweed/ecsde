#!/bin/bash

echo 'digraph {' > deps.dot
for i in $(ls inle_*/Cargo.toml); do
	for line in $(grep '../inle' $i | cut -f1 -d=); do
		echo -e "\t$(dirname $i) -> $line;"
	done
done >> deps.dot
echo '}' >> deps.dot
