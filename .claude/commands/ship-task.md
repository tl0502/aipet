---
description: 完成一个 task 后的标准收口流程:检查 progress、验证、atomic commit、push 当前 feat 分支
allowed-tools: Bash(git status:*), Bash(git branch:*), Bash(git log:*), Bash(git diff:*), Bash(git ls-files:*), Bash(git check-ignore:*), Bash(git add:*), Bash(git commit:*), Bash(git push:*), Bash(pnpm typecheck:*), Bash(pnpm lint:*), Bash(cd src-tauri && cargo check:*), Bash(cd src-tauri && cargo test:*), Read, Grep
argument-hint: [commit-message]
---

## 任务

把"完成 1 个 task"收口为标准流程:确认工作树、检查 progress、跑验证、创建 atomic commit,并把当前 `feat/*` 分支 push 到 GitHub,避免本地 commit 堆积但远程仓库不更新。

## 前置上下文

- 当前分支:
!`git branch --show-current`

- 工作树状态:
!`git status --short`

- 最近 commit:
!`git log --oneline -5`

- progress/CURRENT.md:
@progress/CURRENT.md

- .gitignore:
@.gitignore

## Step 0:dry-run 报告 + 用户批准 gate(必跑,2026-05-04 新增)

进入 § 必做检查 之前,**必先输出**(短而完整,不要"现在我开始..."等冗余):

1. **待 stage 文件清单**(`git status --porcelain` 全列,标记 modified / new / deleted)
2. **验证结果**(`pnpm typecheck` / `pnpm lint` / `cd src-tauri && cargo check` / `cargo test` 通过数 — Rust/DB 改动跑 test,纯文档可省)
3. **commit message 草稿**(完整,含 type / scope / subject / body)
4. **风险点**(如有,例:跨 service / migration 不可逆 / 与遗留改动混合 / 敏感文件嫌疑)

**等用户回复 "approve" / "ship" / "走" / "OK" 之类字样,才能进入 Step 1 必做检查**。
用户提修改意见 → 调整后**重新跑 dry-run**(重新输出 1-4)→ 等批准。

**例外(可绕过 Step 0)**:用户在调用时显式说「直接 ship-task」/「不用确认」/ 加 `--yes` flag,跳过 Step 0(用于自动化场景或用户明确授权)。

> **理由**:M1 W1 D3 SQLITE_CANTOPEN 双重 bug + B.1 hot-fix 验证流程显示,task 边界 / 改动范围由 main session 自判时易过早 commit,Step 0 加一道审核 gate;90% 简单场景用户秒批,复杂场景用户能拦回 main session 误判。详 CLAUDE.md § 提交规范 + decisions-log 2026-05-04「commit 审核 gate」。

## 必做检查

1. **分支检查**
   - 当前分支必须是 `feat/mN-dX-<topic>` 或其他明确的 feature 分支。
   - 如果在 `main` / `milestone/*` 上,停止并要求先切 feature 分支。

2. **工作树检查**
   - 用 `git status --short` 列出所有变更。
   - 明确区分:
     - 本 task 应提交的文件
     - 不应提交的本地/敏感文件
     - 与本 task 无关的遗留变更

3. **敏感文件检查**
   - 不提交 `.env*`,`.claude/settings.local.json`,`docs/AIPET-obsidian/.obsidian/`,运行时 DB(`*.db`,`*.sqlite*`),大型模型资源(`public/avatar/*.vrm|*.glb|*.fbx`)。
   - 如发现敏感文件已被追踪,停止并提示先 `git rm --cached` + rotate secret。

4. **progress 检查**
   - 完成 task 必须更新 `progress/CURRENT.md`。
   - 完成 story 还必须更新 `progress/mN.md` 对应行。
   - 完成 module 还必须更新 `progress/decisions-log.md`。
   - `progress/CURRENT.md` 的 `Last commit` 在 commit 后可能仍需回填,如不能自动回填则在输出中提醒。

