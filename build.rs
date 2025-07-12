extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        
        // 设置清单文件
        res.set_manifest_file("app.manifest");
        
        // 可选：设置图标
        // res.set_icon("icon.ico");
        
        res.set("ProductName", "WeFriends开源微信好友检测工具");
        res.set("ProductVersion", "0.1.0");
        
        res.set("FileDescription", "WeFriends主程序");
        res.set("FileVersion", "0.1.0.0");

        res.set("LegalCopyright", "Copyright © 2025 StrayMeteor3337");
        
        res.compile().unwrap();
    }
}