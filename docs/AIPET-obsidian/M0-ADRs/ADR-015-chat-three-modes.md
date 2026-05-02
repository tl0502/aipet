# ADR-015: 对话面板三形态架构

- **状态**: Accepted
- **决策日期**: 2026-05-02
- **Owner**: M1 D3 实施期(产品 + 前端 + 主进程协议)
- **Reviewers**: 用户(单人项目主)签字 2026-05-02
- **目标完成**: M1 D5 之前需 Accepted(已达成,2026-05-02 Accepted)
- **影响范围**: B 对话(全)、A 桌宠壳层(控制按钮区延伸)、G 设置、H 工坊、L 文件拖入、Q 游戏(launcher 入口)

## 背景

PRD v1.0 §7.2 与架构 v1.0 §3.225 把 ChatPanel 描述为单一"对话面板":全局快捷键 / 点击桌宠唤起,流式渲染,失焦关闭。这是 M0 决策周的简化抽象,够支撑模块依赖图但**未对窗口形态、入口数量、多 conversation、与工坊/设置/游戏的整合方式**做精细决策。

M1 D3 实施 A.5 全局快捷键占位时(commit `da0a6ad`),用户提出**三形态共享一份 conversation 数据**的完整设计:

1. **形态 1 hub 总面板**——含工坊 / 设置 / 对话历史 / 游戏 launcher 的 ChatGPT 式总控制台
2. **形态 2 磁吸浮窗**——黏在角色窗旁,可拖动脱离自由布置(异磁极感)
3. **形态 3 漫画对话气泡**——角色窗内沉浸式叠加

这些设计若不沉淀为 ADR,会扩散到 PRD/架构/flows v1.1 升级时 5 处文档同时变,且后续 M2-M5 的 story 切分将丧失锚点。本 ADR 固化形态架构以稳定后续 6 子 story(B.3.a-B.3.f)的实施切片。

## 备选方案

### 选项 A:维持原 PRD §7.2 单一对话窗

ChatPanel 仅一种形态:独立 Tauri 窗口或角色窗子组件二选一,工坊 / 设置 / 游戏各自独立窗口。

**优点**
- 工程量最小(B.3 仍按原 1d 估算可完结)
- 概念简单,用户学习成本低
- 不需要 ConversationStore 抽象

**缺点**
- 失去"和桌宠对话"的沉浸感(无形态 3 漫画气泡)
- 工坊 / 设置 / 游戏散落多窗口,UX 碎片化(用户记不住每个功能在哪)
- M3 多 conversation 时左侧栏挤在小窗口里体验差
- 与 ADR-012 GameRoom 独立窗口共生但缺统一入口

### 选项 B:仅形态 2 + 形态 3,不做形态 1 hub

实现磁吸浮窗 + 漫画气泡两形态,工坊 / 设置 / 游戏保持各自独立窗口(现状)。

**优点**
- 比选项 A 多了沉浸感(漫画气泡)
- 比选项 C 工程量小(省去 hub 整合工作)
- 可在 M2-M3 完整交付,M4-M5 不挤占带宽

**缺点**
- 工坊 / 设置 / 游戏仍散落,长期 UX 碎片化未解
- 用户从托盘菜单难形成"统一控制台"心智
- conversation 列表无显示载体(只能塞磁吸浮窗的左侧栏,小窗口拥挤)

### 选项 C:三形态共存 + ConversationStore 共享(本 ADR 决策)

3 种形态共享一个 view-agnostic 的 ConversationStore;形态间切换只换视图,数据保留。hub 整合工坊 / 设置 / 对话 / 游戏 launcher,GameRoom 仍独立窗口由 hub 调起。

**优点**
- 沉浸感(形态 3)、磁吸便利性(形态 2)、统一控制台(形态 1)三者兼得
- ConversationStore 是 view-agnostic 抽象,长期演进可复用(M6+ 加新形态零成本)
- 与 ADR-012 共生:hub 仅做 game launcher,GameRoom 沉浸感保留
- 控制按钮区作为模块 A 延伸,为 M2+ 未来按钮(摸鱼快捷 / 设置入口 / 形态切换)预留扩展位

**缺点**
- 总工程量大(B.3 一个 story 拆为 6 子 story,跨 M1-M5)
- 三形态切换的状态机增加测试面
- M4 hub 总面板 2d 工作量挤占装扮模块带宽

## 关键评估维度对比

| 维度 | A: 单一窗 | B: 仅 2+3 | C: 三形态(本决策) |
|---|---|---|---|
| 沉浸感 | ★★ | ★★★★ | ★★★★ |
| "和桌宠对话"语义 | ★★ | ★★★★★ | ★★★★★ |
| 跨模块整合(工坊/设置/游戏) | ★ | ★ | ★★★★★ |
| 多 conversation 体验 | ★★ | ★★★ | ★★★★★ |
| 开发工作量 | ★★★★★ low | ★★★ medium | ★★ high |
| 与摸鱼模式协调 | ★★★★ | ★★★ | ★★★★ |
| 与 Onboarding 解耦 | ★★★★★ | ★★★★★ | ★★★★★ |
| 与 ADR-012 GameRoom 协调 | ★★★ 各自独立 | ★★★ 各自独立 | ★★★★★ hub 做 launcher |
| 风险面(状态机复杂度) | ★ low | ★★ medium | ★★★ high |
| 长期可演进性 | ★★ | ★★★ | ★★★★★ |

