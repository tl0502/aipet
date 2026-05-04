#!/usr/bin/env node
// PostToolUse hook:Edit / Write / MultiEdit 后,根据文件路径建议跑哪些检查
// advisory only — 不拦工具调用,仅在 Claude 上下文注入「下一步可考虑」提示
// 详见 progress/audit-coverage-2026-05-04.md § 三层智能触发(L1 hook)
// 6 个高质量触发点:依赖/构建 / Vue 组件 / 构建发布配置 / DB schema 迁移 /
//                  services sqlx 调用 / commands 新 IPC

const fs = require('fs');

let raw;
try {
  raw = fs.readFileSync(0, 'utf-8');
} catch (e) {
  process.exit(0); // 无 stdin 放行
}

let input;
try {
  input = JSON.parse(raw);
} catch (e) {
  process.exit(0); // 解析失败放行
}

const filePath = (input.tool_input && input.tool_input.file_path) || '';
if (!filePath) process.exit(0);

// 规范化路径(兼容 Win 反斜杠)
const p = filePath.replace(/\\/g, '/');

const suggestions = [];

// 触发 1:依赖 / 锁文件
if (/(^|\/)(Cargo\.toml|package\.json|pnpm-lock\.yaml|Cargo\.lock)$/.test(p)) {
  suggestions.push(
    '依赖 / 构建配置改动 → 模块完成时可考虑:\n' +
    '  • `/deps-audit`(Rust + Node CVE / license 扫描)\n' +
    '  • `/release-check`(若 `[profile.release]` 或 bundle 配置改动)'
  );
}

// 触发 2:Vue 组件
if (/(^|\/)src\/.+\.vue$/.test(p)) {
  suggestions.push(
    `Vue 组件改动 → 完成时可跑 \`/a11y-check ${filePath}\`\n` +
    '  (键盘可达 / ARIA / 文案外置 / 快捷键冲突 / 可控关闭)'
  );
}

// 触发 3:构建 / 发布配置
if (/(^|\/)(vite\.config\.(ts|js|mts|cts)|tauri\.conf\.json|tauri\.windows\.conf\.json)$/.test(p)) {
  suggestions.push(
    '构建 / 发布配置改动 → 模块完成时跑 `/release-check`\n' +
    '  (bundle 体积 / smoke 启动 / WebView2 bootstrap)'
  );
}

// 触发 4:DB schema 迁移
if (/(^|\/)src-tauri\/migrations\/.+\.sql$/.test(p)) {
  suggestions.push(
    'DB 迁移 schema 改动 → 跑 `/code-audit --staged` 维度 3 数据完整性\n' +
    '  关键自查:\n' +
    '  • `ON CONFLICT(cols)` 必须配 cols 的 UNIQUE 约束才生效(SQLite 静默失效坑)\n' +
    '  • 每条 DDL 必须 `IF NOT EXISTS` 或 schema 检查包裹,可重跑'
  );
}

// 触发 5:services/*.rs 含 sqlx 调用 → 测试覆盖底线 fresh_db 集成测试提示
//
// CLAUDE.md § 测试覆盖底线 强约束:DB-touching 改动必须有真实路径集成测试,
// 纯逻辑单测不算 done。M1 W1 D3 SQLITE_CANTOPEN 双重 bug 潜伏 6 commit 的
// 教训(7 services / 62 单测全是纯逻辑)→ hook 提前在文件改动期就提醒,
// 不等 ship-task Step 5 才发现没补集成测试。
if (/(^|\/)src-tauri\/src\/services\/.+\.rs$/.test(p)) {
  let hasSqlx = false;
  try {
    const content = fs.readFileSync(filePath, 'utf-8');
    hasSqlx = /\bsqlx::/.test(content);
  } catch (e) {
    // 文件读不到(被删了 / 路径错)就跳过本提示
  }
  if (hasSqlx) {
    suggestions.push(
      'services/ 含 sqlx 调用 → CLAUDE.md § 测试覆盖底线 强约束:\n' +
      '  • 必须有 `services/test_db.rs::fresh_db()` 集成测试覆盖,纯逻辑单测不算\n' +
      '  • 参考 secrets / nickname / memory / persona 已落地的 22 集成测试\n' +
      '  • [HIGH] sqlx 默认开 PRAGMA foreign_keys=ON,跨表 INSERT 前确保 FK 父行存在'
    );
  }
}

// 触发 6:commands/*.rs 含 #[tauri::command] → dev panel e2e 提示
//
// CLAUDE.md § 测试覆盖底线 强约束:IPC command 暴露给前端时必须 dev panel
// 端到端手测,响应内容贴到 task 收口报告。覆盖纯 cargo test 测不到的真实
// AppHandle / capabilities / 前端调用链路。
if (/(^|\/)src-tauri\/src\/commands\/.+\.rs$/.test(p)) {
  let hasTauriCmd = false;
  try {
    const content = fs.readFileSync(filePath, 'utf-8');
    hasTauriCmd = /#\[tauri::command\]/.test(content);
  } catch (e) {
    // 同上
  }
  if (hasTauriCmd) {
    suggestions.push(
      'commands/ 含 #[tauri::command] → CLAUDE.md § 测试覆盖底线 强约束:\n' +
      '  • IPC 暴露前端必须 dev panel(`Ctrl+Shift+D`)端到端手测一遍真实链路\n' +
      '  • 响应内容(成功 / 异常)贴到 task 收口报告\n' +
      '  • 新增命令记得在 `lib.rs invoke_handler!` 注册 + `src/dev/IpcPlayground` 加 metadata'
    );
  }
}

if (suggestions.length === 0) process.exit(0);

const message = '【check-suggest】下一步检查建议(advisory,不强制):\n\n' + suggestions.join('\n\n');

// PostToolUse 输出 additionalContext 注入到 Claude 上下文
process.stdout.write(JSON.stringify({
  hookSpecificOutput: {
    hookEventName: 'PostToolUse',
    additionalContext: message,
  },
}));

process.exit(0);
