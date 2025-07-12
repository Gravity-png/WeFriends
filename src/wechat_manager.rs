use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::io;
use std::time::Duration;
use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use rand::Rng;
use tokio::time;

// 定义 Windows 类型别名（更清晰）
type DWORD = u32;
type BOOL = i32;

/// 结束微信进程
/// 确保启动微信之前没有正在运行的微信进程,避免出错
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
/// 注意,不要直接调用这个,调用login_wechat实现启动+注入hook
fn start_wechat() -> io::Result<()> {
    #[cfg(target_os = "windows")] {
        // 检查LocalAppData路径下的微信,正常这里应该存在微信
        if let Ok(local_app_data) = std::env::var("LocalAppData") {
            let wechat_path = Path::new(&local_app_data)
                .join("Tencent")
                .join("WeChat");
            
            // 检查版本目录是否存在
            let version_dir = wechat_path.join("[3.7.0.30]");
            if version_dir.exists() {
                let exe_path = wechat_path.join("WeChat.exe");
                if exe_path.exists() {
                    return Command::new(exe_path)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .creation_flags(0x00000008 | 0x08000000)
                        .spawn()
                        .map(|_| ());
                }
            }
        }

        // 检查Client目录下的微信
        let client_path = Path::new("Client").join("WeChat.exe");
        println!("客户端路径: {}",client_path.display());
        
        if client_path.exists() {
            println!("直接启动微信,可能之前安装失败");
            return Command::new(client_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .creation_flags(0x00000008 | 0x08000000)
                .spawn()
                .map(|_| ());
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

fn hook_wechat(pid: DWORD, port: i32) -> Result<i32> {
    unsafe {
        // 加载 DLL 文件（请确保 example.dll 在运行路径下）
        let lib = Library::new("wxdriver64.dll")?;
        // 获取函数指针
        let func: Symbol<unsafe extern "C" fn(DWORD, i32) -> BOOL> = lib.get(b"start_listen")?;

        // 设置参数并调用函数
        let result = func(pid, port);

        // 输出结果
        if result != 0 {
            println!("监听启动成功");
            Ok(port)
        } else {
            println!("监听启动失败");
            Err(anyhow::anyhow!("Hook微信失败,监听启动失败"))
        }
    }
}


/// 登录微信-启动+hook微信
pub async fn login_wechat() -> Result<u16> {
    start_wechat().context("启动微信失败,请重试")?;
    
    // 获取微信进程PID,如果未检测到微信进程就每秒重试一次
    let pid = loop {
        let output = Command::new("tasklist")
            .args(&["/FI", "IMAGENAME eq WeChat.exe", "/FO", "CSV", "/NH"])
            .output()
            .context("执行tasklist命令失败")?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().next() {
                if let Some(pid_str) = line.split(',').nth(1) {
                    if let Ok(pid) = pid_str.trim_matches('"').parse::<DWORD>() {
                        break pid;
                    }
                }
            }
        }
        time::sleep(Duration::from_secs(1)).await;
    };
    
    // 生成49153-65534之间的随机端口
    #[allow(deprecated)]
    let port: i32 = rand::thread_rng().gen_range(49153..=65534);
    
    // 非阻塞等待5秒
    time::sleep(Duration::from_secs(5)).await;

    let port = hook_wechat(pid,port).context("Hook微信失败,请重试,否则所有操作都将无效")?;
    Ok(port as u16)
}

use std::fs;
use std::path::Path;

pub async fn install_wechat() -> Result<()> {
    // 获取LocalAppData路径
    let local_app_data = std::env::var("LocalAppData")
        .context("无法获取LocalAppData路径")?;

    // 检查WeChatWin.dll是否存在
    println!("LocalAppData: {}",local_app_data);

    // 检查WeChatWin.dll是否存在
    let dll_path = Path::new(&local_app_data)
        .join("Tencent")
        .join("WeChat")
        .join("[3.7.0.30]")
        .join("WeChatWin.dll");
    
    println!("dll_path: {}", dll_path.display());

    if dll_path.exists() {
        return Ok(()); // 文件已存在，不执行操作
    }

    // 构建目标路径
    let target_path = Path::new(&local_app_data)
        .join("Tencent")
        .join("WeChat");

    println!("target_path: {}", target_path.display());
    

    // 删除WeChat目录（如果存在）
    if Path::new(&target_path).exists() {
        fs::remove_dir_all(&target_path)
            .context(format!("删除目录失败: {}", target_path.display()))?;
    }

    // 创建目标目录
    fs::create_dir_all(&target_path)
        .context(format!("创建目录失败: {}", target_path.display()))?;
    
    time::sleep(Duration::from_secs(3)).await;

    // 复制Client目录内容（如果存在）
    let client_path = Path::new("Client");
    if client_path.exists() {
        println!("copying");
        copy_dir_all(client_path, Path::new(&target_path))
            .context("复制文件失败")?;
        
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// 清空缓存,就是把install_wechat复制的微信客户端删了
pub fn clear_cache() -> Result<()> {
    // 获取LocalAppData路径
    let local_app_data = std::env::var("LocalAppData")
        .context("无法获取LocalAppData路径")?;

    // 构建目标路径
    let target_path = Path::new(&local_app_data)
        .join("Tencent")
        .join("WeChat");

    println!("尝试删除微信缓存目录: {}", target_path.display());
    
    // 删除WeChat目录（如果存在）
    if Path::new(&target_path).exists() {
        match fs::remove_dir_all(&target_path) {
            Ok(_) => println!("成功删除微信缓存目录"),
            Err(e) => {
                eprintln!("删除目录失败: {}", e);
                return Err(anyhow::anyhow!("删除目录失败: {}", e));
            }
        }
    } else {
        println!("微信缓存目录不存在，无需删除");
    }
    Ok(())
}
