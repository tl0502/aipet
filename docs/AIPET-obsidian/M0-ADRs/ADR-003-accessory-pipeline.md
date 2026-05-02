# ADR-003: 配饰美术管线兼容性

> **⚠️ 状态变更 2026-05-01(post-M0)**:本 ADR 选定的"路径 A:Live2D + native 插槽"已随 ADR-002 一同**废止**(项目切到 VRM 3D)。设计意图(运行时叠加 0-3 件、切换 < 500ms、用户上传 P2)保留;实现路径改为 **VRM humanoid bone attach + VRMC_node_constraint**,配饰为独立 .glb / .vrma 节点,运行时挂到 head/neck/face 等骨骼。M4 实施前需输出新的"VRM 配饰美术规范"。
>
> 受影响:`accessory_*_slot` 参数命名作废(VRM 不需要,直接用 humanoid bone enum:`head` / `neck` / `leftEye` 等)。
>
> 下方原决策内容保留作为历史背景。

- **状态**: ~~Accepted~~ → **Superseded(2026-05-01)**
- **决策日期**: 2026-05-01
- **Owner**: M0 决策周(产品 + 美术 + 前端)
- **Reviewers**: M0 决策周复审通过
- **目标完成**: M0 决策周内(W0,基于 ADR-002 选定方案后立即开始)
- **影响范围**: O 装扮系统(M4 实施)
- **依赖**: ADR-002

## 背景

模块 O 要求:
- 用户可在桌宠上同时叠加 0-3 件配饰(帽子 / 围巾 / 眼镜)
- 配饰是独立资源,与桌宠模型解耦
- 切换 < 500ms
- 节气/节日皮肤为整套替换(不与单件配饰冲突)
- 用户上传自定义配饰为可选(P2)

ADR-002 选定的资源管线对"运行时 layered sticker 叠加"的支持程度,直接决定本 ADR 的实现路径。

## 备选方案(按 ADR-002 不同选定动态展开)

### 路径 A:ADR-002 = Live2D Cubism 4 → 使用 native 插槽

**实现**
- 美术在 Cubism Editor 里为桌宠模型预留"配饰挂载点"参数(parameter group):
  - `accessory_head_slot`(帽子位)
  - `accessory_neck_slot`(围巾位)
  - `accessory_face_slot`(眼镜位)
- 每个配饰是独立 PNG + JSON(锚点 + z-index + scale + 关联 slot)
- 运行时通过 SDK 在指定 slot 加载/卸载 PNG

**优点**
- 切换性能好(< 200ms)
- 美术工作量低(配饰是独立小图层,非完整重建模型)
- 用户上传自定义配饰可行(P2)

**缺点**
- 需要美术在建模时正确预留 slot(后期补做成本高)
- z-index 策略需要前端约定

### 路径 B:ADR-002 = Spine 2D → 使用 skin 部分覆盖

**实现**
- Spine 的 skin 系统支持"部分 attachment 替换"
- 美术为每件配饰做一个 partial skin

**优点**
- Spine native 支持

**缺点**
- 配饰不能任意自由叠加,组合数爆炸
- 美术工作量高(每个组合都要预定义)

### 路径 C:ADR-002 = PNG 序列帧 → DOM/Canvas 上叠加

**实现**
- 桌宠序列帧渲染在底层 Canvas
- 配饰 PNG 渲染在上层 div,根据动作锚点同步位置

**优点**
- 简单直接

**缺点**
- 桌宠动作每帧切换时,配饰位置需要逐帧同步,误差大
- 高 DPI 下抖动明显

### 路径 D(降级):配饰仅做"整套皮肤"

**实现**
- 不支持自由叠加
- 每件配饰其实是一套完整皮肤(包含桌宠 + 该配饰)
- 切换时整体替换

**缺点**
- O.1 配饰系统弱化为"O.2 节气皮肤"的子集
- KPI 11.17 装扮使用率受影响(用户少了"自由搭配"乐趣)

## 我的倾向

**优先级**:
1. **🌟 强烈倾向路径 A**(Live2D + native 插槽)— 假设 ADR-002 = Live2D。
2. 路径 B 仅在 ADR-002 = Spine 时考虑。
3. 路径 D 作为 spike 失败时的降级兜底。

## 决策

**选定:路径 A — Live2D + native 插槽叠加**(基于 ADR-002 选 Live2D)。

每个桌宠模型在 Cubism Editor 中预留挂载点参数:`accessory_head_slot`、`accessory_neck_slot`、`accessory_face_slot`、`accessory_ear_slot`、`accessory_back_slot` 等;每件配饰为独立 PNG(透明通道)+ JSON 锚点描述(x/y/scale/z_index/layer_target)。运行时通过 SDK 在指定 slot 加载/卸载,切换 ≤ 500ms。

**降级兜底**:若 ADR-002 spike 验证失败导致 Live2D 不可用,降级到"路径 D 整套皮肤"(配饰仅作为节气整体皮肤,放弃自由叠加;牺牲 KPI 11.17 但保留 11.15/16/18-20)。

## 后果

### 正面
- 切换性能好(< 200ms),满足 PRD §10.1 装扮切换 < 500ms。
- 美术工作量低:配饰是独立小图层,非完整重建模型;品类扩展(P2 用户上传)可行。
- 与 ADR-011 装扮 schema 的 `anchor.layer_target` 字段直接对接。

### 负面
- 美术建模时必须正确预留 slot;后期补做 slot 成本极高,要求美术规范一次到位。
- z-index 策略需前端约定(配饰之间的叠加顺序),M4 实施前输出规范文档。

## 实施动作

- [ ] 输出"配饰美术规范":图层命名 / 锚点 JSON 字段 / 推荐尺寸 / 透明通道 / z-index 范围
- [ ] 与 ADR-002 的"美术资源交付规范"合并(若来自同一管线)
- [ ] 内置 8 件配饰 + 4 套节气皮肤的产品定稿(品类 / 风格)
- [ ] 输出"配饰资源校验工具"(本地脚本检测 PNG 透明 / 锚点 JSON 合法)
- [ ] 与 ADR-011(装扮付费 schema)对齐 `tier` / `unlock` 字段

## 引用

- PRD v0.6 §7.15(模块 O)
- 架构 v0.3 §2.3(WardrobeService)
- 架构 v0.3 §13.1(风险 #1)
- 人格设计 v0.2 §1.6(装扮归桌宠不归人格)

## 复审签字

| Reviewer | 角色 | 同意? | 备注 |
|---|---|---|---|
| M0 决策周 | 产品 | ☑ | 与 PRD §2.M0 决策 23 一致 |
| M0 决策周 | 美术 | ☑ | 美术交付规范 M0 末输出 |
| M0 决策周 | 前端 | ☑ | z-index 策略 M4 前确定 |
