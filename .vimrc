set path+=ecs_engine/src/**
set path+=ecs_game/src/**
set path+=ecs_runner/src**
set path+=cfg/**
autocmd BufRead *.rs :setlocal tags=./ecs_engine/rusty-tags.vi;/,./ecs_game/rusty-tags.vi,./ecs_runner/rusty-tags.vi
