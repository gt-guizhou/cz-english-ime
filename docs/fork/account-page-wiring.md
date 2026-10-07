# 「账号/同步」设置页接线蓝图（Windows 设置程序）

> 状态：地基已就绪（[sync] 分节进 Config + 模板，qingjian-sync crate 已发布）；
> 本页是最后一块拼图。实现时照此蓝图，参照 `cloud.rs` 的 TestConnection 异步范式。

## 要动的文件（apps/windows/settings/src/panel/）

1. `message.rs`：加 `SyncEnabled(bool)`、`SyncAccount(String)`、`SyncPassword(String)`、
   `SyncLogin`、`SyncLoginDone(Result<String,String>)`、`SyncNow`、`SyncDone(Result<String,String>)`。
2. `pages/account.rs`（新建）：`view(settings, context)` —— 账号/密码框（参考 general.rs 的输入行）、
   「登录」按钮（→SyncLogin）、「立即同步」按钮（→SyncNow）、状态行（登录结果/上次同步与 added/skipped/pending）。
3. `pages/mod.rs`：注册 `account`；`component.rs`：导航项「账号」+ Message 分发
   （登录/同步在后台线程跑，完成后发 `*Done`；参考 cloud 页 TestConnection → CloudTestDone 的现成线程模式）。
4. `component.rs` 落盘：SyncEnabled/账号写进 `config.sync`（token 写 `.env` 或 config，对齐 cloud 页密钥口径）。

## 登录接口（实现时从 kaola-english `api/controller/User.php` 核实字段名）

- `POST {base}/api/user/login`，FastAdmin 惯例表单 `account`+`password`，成功 `code=1`，
  取 `data.userinfo.token` 存入 `[sync]`（或 `CZIME_SYNC_TOKEN` 写 `%APPDATA%\CZEnglishIME\.env`）。

## 同步调用

- 路径：`%APPDATA%\CZEnglishIME\user-vocab.tsv` 与同目录 `sync-state.json`
  （main.rs:14 的 user_dir 解析已存在；settings 与 server 同目录）。
- `qingjian_sync::sync_all(&config.sync, &vocab, &state)` → `SyncOutcome{added,skipped,pending,category_id}`；
  `pending>0` 时提示「还有 N 个新词下次同步继续」。