5. **验证**
   默认运行:
   ```bash
   pnpm typecheck
   pnpm lint
   cd src-tauri && cargo check
   ```
   Rust 逻辑或数据库变更再运行:
   ```bash
   cd src-tauri && cargo test
   ```
   UI/前端交互变更还需说明是否已通过 `pnpm dev` / `pnpm tauri:dev` 手测;若未手测,必须在输出中标明。

   **测试覆盖底线**(2026-05-04 新增,与 CLAUDE.md § 测试覆盖底线 同步):
   - DB-touching 改动(新加 service / sqlx 调用):必须有 `services/test_db.rs::fresh_db()` 集成测试覆盖,纯逻辑单测不算
   - 文件 / Win32 API 改动:必须有 tempfile + 真实 syscall 集成测试(I.2 crypto.rs DPAPI 同款)
   - IPC command 暴露给前端:必须 dev panel(`Ctrl+Shift+D`)端到端手测,响应贴到 task 收口报告
   - 不满足时**停止 commit**,先补测试再回到 Step 0 重新 dry-run

6. **commit**
   - 使用 Conventional Commit:
     - `feat / fix / refactor / docs / test / perf / chore`
   - subject ≤ 100 chars。
   - body 必须说明关联 ADR 或 PRD 模块;若为 scaffold/流程维护,写明"无关联 ADR;PRD 模块:无(scaffold/流程维护)"。
   - commit message 末尾必须包含:
     ```
     Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
     ```

7. **push**
   - commit 成功后必须执行:
     ```bash
     git push -u origin <current-branch>
     ```
   - 如果已有 upstream,普通 `git push` 即可。
   - push 后输出 PR 链接:目标分支是当前 milestone,例如 `milestone/m1`。

## 执行规则

- 如果用户给了 `$ARGUMENTS`,优先用作 commit header,但仍要检查是否符合 Conventional Commit。
- 如果没有 `$ARGUMENTS`,根据 diff 草拟 commit message 后再提交。
- **不要**用 `git add -A`。只 stage 本 task 相关文件。
- **不要**提交本地密钥、大文件、运行时数据。
- **不要**push `main` 或 `milestone/*`；本命令只 push 当前 feat 分支。
- 如果 hook/test 失败,修复根因后重新 commit;不要 `--no-verify`。

## 输出

最终输出 7 行以内:

1. commit SHA + message
2. pushed branch + upstream
3. PR 链接/目标分支
4. 验证结果摘要
5. **下一步检查建议**(基于 commit diff 智能匹配,见下表;不命中则空)
6. follow-up(如需 rotate secret / 回填 Last commit / 手测 UI)

## 下一步检查建议(commit 后智能匹配)

commit 成功后,看本笔 diff 内容,在「输出」第 5 步给出 **0-3** 条最相关检查建议(优先级高先列,不刷数)。

判断表:

| diff 含 | 建议 | 置信度 |
|---|---|---|
| `Cargo.toml` / `package.json` / `*.lock` 新增依赖行 | `/deps-audit`(Rust + Node CVE/license 扫描) | 高 |
| `Cargo.toml [profile.release]` / `vite.config.ts` build 段 / `tauri.conf.json` | `/release-check`(bundle / smoke / WebView2) | 高 |
| `src/components/*.vue` / `src/views/*.vue` 改动 ≥ 1 文件 | `/a11y-check <path>` | 中 |
| `src-tauri/migrations/*.sql` 新增或改动 | `/code-audit --staged` 维度 3 数据完整性 | 高 |
| 跨 ≥ 3 个 service / Rust 文件 大 commit(> 200 行) | `/code-audit --staged`(8 维度漏洞扫) | 中 |
| 完成 module-level 收口(`decisions-log.md` 加行 OR `m{N}.md` 整行 ✅) | `/perf-check --baseline` + `/code-audit --staged` | 高(milestone-relevant) |
| `tracing::` / `log::` / `eprintln!` 改动 ≥ 5 处 | (M1 D6+ ringbuffer logger 落地后)调 `obs-checker` agent(SOP `.claude/agents/obs-checker.md` 已就绪,等实测覆盖)| 低(stub) |

**判断规则**:
- 仅当 diff **明确命中**模式才建议;不命中则第 5 步留空,输出 6 行
- 最多 3 条,按上表「置信度」先列高的
- 一行内简述命中原因(如「本笔含 Cargo.toml 新增 ulid / gray_matter 依赖行」)
- **不重复 PostToolUse hook 已经提醒过的内容** — hook 在文件改动时已提醒,ship-task 这里是 commit 后的 milestone-aware 二次确认,聚焦「这笔 commit 完整性视角」而非「单文件视角」

**输出语气示例**:
```
5. 建议下一步:
   • `/deps-audit`(本笔 Cargo.toml 新增 ulid / gray_matter)
   • `/perf-check --baseline`(本笔含 m1.md F.1 整行 ✅,模块级收口)
```

不命中时:
```
5. (无下一步检查建议)
```
