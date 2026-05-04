---
description: 性能预算实测 — 内存 / 启动 / bundle / 物理交互 / 装扮切换 vs BASELINE.md § 性能预算速查 9 项指标。本命令只测,不修。
allowed-tools: Bash(pnpm tauri build:*), Bash(tasklist:*), Bash(powershell:*), Bash(Get-ChildItem:*), Bash(Get-Process:*), Bash(Measure-Command:*), Bash(Start-Process:*), Bash(Stop-Process:*), Bash(grep:*), Bash(git log:*), Bash(git rev-parse:*), Read, Write
argument-hint: [--quick | --full] [--baseline]
---

## 任务

测量 BASELINE.md § 性能预算速查 9 项指标,对照目标值标 ✅/❌。**只测不修**;超 budget 项写到 `progress/risks.md`,owner = self。

## 前置上下文

- 当前分支:
!`git rev-parse --abbrev-ref HEAD`

- 当前 HEAD:
!`git rev-parse --short HEAD`

- BASELINE.md § 性能预算速查(权威目标值,不复制):
@docs/AIPET-obsidian/BASELINE.md

- progress/CURRENT.md(当前 milestone / 模块状态):
@progress/CURRENT.md

- progress/perf-baseline.json(若存在,作为历史趋势比对源):
@progress/perf-baseline.json

## 范围解析

| `$ARGUMENTS` | 解析 |
|---|---|
| 空 / `--quick` | 只测 3 项可机械跑的:**内存 / bundle / 冷启动**(默认,~30s) |
| `--full` | 跑全 9 项;待接入项标 N/A 留 stub(~5min,需先跑 `pnpm tauri build`) |
| `--baseline` | 把当次结果同步写一份到 `progress/perf-baseline.json`,作为后续比对基线 |

`--baseline` 与 `--quick`/`--full` 可叠加。

## 测量项 → 工具映射

| # | 项 | 目标 | 测量方法 | M{N} 可测 |
|---|---|---|---|---|
| 1 | 总常驻内存 | ≤ 250MB | 启动 `aipet.exe`,等 30s 静态稳态后 `tasklist /FI "IMAGENAME eq aipet.exe" /FO CSV` 取 Working Set | M1+ |
| 2 | 安装包 | ≤ 80MB | `pnpm tauri build` 后 `Get-ChildItem src-tauri/target/release/bundle/msi/*.msi \| Select Length` | M1+ |
| 3 | 冷启动 | ≤ 5s | PowerShell `Measure-Command { Start-Process .\aipet.exe -PassThru \| Wait-Process -Timeout 30 }`(进程退出或 30s timeout) | M1+ |
| 4 | 物理交互响应 | < 100ms | grep ringbuffer logger 中的 `hitbox→action` 时间戳差(P95) | M1 D6+(等 logger) |
| 5 | 装扮切换 | < 500ms | grep `WardrobeService::switch` 时间戳差 | M4+ |
| 6 | 对话首 token | p50 ≤ 1.5s | grep `LLMProvider::stream_first_token` 时间戳差(等 B.1) | M1 D5+ |
| 7 | 声音播放延迟 | < 50ms | grep `VoiceEffectPlayer::play` 时间戳差 | M4+ |
| 8 | 本地游戏每轮 | < 50ms | grep `GameEngine::tick` 时间戳差 | M5+ |
| 9 | 摸鱼切换 | < 200ms | grep `BossKeyService::toggle` 时间戳差 | M2+ |

**测量必须明确「冷启 / 静态稳态 / 含 IO 等待」三种模式**:
- 冷启:首次启动 / 清空缓存
- 稳态:启动后 ≥ 30s,没有 active interaction
- 含 IO:涉及 SQLite / 文件系统 / 网络的路径

## 产物结构

`progress/perf-{YYYY-MM-DD}.md`(同日多次跑追加 `## Run` 节,不覆盖):

```markdown
# Perf check {YYYY-MM-DD}

## Run {HH:MM:SS} — scope {--quick | --full}

**测量上下文**
- HEAD:`<short SHA>`
- 分支:`<branch>`
- Milestone:M{N} W{w} D{d}
- 测量模式:cold / steady / io

## 实测 vs 预算

| # | 项 | 目标 | 实测 | 模式 | 裁决 | 趋势 |
|---|---|---|---|---|---|---|
| 1 | 内存 | ≤ 250MB | 138 MB | steady | ✅ | -2 MB vs baseline |
| 2 | 安装包 | ≤ 80MB | 56 MB | — | ✅ | +3 MB vs baseline |
| ... |

## 超预算项分析(若有)

### 项 N:<名> 实测 X 超 Y MB

- 触发条件:...
- 关联 risks.md:#9 VRM 内存 / 新风险已写入第 14 行
- 建议:LOD 切换 / 资源压缩 / 降低纹理分辨率

## 历史趋势(若 perf-baseline.json 存在)

- 内存 138 MB(基线 140 MB,-1.4%)
- 安装包 56 MB(基线 53 MB,+5.7%,⚠️ 接近 60MB 警戒)
```

## 状态文件(可选)

`--baseline` 时同步写 `progress/perf-baseline.json`:

```json
{
  "baseline_sha": "<HEAD SHA>",
  "baseline_date": "<RFC3339>",
  "baseline_milestone": "M1 W1 D3",
  "metrics": {
    "memory_mb": 138,
    "bundle_mb": 56,
    "cold_start_ms": 3200,
    "interaction_response_ms": null
  }
}
```

下次 perf-check 自动读这里作 trend 比对源。

## 执行规则

- **只读 + 写 progress/** — 不修代码 / 文档 / CI
- **不评判产品决策** — 超 budget 只标 ❌ 与建议,不直接修代码;修复由 module-implementer 接力
- **N/A 必须有理由** — 待接入项标「等 M{N} <模块>」,不能空白
- **测量误差 ≥ 10% 必须重测** — 单次结果不可信,内存 / 启动 至少 3 次取中位

## 节奏

- 模块完成时(尤其 VRM 渲染 / 装扮 / 游戏 / IPC 重交互)— 由 module-implementer 触发
- milestone 末必跑 — 由 gate-checker 触发(`progress/gate-m{N}.md` 引用本期报告)
- 超 budget → 写一行到 `progress/risks.md`,owner = self,期望解锁条件 = 优化方案

## 输出

终端 5 行内:

1. 测量范围(--quick / --full)+ HEAD SHA
2. 9 项实测 vs 预算汇总(✅ N / ❌ N / N/A N)
3. 超预算项数 + 关联 risks.md 行号
4. 报告路径 `progress/perf-{date}.md`
5. 建议下一步(若超预算 → "进 perf 优化清单";若全过 → "可放行 ship-task")