## 决策

**选项 C:三形态共存 + ConversationStore 共享**。

核心理由:对话是桌宠应用的高频核心交互;桌宠产品的差异化在"沉浸式陪伴",仅做单一对话窗会浪费桌宠 3D 角色的视觉资产(VRM / 装扮 / 表情)。三形态对应"全功能(hub)/ 便利(磁吸)/ 沉浸(漫画气泡)"三种使用场景,view-agnostic ConversationStore 是工程上正交的抽象,长期收益超过短期工程量。

### 决策细则

#### 1. 三形态规范

| 形态 | 窗口形态 | 入口 | M 阶段 |
|---|---|---|---|
| **1 hub 总面板** | 独立 Tauri 窗口 `hub`,1024×680 | 托盘菜单 / 各 tab 入口 | M4 (B.3.e) |
| **2 磁吸浮窗** | 独立 Tauri 窗口 `chat`,默认 380×480 | `Ctrl+Alt+Space` / 点击桌宠 | M1 极简(B.3.a)→ M2 完整(B.3.c) |
| **3 漫画气泡** | 角色窗内子组件(非独立窗) | 控制按钮区某按钮 | M5 (B.3.f) |

#### 2. ConversationStore 共享层

- 数据层独立于视图,同一 `conversation_id` 的消息流在 3 形态间切换不丢失
- SQLite schema 加 `conversations` 表(`id / title / created_at / archived`),`messages` 表加 `conversation_id` 索引
- IPC 加 `conversation.list / create / rename / archive / delete / activate`
- 当前活跃 conversation_id 持久化到 `user_state.active_conversation_id`

#### 3. hub 与 GameRoom 关系(Q1)

**共存 launcher**。hub 的"游戏"tab 是游戏列表 + 启动按钮,点击调 `game_room.launch(id)` 创建独立 GameRoom 窗口(沿用 ADR-012 决策)。**ADR-012 不 Superseded**,GameRoom 沉浸感保留。

#### 4. hub 与 Onboarding 关系(Q2)

**Onboarding 保持独立**(label `onboarding`),完成后销毁;hub 不掺和。生命周期不同(一次性 vs 常驻),Onboarding UI 需全屏沉浸不被 tab 干扰。

#### 5. 形态切换语义(Q3)

**数据保留 + view 切换**。同一 `conversation_id`:
- 切到形态 3 时:形态 2 窗口 hide(不销毁),形态 3 子组件渲染同一消息流
- 切回形态 2 时:形态 3 子组件 unmount,形态 2 窗口 show 恢复
- 形态 1 hub 内对话 tab:可设为同一 conversation 或独立 conversation,用户拍板(默认同步当前活跃 conversation)

#### 6. 控制按钮区(模块 A 延伸,新概念)

- 角色下方一排圆角长方形按钮
- 每按钮带"自动隐藏开关":鼠标移开时是否隐藏(用户可关掉自动隐藏)
- 归属模块 A,不归 B(被 3 形态共用)
- M2 W3 启动 B.3.b 时定第一批按钮清单(见 Q5 TBD)

#### 7. M1 范围(Q6)

仅 **B.3.a 形态 2 极简版**:独立 chat 窗口、单 conversation、流式渲染、`Ctrl+Alt+Space` toggle、ESC / 失焦 hide。**不做**:磁吸 / 控制按钮区 / 多 conversation / 形态 1/3。

## 后果

### 正面

1. **架构稳定性**:ConversationStore view-agnostic 抽象,M6+ 加新形态(如 AR 眼镜端 / 移动端)零返工
2. **UX 一致性**:三形态共享数据,用户在不同场景切换无认知断层
3. **跨模块整合**:hub 在 M4 做 工坊 + 设置 + 对话 + 游戏 launcher 整合,消除现有"功能散落多窗"碎片化
4. **M1 风险可控**:M1 仅做形态 2 极简版,出口 KPI("核心 UI 跑通")可达成,复杂部分推迟到 M2-M5 渐进交付
5. **与 ADR-012 共生**:GameRoom 沉浸感保留,hub 只做入口,二者不互斥

### 负面

1. **总工程量增加**:B.3 从原 1d 估算扩展到 6 子 story 合计 ~7d 跨 M1-M5
2. **状态机复杂度**:磁吸 ↔ 断开 / 形态 2 ↔ 形态 3 / hub 内 conversation 切换,状态机面变大,测试面随之增加
3. **跨窗口 IPC 开销**:形态 1/2 是独立窗口,与角色窗 / GameRoom 之间消息同步需走 emit/listen,增加 5-15MB 内存(可吸收)
4. **首次签字延迟**:本 ADR Accepted 是 M1 后期 B.3.a 启动的硬前置;若 Q4/Q5 TBD 拖到 M2 W3,可能阻塞 B.3.b/c

## 实施动作

