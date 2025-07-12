use egui;
use egui_modal::Modal;
use tokio::time;
use serde_json::json;
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
    contact_list: Arc<Mutex<serde_json::Value>>,//ui更新时不会访问
    deleted_me_list: Arc<Mutex<serde_json::Value>>,//ui更新时不会访问
    blocked_me_list: Arc<Mutex<serde_json::Value>>,//ui更新时不会访问
    total_friends: Arc<Mutex<usize>>,
    deleted_me: Arc<Mutex<usize>>,
    blocked_me: Arc<Mutex<usize>>,
    logs: Arc<Mutex<Vec<String>>>,
    port: Arc<Mutex<u16>>,
    confirm_login: bool,
    can_check_relation: Arc<Mutex<bool>>,
    can_set_remark: Arc<Mutex<bool>>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            nickname: Arc::new(Mutex::new("微信未登录".to_owned())),
            wxid: Arc::new(Mutex::new("微信未登录".to_owned())),
            wxsign: Arc::new(Mutex::new("微信未登录".to_owned())),
            contact_list: Arc::new(Mutex::new(json!([]))),//好友列表
            /*
            contact_list元素json格式:
            {
                "wxNickName": "昵称",
                "wxNumber": "微信号",
                "wxRemark": "好友备注",
                "wxType": 3,
                "wxVerifyFlag": 0,
                "wxid": "wxid"
            }
             */
            deleted_me_list: Arc::new(Mutex::new(json!([]))),//删除我的人的列表
            blocked_me_list: Arc::new(Mutex::new(json!([]))),//拉黑我的人的列表
            total_friends: Arc::new(Mutex::new(0)),//好友列表的总数
            deleted_me: Arc::new(Mutex::new(0)),//删除我的人的列表的总数
            blocked_me: Arc::new(Mutex::new(0)),//拉黑我的人的列表总数
            logs: Arc::new(Mutex::new(vec![
                "欢迎使用WeFriends——开源、免费的微信好友关系检测工具".to_string(),
                "开发者:StrayMeteor3337".to_string(),
            ])),//日志列表
            port: Arc::new(Mutex::new(1)),//和hook模块通信的端口号,ip为127.0.0.1
            confirm_login: false,//登录微信对话框确认用的
            can_check_relation: Arc::new(Mutex::new(false)),//是否可以开始检测好友,获取好友列表成功之后为真,开始检测后为假
            can_set_remark: Arc::new(Mutex::new(false)),//是否可以添加备注,检测完毕后为真,添加开始后为假
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

