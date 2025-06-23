extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        
        // 设置清单文件
        res.set_manifest_file("app.manifest");
        
        // 可选：设置图标
        // res.set_icon("icon.ico");
        
        // 可选：设置版本信息
        //res.set("ProductName", "My Application");
        res.set("FileDescription", "WeCheck主程序");
        // res.set("LegalCopyright", "Copyright © 2023");
        
        res.compile().unwrap();
    }
}