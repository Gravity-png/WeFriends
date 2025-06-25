pub mod wechat_manager;
pub mod wechat_controller;
pub mod util;

// 导出主要功能
pub use wechat_manager::{kill_wechat, login_wechat, restart_wechat};
pub use wechat_controller::{overwrite_wechat_version};
