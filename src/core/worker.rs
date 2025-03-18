use crate::auth::{
    oauth::{self, SpotifyAuthResponse, listen_for_callback, refresh_token},
    token_manager::{self, StoredTokens},
};
use log::{debug, error, info};
use std::thread;
use tokio::{
    runtime::Runtime,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

// Commands that can be sent to the background worker
#[derive(Debug)]
pub enum WorkerCommand {
    StartOAuth,
    TryLoadTokens,
    Shutdown,
}

// Responses from the background worker
#[derive(Debug)]
pub enum WorkerResponse {
    OAuthStarted { auth_url: String },
    OAuthComplete { auth_response: SpotifyAuthResponse },
    OAuthFailed { error: String },
    TokensLoaded { auth_response: SpotifyAuthResponse },
    NoStoredTokens,
}

pub struct Worker {
    command_sender: UnboundedSender<WorkerCommand>,
    response_receiver: UnboundedReceiver<WorkerResponse>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new() -> Self {
        let (command_sender, mut command_receiver) = mpsc::unbounded_channel();
        let (response_sender, response_receiver) = mpsc::unbounded_channel();

        let thread_handle = thread::spawn(move || {
            let runtime = Runtime::new().unwrap();

            runtime.block_on(async {
                loop {
                    match command_receiver.recv().await {
                        Some(WorkerCommand::StartOAuth) => {
                            let response_sender_clone = response_sender.clone();
                            runtime.spawn(async move {
                                let (auth_url, code_verifier) = oauth::get_spotify_auth_url();
                                // Send the auth URL back to the UI
                                response_sender_clone
                                    .send(WorkerResponse::OAuthStarted {
                                        auth_url: auth_url.clone(),
                                    })
                                    .unwrap();

                                let auth_response = listen_for_callback(code_verifier).await;

                                match auth_response {
                                    Ok(auth_response) => {
                                        debug!("OAuth complete: {:?}", auth_response);

                                        // Save tokens for later use
                                        let stored_tokens =
                                            StoredTokens::from(auth_response.clone());
                                        if let Err(e) = token_manager::save_tokens(&stored_tokens) {
                                            error!("Failed to save tokens: {}", e);
                                        }

                                        response_sender_clone
                                            .send(WorkerResponse::OAuthComplete { auth_response })
                                            .unwrap();
                                    }
                                    Err(e) => {
                                        response_sender_clone
                                            .send(WorkerResponse::OAuthFailed {
                                                error: e.to_string(),
                                            })
                                            .unwrap();
                                    }
                                }
                            });
                        }
                        Some(WorkerCommand::TryLoadTokens) => {
                            let response_sender_clone = response_sender.clone();
                            runtime.spawn(async move {
                                match token_manager::load_tokens() {
                                    Ok(tokens) => {
                                        if tokens.is_expired() {
                                            info!("Stored tokens are expired, refreshing...");
                                            // Try to refresh the token
                                            match refresh_token(&tokens.refresh_token).await {
                                                Ok(new_auth) => {
                                                    let new_tokens =
                                                        StoredTokens::from(new_auth.clone());
                                                    if let Err(e) =
                                                        token_manager::save_tokens(&new_tokens)
                                                    {
                                                        error!(
                                                            "Failed to save refreshed tokens: {}",
                                                            e
                                                        );
                                                    }

                                                    response_sender_clone
                                                        .send(WorkerResponse::TokensLoaded {
                                                            auth_response: new_auth,
                                                        })
                                                        .unwrap();
                                                }
                                                Err(e) => {
                                                    error!("Failed to refresh token: {}", e);
                                                    response_sender_clone
                                                        .send(WorkerResponse::NoStoredTokens)
                                                        .unwrap();
                                                }
                                            }
                                        } else {
                                            info!("Using stored valid tokens");
                                            response_sender_clone
                                                .send(WorkerResponse::TokensLoaded {
                                                    auth_response: tokens.to_auth_response(),
                                                })
                                                .unwrap();
                                        }
                                    }
                                    Err(e) => {
                                        debug!("No stored tokens found: {}", e);
                                        response_sender_clone
                                            .send(WorkerResponse::NoStoredTokens)
                                            .unwrap();
                                    }
                                }
                            });
                        }
                        Some(WorkerCommand::Shutdown) => {
                            break;
                        }
                        None => {
                            error!("Worker command channel closed unexpectedly");
                            break;
                        }
                    }
                }
            });
        });

        Self {
            command_sender,
            response_receiver,
            thread_handle: Some(thread_handle),
        }
    }

    pub fn start_oauth(&self) {
        self.command_sender.send(WorkerCommand::StartOAuth).unwrap();
    }

    pub fn try_load_tokens(&self) {
        self.command_sender
            .send(WorkerCommand::TryLoadTokens)
            .unwrap();
    }

    pub fn shutdown(&mut self) {
        self.command_sender.send(WorkerCommand::Shutdown).unwrap();
        self.thread_handle.take().unwrap().join().unwrap();
    }

    pub fn poll_responses(&mut self) -> Option<WorkerResponse> {
        self.response_receiver.try_recv().ok()
    }
}
