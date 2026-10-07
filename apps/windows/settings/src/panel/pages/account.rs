//! 「账号」页：疯狂听抄账号登录与生词同步。登录拿 token 写进 `[sync]`；
//! 「立即同步」在后台线程跑 `qingjian_sync::sync_all`（生词本 → 服务端个人词本），结果显示在页内。

use qingjian_sync::SyncConfig;
use windows_reactor::*;

use crate::panel::controls::{field, labeled, note, page};
use crate::panel::{Message, Settings};

/// 登录：`POST /api/user/login`（表单 `account`/`password`），成功取 `data.userinfo.token`。
pub(crate) fn run_login(
    config: &SyncConfig,
    account: &str,
    password: &str,
) -> Result<String, String> {
    let url = format!("{}/api/user/login", config.base_url.trim_end_matches('/'));
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_millis(config.timeout_ms))
        .build()
        .map_err(|error| error.to_string())?;
    let value: serde_json::Value = client
        .post(&url)
        // JSON 体：TP5 的 post() 会解析 application/json（仓库备注 25 证实），字符串字段不受全局 filter 影响
        .json(&serde_json::json!({ "account": account, "password": password }))
        .send()
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.json())
        .map_err(|error| error.to_string())?;
    if value["code"].as_i64() != Some(1) {
        return Err(value["msg"].as_str().unwrap_or("登录失败").to_owned());
    }
    value["data"]["userinfo"]["token"]
        .as_str()
        .map(|token| token.to_owned())
        .ok_or_else(|| "响应里没有 token".to_owned())
}

/// 同步一次：生词本与水位都在用户数据目录（`%APPDATA%\CZEnglishIME`）。
pub(crate) fn run_sync(config: &SyncConfig, data_dir: &std::path::Path) -> Result<String, String> {
    match qingjian_sync::sync_all(
        config,
        &data_dir.join("user-vocab.tsv"),
        &data_dir.join("sync-state.json"),
    ) {
        Ok(outcome) if outcome.pending > 0 => Ok(format!(
            "已同步：新增 {} 词，跳过 {}；还有 {} 个新词下次同步继续",
            outcome.added, outcome.skipped, outcome.pending
        )),
        Ok(outcome) => Ok(format!(
            "已同步：新增 {} 词，跳过 {}",
            outcome.added, outcome.skipped
        )),
        Err(qingjian_sync::SyncError::Disabled) => Err("同步未开启：先打开上面的开关".to_owned()),
        Err(error) => Err(error.to_string()),
    }
}

pub(crate) fn view(settings: &Settings, context: &mut ViewContext<Settings>) -> View {
    let sync = &settings.config.sync;
    let rows = [
        field(
            "启用生词同步",
            "把生词本里值得复习的英文词推给疯狂听抄账号下的「疯狂听抄输入法生词」词本，\
             之后可在疯狂听抄里听写、进遗忘曲线复习。只上传生词表，不含输入日志。",
            ToggleSwitch::new()
                .is_on(sync.enabled)
                .on_toggled(context.callback(Message::SyncEnabled)),
        ),
        field(
            "服务地址",
            "",
            TextBox::new()
                .text(sync.base_url.clone())
                .on_text_changed(context.callback(Message::SyncBaseUrl)),
        ),
        field(
            "账号",
            "疯狂听抄的登录账号；登录只把令牌保存在这台电脑的配置文件里。",
            TextBox::new()
                .text(settings.sync_account.clone())
                .placeholder_text("疯狂听抄账号")
                .on_text_changed(context.callback(Message::SyncAccount)),
        ),
        field(
            "密码",
            "",
            PasswordBox::new()
                .password(settings.sync_password.clone())
                .on_password_changed(context.callback(Message::SyncPassword)),
        ),
        labeled(
            "",
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(12.0)
                .children((
                    Button::new()
                        .on_click(context.message(Message::SyncLogin))
                        .content("登录"),
                    Button::new()
                        .on_click(context.message(Message::SyncNow))
                        .content("立即同步"),
                    TextBlock::new().text(settings.sync_status.clone()),
                )),
        ),
        note("登录后点「立即同步」推一次生词；token 为空时同步会用环境变量 CZIME_SYNC_TOKEN。"),
    ];
    page("账号", StackPanel::new().spacing(16.0).children(rows))
}
