use egui;
use egui_modal::Modal;
use std::sync::{Arc, Mutex};
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
                //PS: 这个字体文件貌似有点大，以后精简一下
                Arc::new(egui::FontData::from_static(include_bytes!("NotoSansSC-Regular.ttf"))),
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
    nickname: String,
    wxid: String,
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
            nickname: "张三".to_owned(),
            wxid: "123456".to_owned(),
            total_friends: 150,
            deleted_me: 5,
            blocked_me: 3,
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
                login_tip_dialog.body(ui, "请等待系统日志中输出hook成功的提示后再登录微信,不然会提示版本过低");
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
                        let _ = WeFriends::wechat_manager::kill_wechat();
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
                    ui.label(format!("昵称：{}", self.nickname));
                    ui.label(format!("账号ID：{}", self.wxid));
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
