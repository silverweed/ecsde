#!/bin/bash

find inle ecs_game/src -type f -name \*.rs -exec grep -B1 unsafe {} + |
    tee > >(cat) >(wc -l | xargs -i expr {} / 3 | xargs -i echo Found {} unsafes)
