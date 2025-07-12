use serde_json::Value;
/* 
这里提一下微信的机制,不知道会不会有人看: 
微信你手机的"微信"页面里面所有的内容都有wxid,这是微信识别这些内容的唯一标识

>>>>好友的wxid不是微信号,wxid是你账号刚注册时的"微信号"一栏里面显示的内容,你之后改过的微信号在微信后台是另一个字段,wxid在账号注册时随机生成且不可修改,这个才是你账号的唯一凭证,微信号本质就是一个不可重复且可以用来登录和加好友的昵称

微信中好友的wxid以"wxid_"开头、公众号以"gh_"开头 、群聊以"@chatroom"结尾
还有, wxid为weixin是微信团队, filehelper是文件传输助手 ,qqmail是qq邮箱提醒的聊天, fmessage是朋友推荐消息, medianote是语音记事本, floatbottle是漂流瓶(这个功能不是早没了吗,为什么还留着)
而微信支付,微信运动是由公众号提供的内容所以wxid格式和普通公众号相同

提一些无关的:
腾讯你明明都把所有群聊编号了而且还都加载到客户端内存里了,为什么不搞一个群聊列表功能???
还必须“保存到通讯录”才能看到,不小心删了聊天记录群都找不到,你是故意恶心人吗!!!!???
*/


/// 用来判断wxid是否是好友, 微信中 好友的wxid以"wxid_"开头、公众号以"gh_"开头 、群聊以"@chatroom"结尾
/// 过滤包含"wxid"字段的JSON数组，保留以"wxid_"开头的元素
/// 
/// # 参数
/// * `items` - 包含JSON对象的数组
/// 
/// # 返回
/// 过滤后的数组
pub fn filter_wxid_items(items: Vec<Value>) -> Vec<Value> {
    items.into_iter()
        .filter(|item| {
            item.get("wxid")
                .and_then(Value::as_str)
                .map_or(false, |s| s.starts_with("wxid_"))
        })
        .collect()
}
