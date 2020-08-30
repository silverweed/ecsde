#!/bin/bash

DIR="$(dirname $(readlink -f $0))"
find "$DIR/inle/" "$DIR/ecs_game/src/" -type f -name \*.rs -exec egrep -n '@\S+' {} + |
	awk -vFS="//" '{print $2,"%",$1}' |
	sort |
	awk -vFS='%' '{print "\033[2m" $2 "\033[0;0m\n\t",$1}'
