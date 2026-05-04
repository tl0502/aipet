#!/usr/bin/env node
// PostToolUse hook:Edit / Write / MultiEdit 后,根据文件路径建议跑哪些检查
// advisory only — 不拦工具调用,仅在 Claude 上下文注入「下一步可考虑」提示
// 详见 progress/audit-coverage-2026-05-04.md § 三层智能触发(L1 hook)
// 4 个高质量触发点:依赖/构建 / Vue 组件 / 构建发布配置 / DB schema 迁移

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
