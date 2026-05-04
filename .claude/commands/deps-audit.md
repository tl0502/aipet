---
description: 依赖 CVE + license 扫描 — Rust 走 cargo-deny(advisories + licenses + bans 三合一),Node 走 pnpm audit + license 白名单。失败应拦 PR 合入。
allowed-tools: Bash(cargo deny:*), Bash(cargo audit:*), Bash(cargo install:*), Bash(pnpm audit:*), Bash(pnpm licenses:*), Bash(git log:*), Bash(git diff:*), Bash(git rev-parse:*), Read, Write
argument-hint: [--rust | --node | --all] [--update-config]
---

## 任务

扫 Rust + Node.js 依赖的 CVE / license / yanked / unmaintained,产 `progress/deps-audit-{date}.md`。

**不修依赖**;升级 / 替换决策由 module-implementer 接力。本命令只判「是否有违规」与「建议路径」。

## 前置上下文

- 当前分支:
!`git rev-parse --abbrev-ref HEAD`

- 当前 HEAD:
!`git rev-parse --short HEAD`

- 最近改动 Cargo.toml / package.json:
!`git log --oneline -5 -- src-tauri/Cargo.toml package.json`

- src-tauri/Cargo.toml(直接依赖清单):
@src-tauri/Cargo.toml

- package.json(直接依赖清单):
@package.json

- src-tauri/deny.toml(若已配置):
@src-tauri/deny.toml

## 范围解析

| `$ARGUMENTS` | 解析 |
|---|---|
| 空 / `--all` | 跑 Rust + Node 全套(默认) |
| `--rust` | 只跑 cargo-deny |
| `--node` | 只跑 pnpm audit + licenses |
| `--update-config` | 若 `src-tauri/deny.toml` 不存在,生成 starter 配置(见下) |

## 工具栈选型

### Rust:cargo-deny(优先)

理由:**单工具兼顾 advisory / license / source / bans 4 维度**,替代 cargo-audit 的单一 CVE 扫。

- 安装:`cargo install --locked cargo-deny`(本机一次,CI 缓存)
- 跑:`cd src-tauri && cargo deny check`
- 配置:`src-tauri/deny.toml`(本命令负责生成 starter)

### Node.js:pnpm audit + pnpm licenses

- CVE:`pnpm audit --audit-level=high --json`(level high 以上才算违规;moderate 仅记录)
- license:`pnpm licenses ls --json`,与白名单对照
- License 白名单:`MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unlicense, Zlib, MPL-2.0, CC0-1.0`
- License 拦截名单:`GPL-*, AGPL-*, LGPL-*, SSPL-*, Commons-Clause, Copyleft, OSL-*`(任一命中 → 必须 review)

## starter 配置(`--update-config` 触发)

`src-tauri/deny.toml`:

```toml
[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
notice = "warn"
ignore = []  # 仅在用户签字后填 RUSTSEC-XXXX-XXXX 加豁免

[licenses]
unlicensed = "deny"
allow = [
  "MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause",
  "ISC", "Unlicense", "Zlib", "MPL-2.0", "CC0-1.0",
]
deny = ["GPL-2.0", "GPL-3.0", "AGPL-3.0", "LGPL-3.0", "SSPL-1.0"]
copyleft = "warn"
allow-osi-fsf-free = "neither"
default = "deny"
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"
wildcards = "deny"
highlight = "all"
deny = []  # 在 ADR 拍板后填具体 crate 名(如 openssl 替代为 rustls)

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

生成时若 `deny.toml` 已存在 → **不覆盖**,仅 echo 「已存在,如需重置请手动删除」。

## 产物

`progress/deps-audit-{YYYY-MM-DD}.md`:

```markdown
# Deps audit {YYYY-MM-DD}

## Run {HH:MM:SS} — scope {--rust | --node | --all}

**上下文**
- HEAD:`<short SHA>`
- 总裁决:**Pass / Conditional / Block**

## Rust(cargo-deny)

- advisories:**N hits**(critical X / high Y / medium Z)
- licenses:**N violations**(deny X / copyleft warn Y)
- bans:**N hits**(multiple-versions / wildcards / explicit deny)

### 详情(若有)

#### A-1 [`<crate>@<ver>`] RUSTSEC-XXXX-YYYY 一句描述

- 触发路径:`<crate> -> <transitive> -> ...`
- 修复:升级到 `>= X.Y.Z` / 替换为 `<alt>` / accept 加 ignore(需用户签字)

## Node(pnpm)

- vulnerabilities:**critical N / high N / moderate N**(只 critical+high 算违规)
- license violations:**N hits**(GPL X / AGPL Y / unknown Z)

### 详情(若有)

#### N-1 [`<pkg>@<ver>`] CVE-XXXX-YYYY 一句描述

- 触发路径:`pnpm why <pkg>` 输出
- 修复:`pnpm update <pkg>` / `pnpm add <alt>` / 等上游修复

## 建议

- 升级:list crate / pkg
- 替换:list crate -> alt
- accept(白名单):需用户签字 → 写到 `deny.toml` ignore 段并标 ADR-NNN

## 后续动作

- 若 Block → module-implementer 接力修复
- 若 Pass / Conditional → 可放行 ship-task
- 若新引入依赖 → 同步更新 `progress/decisions-log.md` 加一行(decisions 历史已有先例)
```

## CI 集成(分离任务,不在本命令范围)

后续单独 commit,把以下加到 `.github/workflows/ci.yml`:

```yaml
- name: Install cargo-deny
  run: cargo install --locked cargo-deny || true

- name: Cargo deny check
  working-directory: src-tauri
  run: cargo deny check

- name: Pnpm audit
  run: pnpm audit --audit-level=high

- name: License check
  run: pnpm licenses ls --json | node scripts/license-check.mjs
```

`scripts/license-check.mjs` 读 stdin JSON,对照白名单/拒绝名单退出 0/1。本命令不实施 CI 集成,只在报告 § 后续动作 提示。

## 执行规则

- **只读 + 写 progress/** — 除 `--update-config` 触发的 deny.toml starter 外,不修任何文件
- **每条违规必须有 CVE / RUSTSEC / license SPDX 编号** — 没编号的不算违规
- **accept 必须签字** — Won't fix / ignore 需要用户在报告中明确批准,本命令不自动加 ignore
- **CI 集成单独 commit** — 本命令只生成报告与 starter 配置,不动 ci.yml

## 节奏

- 每 PR(CI 触发)
- nightly schedule(cron 抓最新 advisory)
- module 引入新依赖时(decisions-log 触发)
- milestone 末由 gate-checker 调用,出口报告引用最新 audit 结果

## 输出

终端 5 行内:

1. 范围 + HEAD SHA
2. Rust:advisories N / licenses N / bans N
3. Node:vuln critical N / high N / license violations N
4. 总裁决 + 报告路径
5. 建议下一步(Block → 修复 / Pass → ship-task)
