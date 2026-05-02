# 风险登记表

> 来源:[路线图 §6.1](../docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md)。每周末 retro 时同步状态。

状态枚举:
- `closed` — 风险已消除(决策替代 / 完成实施)
- `mitigated` — 已应用缓解动作,持续监控
- `active` — 显化窗口期内,主动监控
- `upcoming` — 显化窗口期未到,提前准备
- `triggered` — 已显化,正在处理

---

| # | 风险 | 显化时机 | 影响 | 缓解 | 状态 | 备注 |
|---|---|---|---|---|---|---|
| 1 | ~~Live2D 商用授权~~ | M0 末 spike | M0 选型 blocked | 已切 VRM(MIT 开源,无授权风险) | **closed** | ADR-002/003 Superseded(2026-05-01) |
| 2 | Tauri AV 软件误报 | M1 测试 | 用户启动失败 | SmartScreen 信誉申请、AV 厂商白名单 | **upcoming** | M1 W1 D3+ 进入测试期监控 |
| 3 | WebView2 缺失(老 Win10) | M1 安装期 | 应用打不开 | 安装包内置 Bootstrapper | **upcoming** | M1 末打包测试时验证 |
| 4 | RAWINPUT 实现复杂 | M2 spike | N.4 键鼠协同延期 | 降级"快速 idle 切换"近似信号(不影响其他 N 子项) | **upcoming** | M2 关键路径,W3 必须决断 |
| 5 | 物理动作美术工作量大 | M2 实施 | 美术延期 | 12 个核心动作上限(ADR-004),复用 stretch/yawn | **upcoming** | M2 动作清单已锁定,降低风险 |
| 6 | GetLastInputInfo RDP 不一致 | M3 测试 | 主动关心误触发 | RDP 场景默认关闭模块 J | **upcoming** | M3 测试期覆盖 |
| 7 | Tauri file-drop 跨版本断裂 | M3 集成测试 | 文件拖入功能断裂 | M0 锁定 Tauri 2.x 版本,M3 集成测试覆盖 | **mitigated** | Tauri 2.1.0 已锁,版本固定 |
| 8 | Milestone 时区跨日 | M3 + M5 | 重复 / 漏触发 | 本地时区 + 启动期幂等检查 + `milestones.id` PK 唯一 | **upcoming** | M3 实施 MilestoneService 时单测必含跨午夜场景 |
| 9 | VRM 渲染内存超 250MB | M4-M5 性能调优 | 性能预算超 | LOD 切换 / 低多边形 / 低分辨率贴图模式兜底 | **upcoming** | M0 spike 已验证 < 150MB,但叠加配饰后需重测 |
| 10 | 节气推送被认为打扰 | M5 灰度 | 装扮使用率(KPI 11.17)不达标 | 默认每节气仅推 1 次;用户拒绝当年不再推 | **upcoming** | M4 实施时设默认值 |
| 11 | DPAPI 跨用户切换异常 | M3 + M5 测试 | 多用户机器混用 | 作为 feature 暴露(账户绑定) | **upcoming** | M3 实施 CryptoService 时文档化 |
| 12 | LLM 游戏 token 月成本 | M5 上线后 | 用户账单爆炸 | 单次 2000 token 上限 + 设置可见消耗统计 + 告警 | **upcoming** | M5 GameEngine 必含 token 上限熔断 |
| 13 | 自由活动被关(KPI 11.12) | M5 灰度后 | 生命感关闭率 > 15% | 灰度数据驱动;> 15% 默认关闭"逛桌面"子项 | **upcoming** | M5 灰度数据决定 |

---

## 实施期新增风险(M0 后发现)

(暂无)

---

## Retro 周期

- 每周末扫一遍本表 → 状态更新 → 即将进入显化窗口的提前准备
- Milestone 出口由 gate-checker 在 `progress/gate-m{N}.md` 中引用本表状态
