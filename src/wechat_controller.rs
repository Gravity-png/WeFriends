const WEHCAT_API:&str = "http://127.0.0.1:{port}/api/?type={type}";

const WECHAT_SET_VERSION:u8 = 35; //修改微信版本号
const WECHAT_CHECK_LOGIN:u8 = 0; //检测微信登录状态
const WECHAT_GET_PROFILE:u8 = 1; //获取个人信息

/// 修改微信版本号,避免登录的时候提示版本过低
pub async fn overwrite_wechat_version(port:u16,wechat_version:&str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let url = WEHCAT_API
        .replace("{port}", &port.to_string())
        .replace("{type}", &WECHAT_SET_VERSION.to_string());
    
    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "version": wechat_version
        }))
        .send()
        .await?;
    
    if response.status().is_success() {
        //println!("Response: {}", &response.text().await?);
        Ok(())
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}

/// 检测微信是否登录
pub async fn check_wechat_login(port:u16) -> Result<bool, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = WEHCAT_API
        .replace("{port}", &port.to_string())
        .replace("{type}", &WECHAT_CHECK_LOGIN.to_string());
    
    let response = client
        .get(&url)
        .send()
        .await?;
    
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        let is_login = json["is_login"].as_i64().unwrap_or(0) == 1;
        Ok(is_login)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}

/// 获取微信个人信息
pub async fn get_wechat_profile(port:u16) -> Result<serde_json::Value, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = WEHCAT_API
        .replace("{port}", &port.to_string())
        .replace("{type}", &WECHAT_GET_PROFILE.to_string());
    
    let response = client
        .get(&url)
        .send()
        .await?;
    
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}
