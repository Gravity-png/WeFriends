use egui;
use egui_modal::Modal;
use tokio::time;
use std::{sync::{Arc, Mutex}, time::Duration};
use chrono::Local;

fn main() -> Result<(), eframe::Error> {
    // 创建视口选项，设置视口的内部大小为800x600像素
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "WeFriends",
        options,
        Box::new(|_cc| {
            // 自定义字体加载
            let mut fonts = egui::FontDefinitions::default();

            // 添加中文字体，并使用 Arc 包装
            fonts.font_data.insert(
                "my_font".to_owned(),
                //PS: 源文件在https://fonts.google.com/noto/specimen/Noto+Sans+SC ,这个是精简过的,原来的太大了
                Arc::new(egui::FontData::from_static(include_bytes!("NotoSansSC-Regular-3500.ttf"))),
            );

            // 设置默认字体
            fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
                .insert(0, "my_font".to_owned());

            // 将自定义字体应用到上下文
            _cc.egui_ctx.set_fonts(fonts);

            // 返回一个 Result 类型
            Ok(Box::new(MyApp::default()) as Box<dyn eframe::App>)
        }),
    )
}


#[derive(Clone)]
pub struct MyApp {
    nickname: Arc<Mutex<String>>,
    wxid: Arc<Mutex<String>>,
    wxsign: Arc<Mutex<String>>,
    total_friends: usize,
    deleted_me: usize,
    blocked_me: usize,
    logs: Arc<Mutex<Vec<String>>>,
    port: Option<u16>,
    confirm_login: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            nickname: Arc::new(Mutex::new("微信未登录".to_owned())),
            wxid: Arc::new(Mutex::new("微信未登录".to_owned())),
            wxsign: Arc::new(Mutex::new("微信未登录".to_owned())),
            total_friends: 0,
            deleted_me: 0,
            blocked_me: 0,
            logs: Arc::new(Mutex::new(vec![
                "欢迎使用WeFriends——开源、免费的微信好友关系检测工具".to_string(),
                "开发者:StrayMeteor3337".to_string(),
            ])),
            port: None,
            confirm_login: false,
        }
    }
}


