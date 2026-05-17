use crate::api::{self, HomeData};
use crate::auth::oauth::{self, SpotifyAuthResponse, listen_for_callback, refresh_token};
use crate::auth::token_manager::{self, StoredTokens};
use frostify_gfx::WakeHandle;
use log::{debug, error, info, warn};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self as tmpsc, UnboundedSender};

#[derive(Debug)]
pub enum WorkerCommand {
    StartOAuth,
    TryLoadTokens,
    FetchHome { access_token: String },
    #[allow(dead_code)]
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum WorkerResponse {
    OAuthStarted { auth_url: String },
    OAuthComplete { auth: SpotifyAuthResponse },
    OAuthFailed { error: String },
    TokensLoaded { auth: SpotifyAuthResponse },
    NoStoredTokens,
    HomeData { data: HomeData },
}

pub struct Worker {
    cmd_tx: UnboundedSender<WorkerCommand>,
    resp_rx: Receiver<WorkerResponse>,
    #[allow(dead_code)]
    handle: Option<thread::JoinHandle<()>>,
}

#[derive(Clone)]
struct Responder {
    tx: Sender<WorkerResponse>,
    wake: Arc<WakeHandle>,
}

impl Responder {
    fn send(&self, r: WorkerResponse) {
        let _ = self.tx.send(r);
        self.wake.wake();
    }
}

impl Worker {
    pub fn new(wake: Arc<WakeHandle>) -> Self {
        let (cmd_tx, mut cmd_rx) = tmpsc::unbounded_channel::<WorkerCommand>();
        let (resp_tx, resp_rx): (Sender<WorkerResponse>, Receiver<WorkerResponse>) = channel();
        let resp = Responder { tx: resp_tx, wake };

        let handle = thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                while let Some(cmd) = cmd_rx.recv().await {
                    match cmd {
                        WorkerCommand::StartOAuth => spawn_oauth(resp.clone()),
                        WorkerCommand::TryLoadTokens => spawn_try_load(resp.clone()),
                        WorkerCommand::FetchHome { access_token } => {
                            spawn_fetch_home(resp.clone(), access_token)
                        }
                        WorkerCommand::Shutdown => break,
                    }
                }
            });
        });

        Self { cmd_tx, resp_rx, handle: Some(handle) }
    }

    pub fn start_oauth(&self) {
        let _ = self.cmd_tx.send(WorkerCommand::StartOAuth);
    }
    pub fn try_load_tokens(&self) {
        let _ = self.cmd_tx.send(WorkerCommand::TryLoadTokens);
    }
    pub fn fetch_home(&self, access_token: String) {
        let _ = self.cmd_tx.send(WorkerCommand::FetchHome { access_token });
    }
    pub fn poll(&self) -> Option<WorkerResponse> {
        self.resp_rx.try_recv().ok()
    }
    #[allow(dead_code)]
    pub fn shutdown(mut self) {
        let _ = self.cmd_tx.send(WorkerCommand::Shutdown);
        if let Some(h) = self.handle.take() { let _ = h.join(); }
    }
}

fn spawn_oauth(resp: Responder) {
    tokio::spawn(async move {
        let (url, verifier) = oauth::get_spotify_auth_url();
        resp.send(WorkerResponse::OAuthStarted { auth_url: url });
        match listen_for_callback(verifier).await {
            Ok(auth) => {
                debug!("OAuth complete");
                let stored = StoredTokens::from(auth.clone());
                if let Err(e) = token_manager::save_tokens(&stored) {
                    error!("save tokens: {e}");
                }
                resp.send(WorkerResponse::OAuthComplete { auth });
            }
            Err(e) => {
                resp.send(WorkerResponse::OAuthFailed { error: e.to_string() });
            }
        }
    });
}

fn spawn_fetch_home(resp: Responder, access_token: String) {
    tokio::spawn(async move {
        let (profile, playlists, recent) = tokio::join!(
            api::get_me(&access_token),
            api::get_playlists(&access_token),
            api::get_recently_played(&access_token),
        );
        let mut data = HomeData::default();
        match profile {
            Ok(p) => data.profile = Some(p),
            Err(e) => warn!("get_me failed: {e}"),
        }
        match playlists {
            Ok(ps) => data.playlists = ps,
            Err(e) => warn!("get_playlists failed: {e}"),
        }
        match recent {
            Ok(rs) => data.recent = rs,
            Err(e) => warn!("get_recently_played failed: {e}"),
        }
        info!(
            "home data: profile={} playlists={} recent={}",
            data.profile.is_some(),
            data.playlists.len(),
            data.recent.len()
        );
        resp.send(WorkerResponse::HomeData { data });
    });
}

fn spawn_try_load(resp: Responder) {
    tokio::spawn(async move {
        match token_manager::load_tokens() {
            Ok(tokens) => {
                if tokens.is_expired() {
                    info!("refreshing expired token");
                    match refresh_token(&tokens.refresh_token).await {
                        Ok(auth) => {
                            let _ = token_manager::save_tokens(&StoredTokens::from(auth.clone()));
                            resp.send(WorkerResponse::TokensLoaded { auth });
                        }
                        Err(e) => {
                            error!("refresh failed: {e}");
                            resp.send(WorkerResponse::NoStoredTokens);
                        }
                    }
                } else {
                    resp.send(WorkerResponse::TokensLoaded {
                        auth: tokens.to_auth_response(),
                    });
                }
            }
            Err(e) => {
                debug!("no stored tokens: {e}");
                resp.send(WorkerResponse::NoStoredTokens);
            }
        }
    });
}
