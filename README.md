<h2 align="center">WeFriends</h2>
<h4 align="center">Are we still friends?</h4>

---

WeFriends是一个**开源、免费、安全的微信好友检测工具**，快去看看有没有朋友偷偷删掉或者拉黑你

---

> [!NOTE]
>
> WeFriends仍处于beta版本阶段,可能出现问题,报告问题请到issues



### 为什么选择WeFriends?

WeFriends具有以下优点:
- 免费使用,无需购买
- **好友不会收到任何提示**
- 代码全部开源,安全可控
- 所有数据本地处理,不会上传您的隐私
- 检测速度快,支持不限量好友
- 基于微信pc hook,封号概率低

### 使用指南

1. 从Releases下载最新版本(或者自行编译 查看方法)
2. 解压后阅读txt文档里的说明操作,也要阅读软件内对话框里的提示

### 软件截图

![主界面](https://gitee.com/StrayMeteor3337/strayImg/raw/master/WFmainUI.png)

主界面

### 软件原理

WeFriends会向微信客户端进程注入一个dll模块,该模块会在内存特定位置修改微信程序的指令,实现hook操作——调用一个腾讯非开放的接口直接查询好友关系,并拦截接口返回值,其他功能原理也差不多

### 开源协议

使用**MIT开源许可**,您可以在遵守协议的前提下自由地获取、使用、修改、分发本软件并可用于商业活动

### 贡献

问渠那得清如许,为有源头活水来  您可以向本仓库提交PR来贡献代码

### 感谢
##### 本项目的所有hook功能均来自大佬ljc545w，没有他的开源项目[ComWeChatRobot](https://github.com/ljc545w/ComWeChatRobot)，就不会有WeFriends工具

