use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::io;

pub fn kill_wechat() -> io::Result<()> {
    #[cfg(target_os = "windows")] {
        // 使用系统命令方式
        let output = Command::new("taskkill")
            .args(&["/F", "/IM", "WeChat.exe"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("没有找到进程") && !stderr.contains("not found") {
                println!("无法杀死微信进程,这不会影响正常使用");
                println!("错误信息: {}", stderr);
                return Err(io::Error::new(io::ErrorKind::Other, stderr.to_string()));
            }
        }
    }
    Ok(())
}

/// 启动微信
pub fn start_wechat() -> io::Result<()> {
    #[cfg(target_os = "windows")] {
        let paths = [
            r"D:\Program Files (x86)\Tencent\WeChat\WeChat.exe",
            r"D:\Program Files\Tencent\Weixin\WeChat.exe",
        ];

        for path in &paths {
            match Command::new(path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .creation_flags(0x00000008 | 0x08000000)
                .spawn() 
            {
                Ok(_) => return Ok(()),
                Err(_e) => continue,
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "WeChat executable not found"))
    }
    #[cfg(not(target_os = "windows"))] {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Only supported on Windows"))
    }
}

/// 重启微信
pub fn restart_wechat() -> io::Result<()> {
    kill_wechat()?;
    start_wechat()
}
