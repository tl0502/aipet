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

最终输出 5 行以内:

1. commit SHA + message
2. pushed branch + upstream
3. PR 链接/目标分支
4. 验证结果摘要
5. follow-up(如需 rotate secret / 回填 Last commit / 手测 UI)