/// 更新界面上列表元素的总数
/// update_total为真时,会更新好友总数,正式开始检测好友关系之后不要启用
/// 
/// 注意: 每次修改列表时都要调用,不然用户看不到
pub fn update_list_num_all(app: &mut MyApp, update_total:bool) {
    if update_total {
        *app.total_friends.lock().unwrap() = app.contact_list.lock().unwrap().as_array().map_or(0, |v| v.len());
    }
    *app.deleted_me.lock().unwrap() = app.deleted_me_list.lock().unwrap().as_array().map_or(0, |v| v.len());
    *app.blocked_me.lock().unwrap() = app.blocked_me_list.lock().unwrap().as_array().map_or(0, |v| v.len());
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let not_login_dialog = Modal::new(ctx, "not_login");
        // 构建模态窗口内容,此窗口在没有登录微信时点击"开始检测"按钮时弹出
        not_login_dialog.show(|ui| {
            not_login_dialog.title(ui, "错误");

            not_login_dialog.frame(ui, |ui| {
                not_login_dialog.body(ui, "请先登录微信或等待数据加载完成。");
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
                login_tip_dialog.body(ui, "点击确定将会安装微信到%LocalAppData%/Tencent/WeChat目录,在设置清除缓存即可卸载\n请等待系统日志中输出修改微信版本成功的提示后再登录微信,不然会提示版本过低");
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

        let check_tip_dialog = Modal::new(ctx, "check_tip");

        check_tip_dialog.show(|ui| {
            check_tip_dialog.title(ui, "提示");

            check_tip_dialog.frame(ui, |ui| {
                check_tip_dialog.body(ui, "检测已开始,微信聊天中会显示“你已添加了xxx”,但对方不会看到这条消息,之后会解决");
            });

            check_tip_dialog.buttons(ui, |ui| {
                if check_tip_dialog.button(ui, "确定").clicked() {
                    check_tip_dialog.close();
                }
            });
        });

        let check_finished_dialog = Modal::new(ctx, "check_finished");

        check_finished_dialog.show(|ui| {
            check_finished_dialog.title(ui, "检测完成,请去添加备注");

            check_finished_dialog.frame(ui, |ui| {
                check_finished_dialog.body(ui, "如果想保留检测结果,则必须点击添加备注按钮");
            });

            check_finished_dialog.buttons(ui, |ui| {
                if check_finished_dialog.button(ui, "我知道了").clicked() {
                    check_finished_dialog.close();
                }
                
            });
        });

        let all_finished_dialog = Modal::new(ctx, "all_finished");

        all_finished_dialog.show(|ui| {
            all_finished_dialog.title(ui, "大功告成,请仔细阅读之后的操作");

            all_finished_dialog.frame(ui, |ui| {
                all_finished_dialog.body(ui,
                     r##"在打开的pc微信中-"通讯录"页面-"通讯录管理"-标签 中新建一个标签(名称随意),并在顶部的搜索框里面搜索"删除我的人"(拉黑同理,搜索"拉黑我的人"),点击"昵称"左边的框来全选,最后添加标签,之后可批量删除或者查看这些人"##
                    );
            });

            all_finished_dialog.buttons(ui, |ui| {
                if all_finished_dialog.button(ui, "我知道了").clicked() {
                    all_finished_dialog.close();
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
                    
                    //                 shit hill by StrayMeteor3337
                    //                      |
                    //                      V
                    // 处理登录确认
                    if self.confirm_login {
                        self.confirm_login = false;
                        // 启动WeChat.exe, 警告: 这会先杀死所有正在运行的微信进程
                        let _ = WeFriends::wechat_manager::kill_wechat();
                        log_message(self,"复制并启动微信和hook可能需要一会,请耐心等待,不要现在登录微信");
                        let ctx = ctx.clone();
                        let app = self.clone();
                        
                        // 使用tokio运行时执行异步任务
                        std::thread::spawn(move || {
                            tokio::runtime::Runtime::new()
                                .unwrap()
                                .block_on(async {
                                    match WeFriends::wechat_manager::install_wechat().await {
                                        Ok(_) => {
                                            let mut app = app.clone();
                                            log_message(&mut app, "微信安装完成")
                                        }
                                        Err(e) => {
                                            let mut app = app.clone();
                                            log_message(&mut app, &format!("微信安装失败,将会直接启动: {}", e))
                                        }
                                    }

                                    match WeFriends::wechat_manager::login_wechat().await {
                                        Ok(port) => {
                                            let mut app = app.clone();
                                            *app.port.lock().unwrap() = port;
                                            log_message(&mut app, &format!("hook微信成功，监听端口: {}", port));

                                            //不手动更新的话要等半天才会显示日志
                                            ctx.request_repaint();

                                            // 修改微信版本号,同时也算测试和hook模块的通信
                                            time::sleep(Duration::from_secs(1)).await;
                                            if let Err(e) = WeFriends::wechat_controller::overwrite_wechat_version(port, "3.9.12.51").await {
                                                log_message(&mut app, &format!("覆写微信版本号失败,必须退出微信重试,否则你将无法登录: {}", e));
                                                ctx.request_repaint();
                                            } else {
                                                log_message(&mut app, "已修改微信版本号为3.9.12.51,请登录微信,如果还是提示版本过低,等待几分钟二维码刷新之后再登录");
                                                ctx.request_repaint();
                                                
                                                // 循环检测微信登录状态
                                                loop {
                                                    time::sleep(Duration::from_secs(1)).await;
                                                    match WeFriends::wechat_controller::check_wechat_login(port).await {
                                                        Ok(true) => {//如果已经登录,则获取账号信息
                                                            log_message(&mut app, "微信已登录");
                                                            ctx.request_repaint();
                                                            
                                                            //登录以后获取账号信息
                                                            match WeFriends::wechat_controller::get_wechat_profile(port).await {
                                                                Ok(profile) => {//如果获取账号信息成功,则将其存储到全局对象中并显示给用户,然后获取好友列表
                                                                    let nickname = profile["data"]["wxNickName"].as_str().unwrap_or("").to_string();
                                                                    let wxid = profile["data"]["wxId"].as_str().unwrap_or("").to_string();
                                                                    let wxsign = profile["data"]["wxSignature"].as_str().unwrap_or("").to_string();
                                                                    
                                                                    *app.nickname.lock().unwrap() = nickname;
                                                                    *app.wxid.lock().unwrap() = wxid;
                                                                    *app.wxsign.lock().unwrap() = wxsign;
                                                                    
                                                                    log_message(&mut app, "正在获取好友列表");
                                                                    ctx.request_repaint();

                                                                    loop {
                                                                        match WeFriends::wechat_controller::get_wechat_contact(port).await {//获取好友列表
                                                                            Ok(contact_list) => {//如果获取好友列表成功,从中提取出好友(具体原理看util.rs中的注释)
                                                                                let filtered_contacts = WeFriends::util::filter_wxid_items(
                                                                                    contact_list["data"].as_array().unwrap().to_vec()
                                                                                );
                                                                                if !filtered_contacts.is_empty() {
                                                                                    *app.contact_list.lock().unwrap() = serde_json::Value::Array(filtered_contacts);
                                                                                    //这个可不能忘了
                                                                                    update_list_num_all(&mut app, true);

                                                                                    log_message(&mut app, "[提示]好友总数应该比你手机上显示的少一个(因为不算你自己)");
                                                                                    ctx.request_repaint();

                                                                                    *app.can_check_relation.lock().unwrap() = true;
                                                                                    log_message(&mut app, "微信客户端进入主界面后即可开始检测好友关系");

                                                                                    break;
                                                                                    //shit hill到此结束
                                                                                } else {
                                                                                    log_message(&mut app, "好友列表为空，等待2秒后重试...");
                                                                                    ctx.request_repaint();
                                                                                    time::sleep(Duration::from_secs(2)).await;
                                                                                }
                                                                            }
                                                                            Err(e) => {
                                                                                log_message(&mut app, &format!("获取好友列表出错: {}", e));
                                                                                ctx.request_repaint();
                                                                                break;
                                                                            }
                                                                        }
                                                                    }

                                                                    ctx.request_repaint();
                                                                }
                                                                Err(e) => {
                                                                    log_message(&mut app, &format!("获取账号信息出错: {}", e));
                                                                    ctx.request_repaint();
                                                                }
                                                            }

                                                            break;
                                                        }
                                                        Ok(false) => continue,
                                                        Err(e) => {
                                                            log_message(&mut app, &format!("检测登录状态出错: {}", e));
                                                            ctx.request_repaint();
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
                        if *self.can_check_relation.lock().unwrap() {
                            //禁用开始检测,不然用户狂点就炸了
                            *self.can_check_relation.lock().unwrap() = false;

                            let ctx = ctx.clone();
                            let app = self.clone();

                            // 使用tokio运行时执行异步任务
                            std::thread::spawn(move || {
                                tokio::runtime::Runtime::new()
                                    .unwrap()
                                    .block_on(async {
                                        let mut app = app.clone();
                                        let contact_list = app.contact_list.lock().unwrap().clone();
                                        let port = *app.port.lock().unwrap();

                                        let total = app.total_friends.lock().unwrap().clone();
                                        let mut checked = 0;
                                        
                                        //遍历好友列表,查询与所有好友的关系
                                        for wxuser in contact_list.as_array().unwrap() {
                                            let wxid = wxuser["wxid"].as_str().unwrap();
                                            match WeFriends::wechat_controller::check_relation(port, wxid).await {
                                                Ok(status) => {
                                                    if status["result"] == "OK" {
                                                        if status["status"] == 176 {
                                                            //被删除
                                                            {
                                                                let mut deleted_list = app.deleted_me_list.lock().unwrap();
                                                                if let Some(arr) = deleted_list.as_array_mut() {
                                                                    arr.push(wxuser.clone());
                                                                }
                                                            }
                                                            update_list_num_all(&mut app, false);
                                                        }else if status["status"] == 181 {
                                                            //被拉黑
                                                            {
                                                                let mut blocked_list = app.blocked_me_list.lock().unwrap();
                                                                if let Some(arr) = blocked_list.as_array_mut() {
                                                                    arr.push(wxuser.clone());
                                                                }
                                                            }
                                                            update_list_num_all(&mut app, false);
                                                        }
                                                        //正常关系(对方账号异常也算正常关系)

                                                        checked+=1;
                                                        log_message(&mut app, &format!("正在检测第{}/{}个好友",checked,total));
                                                    } else {
                                                        log_message(&mut app, &format!("查询好友关系时出错: 请检查网络连接"));
                                                    }
                                                }
                                                Err(e) => {
                                                    log_message(&mut app, &format!("查询好友关系时出错: {}", e));
                                                    ctx.request_repaint();
                                                }
                                            }
                                        }
                                        //检测完毕,提示用户,这时候拉黑删除都已经添加到对应list了
                                        *app.can_set_remark.lock().unwrap() = true;
                                        check_finished_dialog.open();
                                    })
                            });
                        } else {
                            not_login_dialog.open();
                        }
                    }

                    // 按钮之间添加间距
                    ui.add_space(20.0);

                    // 添加备注按钮
                    let b2 = egui::Button::new("添加备注").min_size(button_size);
                    if ui.add(b2).clicked() {
                        // 处理点击事件
                        if *self.can_set_remark.lock().unwrap() {
                            // 禁用添加备注按钮
                            *self.can_set_remark.lock().unwrap() = false;
                            
                            let ctx = ctx.clone();
                            let app = self.clone();
                            
                            // 使用tokio运行时执行异步任务
                            std::thread::spawn(move || {
                                tokio::runtime::Runtime::new()
                                    .unwrap()
                                    .block_on(async {
                                        let mut app = app.clone();
                                        let port = *app.port.lock().unwrap();
                                        
                                        // 处理被删除的好友
                                        {
                                            let deleted_list = app.deleted_me_list.lock().unwrap().clone();
                                            let total = deleted_list.as_array().unwrap().len();
                                            let mut processed = 0;
                                            
                                            for user in deleted_list.as_array().unwrap() {
                                                processed += 1;
                                                let wxid = user["wxid"].as_str().unwrap();
                                                let nickname = user["wxNickName"].as_str().unwrap();
                                                let remark = user["wxRemark"].as_str().unwrap_or("");
                                                
                                                let new_remark = if remark.is_empty() {
                                                    format!("删除我的人-{}", nickname)
                                                } else {
                                                    format!("删除我的人-{}", remark)
                                                };
                                                
                                                match WeFriends::wechat_controller::set_remark(port, wxid, &new_remark).await {
                                                    Ok(_) => log_message(&mut app, &format!("标记删除: 已为第 {}/{} 添加备注", processed, total)),
                                                    Err(e) => log_message(&mut app, &format!("标记删除: 第 {}/{} 添加备注失败: {}", processed, total, e)),
                                                }
                                                ctx.request_repaint();
                                            }
                                        }
                                        
                                        // 处理被拉黑的好友
                                        {
                                            let blocked_list = app.blocked_me_list.lock().unwrap().clone();
                                            let total = blocked_list.as_array().unwrap().len();
                                            let mut processed = 0;
                                            
                                            for user in blocked_list.as_array().unwrap() {
                                                processed += 1;
                                                let wxid = user["wxid"].as_str().unwrap();
                                                let nickname = user["wxNickName"].as_str().unwrap();
                                                let remark = user["wxRemark"].as_str().unwrap_or("");
                                                
                                                let new_remark = if remark.is_empty() {
                                                    format!("拉黑我的人-{}", nickname)
                                                } else {
                                                    format!("拉黑我的人-{}", remark)
                                                };
                                                
                                                match WeFriends::wechat_controller::set_remark(port, wxid, &new_remark).await {
                                                    Ok(_) => log_message(&mut app, &format!("标记拉黑: 已为第 {}/{} 添加备注", processed, total)),
                                                    Err(e) => log_message(&mut app, &format!("标记拉黑: 第 {}/{} 添加备注失败: {}", processed, total, e)),
                                                }
                                                ctx.request_repaint();
                                            }
                                        }
                                        
                                        log_message(&mut app, "备注添加完成");
                                        ctx.request_repaint();
                                    });
                            });

                            //添加完成备注,提示用户
                            all_finished_dialog.open();
                        } else {
                            log_message(self, "现在不能添加备注");
                        }
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
                ui.label(egui::RichText::new(format!("好友总数: {}", *self.total_friends.lock().unwrap())).size(24.0));
                ui.label(egui::RichText::new(format!("删除我的人: {}", *self.deleted_me.lock().unwrap())).size(24.0));
                ui.label(egui::RichText::new(format!("拉黑我的人: {}", *self.blocked_me.lock().unwrap())).size(24.0));
                
                // 日志控制台
                ui.separator();
                ui.add_space(10.0);
                ui.heading("系统日志: ");
                let scroll_area = egui::ScrollArea::vertical()
                    .max_height(ui.available_height());
                
                scroll_area.show(ui, |ui| {
                    if let Ok(logs) = self.logs.lock() {
                        for log in logs.iter() {
                            ui.label(log);
                        }
                    }
                    // 自动滚动到底部
                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                });
            });
        });
    }

}
