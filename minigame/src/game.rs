use crate::phases::Phase_Args;
use inle_audio::audio_system::Sound_Handle;
use inle_cfg::Cfg_Var;
use inle_input::core_actions::Core_Action;
use std::time::Duration;

#[cfg(debug_assertions)]
use super::debug;

pub type Phase_Manager = inle_app::phases::Phase_Manager<crate::phases::Phase_Args>;

pub struct Game_State {
    pub should_quit: bool,
    pub env: inle_core::env::Env_Info,
    pub config: inle_cfg::config::Config,
    pub app_config: inle_app::app_config::App_Config,
    pub sleep_granularity: Option<Duration>,
    pub loggers: inle_diagnostics::log::Loggers,
    pub rng: inle_core::rand::Default_Rng,

    pub time: inle_core::time::Time,
    pub cur_frame: u64,
    pub prev_frame_time: std::time::Duration,

    pub frame_alloc: inle_alloc::temp::Temp_Allocator,

    pub window: inle_gfx::render_window::Render_Window_Handle,
    pub batches: inle_gfx::render::batcher::Batches,
    pub lights: inle_gfx::light::Lights,

    pub audio_system: inle_audio::audio_system::Audio_System,

    pub input: inle_input::input_state::Input_State,

    pub phys_world: inle_physics::phys_world::Physics_World,
    pub entities: crate::entity::Entity_Container,

    // XXX: ??
    pub default_font: inle_gfx::res::Font_Handle,

    pub engine_cvars: inle_app::app::Engine_CVars,

    pub ui: inle_ui::Ui_Context,

    pub phase_mgr: Phase_Manager,

    pub bg_music: inle_audio::audio_system::Sound_Handle,

    #[cfg(debug_assertions)]
    pub debug_systems: inle_app::debug::systems::Debug_Systems,
    #[cfg(debug_assertions)]
    pub fps_counter: inle_debug::fps::Fps_Counter,
}

pub struct Game_Resources {
    pub gfx: inle_gfx::res::Gfx_Resources,
    pub audio: inle_audio::res::Audio_Resources,
    pub shader_cache: inle_gfx::res::Shader_Cache,
}

pub struct Game_Args {
    pub starting_phase: inle_app::phases::Phase_Id,
}

//
// Init
//
pub fn parse_game_args(args: &[String]) -> Game_Args {
    use crate::phases::*;

    let mut game_args = Game_Args {
        starting_phase: Main_Menu::PHASE_ID,
    };

    let mut i = 1; // skip executable name
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--phase" => {
                if i < args.len() - 1 {
                    let phase = &args[i + 1];
                    i += 1;
                    match phase.as_str() {
                        "menu" => game_args.starting_phase = Main_Menu::PHASE_ID,
                        "game" => game_args.starting_phase = In_Game::PHASE_ID,
                        _ => fatal!("unexpected phase {}", phase),
                    }
                }
            }
            _ => {
                lwarn!("Ignoring command line argument {}", arg);
            }
        }
        i += 1;
    }

    game_args
}

