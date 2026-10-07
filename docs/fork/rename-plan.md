# 改名映射表（青简 → 疯狂听抄输入法）

> 本仓库对上游的改名工程清单。逐项替换并在 commit message 引用本表的编号。
> 原则：代码标识符里的 `qingjian`（crate 名、模块名、二进制名）**第一阶段保留**，只改
> 用户可见的品牌面（产品名、bundle id、安装器、设置页、进程显示名、更新端点），
> 降低与上游 rebase 的冲突面；等上游节奏放缓或我们做大分叉时再评估全面改名。

| # | 上游值 | 本仓库值 | 落点 |
|---|---|---|---|
| 1 | 产品显示名：青简 / Qingjian | 疯狂听抄输入法 / CZ-English IME | 两端设置页「关于」、安装器欢迎页、菜单 |
| 2 | macOS bundle id：`app.qingjian.inputmethod` | `cn.raptrans.ime`（TISInputSourceID、连接名同步改） | `apps/macos` bundle.sh + Info.plist |
| 3 | macOS 进程/显示名：Qingjian | CZEnglishIME | bundle.sh 生成名 |
| 4 | Windows 服务/进程名：qingjian-server 等 | czime-server（显示名「疯狂听抄输入法」） | `apps/windows` |
| 5 | Windows 安装器名与安装目录 | 疯狂听抄输入法 / CZEnglishIME | `apps/windows/installer`（Inno） |
| 6 | TSF 注册显示名 | 疯狂听抄输入法 | `apps/windows/tsf` 注册表资源 |
| 7 | 更新检查端点 `https://qingjian.app/releases.json` | 自有域名（初版可先 GitHub Releases 静态 JSON） | `crates/qingjian-update/src/index/fetch.rs` |
| 8 | 「关于」页官网/GitHub 链接 qingjian.app、qingjian-team | raptrans.cn、gt-guizhou/cz-english-ime | 两端 about 页 |
| 9 | 图标 `assets/icon/`（竹简 mark） | 新图标（**上游 logo 不在授权内，必须替换**；先移除展示位） | `assets/icon/`、两端 bundle |
| 10 | 配置目录 `~/Library/Application Support/Qingjian/` 等 | CZEnglishIME | 两端路径常量（注意老用户迁移不适用——我们无存量用户） |
| 11 | 品牌词表 `brand.tsv` 里的「青简」词条 | 「疯狂听抄」等自有词 | `assets/lexicon/brand.tsv` |
| 12 | README/CHANGELOG/SECURITY 中的品牌叙述 | 已重写 README、新增 NOTICE；CHANGELOG 保留上游历史、新增「Fork 后变更」分节 | 根目录文档 |

## 不改的（第一阶段）

- crate 名 `qingjian-core` 等、二进制 `qingjian-cli`、`.qj`/`.qjm` 数据格式名——内部标识符，无品牌暴露（候选窗不显示），保留可最小化 rebase 冲突。
- `docs/` 开发文档里的历史叙述——历史事实，保留出处感。
- LICENSE 与 `assets/` 数据许可文件——必须原样保留。

## 执行注记（2026-10-07 第一批）

- 已完成：上表 1–8、10–12（bundle id `cn.raptrans.ime`、配置目录 `CZEnglishIME`、更新端点暂指
  `raw.githubusercontent.com/gt-guizhou/cz-english-ime/main/releases.json`（发版时定稿）、Inno 换新 AppId
  `F5D4CB91-4B16-4E6B-A314-40C6A78B7481` 且输出名 `czime-*`、brand.tsv 换「疯狂听抄」）。
- 教训：批量 sed `Qingjian→CZEnglishIME` 会误伤代码标识符。处置原则：**定义与使用同在 apps/ 内的自洽改名保留**
  （macOS `CZEnglishIMEInputController`、fcitx5 `CZEnglishIME*` C++ 类）；**跨 crate 的引用回退**
  （`CandidateRenderer::Qingjian` 定义在 crates/，两处使用已回退）。后续批次对 apps/ 做标识符级替换前先比对 crates/。
- 待第二批：二进制名/进程名（qingjian-server→czime-server，牵动 build.rs/embed-manifest/协议字符串/安装器引用）、
  新图标（替换 assets/icon/ 竹简 mark，上游 logo 不在授权内）、「关于」页 UPSTREAM_URL 按钮（随账号/同步页接线）。
