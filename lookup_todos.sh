#!/bin/bash

DIR="$(dirname $(readlink -f $0))"
find "$DIR/ecs_engine/src/" "$DIR/ecs_game/src/" -type f -name \*.rs -exec egrep -n '@\S+' {} + |
	awk -vFS="//" '{print $2,"%",$1}' |
	sort |
	awk -vFS='%' '{print $2,"\n\t",$1}'
