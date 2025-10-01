use serde_json::json;

const WEHCAT_API:&str = "http://127.0.0.1:{port}/api/?type={type}";

const WECHAT_SET_VERSION:u8 = 35; //修改微信版本号
const WECHAT_CHECK_LOGIN:u8 = 0; //检测微信登录状态
const WECHAT_GET_PROFILE:u8 = 1; //获取个人信息
const WECHAT_GET_CONTACT:u8 = 15;//获取微信联系人列表
const WECHAT_CHATROOM_ADD_MEMBER:u8 = 28;//添加群成员
const WECHAT_CHATROOM_DEL_MEMBER:u8 = 27;//删除群成员
const WECHAT_CHATROOM_GET_MEMBER:u8 = 25;//查询群成员列表
const WECHAT_SET_REMARK:u8 = 24;//给异常好友添加备注

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

/// 获取微信联系人列表
pub async fn get_wechat_contact(port:u16) -> Result<serde_json::Value, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = WEHCAT_API
        .replace("{port}", &port.to_string())
        .replace("{type}", &WECHAT_GET_CONTACT.to_string());
    
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

/// 获取群成员列表
pub async fn chatroom_get_member(port:u16,chatroom_id:&str) -> Result<serde_json::Value, reqwest::Error>
{
    let client = reqwest::Client::new();
    let url = WEHCAT_API
        .replace("{port}", &port.to_string())
        .replace("{type}", &WECHAT_CHATROOM_GET_MEMBER.to_string());
    
    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "chatroom_id": chatroom_id
        }))
        .send()
        .await?;
    
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}

/// 将好友添加到群聊
pub async fn chatroom_add_member(port:u16,chatroom_id:&str,wxids:&[&str]) -> Result<serde_json::Value, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = WEHCAT_API
        .replace("{port}", &port.to_string())
        .replace("{type}", &WECHAT_CHATROOM_ADD_MEMBER.to_string());
    
    let response = client
        .post(&url)
        .json(&serde_json::json!({"chatroom_id":chatroom_id,
            "wxids":wxids
        }))
        .send()
        .await?;
    
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}

/// 将好友移出群聊
pub async fn chatroom_del_member(port:u16,chatroom_id:&str,wxids:&[&str]) -> Result<serde_json::Value, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = WEHCAT_API
        .replace("{port}", &port.to_string())
        .replace("{type}", &WECHAT_CHATROOM_DEL_MEMBER.to_string());
    
    let response = client
        .post(&url)
        .json(&serde_json::json!({"chatroom_id":chatroom_id,
            "wxids":wxids
        }))
        .send()
        .await?;
    
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}


/// 查询微信好友关系
/// 函数返回值: 
/// ### 包含所有异常好友的向量
pub async fn check_relation(port:u16,chatroom_id:&str,wxids:&[&str]) -> Result<serde_json::Value, ()> {
    let response_add_member = match chatroom_add_member(port, chatroom_id, wxids).await {
        Ok(res) => res,
        Err(_) => return Err(()),
    };
    if response_add_member["result"] != "OK" {
        return Err(());
    }
    //查询群成员列表判断是否有异常好友


    Ok(json!({"delete":[],"block":[]}))
}

pub async fn set_remark(port:u16,wxid:&str,remark:&str) -> Result<serde_json::Value, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = WEHCAT_API
        .replace("{port}", &port.to_string())
        .replace("{type}", &WECHAT_SET_REMARK.to_string());
    
    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "wxid": wxid,
            "remark": remark
        }))
        .send()
        .await?;
    
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}
