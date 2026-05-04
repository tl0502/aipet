---
description: UI 组件 a11y / i18n 巡检 — 键盘可达 + 焦点环 + ARIA + 中文文案硬编码 + 快捷键冲突。MVP 期目测,M5 灰度前接 Playwright + axe-core。
allowed-tools: Bash(grep:*), Bash(git rev-parse:*), Read, Grep, Glob, Write
argument-hint: [<vue 组件路径> | --all] [--report]
---

## 任务

巡检 Vue 组件的可访问性(a11y)与文案外置(i18n 准入)。

**只读**;每条 finding 给修复路径,修复由 module-implementer 接力。

## 前置上下文

- 当前 HEAD:
!`git rev-parse --short HEAD`

- src/components 清单:
!`Get-ChildItem src/components -Recurse -Filter *.vue -ErrorAction SilentlyContinue | Select-Object FullName`

- src/views 清单:
!`Get-ChildItem src/views -Recurse -Filter *.vue -ErrorAction SilentlyContinue | Select-Object FullName`

- progress/CURRENT.md:
@progress/CURRENT.md

## 范围解析

| `$ARGUMENTS` | 解析 |
|---|---|
| 空 / `--all` | 扫 `src/components/**/*.vue` + `src/views/**/*.vue` |
| `<path>` | 扫单个组件 / 单个目录 |
| `--report` | 同时写 `progress/a11y-{date}.md`(默认仅终端输出) |

## 检查项

### 关 1:键盘可达

- 每个 popup / modal / panel 必须支持 `@keydown.esc` 关闭
- 交互按钮必须 `tabindex="0"` 或原生 `<button>`(不要用 `<div @click>` 模拟按钮)
- 焦点环 `outline` 不可全局 `outline: none`(可定制但保留可见 focus indicator)

**grep 模式**:
- 找 `<div @click=` 但无 `tabindex` / `role="button"` → 违规
- 找 `outline: none;` 在 `*:focus` 上下文 → 违规
- 找 modal / popup 但无 `@keydown.esc` → 违规

### 关 2:屏幕阅读器(ARIA)

- VRM Canvas 应当有 `aria-label="桌宠 momo 渲染窗口"` 或同义替代描述
- 关键交互元素(发送按钮 / 关闭按钮 / 设置入口)必须 `aria-label`
- 装饰性元素 `aria-hidden="true"`

**grep 模式**:
- 找 `<canvas` 但无 `aria-label` → 违规(VRM 渲染必须有替代描述)
- 找 `<button>` 仅含 emoji / icon(无文字)但无 `aria-label` → 违规

### 关 3:文案硬编码(i18n 准入)

MVP 期仅 zh-CN,但禁止文案散落在组件里。统一外置到 `src/locales/zh-CN.ts`:

```ts
export const messages = {
  chat: {
    placeholder: "和默默说点什么…",
    sendButton: "发送",
    // ...
  },
  // ...
}
```

**grep 模式**:
- `grep -rn "[一-鿿]" src/components/ src/views/`
- 排除注释行(`// ...` / `/* ... */`)
- 排除已经在 `<i18n>` block 内的或 `t('key')` 调用的
- 命中 → 列出 file:line + 文本片段 → 建议外置 key

### 关 4:快捷键冲突

- 扫 `globalShortcut.register` / `useKeydown` / `@keydown.*.exact`
- 与 Win 系统级保留对照(不可绕过):
  - `Ctrl+Esc` (开始菜单)
  - `Win+L` (锁屏)
  - `Win+D` (显示桌面)
  - `Alt+Tab` / `Alt+F4`
  - `Ctrl+Alt+Del` / `Ctrl+Shift+Esc`
- 当前已用快捷键(进度记录 / decisions-log):
  - `Ctrl+Alt+Space`(总开关 ✅ 不冲突)
  - `Ctrl+Shift+B`(BossKey 摸鱼 ✅ 不冲突)
  - `Ctrl+Shift+D`(DEV-1 调试面板 ✅ 不冲突,但与浏览器开发者工具同名,需文档说明)

