// Library root. The binary only needs startup dependencies; rendering and
// input mapping stay private to the crate.
pub mod app;
pub mod cli;
pub mod config;
pub mod db;
pub mod notify;
mod ui;
mod vim_motion;
