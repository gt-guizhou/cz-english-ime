//! 生词同步：把本地生词本（`user-vocab.tsv`）里值得复习的英文词推给疯狂听抄服务端，
//! 进入账号下的「疯狂听抄输入法生词」栏目。筛选纯本地，上传是一次 HTTPS POST，
//! 由设置程序（后续「账号/同步」页）或容器 App 显式触发，不在输入热路径上调用。
//!
//! 协议：`POST {base_url}/api/ime/sync_vocab`，请求头 `token` 为疯狂听抄登录令牌，
//! 请求体走原始 JSON 流（服务端全局 filter 会改写 `post()` 取到的 JSON，直读 `php://input`
//! 是既有约定）；响应是 FastAdmin 统一外壳 `{code, msg, data}`，`data` 含
//! `{added, skipped, pending, category_id}`——`pending` 是本请求没建完的新词（服务端每请求
//! 限建新词数），客户端下次同步原样重发即可，服务端按词去重、天然幂等。

use std::path::Path;
use std::time::Duration;

use jiff::civil::Date;
use qingjian_core::storage::{read_text_lossy, write_atomic_str};
use serde::{Deserialize, Serialize};

/// 同步配置，对应 `config.toml` 的 `[sync]` 分节（默认**关闭**）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SyncConfig {
    /// 是否启用。
    pub enabled: bool,

    /// 服务端地址（不带末尾斜杠）。
    pub base_url: String,

    /// 登录令牌。留空则读 `token_env` 指定的环境变量。
    pub token: Option<String>,

    /// 存放令牌的环境变量名（密钥不进 git，与云联想 `[predict]` 同一口径）。
    pub token_env: String,

    /// 单次请求超时（毫秒）。
    pub timeout_ms: u64,

    /// 单次请求最多带多少条（服务端也有一致的上限，取小者）。
    pub max_batch: usize,

    /// 「生词线」：看到轮次不超过它的词即使没打过也算生词（与内核 `FRESH_UNTIL - 1` 同口径）。
    pub fresh_seen_max: u32,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "https://www.raptrans.cn".to_string(),
            token: None,
            token_env: "CZIME_SYNC_TOKEN".to_string(),
            timeout_ms: 30_000,
            max_batch: 100,
            fresh_seen_max: 2,
        }
    }
}

/// 生词本的一行记录（`user-vocab.tsv`：`语言\t译词\t看到\t上屏\t用过\t首见\t末见`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VocabEntry {
    pub language: String,
    pub word: String,
    pub seen: u32,
    pub committed: u32,
    pub used: u32,
    pub first: Date,
    pub last: Date,
}

/// 解析生词本文件；坏行跳过（与生词本自身的容错口径一致）。
pub fn parse_vocab_tsv(path: &Path) -> Vec<VocabEntry> {
    let text = read_text_lossy(path).ok().flatten().unwrap_or_default();
    let mut entries = Vec::new();
    for line in text.lines() {
        if let Some(entry) = parse_line(line) {
            entries.push(entry);
        }
    }
    entries
}

fn parse_line(line: &str) -> Option<VocabEntry> {
    let mut parts = line.split('\t');
    let language = parts.next()?.trim().to_string();
    let word = parts.next()?.trim().to_string();
    let seen = parts.next()?.trim().parse().ok()?;
    let committed = parts.next()?.trim().parse().ok()?;
    let used = parts.next()?.trim().parse().ok()?;
    let first = parts.next()?.trim().parse().ok()?;
    let last = parts.next()?.trim().parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(VocabEntry {
        language,
        word,
        seen,
        committed,
        used,
        first,
        last,
    })
}

/// 客户端侧的词形预校验：字母开头，只含字母、空格、连字符、撇号，≤60 字符。
/// 词性前缀（`int. hello`）与多义拼接（`hello · hi`）都不是干净词形，直接跳过。
fn is_clean_english_word(word: &str) -> bool {
    let mut chars = word.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    let len = word.chars().count();
    len <= 60 && chars.all(|c| c.is_ascii_alphabetic() || c == ' ' || c == '-' || c == '\'')
}

/// 挑出值得同步的条目：英文、词形干净，且满足其一——上屏过、直接打出过、还是生词（看到次数未过生词线）。
/// 只看英文是产品边界：疯狂听抄是英语学习平台，其他学习语言的条目留在本机。
pub fn select_for_sync<'a>(entries: &'a [VocabEntry], config: &SyncConfig) -> Vec<&'a VocabEntry> {
    entries
        .iter()
        .filter(|entry| {
            entry.language == "en"
                && is_clean_english_word(&entry.word)
                && (entry.committed >= 1 || entry.used >= 1 || entry.seen <= config.fresh_seen_max)
        })
        .collect()
}

/// 上传成功后的服务端回执。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncOutcome {
    /// 本次新加入词本的词数。
    pub added: u32,
    /// 已在词本或非法而跳过的词数。
    pub skipped: u32,
    /// 因单请求建新词上限未处理的词数（下次同步重发即可）。
    pub pending: u32,
    /// 目标栏目 id。
    pub category_id: u64,
}

/// 同步水位：上次成功同步的日期与累计上传条数，与生词本同目录落盘（`sync-state.json`）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncState {
    pub last_sync: Option<Date>,
    pub total_pushed: u64,
}