### 关 5:颜色对比度

- Chat 文字色 vs 透明窗口背景:WCAG 2.1 AA 对比度 ≥ 4.5:1
- MVP 期目测;M5 灰度前用 Lighthouse-CLI / axe-core 自动化

**MVP 期不强制**(标 N/A,M5 前补)

### 关 6:可控关闭

每个 popup / panel / modal 必须支持两种关闭途径:
- 点击外部区域关闭(click outside)
- 按 ESC 关闭(已在关 1 覆盖)

## 产物

终端输出 5 行内汇总;`--report` 触发时同时写 `progress/a11y-{YYYY-MM-DD}.md`:

```markdown
# A11y check {YYYY-MM-DD}

## Run {HH:MM:SS} — scope `<path>` / `--all`

**上下文**
- HEAD:`<short SHA>`
- 扫描组件数:N
- 总裁决:**Pass / Conditional / Block**

## §1 键盘可达

- 已检 N 个交互元素
- 违规:M 处

### 详情(若有)

#### A-1 [`<file>:<line>`] `<div @click>` 无 tabindex

- 现状:`<div class="action-btn" @click="...">点我</div>`
- 修复:改 `<button>` 或加 `tabindex="0" role="button"`

## §2 ARIA

- canvas 元素:N 个 / M 个有 aria-label
- 仅图标按钮:N 个 / M 个有 aria-label

### 详情(若有)

#### B-1 [`<file>:<line>`] canvas 无 aria-label

- 现状:`<canvas ref="petCanvas">`
- 修复:加 `aria-label="桌宠 momo 渲染窗口"`

## §3 文案硬编码

- 中文硬编码命中:N 处 in K 文件
- 已外置到 zh-CN.ts:M 处

### 详情(top 5)

- `src/components/ChatInput.vue:42` 「和默默说点什么…」 → 外置到 `chat.placeholder`
- ...

## §4 快捷键冲突

- 已注册:N 个
- 与系统保留冲突:M 个

### 详情(若有)

(无冲突 / 列出冲突项 + 备选快捷键)

## §5 颜色对比度

- (MVP 目测,M5 前用 Lighthouse / axe-core 自动化)

## §6 可控关闭

- popup / modal:N 个
- 缺 click-outside:M 个
- 缺 ESC:已在 §1 覆盖

### 详情(若有)

#### F-1 [`<file>:<line>`] modal 缺 click-outside

- 现状:`<Teleport to="body"> <div class="modal">...</div> </Teleport>`
- 修复:加 `v-click-outside` directive 或 `@click.self="close"` 在背景层

## 建议下一步

- P0:N 条键盘可达 + ARIA 违规(影响基本可用性)
- P1:M 条文案外置(M2 chat 模块前清理)
- P2:M5 RC 前接 Lighthouse / axe-core
```

## 执行规则

- **只读 + 写 progress/(可选)** — 不修组件
- **不引入 i18n 框架** — MVP 仅 zh-CN,先把文案集中到 `src/locales/zh-CN.ts` 文件;Vue I18n 等到 P1+ 多语言时再装
- **每条 finding 必须给修复路径** — 没修复路径的不算 finding
- **MVP 期 N/A 项可标过**(颜色对比度 / 完整 i18n)
- **不与 module-implementer 重复劳动** — module-implementer DoD 已含「自查 a11y」时,本命令仅做 milestone 末巡检

## 节奏

- UI 组件完成时 module-implementer 自查(默认终端输出)
- milestone 末巡检(`--report` 写报告 + gate-checker 引用)
- M5 灰度前接入 Playwright + axe-core E2E(超出本命令范围)

## 输出

终端 5 行内:

1. 扫描范围 + 组件数
2. 6 关裁决汇总(✅ / ❌ / N/A)
3. 总违规数(P0 / P1 / P2)
4. 报告路径(若 `--report`)
5. 建议下一步(模块级修 / milestone 末再扫)
