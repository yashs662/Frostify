pub mod login;
pub mod home;
pub mod chrome;
pub mod icon;
pub mod settings;
pub mod splitter;
pub mod tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Splash,
    Login,
    Home,
}
