---
description: 检查 progress/CURRENT.md 与最近 git log 的一致性,列出可能漏更的 commit
allowed-tools: Bash(git log:*), Bash(git status:*), Bash(git branch:*), Read
argument-hint: [N]
---

## 任务

检查 `progress/CURRENT.md` 与最近 N 条 git log(默认 5)是否一致,输出"建议补充行" markdown 片段。

## 上下文

- 当前 progress/CURRENT.md:
@progress/CURRENT.md

- 最近 git log:
!`git log --oneline -${1:-5}`

- 当前分支:
!`git branch --show-current`

- 工作树状态:
!`git status --short`

## 输出

1. 列出 git log 中**未在 "Recently Completed" 反映的 commit**(SHA 比对)
2. 校验 `Active branch` 行是否与实际一致
3. 校验 `Last commit` 行是否与最新 commit 一致
4. 输出可直接 paste 到 CURRENT.md 的"建议补充行"片段

⚠️ 只输出建议,不修改 CURRENT.md(由用户判断后手动 paste)。
