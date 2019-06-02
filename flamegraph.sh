#!/bin/bash

APP=ecs

declare -r F=/usr/local/src/FlameGraph
set -x
./target/debug/$APP $@ &>/dev/null&
sudo perf record -F 80 -a -g -p $! -- sleep 30
sudo perf script > /tmp/out.perf
$F/stackcollapse-perf.pl /tmp/out.perf > /tmp/out.folded
$F/flamegraph.pl /tmp/out.folded > /tmp/out.svg
firefox /tmp/out.svg&
pkill $APP