pub fn load_sync_state(path: &Path) -> SyncState {
    read_text_lossy(path)
        .ok()
        .flatten()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_sync_state(path: &Path, state: &SyncState) {
    if let Ok(text) = serde_json::to_string_pretty(state)
        && let Err(error) = write_atomic_str(path, &text)
    {
        tracing::warn!(%error, path = %path.display(), "同步水位写回失败，下次同步会重试全量");
    }
}

/// 一次完整同步：解析生词本 → 筛选 → 分批推给服务端。
/// 未启用 / 未配置令牌时返回 [`SyncError::Disabled`] / [`SyncError::NoToken`]，不算错误路径。
pub fn sync_all(
    config: &SyncConfig,
    vocab_path: &Path,
    state_path: &Path,
) -> Result<SyncOutcome, SyncError> {
    if !config.enabled {
        return Err(SyncError::Disabled);
    }
    let token = config
        .token
        .clone()
        .or_else(|| std::env::var(&config.token_env).ok())
        .filter(|token| !token.is_empty())
        .ok_or_else(|| SyncError::NoToken(config.token_env.clone()))?;

    let entries = parse_vocab_tsv(vocab_path);
    let selected = select_for_sync(&entries, config);
    if selected.is_empty() {
        return Err(SyncError::NothingToSync);
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(config.timeout_ms))
        .build()?;
    let url = format!(
        "{}/api/ime/sync_vocab",
        config.base_url.trim_end_matches('/')
    );
    let mut outcome = SyncOutcome {
        added: 0,
        skipped: 0,
        pending: 0,
        category_id: 0,
    };
    for chunk in selected.chunks(config.max_batch.max(1)) {
        let body: Vec<_> = chunk
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "word": entry.word,
                    "seen": entry.seen,
                    "committed": entry.committed,
                    "used": entry.used,
                    "first_seen": entry.first.to_string(),
                    "last_seen": entry.last.to_string(),
                })
            })
            .collect();
        let response: serde_json::Value = client
            .post(&url)
            .header("token", &token)
            .json(&serde_json::json!({ "entries": body }))
            .send()?
            .error_for_status()?
            .json()?;
        if response["code"].as_i64() != Some(1) {
            let msg = response["msg"].as_str().unwrap_or("服务端拒绝").to_string();
            return Err(SyncError::Server(msg));
        }
        let data = &response["data"];
        outcome.added += data["added"].as_u64().unwrap_or(0) as u32;
        outcome.skipped += data["skipped"].as_u64().unwrap_or(0) as u32;
        outcome.pending += data["pending"].as_u64().unwrap_or(0) as u32;
        outcome.category_id = data["category_id"].as_u64().unwrap_or(0);
    }

    let mut state = load_sync_state(state_path);
    state.last_sync = Some(jiff::Zoned::now().date());
    state.total_pushed += selected.len() as u64;
    save_sync_state(state_path, &state);
    Ok(outcome)
}

/// 同步失败。`Disabled` / `NoToken` / `NothingToSync` 是正常分支，调用方按文案提示即可。
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("生词同步未开启")]
    Disabled,
    #[error("没有登录令牌：在设置里登录疯狂听抄账号，或配置 {0} 环境变量")]
    NoToken(String),
    #[error("没有值得同步的生词")]
    NothingToSync,
    #[error("服务端返回：{0}")]
    Server(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(word: &str, seen: u32, committed: u32, used: u32) -> VocabEntry {
        VocabEntry {
            language: "en".to_string(),
            word: word.to_string(),
            seen,
            committed,
            used,
            first: Date::constant(2026, 9, 1),
            last: Date::constant(2026, 10, 5),
        }
    }

    #[test]
    fn parse_roundtrip_and_bad_lines() {
        let text = "en\thello\t3\t1\t0\t2026-09-01\t2026-10-05\n\
                    en\tyou seem\t1\t0\t0\t2026-09-02\t2026-09-02\n\
                    ja\tこんにちは\t1\t0\t0\t2026-09-02\t2026-09-02\n\
                    坏行\n\
                    en\twrong\tcols\t2026-09-01\t2026-09-01\n";
        let entries: Vec<VocabEntry> = text.lines().filter_map(parse_line).collect();
        // parse 不挑语言（ja 行也合法入表，语言过滤在 select 阶段做）；坏行与列数不对的行跳过
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].word, "hello");
        assert_eq!(entries[0].seen, 3);
        assert_eq!(entries[1].word, "you seem");
        assert_eq!(entries[1].language, "en");
        assert_eq!(entries[2].language, "ja");
    }

    #[test]
    fn select_keeps_engaged_or_fresh_english_only() {
        let config = SyncConfig::default();
        let entries = vec![
            entry("hello", 5, 1, 0),      // 上屏过 → 同步
            entry("world", 1, 0, 0),      // 生词线内 → 同步
            entry("boring", 9, 0, 0),     // 看熟且没打过 → 不同步
            entry("int. hello", 1, 1, 0), // 带词性前缀 → 不同步
            entry("hi · hey", 1, 0, 1),   // 多义拼接 → 不同步
            VocabEntry {
                language: "ja".into(),
                ..entry("konn", 1, 1, 1)
            }, // 非英语 → 不同步
        ];
        let selected = select_for_sync(&entries, &config);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].word, "hello");
        assert_eq!(selected[1].word, "world");
    }

    #[test]
    fn state_roundtrip() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("czime-sync-state-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        assert_eq!(load_sync_state(&path), SyncState::default());
        let state = SyncState {
            last_sync: Some(Date::constant(2026, 10, 7)),
            total_pushed: 42,
        };
        save_sync_state(&path, &state);
        assert_eq!(load_sync_state(&path), state);
        let _ = std::fs::remove_file(&path);
    }
}