pub fn internal_game_init(_args: &Game_Args) -> Box<Game_State> {
    use inle_core::env::Env_Info;

    let mut loggers = unsafe { inle_diagnostics::log::create_loggers() };
    inle_diagnostics::log::add_default_logger(&mut loggers);

    let env = Env_Info::gather().unwrap();
    let mut config = inle_cfg::Config::new_from_dir(&env.cfg_root);
    #[cfg(debug_assertions)]
    {
        inle_app::app::start_config_watch(&env, &mut config)
            .unwrap_or_else(|err| lerr!("Failed to start config watch: {}", err));
    }
    let sleep_granularity = inle_core::sleep::init_sleep().map_or_else(
        |err| {
            lerr!("Failed to init sleep: {:?}", err);
            None
        },
        Option::Some,
    );

    let app_config = {
        let win_width: Cfg_Var<i32> = Cfg_Var::new("engine/window/width", &config);
        let win_height: Cfg_Var<i32> = Cfg_Var::new("engine/window/height", &config);
        let win_title: Cfg_Var<String> = Cfg_Var::new("engine/window/title", &config);
        inle_app::app_config::App_Config {
            title: win_title.read(&config).clone(),
            target_win_size: (
                win_width.read(&config) as u32,
                win_height.read(&config) as u32,
            ),
        }
    };

    let mut window = {
        let window_create_args = inle_win::window::Create_Window_Args { vsync: true };
        let window = inle_win::window::create_window(
            &window_create_args,
            app_config.target_win_size,
            &app_config.title,
        );
        inle_gfx::render_window::create_render_window(window)
    };

    let batches = inle_gfx::render::batcher::Batches::default();
    let lights = inle_gfx::light::Lights::default();

    let mut input = inle_input::input_state::create_input_state(&env);
    inle_input::joystick::init_joysticks(&window, &env, &mut input.raw.joy_state);

    //inle_ui::init_ui(&mut engine_state.systems.ui, gres, &engine_state.env);

    let seed = inle_core::rand::new_random_seed().unwrap();
    let rng = inle_core::rand::new_rng_with_seed(seed);

    #[cfg(debug_assertions)]
    let debug_systems = inle_app::debug::systems::Debug_Systems::new(&config);

    let time = inle_core::time::Time::default();
    let frame_alloc =
        inle_alloc::temp::Temp_Allocator::with_capacity(inle_common::units::gigabytes(1));

    let engine_cvars = inle_app::app::create_engine_cvars(&config);

    let ui = inle_ui::Ui_Context::default();

    let phase_mgr = Phase_Manager::default();

    let audio_config = inle_audio::audio_system::Audio_System_Config {
        max_concurrent_sounds: 6,
    };
    let audio_system = inle_audio::audio_system::Audio_System::new(&audio_config);

    let phys_world = inle_physics::phys_world::Physics_World::default();
    let entities = crate::entity::Entity_Container::default();

    Box::new(Game_State {
        env,
        config,
        app_config,
        sleep_granularity,
        window,
        batches,
        lights,
        audio_system,
        loggers,
        time,
        prev_frame_time: Duration::default(),
        input,
        rng,
        cur_frame: 0,
        frame_alloc,
        should_quit: false,
        default_font: None,
        ui,
        phys_world,
        entities,
        engine_cvars,
        phase_mgr,
        bg_music: Sound_Handle::INVALID,
        #[cfg(debug_assertions)]
        debug_systems,
        #[cfg(debug_assertions)]
        fps_counter: inle_debug::fps::Fps_Counter::with_update_rate(&Duration::from_secs(1)),
    })
}

pub fn create_game_resources() -> Box<Game_Resources> {
    let gfx = inle_gfx::res::Gfx_Resources::new();
    let audio = inle_audio::res::Audio_Resources::new();
    let shader_cache = inle_gfx::res::Shader_Cache::new();
    Box::new(Game_Resources {
        gfx,
        audio,
        shader_cache,
    })
}

// Used to initialize game state stuff that needs resources
pub fn game_post_init(
    game_state: &mut Game_State,
    game_res: &mut Game_Resources,
    game_args: &Game_Args,
) {
    let font_name = inle_cfg::Cfg_Var::<String>::new("engine/debug/ui/font", &game_state.config);
    game_state.default_font = game_res.gfx.load_font(&inle_gfx::res::font_path(
        &game_state.env,
        font_name.read(&game_state.config),
    ));

    let enable_audio = inle_cfg::Cfg_Var::<bool>::new("game/audio/enable", &game_state.config);
    if enable_audio.read(&game_state.config) {
        let snd_buf = game_res.audio.load_sound(&inle_audio::res::sound_path(
            &game_state.env,
            "blockster_theme.ogg",
        ));
        game_state.bg_music = game_state.audio_system.play_sound(&game_res.audio, snd_buf);
        if let Some(snd) = game_state.audio_system.get_sound_mut(game_state.bg_music) {
            inle_audio::sound::set_sound_looping(snd, true);
        }
    }

    // DEBUG
    //   game_state.should_quit = true;

    inle_ui::init_ui(&mut game_state.ui, &mut game_res.gfx, &game_state.env);

    register_game_phases(game_state);
    let mut args = Phase_Args::new(game_state, game_res);
    game_state
        .phase_mgr
        .push_phase(game_args.starting_phase, &mut args);

    #[cfg(debug_assertions)]
    {
        debug::init_debug(game_state, game_res);
    }
}

pub fn register_game_phases(game_state: &mut Game_State) {
    use crate::phases;

    game_state.phase_mgr.register_phase(
        phases::Main_Menu::PHASE_ID,
        Box::new(phases::Main_Menu::new(&mut game_state.window)),
    );
    game_state.phase_mgr.register_phase(
        phases::In_Game::PHASE_ID,
        Box::new(phases::In_Game::new(&game_state.config)),
    );
}