/// 日志函数，向日志栏输出日志信息
pub fn log_message(app: &mut MyApp, message: &str) {
    let now = Local::now();
    let log_entry = format!("[{}] {}", now.format("%Y-%m-%d %H:%M:%S"), message);
    let mut logs = app.logs.lock().unwrap();
    logs.push(log_entry);
    
    // 限制日志数量
    if logs.len() > 100 {
        logs.remove(0);
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let not_login_dialog = Modal::new(ctx, "not_login");
        // 构建模态窗口内容,此窗口在没有登录微信时点击"开始检测"按钮时弹出
        not_login_dialog.show(|ui| {
            not_login_dialog.title(ui, "错误");

            not_login_dialog.frame(ui, |ui| {
                not_login_dialog.body(ui, "请先登录微信。");
            });

            not_login_dialog.buttons(ui, |ui| {
                if not_login_dialog.button(ui, "关闭").clicked() {
                    not_login_dialog.close();
                }
            });
        });

        // 构建模态窗口内容,此窗口在启动微信之前提醒用户注意事项
        let login_tip_dialog = Modal::new(ctx, "login_tip");
        
        login_tip_dialog.show(|ui| {
            login_tip_dialog.title(ui, "注意");

            login_tip_dialog.frame(ui, |ui| {
                login_tip_dialog.body(ui, "请等待系统日志中输出修改微信版本成功的提示后再登录微信,不然会提示版本过低");
            });

            login_tip_dialog.buttons(ui, |ui| {
                if login_tip_dialog.button(ui, "确定").clicked() {
                    self.confirm_login = true;
                    login_tip_dialog.close();
                }
                if login_tip_dialog.button(ui, "取消").clicked() {
                    login_tip_dialog.close();
                }
            });
        });


        // 顶部标题
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("WeFriends主程序——微信好友检测");
        });

        // 底部按钮面板
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            //Frame是为了设置按钮的外边距
            egui::containers::Frame::default()
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui|{
                // 设置按钮大小
                let button_size = egui::vec2(120.0, 30.0); // 宽度: 120px, 高度: 30px

                // 设置水平布局，并将按钮组水平居中
                let layout = egui::Layout::left_to_right(egui::Align::Center);
                ui.with_layout(layout, |ui| {
                    // 启动微信按钮
                    let b1 = egui::Button::new("登录微信").min_size(button_size);
                    if ui.add(b1).clicked() {
                        login_tip_dialog.open();
                    }
                    
                    // 处理登录确认
                    if self.confirm_login {
                        self.confirm_login = false;
                        // 启动WeChat.exe, 警告: 这会先杀死所有正在运行的微信进程
                        //let _ = WeFriends::wechat_manager::kill_wechat();
                        log_message(self,"启动hook可能需要一会,请耐心等待");
                        let ctx = ctx.clone();
                        let app = self.clone();
                        
                        // 使用tokio运行时执行异步任务
                        std::thread::spawn(move || {
                            tokio::runtime::Runtime::new()
                                .unwrap()
                                .block_on(async {
                                    match WeFriends::wechat_manager::login_wechat().await {
                                        Ok(port) => {
                                            let mut app = app.clone();
                                            app.port = Some(port);
                                            log_message(&mut app, &format!("hook微信成功，监听端口: {}", port));

                                            //不手动更新的话要等半天才会显示日志
                                            ctx.request_repaint();

                                            // 修改微信版本号,同时也算测试和hook模块的通信
                                            time::sleep(Duration::from_secs(1)).await;
                                            if let Err(e) = WeFriends::wechat_controller::overwrite_wechat_version(port, "3.9.12.51").await {
                                                log_message(&mut app, &format!("覆写微信版本号失败,必须退出微信重试,否则你将无法登录: {}", e));
                                                ctx.request_repaint();
                                            } else {
                                                log_message(&mut app, "已修改微信版本号为3.9.12.51,请登录微信");
                                                ctx.request_repaint();
                                                
                                                // 循环检测微信登录状态
                                                loop {
                                                    time::sleep(Duration::from_secs(1)).await;
                                                    match WeFriends::wechat_controller::check_wechat_login(port).await {
                                                        Ok(true) => {
                                                            log_message(&mut app, "微信已登录");
                                                            ctx.request_repaint();
                                                            
                                                            //登录以后获取账号信息
                                                            match WeFriends::wechat_controller::get_wechat_profile(port).await {
                                                                Ok(profile) => {
                                                                    let nickname = profile["data"]["wxNickName"].as_str().unwrap_or("").to_string();
                                                                    let wxid = profile["data"]["wxId"].as_str().unwrap_or("").to_string();
                                                                    let wxsign = profile["data"]["wxSignature"].as_str().unwrap_or("").to_string();
                                                                    
                                                                    *app.nickname.lock().unwrap() = nickname;
                                                                    *app.wxid.lock().unwrap() = wxid;
                                                                    *app.wxsign.lock().unwrap() = wxsign;
                                                                    
                                                                    
                                                                    log_message(&mut app, "获取账号信息成功");

                                                                    ctx.request_repaint();
                                                                }
                                                                Err(e) => {
                                                                    log_message(&mut app, &format!("获取账号信息出错: {}", e));
                                                                }
                                                            }

                                                            break;
                                                        }
                                                        Ok(false) => continue,
                                                        Err(e) => {
                                                            log_message(&mut app, &format!("检测登录状态出错: {}", e));
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            ctx.request_repaint();
                                        }
                                        Err(e) => {
                                            let mut app = app.clone();
                                            log_message(&mut app, &format!("hook微信出错: {}", e));
                                            ctx.request_repaint();
                                        }
                                    }
                                });
                        });
                    }

                    // 按钮之间添加间距
                    ui.add_space(20.0);

                    // 开始检测按钮
                    let b2 = egui::Button::new("开始检测").min_size(button_size);
                    if ui.add(b2).clicked() {
                        // 处理点击事件
                        not_login_dialog.open();
                    }

                    // 按钮之间添加间距
                    ui.add_space(20.0);

                    // 退出登录按钮
                    let b2 = egui::Button::new("退出登录").min_size(button_size);
                    if ui.add(b2).clicked() {
                        // 处理点击事件
                    }
                });
            })
        });

        // 左侧面板（直接挂载到 ctx）
        egui::SidePanel::left("user_info_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("👤 用户信息");
                    ui.separator();
                    ui.add_space(10.0);
                    ui.label(format!("昵称：{}", self.nickname.lock().unwrap()));
                    ui.label(format!("账号：{}", self.wxid.lock().unwrap()));
                    ui.label(format!("签名：{}", self.wxsign.lock().unwrap()))
                });
            });

        // 主内容区域（右侧）
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("📊 好友统计");
                ui.separator();
                ui.add_space(10.0);
                ui.label(egui::RichText::new(format!("好友总数: {}", self.total_friends)).size(24.0));
                ui.label(egui::RichText::new(format!("删除我的人: {}", self.deleted_me)).size(24.0));
                ui.label(egui::RichText::new(format!("拉黑我的人: {}", self.blocked_me)).size(24.0));
                
                // 日志控制台
                ui.separator();
                ui.add_space(10.0);
                ui.heading("系统日志: ");
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        if let Ok(logs) = self.logs.lock() {
                            for log in logs.iter() {
                                ui.label(log);
                            }
                        }
                    });
            });
        });
    }

}
