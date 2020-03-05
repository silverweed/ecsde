autocmd BufRead *.rs :setlocal tags=./ecs_engine/rusty-tags.vi;/,./ecs_game/rusty-tags.vi,./ecs_runner/rusty-tags.vi
set path+=./ecs_game
set path+=./ecs_engine
set path+=./ecs_runner