pub fn start_frame(game_state: &mut Game_State) {
    inle_gfx::render_window::start_new_frame(&mut game_state.window);

    #[cfg(debug_assertions)]
    {
        debug::start_debug_frame(
            &mut game_state.debug_systems,
            &game_state.time,
            game_state.cur_frame,
        );
    }

    game_state.time.update();
}

pub fn end_frame(game_state: &mut Game_State) {
    #[cfg(debug_assertions)]
    {
        let refresh_rate = Cfg_Var::new("engine/debug/trace/refresh_rate", &game_state.config);
        debug::update_traces(
            refresh_rate,
            &mut game_state.debug_systems,
            &game_state.config,
            &game_state.time,
            game_state.cur_frame,
            &mut game_state.frame_alloc,
        );
    }

    unsafe {
        game_state.frame_alloc.dealloc_all();
    }
}

//
// Input
//
pub fn process_input(game_state: &mut Game_State, game_res: &mut Game_Resources) {
    let process_game_actions;
    #[cfg(debug_assertions)]
    {
        process_game_actions = game_state.debug_systems.console.lock().unwrap().status
            != inle_debug::console::Console_Status::Open;
    }
    #[cfg(not(debug_assertions))]
    {
        process_game_actions = true;
    }

    inle_input::input_state::update_raw_input(&mut game_state.window, &mut game_state.input.raw);
    inle_input::input_state::process_raw_input(
        &game_state.input.raw,
        &game_state.input.bindings,
        &mut game_state.input.processed,
        process_game_actions,
    );

    if handle_core_actions(&mut game_state.window, &mut game_state.input) {
        game_state.should_quit = true;
        return;
    }

    let actions = game_state.input.processed.game_actions.clone();
    let mut args = Phase_Args::new(game_state, game_res);
    game_state.phase_mgr.handle_actions(&actions, &mut args);
}

fn handle_core_actions(
    window: &mut inle_gfx::render_window::Render_Window_Handle,
    input: &mut inle_input::input_state::Input_State,
) -> bool {
    for action in &input.processed.core_actions {
        match action {
            Core_Action::Quit => return true,
            Core_Action::Resize(new_width, new_height) => {
                inle_gfx::render_window::resize_keep_ratio(window, *new_width, *new_height);
            }
            Core_Action::Focus_Lost => {
                input.raw.kb_state.modifiers_pressed = 0;
                inle_input::mouse::reset_mouse_state(&mut input.raw.mouse_state);
            }
            _ => unimplemented!(),
        }
    }
    false
}

//
// Update
//
pub fn update(game_state: &mut Game_State, game_res: &mut Game_Resources) {
    inle_gfx::render::batcher::clear_batches(&mut game_state.batches);

    let mut args = Phase_Args::new(game_state, game_res);
    let should_quit = game_state.phase_mgr.update(&mut args);
    if should_quit {
        game_state.should_quit = should_quit;
    }
}

//
// Render
//
pub fn render(game_state: &mut Game_State, game_res: &mut Game_Resources) {
    trace!("render");

    let cur_vsync = inle_win::window::has_vsync(&game_state.window);
    let desired_vsync = game_state.engine_cvars.vsync.read(&game_state.config);
    if cur_vsync != desired_vsync {
        inle_win::window::set_vsync(&mut game_state.window, desired_vsync);
    }

    let win = &mut game_state.window;
    let clear_color = inle_common::colors::color_from_hex_no_alpha(
        game_state.engine_cvars.clear_color.read(&game_state.config),
    );
    inle_gfx::render_window::set_clear_color(win, clear_color);
    inle_gfx::render_window::clear(win);

    let cam_xform = inle_math::transform::Transform2D::default();
    let draw_params = inle_gfx::render::batcher::Batcher_Draw_Params::default();
    inle_gfx::render::batcher::draw_batches(
        win,
        &game_res.gfx,
        &mut game_state.batches,
        &mut game_res.shader_cache,
        &cam_xform,
        &mut game_state.lights,
        draw_params,
        &mut game_state.frame_alloc,
    );

    inle_ui::draw_all_ui(win, &game_res.gfx, &mut game_state.ui);

    #[cfg(debug_assertions)]
    {
        debug::render_debug(
            &mut game_state.debug_systems,
            win,
            &game_state.input,
            &game_state.config,
            &mut game_state.frame_alloc,
            &mut game_state.time,
            &mut game_res.gfx,
        );
    }

    inle_win::window::display(win);
}
