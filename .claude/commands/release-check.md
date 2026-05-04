---
description: 发布健康度 — bundle 体积 vs 80MB 预算 + smoke test(启动 5s 不自杀)+ WebView2 bootstrap 验证 + 升级路径(M3+)
allowed-tools: Bash(pnpm tauri build:*), Bash(Get-ChildItem:*), Bash(Get-ChildItem -Recurse:*), Bash(Measure-Object:*), Bash(powershell:*), Bash(Start-Process:*), Bash(Wait-Process:*), Bash(Stop-Process:*), Bash(git rev-parse:*), Read, Write
argument-hint: [--bundle-only | --smoke-only | --full]
---

## 任务

跑 Tauri release build,验证发布候选版的 4 道关:**bundle 体积 / bundle 内文件清单 / smoke 启动 / WebView2 bootstrap**。M3+ 接入 UpdaterService 后增加升级路径验证。

**只测不修**;超预算或 smoke 失败 → 写到 `progress/risks.md`,owner = self,修复由 module-implementer 接力。

## 前置上下文

- 当前分支 / HEAD:
!`git rev-parse --abbrev-ref HEAD`
!`git rev-parse --short HEAD`

- BASELINE.md § 性能预算速查(80MB bundle 硬约束):
@docs/AIPET-obsidian/BASELINE.md

- progress/risks.md(#3 WebView2 / 关联条目):
@progress/risks.md

- 现有 bundle(若已 build):
!`Get-ChildItem src-tauri/target/release/bundle -Recurse -File -ErrorAction SilentlyContinue | Measure-Object Length -Sum`

## 范围解析

| `$ARGUMENTS` | 解析 |
|---|---|
| 空 / `--full` | 跑全套(bundle + smoke + WebView2 检查),需要 fresh `pnpm tauri build`(~5-10min) |
| `--bundle-only` | 跑 build,只检 bundle 体积与内容清单(~5min) |
| `--smoke-only` | 跳过 build(用现有 release 产物),只做 smoke 启动测试(~30s) |

## 检查项

### 关 1:Bundle 体积

- 命令:`Get-ChildItem src-tauri/target/release/bundle/msi/*.msi | Select Length`
- 预算:**≤ 80MB**(BASELINE 硬约束)
- 趋势:diff > 5MB vs 上次 → ⚠️ 警示,排查最近依赖增量

### 关 2:Bundle 内文件清单

- 命令:`Get-ChildItem src-tauri/target/release/bundle -Recurse -File`
- 不该带:
  - `*.pdb`(调试符号,~50MB)
  - `*.exp` / `*.lib`(导入库)
  - `*test*` / `*spec*`(测试资源)
  - 未使用的 `.vrm` / `.glb`(只该带 momo.vrm)
- Cargo.toml `[profile.release]` 已设 `strip = true`,但需复核 bundle plugin 是否覆盖

### 关 3:Smoke 启动

- 命令:
  ```powershell
  $proc = Start-Process .\src-tauri\target\release\aipet.exe -PassThru
  Start-Sleep -Seconds 5
  if ($proc.HasExited) {
    Write-Error "Process died within 5s, exit code: $($proc.ExitCode)"
    exit 1
  }
  Stop-Process -Id $proc.Id
  ```
- 通过:5s 内进程仍存活
- 失败:进程 exit code != 0(常见:DLL 缺失 / WebView2 缺失 / 配置文件读失败)

### 关 4:WebView2 bootstrap(关联 risks.md #3)

- 检查 `tauri.conf.json` 的 `windows.webviewInstallMode`:
  - `embedBootstrapper` ✅(默认推荐,无网安装)
  - `downloadBootstrapper` ⚠️(联网下载,弱网环境失败)
  - `offlineInstaller` ✅(老 Win10 兜底)
  - `skip` ❌(MVP 期不允许)
- M1 末打包测试时:在干净 Win10 VM 上验证 WebView2 自动安装

### 关 5:升级路径(M3+ stub)

- 等 UpdaterService 接入后:
  - 验证 `tauri-updater` 配置(public key / endpoint / signature)
  - 模拟 v0.M{N-1}.0 → v0.M{N}.0 的 patch 流程
  - 用户数据迁移脚本是否幂等(关联 migrations idempotent)
- 当前期:本节标 N/A(M3+ 接入)

## 产物

`progress/release-check-{YYYY-MM-DD}.md`:

```markdown
# Release check {YYYY-MM-DD} — bundle v{0.0.0}

## Run {HH:MM:SS} — scope {--full | --bundle-only | --smoke-only}

**上下文**
- HEAD:`<short SHA>`
- Cargo profile:release(opt-level=3 / lto=true / strip=true)
- 总裁决:**Pass / Conditional / Block**

## 关 1:Bundle 体积

- msi:`<X> MB / 80 MB` → ✅/❌
- 趋势:`+X MB / -Y MB` vs 上次

## 关 2:Bundle 内文件清单

- 总文件数:N
- 异常项:无 / [...]
- strip 验证:✅/❌

## 关 3:Smoke 启动

- 5s 自杀:✅/❌
- exit code(若失败):...
- 失败日志(若有):tail of stderr

## 关 4:WebView2 bootstrap

- webviewInstallMode 配置:`embedBootstrapper` ✅
- 干净 VM 测试:**未跑 / Pass / Fail**(M1 末必跑)

## 关 5:升级路径

- (M3+ 接入,本期 N/A)

## 下一步

- 若 Block → module-implementer 修复 / 关联 risks.md 哪条
- 若 Pass → 可放行 ship-task
```

## 执行规则

- **只读 + 写 progress/** — 不修代码 / 配置
- **build 必须从干净状态** — `--full` 模式建议先 `cd src-tauri && cargo clean`(本命令不自动 clean,提示用户)
- **测试 binary 必须干净退出** — `Stop-Process` 之前确认进程未崩溃,不留 zombie
- **WebView2 干净 VM 测试不在本命令范围** — 本命令只检 `tauri.conf.json` 配置;实际 VM 测试需用户手动

## 节奏

- milestone 末必跑(由 gate-checker 触发,出口报告引用本期结果)
- bundle 体积突增(diff > 5MB)→ 排查依赖增量
- M5 RC 前完整跑一次,含 WebView2 干净 VM 测试
- 不在每次 commit 跑(`pnpm tauri build` 慢)

## 输出

终端 5 行内:

1. 范围 + HEAD SHA + bundle 版本号
2. 关 1-4 裁决汇总(✅/❌/N/A)
3. bundle 体积 X MB / 80 MB / 趋势
4. 报告路径 `progress/release-check-{date}.md`
5. 建议下一步(Block / RC / 进 perf-check)
