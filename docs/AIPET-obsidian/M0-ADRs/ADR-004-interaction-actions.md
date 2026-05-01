# ADR-004: 物理交互动作清单

- **状态**: Accepted
- **决策日期**: 2026-05-01
- **Owner**: M0 决策周(产品 + 美术)
- **Reviewers**: M0 决策周复审通过
- **目标完成**: M0 决策周内(W0,可与 ADR-002 并行)
- **影响范围**: N 物理交互(M2 实施)
- **依赖**: ADR-002

## 背景

模块 N 要求每个 hitbox + 每种交互方式都有差异化反应。如果不限制动作总数,美术会陷入组合爆炸:

- hitbox: head / body / tail / edge → 4
- 交互方式: click / double_click / long_press / right_click / drag(普通/晕眩/抗议)/ keyboard_burst → 7+

笛卡尔积 = 28+ 个独立动作 ID。**不可承受**。

需要决定:
1. 动作 ID 总数上限(架构 §13.1 提到 ≤ 12 个)
2. 哪些 hitbox × 交互方式共用一个动作(降维)
3. 默认 reaction_table 与 .soul.md `# 反应配置` 自定义的边界

## 备选方案

### 选项 A:**12 个核心动作 ID(推荐)**

| # | 动作 ID | 触发场景 |
|---|---|---|
| 1 | `head_pat` | head click |
| 2 | `tilt_head` | body click / 接受赞美 |
| 3 | `tail_wiggle` | tail / edge click(痒) |
| 4 | `lean_in` | body double_click(欢迎) |
| 5 | `surprised` | head double_click(被吓) |
| 6 | `fall_asleep` | body long_press / 时段 22:00+ |
| 7 | `stretch` | 06:00-09:00 时段 / 长时间未互动后回归 |
| 8 | `yawn` | 11:30-13:30 时段 |
| 9 | `dizzy` | drag 长距离/快速 |
| 10 | `protest` | drag 抗议(N.3)|
| 11 | `cheer` / `fist_pump` | keyboard_burst(N.4) |
| 12 | `rub_eyes` | 早晨 / 心情切换 sleepy |

right_click 不需要动作(直接弹快捷菜单)。

**优点**
- 美术工作量可控(12 个动作 × ~30 帧/动作 = 360 帧上限,Live2D 表达式则更少)
- 覆盖核心场景
- 复用率高(stretch / yawn / fall_asleep 同时被生命感和物理交互使用)

### 选项 B:**8 个极简动作 ID**(裁剪到核心)

砍掉:`lean_in / surprised / rub_eyes / cheer`,合并到 `head_pat / tilt_head` 的变体。

**优点**
- 美术工作量更低
- 上线更快

**缺点**
- N 模块体验单薄,各 hitbox × 交互方式的差异化不明显
- KPI 11.15(物理交互密度)可能受损(用户感觉点了没区别就懒得点)

### 选项 C:**24 个全覆盖动作 ID**

每个 hitbox × 主要交互组合一个独立动作。

**优点**
- 体验丰富

**缺点**
- 美术工作量超 MVP 范围
- 内存 / 包体积压力大

## 我的倾向

**🌟 倾向选项 A(12 个核心动作 ID)**:
1. 与架构 §13.1 风险缓解条款一致("动作清单上限 ≤ 12 个独立动作 ID")。
2. 高复用(stretch/yawn 同时服务 N + I,降低美术总工作量)。
3. 留 `.soul.md` 反应配置区段空间,允许人格自定义但不要求美术为每个人格做不同动作(用 tone 调速 + 文案差异)。

**默认 reaction_table 草案**(伪代码,M0 内确认):

```yaml
click:
  head:    { action: head_pat }
  body:    { action: tilt_head }
  tail:    { action: tail_wiggle }
  edge:    { action: tail_wiggle }
double_click:
  head:    { action: surprised }
  body:    { action: lean_in }
long_press:
  head:    { action: fall_asleep }
  body:    { action: fall_asleep }
right_click: 弹快捷菜单(无动作)
drag:
  normal:  { action: tilt_head }
  fast:    { action: dizzy }
  protest: { action: protest }
keyboard_burst: { action: cheer }
```

## 决策

**选定:选项 A — 12 个核心动作 ID**:
`head_pat / tilt_head / tail_wiggle / lean_in / surprised / fall_asleep / stretch / yawn / dizzy / protest / cheer / rub_eyes`。

right_click 不需要动作(直接弹快捷菜单)。

默认 reaction_table 按"备选方案"段草案落地;`.soul.md` 的 `# 反应配置` 区段(人格设计 v1.0 §2.2)可覆盖默认。

**N.4 键鼠协同的 RAWINPUT 实现**作为 M2 内 spike,如成本过高则降级为"快速 idle 切换"近似信号(架构 v1.0 §13.1 风险登记)。

## 后果

### 正面
- 美术工作量可控:12 动作 × ~30 帧 = 360 帧上限,Live2D 表达式更少,符合预算。
- 复用率高:`stretch / yawn / fall_asleep / rub_eyes` 同时被生命感(模块 I 日常时段)和物理交互(模块 N)使用,降低总美术工作量。
- 留出 `.soul.md` 反应配置覆盖空间,人格差异化由 tone 调速 + 文案差异承担,不要求美术为每个人格做不同动作。

### 负面
- hitbox × 交互方式不全是独立动作,需要默认 mapping 表(reaction_table)兜底,设计文案/UAT 需要明确边界。
- N.4 键盘 burst 的 RAWINPUT 实现可行性 spike 推到 M2,若失败需降级近似信号(不影响其他 N 子项)。

## 实施动作

- [ ] 锁定 12 个动作 ID 的命名规范(贴合 Live2D 模型 motion ID 命名)
- [ ] 美术输出每个动作的 spike 视频或 GIF 用于 verify
- [ ] 输出 default `reaction_table`(代码硬编码 + .soul.md 可覆盖)
- [ ] 跨人格的反应文案差异化(从 .soul.md 离线模板 `## 调侃` 抽取)
- [ ] N.4 键鼠协同的 RAWINPUT 实现可行性 spike(M2 内决定;ADR 不阻塞)

## 引用

- PRD v0.6 §7.14(模块 N)
- 架构 v0.3 §2.1(InteractionRouter)
- 架构 v0.3 §13.1(风险 #5)
- 人格设计 v0.2 §2.2(`# 反应配置` 区段)

## 复审签字

| Reviewer | 角色 | 同意? | 备注 |
|---|---|---|---|
| M0 决策周 | 产品 | ☑ | 与 PRD §2.M0 决策 24 一致 |
| M0 决策周 | 美术 | ☑ | 12 动作清单可控,Live2D 模型预留对应 motion ID |
