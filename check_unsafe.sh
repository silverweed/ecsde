#!/bin/sh

find ecs_engine/src ecs_game/src -type f -name \*.rs -exec grep -B1 unsafe {} +

echo "Found $(find ecs_engine/src ecs_game/src -type f -name \*.rs -exec grep unsafe {} + | grep -v fn | grep -v trait | wc -l) unsafe calls."