实施切片由 6 子 story 渐进交付:

```
M1 后期 (D5-D10)
[ ] I.1 SQLite schema:messages 表 + conversations 表(1d)
[ ] B.3.a 形态 2 极简版 chat 窗口(1d)
    - 独立 Tauri 窗口 label=chat
    - 单 conversation 消息流 + 输入区 + 流式渲染
    - Ctrl+Alt+Space toggle / ESC / 失焦 hide
    - 不做磁吸 / 不做控制按钮区 / 不做多 conversation

M2 (W3-W4)
[ ] B.3.b 控制按钮区骨架(模块 A 延伸,0.5d)
    - 角色下方按钮区组件 + 自动隐藏开关
    - 第一批按钮清单(Q5 TBD,启动前拍板)
[ ] B.3.c 形态 2 磁吸交互(1d)
    - 吸附 ↔ 断开状态机
    - 物理阈值(Q4 TBD,启动前拍板)
    - 失焦收缩到控制按钮区

M3 (W5-W6)
[ ] B.3.d 多 conversation 左侧栏(形态 2 内,1d)
    - conversations CRUD UI
    - 小窗口时左侧栏自动收起
    - 切换 / 重命名 / 归档 / 删除
[ ] L 模块文件拖入扩展(0.5d)
    - 接收源:形态 2/3 输入区 + 角色窗
    - 角色窗作为统一接收口

M4 (W7-W8)
[ ] B.3.e hub 总面板(2d)
    - 独立 Tauri 窗口 label=hub,1024×680
    - 4 tab:对话 / 工坊 / 设置 / 游戏 launcher
    - 工坊 + 设置从原各自独立窗收纳进 hub
    - 游戏 tab 调 game_room.launch 创建 GameRoom

M5 (W9-W10)
[ ] B.3.f 形态 3 漫画气泡(1.5d)
    - 角色窗内子组件叠加
    - 智能位置(左上 / 右上 / 正上,看屏幕剩余空间)
    - 通过控制按钮区某按钮激活
    - 与形态 2 数据保留切换
```

## 待决策项(TBD)

以下决策不阻塞当前 M1,但 M2 启动前必须拍板:

### TBD-1: 磁吸物理阈值(Q4)

启动 B.3.c 前定:
- 吸附判定距离:候选 30 / 50 / 80 px(逻辑像素)
- 吸附边偏好:自动选屏幕剩余空间 vs 用户固定
- 吸附动效:0.2s 滑入 vs 硬贴
- 边缘 case:多屏 / 桌宠靠近屏幕边缘时如何吸附

### TBD-2: 控制按钮区初始按钮清单(Q5)

启动 B.3.b 前定。候选按钮:
- [必] 形态 3 漫画气泡激活
- [建议] 形态 2 chat 窗口唤起
- [建议] 形态 1 hub 入口
- [可选] 摸鱼模式按钮(`Ctrl+Shift+B` GUI 替代)
- [可选] 设置入口
- [可选] 当前 conversation 切换器

### TBD-3: hub 内对话 tab 与磁吸 chat 窗口的同步语义

M4 启动 B.3.e 前定:
- 选项 A:hub 内对话 tab 与磁吸 chat 窗口 **同一 active_conversation_id**(默认)
- 选项 B:hub 内独立 active conversation,与磁吸 chat 窗各自记录
- 选项 C:hub 打开时自动同步 chat 窗口当前 conversation,之后独立

## 关联 ADR

- **ADR-005 默认 LLM Provider**:形态 1 hub 设置 tab 内承载 6 preset 配置(OpenAI/DeepSeek/Moonshot/通义/Ollama/自定义),不影响形态 2/3
- **ADR-006 安全前缀**:3 形态对话**全部**走 SecurityGuard,不可绕过(关键约束 5)
- **ADR-008 灵魂宣誓页文案**:与 Onboarding 关联;Q2 决策 Onboarding 不进 hub,故本 ADR 与 ADR-008 无重叠
- **ADR-009 三个内置人格**:人格切换路径不受形态影响;hub 内工坊 tab 承载人格切换,形态 2/3 内显示当前激活人格(仅显示,不切换)
- **ADR-012 小游戏 UI 风格**:**共生不替代**。GameRoom 独立窗口决策保持,hub 仅做 game launcher 入口。ADR-012 状态保持 Accepted 不变

## 跨 ADR 共识遵守

- ✅ Local-first:所有 conversation 数据本地 SQLite,不上传
- ✅ 用户自主权:用户可在 hub 设置 tab 删除全部 conversation 数据
- ✅ 非养成原则:形态切换不引入流失/必须签到机制
- ✅ 隐私边界:不引入读其他应用的能力
- ✅ 安全护栏不可绕过:SecurityGuard 在所有形态生效

## 复审签字

- **Owner 签字**:2026-05-02
- **Reviewer 1 签字**:用户(单人项目主)2026-05-02

签字完成,状态 Accepted,已同步更新 `M0-ADRs/README.md` 总表。

## 变更日志

- 2026-05-02:起草 v1,状态 Proposed
- 2026-05-02:Accepted(用户签字),进入实施;B.3.a 解锁可启动
