#!/usr/bin/env node
// 拦截对 docs/AIPET-obsidian/_archive/ 与 ADR-001~014 的误改
// 触发于 Edit / Write / MultiEdit;ADR-015+ 与其他文件放行
// 详见 CLAUDE.md § 守则
const fs = require('fs');

let raw;
try {
  raw = fs.readFileSync(0, 'utf-8');
} catch (e) {
  process.exit(0); // 无 stdin 则放行
}

let input;
try {
  input = JSON.parse(raw);
} catch (e) {
  process.exit(0); // 解析失败则放行
}

const filePath = (input.tool_input && input.tool_input.file_path) || '';
const isArchive = /[/\\]_archive[/\\]/.test(filePath);
const isAcceptedAdr = /M0-ADRs[/\\]ADR-(00[1-9]|01[0-4])-[\w-]+\.md$/i.test(filePath);

if (isArchive || isAcceptedAdr) {
  const reason = isArchive
    ? '禁改 _archive/(v0.1-v0.7 历史归档,实施期参考即误)。详 CLAUDE.md § 守则。'
    : '禁改 ADR-001~014(已 Accepted)。新决策走 M0-ADRs/ADR-015+(由 adr-author 角色起草)。详 CLAUDE.md § 守则。';

  process.stdout.write(JSON.stringify({
    hookSpecificOutput: {
      hookEventName: 'PreToolUse',
      permissionDecision: 'deny',
      permissionDecisionReason: reason,
    },
  }));
  process.exit(0);
}

process.exit(0);
