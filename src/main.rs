#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod api;
mod auth;
mod constants;
mod errors;
mod ui;
mod worker;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use frostify_gfx::App;

use crate::api::HomeData;
use crate::auth::oauth::SpotifyAuthResponse;
use crate::ui::View;
use crate::worker::{Worker, WorkerResponse};

const W: u32 = 1280;
const H: u32 = 780;

#[derive(Default)]
struct AppState {
    view: Cell<View>,
    auth: RefCell<Option<SpotifyAuthResponse>>,
    home: RefCell<HomeData>,
}

impl Default for View {
    fn default() -> Self { View::Splash }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wgpu_hal=warn,wgpu_core=warn,frostify=debug"),
    )
    .init();

    let state = Rc::new(AppState::default());
    if std::env::var_os("FROSTIFY_FORCE_HOME").is_some() {
        state.view.set(View::Home);
    }

    let mut app = App::new("Frostify", W, H).decorations(false).capture_from_env();
    let icons = std::rc::Rc::new(ui::icon::load_all(&mut app));
    let rebuild = app.rebuild_token();
    let worker = Rc::new(Worker::new(app.wake_handle()));
    worker.try_load_tokens();

    let on_login: Rc<dyn Fn()> = {
        let worker = worker.clone();
        Rc::new(move || worker.start_oauth())
    };

    let app = {
        let state = state.clone();
        let on_login = on_login.clone();
        let icons = icons.clone();
        app.scene(move |s| match state.view.get() {
            View::Splash | View::Login => {
                let checking = matches!(state.view.get(), View::Splash);
                ui::login::build(s, &icons, on_login.clone(), checking)
            }
            View::Home => ui::home::build(s, &icons, &state.home.borrow()),
        })
    };

    let state_for_frame = state.clone();
    let worker_for_frame = worker.clone();
    let rebuild_for_frame = rebuild.clone();
    let app = app.on_frame(move |_ctx, _tl, _now| {
        while let Some(resp) = worker_for_frame.poll() {
            handle_worker_response(
                &state_for_frame,
                &rebuild_for_frame,
                &worker_for_frame,
                resp,
            );
        }
    });

    app.run()
}

fn handle_worker_response(
    state: &Rc<AppState>,
    rebuild: &Rc<Cell<bool>>,
    worker: &Rc<Worker>,
    resp: WorkerResponse,
) {
    match resp {
        WorkerResponse::OAuthStarted { auth_url } => {
            log::info!("opening browser for OAuth");
            if let Err(e) = webbrowser::open(&auth_url) {
                log::error!("open browser: {e}");
            }
        }
        WorkerResponse::OAuthComplete { auth } | WorkerResponse::TokensLoaded { auth } => {
            log::info!("auth ok — switching to Home");
            worker.fetch_home(auth.access_token.clone());
            *state.auth.borrow_mut() = Some(auth);
            if state.view.get() != View::Home {
                state.view.set(View::Home);
                rebuild.set(true);
            }
        }
        WorkerResponse::OAuthFailed { error } => {
            log::error!("OAuth failed: {error}");
            if state.view.get() != View::Login {
                state.view.set(View::Login);
                rebuild.set(true);
            }
        }
        WorkerResponse::NoStoredTokens => {
            log::info!("no stored tokens — showing Login");
            if state.view.get() != View::Login {
                state.view.set(View::Login);
                rebuild.set(true);
            }
        }
        WorkerResponse::HomeData { data } => {
            log::info!(
                "home data ready: playlists={} recent={}",
                data.playlists.len(),
                data.recent.len()
            );
            *state.home.borrow_mut() = data;
            rebuild.set(true);
        }
    }
}
