<h1 align="center">疯狂听抄输入法 CZ-English IME</h1>

<p align="center"><strong>好好输入，顺便多认识一个词。</strong></p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0--or--later-blue" alt="License: GPL-3.0-or-later"></a>
  <a href="https://github.com/qingjian-team/qingjian"><img src="https://img.shields.io/badge/基于-青简%20Qingjian%20二次开发-2563eb" alt="基于青简 Qingjian 二次开发"></a>
</p>

疯狂听抄输入法是一款学习型拼音输入法。你可以像平常一样打字：输入拼音、选择候选、写完整句；候选旁的一条英语译词，让背单词自然发生在日常输入里。译词始终只是辅助信息，不会盖过你要输入的文字。

打字攒下的生词，可以登录「疯狂听抄」账号同步到你的个人词本，进入复习流程——在键盘上认识，在复习里记住。

## 输入时，你会看到什么

```text
1  开发        development
2  编程        programming
3  架构        architecture
```

候选旁显示英语译词，出现不足三轮的生词会标橙色，看熟了自动变灰。支持整句输入、简拼、拼写纠错、双拼、五笔；本地整句模型会在停顿后调整句子候选。词频与输入习惯在本机学习，越用越顺手。

## 数据与隐私

拼音转换、词库查询、本地模型和输入习惯学习在你的设备上完成。输入量与词汇统计只保存在本机。输入日志与统计分开保存，日志可在设置中关闭或清空。检查更新会向我们的服务器请求版本列表，可在设置中关闭。

**生词同步与账号登录默认关闭**：开启后才会上传生词表（语言、译词、看到次数等少量字段），用于在你的「疯狂听抄」账号下生成个人词本；不发输入日志、不发按键记录。可选的**云联想默认关闭**，开启后把当前输入发送给云端服务获取候选补全。

## 致谢与许可

本仓库是 [青简 Qingjian](https://github.com/qingjian-team/qingjian) 的二次开发（GPL-3.0-or-later），感谢 qingjian-team 的开源工作。修改说明见 [NOTICE](NOTICE)。

- 代码采用 [GPL-3.0-or-later](LICENSE) 许可；「青简 / Qingjian」名称与 logo 归其作者所有，不包含在代码授权中，本仓库不使用。
- 随包数据有各自的来源与许可，见[数据来源清单](docs/design/landscape.md)与 `assets/` 各目录 README。
- 想参与开发？从[开发文档](docs/)和[开发约定](docs/contributing.md)开始。
