pub mod wechat_manager;
pub mod util;

// 重新导出主要功能
pub use wechat_manager::{kill_wechat, start_wechat, restart_wechat};
