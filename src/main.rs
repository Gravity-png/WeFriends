use egui;
use egui_modal::Modal;
use std::sync::Arc;

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

struct MyApp {
    nickname: String,
    wxid: String,
    abnormal_friends: Vec<(String, String)>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            nickname: "张三".to_owned(),
            wxid: "123456".to_owned(),
            abnormal_friends: vec![
                ("wxid_1".to_owned(), "拉黑".to_owned()),
                ("wxid_2".to_owned(), "删除".to_owned()),
                ("wxid_3".to_owned(), "对方账号异常".to_owned()),
                ("wxid_4".to_owned(), "检测失败".to_owned()),
                ("wxid_5".to_owned(), "---".to_owned()),
            ],
        }
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
                        // 启动WeChat.exe, 警告: 这会先杀死所有正在运行的微信进程
                        let _ = WeFriends::wechat_manager::kill_wechat();
                        let _ = WeFriends::wechat_manager::start_wechat();
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
            egui::containers::Frame::default()
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    // 列表标题
                    ui.heading("📋 异常好友列表");
                    ui.separator();

                    // 带滚动条的可扩展列表
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, true])
                        .max_height(ui.available_height())
                        .show(ui, |ui| {
                            egui::Grid::new("friends_grid")
                                .striped(true)
                                .spacing([20.0, 8.0])
                                .min_col_width(250.0)
                                .show(ui, |ui| {
                                    // 表头
                                    ui.strong("好友账号ID");
                                    ui.strong("好友状态");
                                    ui.end_row();

                                    for (friend_wxid, friend_status) in &self.abnormal_friends {
                                        ui.label(
                                            egui::RichText::new(friend_wxid)
                                                .text_style(egui::TextStyle::Body),
                                        );

                                        let (color, text) = match friend_status.as_str() {
                                            "已完成" => (
                                                egui::Color32::from_rgb(46, 204, 113),
                                                "✔ 已完成",
                                            ),
                                            "拉黑" => (
                                                egui::Color32::from_rgb(52, 152, 219),
                                                "拉黑",
                                            ),
                                            "删除" => (
                                                egui::Color32::from_rgb(231, 76, 60),
                                                "删除",
                                            ),
                                            _ => (egui::Color32::GRAY, friend_status.as_ref()),
                                        };

                                        ui.colored_label(color, text)
                                            .on_hover_text("双击查看详情");

                                        ui.end_row();
                                    }
                                });
                        });
                });
        });
    }

}
