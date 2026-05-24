# Mindustry Java → Rust 迁移总控文档

本文档用于约束后续 AI/开发者持续迁移，目标是防止漏迁移、跑偏目录、把工程做成孤立模块，或忘记最终要交付的是可整合、可联机、可游玩的 Rust 版 Mindustry/MDT。

## 1. 最终目标（必须保持）

把 Java 参考仓库：

```text
D:/MDT/mindustry-upstream-v157.4
```

逐文件、逐目录对照迁移/重写到 Rust 工作仓库：

```text
D:/MDT/rust-mindustry
```

最终交付物必须是一个完整 Rust 版 Mindustry/MDT：

- 能作为统一工程构建，而不是互相独立的 helper 集合；
- 目录和模块尽量贴近原 Java 项目结构；
- 游戏数据、规则、世界、实体、方块、单位、AI、网络、客户端、服务端逐步闭环；
- 尽量实现 Rust 客户端与原版 Java 服务端/客户端在联机协议层面的互通；
- 长期目标是可启动、可进入世界、可联机、可游玩。

不要把目标降级成：

- 只写协议 demo；
- 只写数据结构翻译；
- 只写测试 helper；
- 只做不可运行的模块堆砌。

## 2. 目录与仓库规则

### 2.1 唯一参考目录

只以此目录作为 Java 上游参考：

```text
D:/MDT/mindustry-upstream-v157.4
```

当前参考基线：

```text
tag: v158
commit: ed54566
```

如果用户再次要求“更新至 158 / 拉取覆盖本地参考基线”，必须先确认该目录当前 tag/commit，再继续迁移。

### 2.2 唯一工作目录

只在此目录修改 Rust 工程：

```text
D:/MDT/rust-mindustry
```

远端：

```text
https://github.com/Anon-deisu/mindustry-rust
```

只推送：

```text
main
```

禁止推送 `master`。

### 2.3 废案目录

不要读取、参考或写入此废案目录，除非用户明确要求考古：

```text
D:/MDT/mindustry-rust
```

## 3. 交互与工作风格

- 始终使用简体中文回复；
- 可以适当同步“正在做什么 / 下一步做什么”，但不要因为同步而停止长期工作；
- 用户多次要求“继续 / 不要停”，应理解为继续推进迁移闭环；
- 遇到明确可执行的迁移、修复、测试、文档补齐任务，直接执行；
- 只有在实现路径存在高风险分歧且无法从本地上下文判断时，才停下来问一个关键问题；
- 中大型任务要主动使用子代理：
  - `explorer`：只读探索、定位 Java/Rust 对应关系、梳理缺口；
  - `worker`：边界清晰的局部实现、测试补齐、文档更新；
  - `ultra_worker`：复杂核心迁移、疑难调试、高风险联机互通；
- 子代理池满时不要卡住，主线程继续推进；
- 遇到文字乱码时，优先按 UTF-8 重新读取，再尝试其他编码；不要直接判定文件损坏。

## 4. 每个迁移点的标准流程

每次迁移一个 Java 文件、一个 Rust 文件、一个行为闭环或一个明确功能点时，按以下顺序执行：

1. **确认 Java 源头**
   - 找到 `D:/MDT/mindustry-upstream-v157.4` 下对应 Java 文件；
   - 记录类名、内部类、关键字段、生命周期方法、序列化/网络方法、测试相关行为。
2. **确认 Rust 落点**
   - 优先修改现有 Rust 文件；
   - 新文件必须放在与 Java 模块结构对应的位置；
   - 不要为方便而创建脱离工程主线的临时模块。
3. **对照行为而不是只翻译语法**
   - 字段默认值；
   - update/tick/lifecycle；
   - init/load/afterPatch；
   - read/write/network wire format；
   - Java 集合顺序与边界条件；
   - 浮点阈值、tile/world 坐标、team/player/id 语义。
4. **接入运行态**
   - 新结构必须尽量被现有 `GameState`、`World`、`NetClient`、`NetServer`、实体/方块/AI 生命周期使用；
   - 避免只添加未调用 helper。
5. **补测试**
   - 最少补一个锁定 Java 行为的 Rust 单元测试；
   - 网络/序列化要补 Java-like payload 或 roundtrip；
   - AI/世界/方块要补状态转移或边界条件。
6. **格式化与验证**
   - 运行定向测试；
   - 运行 `cargo check -p mindustry-core` 或相关 crate check；
   - 需要时再跑 workspace 测试。
7. **提交与推送**
   - 中文提交标题；
   - 只推送到 `origin main`；
   - 每个明确迁移闭环或文件级重构完成后提交一次。
8. **更新清单**
   - 在本文档或交接文档中记录已完成/下一步；
   - 如果发现 Java 文件已覆盖，写入已迁移清单；
   - 如果只是骨架，必须标记“部分迁移”，不要假装完成。

## 5. 防漏迁移方法

### 5.1 文件级对照

每个 Java 文件必须最终归入一种状态：

- `未开始`：没有 Rust 对应实现；
- `骨架`：只有类型/模块占位；
- `部分迁移`：已有部分字段/方法/测试，但未闭环；
- `行为迁移`：关键行为、生命周期、序列化或网络格式已迁移；
- `运行态接入`：已被 Rust 主流程调用；
- `完成待回归`：通过定向测试，但需要后续全局回归；
- `完成`：已接入主流程并有测试覆盖。

禁止只按“有同名 rs 文件”判断完成。

### 5.2 目录级进度快照

当前粗略文件数快照：

```text
upstream core/src .java: 774
rust core/src .rs:      352
```

按 `core/src/mindustry` 子目录粗略统计：

| 子目录 | Java 文件数 | Rust 文件数 | 说明 |
| --- | ---: | ---: | --- |
| world | 258 | 59 | 最大缺口，方块/建筑/世界行为需要长期推进 |
| entities | 135 | 71 | 已有一定迁移，但实体运行态仍需闭环 |
| ui | 71 | 6 | 客户端可游玩前的大缺口 |
| graphics | 38 | 5 | 渲染与图形资源缺口大 |
| maps | 34 | 5 | 地图加载/编辑/规则仍需推进 |
| type | 33 | 30 | 类型结构较接近，但行为仍需核查 |
| logic | 33 | 50 | Rust 拆分较细，需对照行为而非数量 |
| ai | 29 | 14 | BuilderAI/PrebuildAI 已推进，其他 AI 待补 |
| io | 24 | 18 | Save/NetworkIO 仍需补后半段 |
| editor | 22 | 1 | 基本未迁移 |
| game | 21 | 21 | 数量接近但必须核查运行态 |
| net | 15 | 16 | 网络骨架存在，互通仍需持续验证 |
| content | 15 | 16 | 内容声明/加载需要继续对照 |
| core | 13 | 10 | 主生命周期仍需补齐 |

该表只用于发现风险，不代表完成度。

### 5.3 推荐生成后续清单

后续应创建机器可更新清单，例如：

```text
docs/migration/file-map-core.csv
```

建议字段：

```text
java_path,rust_path,status,owner,last_commit,tests,notes
```

每次迁移更新对应行，避免“凭记忆继续”。

## 6. 常用命令

### 6.1 Git Bash 推荐入口

在当前 Windows 环境中优先显式调用 Git Bash：

```powershell
& "C:/Program Files/Git/bin/bash.exe" -lc 'git -C "D:/MDT/rust-mindustry" status --short'
```

避免 PowerShell 误解析 Bash 的 `&&`、重定向、引号。

### 6.2 状态核查

```bash
git -C "D:/MDT/rust-mindustry" status --short
git -C "D:/MDT/rust-mindustry" branch --show-current
git -C "D:/MDT/rust-mindustry" log --oneline -10
git -C "D:/MDT/rust-mindustry" remote -v

git -C "D:/MDT/mindustry-upstream-v157.4" describe --tags --always --dirty
git -C "D:/MDT/mindustry-upstream-v157.4" rev-parse --short HEAD
git -C "D:/MDT/mindustry-upstream-v157.4" status --short
```

### 6.3 统计对照

```bash
find "D:/MDT/mindustry-upstream-v157.4/core/src" -type f -name "*.java" | wc -l
find "D:/MDT/rust-mindustry/core/src" -type f -name "*.rs" | wc -l

find "D:/MDT/mindustry-upstream-v157.4/core/src/mindustry" -type f -name "*.java" \
  | sed 's#D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/##' \
  | awk -F/ '{print $1}' | sort | uniq -c | sort -nr
```

### 6.4 Cargo

Cargo 路径：

```text
C:/Users/yuyu/.cargo/bin/cargo.exe
```

常用：

```bash
"C:/Users/yuyu/.cargo/bin/cargo.exe" fmt -p mindustry-core
"C:/Users/yuyu/.cargo/bin/cargo.exe" check -p mindustry-core
"C:/Users/yuyu/.cargo/bin/cargo.exe" test -p mindustry-core <test_filter> -- --nocapture
"C:/Users/yuyu/.cargo/bin/cargo.exe" test --workspace -- --test-threads=1
```

已知 `cargo fmt -p mindustry-core` 可能带出无关格式化：

```text
core/src/mindustry/game/rules.rs
```

如果只是历史格式噪音，提交前撤销：

```bash
git -C "D:/MDT/rust-mindustry" checkout -- "core/src/mindustry/game/rules.rs"
```

### 6.5 提交与推送

```bash
git -C "D:/MDT/rust-mindustry" add <files>
git -C "D:/MDT/rust-mindustry" commit -m "<中文提交标题>"
git -C "D:/MDT/rust-mindustry" -c http.version=HTTP/1.1 \
  -c http.proxy=http://127.0.0.1:10808 \
  -c https.proxy=http://127.0.0.1:10808 \
  push origin main
```

如果代理失败，按实际网络情况重试；不要推送到 `master`。

## 7. 已完成/已推进的关键区域

### 7.1 网络与 world stream

已推进：

- 服务端/客户端真实网络入口；
- 服务端端口参数；
- 桌面客户端参数连接；
- 最小 world stream 下发；
- Rust 客户端解析 world stream 前置信息；
- `ConnectConfirmCallPacket` 确认逻辑；
- SaveIO 尾部前缀解析：content header、content patches、map、team blocks；
- `GameState::apply_network_world_data(...)` 接入部分地图/波次/locales/patcher 状态。

仍需：

- 完整 Java 兼容 `NetworkIO.writeWorld/loadWorld`；
- markers/custom chunks 的 UBJSON/JsonIO 兼容；
- player/groups/world/entity 的完整应用；
- 与 Java 原版服务端/客户端的持续互通测试。

### 7.2 BuilderAI / PrebuildAI

已推进：

- `BuilderAiRuntimeState`
- `BuilderAiRuntimeInput`
- `BuilderAiRuntimeStep`
- `BuilderAiRuntimeBranch`
- `BuilderComp::apply_builder_ai_tick(...)`
- `UnitComp::tick_builder_ai(...)`
- following/assist 搜索 helper；
- 移动范围校准；
- 部分 PrebuildAI 单位状态挂载。

仍需：

- 继续对照其他 AI 类型；
- 确保 AI 输出实际接入单位控制、建筑队列和世界状态；
- 扩充 Java 行为测试。

### 7.3 BuildTurret

已推进：

- `BuildTurretState`
- `BuildTurretPlanAction`
- `BuildTurretPlanValidation`
- `BuildTurretUpdateAction`
- `BuildTurretUpdateStep`
- `BuildTurretUnitConstructor`
- `BuildTurretUnitTypeConfig`
- `BuildTurretUnitTickInput`
- `BuildTurretUnitTickStep`
- `BuildTurretUnitBinding`
- `BuildTurretDrawCommand`
- `BuildTurretDrawPlan`
- `build_turret_update_tick(...)`
- `build_turret_unit_tick(...)`
- `apply_build_turret_unit_tick(...)`
- `build_turret_draw_plan(...)`
- `build_turret_unit_type(...)`
- `apply_build_turret_unit_type_defaults(...)`
- `build_turret_after_patch_unit_type(...)`
- `build_turret_after_patch_unit_type_config(...)`
- `build_turret_write_child_with_loader(...)`
- `build_turret_read_child_with_loader(...)`
- `build_turret_capture_unit_plans(...)`
- `build_turret_apply_unit_plans(...)`
- `build_turret_sense_from_plan(...)`
- `build_turret_sense_object_from_plan(...)`
- following/队伍 plan/自建 plan 清理等部分 `BuildTurretBuild.updateTile()` 逻辑。
- `BuildTurret.init()/afterPatch()` 的内部 `unitType` 配置与同步逻辑。
- `BuildTurretBuild.updateTile()` 前半段单位刷新 planner，并已薄接入 `UnitComp`：
  - unit 绑定到炮台位置/队伍；
  - rotation/warmup 写回 `BuildTurretState`；
  - `lookAt` 写回 unit rotation；
  - `buildSpeedMultiplier/speedMultiplier` 写回 unit status 与 builder view。
- `BuildTurretBuild.draw()` 的纯 draw plan：
  - base；
  - turret shadow/body 使用 `rotation - 90`；
  - glowRegion 存在时用 `warmup`；
  - `efficiency > 0` 时绘制 unit building beam。
- `BuildTurretBuild.write/read()` 已新增基于 `ContentLoader` 与 `TypeIO.writePlans/readPlans` 的 typed plans 读写路径；旧 raw 路径保留作兼容兜底。
- `BuildTurretState.plans` 与 `UnitComp.builder.plans` 已有双向桥接，typed read/write 后可恢复到单位建造队列。
- `BuildTurretBuild.sense/senseObject()` 对 `buildX/buildY/building/breaking` 的 unit build plan 转发语义已有轻量 helper。

仍需：

- 继续补完整 `BuildTurretBuild` 运行态；
- 接入完整 world/building runtime。

## 8. 当前真实完成度口径

如果以“完整可游玩 Rust Mindustry/MDT，且尽量联机互通”为 100%，当前仍是早期阶段，建议对用户口径保持保守：

```text
约 6%～9%
```

原因：

- Rust workspace、网络骨架和部分 world stream 已经存在；
- 一些 AI/方块局部行为已迁移并有测试；
- 但 UI、渲染、完整世界、实体、内容加载、存档、地图、方块运行态、完整客户端/服务端 gameplay 仍大量缺失。

不要因为已有很多 Rust 文件就宣称接近完成。

## 9. 下一步推荐路线

### 9.1 优先补迁移总索引

创建机器可更新清单：

```text
docs/migration/file-map-core.csv
```

先覆盖 `core/src/mindustry`，后续扩展到 desktop/server/tools/android/ios/annotations。

### 9.2 继续 BuildTurret

参考：

```text
D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/world/blocks/defense/BuildTurret.java
```

优先补：

- `BuildTurretBuild` 剩余运行态。
- `BuildTurretBuild.unit()/canControl()/buildRotation()/warmup()` 与真实 block runtime trait 接口；
- 将当前 helper 汇总为真实 `BuildTurretBuild` runtime adapter，而不是只由测试直接调用。

### 7.4 Thruster

已推进：

- `ThrusterBlockConfig`
- `ThrusterDrawCommand`
- `ThrusterDrawPlan`
- `thruster_top_rotation(...)`
- `thruster_draw_plan(...)`
- 已对照 `Thruster.java` 锁定：
  - 构造器 `rotate = true`；
  - 构造器 `quickRotate = false`；
  - plan/build 绘制顺序为 base region 后 topRegion；
  - top rotation = `rotation * 90` / `rotdeg()`。

仍需：

- 接入真实 block rendering adapter；
- 将 Thruster 类型配置接入内容块声明，而不是只在 helper 层测试。

### 7.5 Door / AutoDoor

已推进：

- `DoorState`
- `door_check_solid(...)`
- `door_sense_enabled(...)`
- `door_can_toggle(...)`
- `door_tapped_should_configure(...)`
- `write_door_state(...)`
- `read_door_state(...)`
- `auto_door_should_open(...)`
- `auto_door_trigger_size(...)`
- `AutoDoorUpdatePlan`
- `AutoDoorSetOpenPlan`
- `auto_door_update_plan(...)`
- `auto_door_set_open_plan(...)`
- 已对照 `AutoDoor.updateTile()` 锁定：
  - timer 未到或 net client 时不扫描/不发送 toggle；
  - 触发范围内存在 ground 且非 allowLegStep 单位时应打开；
  - open 状态变化时才发送 toggle；
  - `setOpen` 总是更新 pathfinder，只有 `wasVisible` 时播放 effect/sound。

仍需：

- `Door` 的 chained doors flood-fill 与批量 configure 规划；
- `Door.getPlanRegion()` / `AutoDoor.draw()` / `Door.draw()` 的 open-region 选择 helper；
- 接入真实 Units/tree/pathfinder/Call runtime。

已完成 Rust 结构：

```text
BuildTurretUnitTypeConfig
```

已测试锁定：

- name = `turret-unit-{block_name}`；
- hidden/internal = true；
- speed/hitSize/itemCapacity = 0；
- health = 1；
- afterPatch 同步 rotateSpeed/buildBeamOffset/range/buildSpeed。

### 9.3 继续 NetworkIO / SaveVersion

参考：

```text
D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/net/NetworkIO.java
D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/io/SaveIO.java
D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/io/versions/SaveVersion.java
```

优先补：

- marker/custom chunks 精确拆分；
- UBJSON/JsonIO bytes；
- world stream 应用到 `World`；
- player/groups/entity 生命周期。

### 9.4 后续大型目录

建议按“能形成可游玩闭环”的优先级，而不是纯字母顺序：

1. `core/net` + `core/core`：联机协议和主生命周期；
2. `core/io` + `core/maps`：存档、地图、规则；
3. `core/world`：方块、建筑、地形；
4. `core/entities` + `core/ai`：单位、子弹、AI；
5. `core/content` + `core/type`：内容声明、物品、液体、单位类型；
6. `desktop` + `core/ui` + `core/graphics`：窗口、输入、渲染、UI；
7. `server`：独立服务端完整运行；
8. `tools/android/ios/annotations/tests`：平台与工具链补齐。

## 10. 已知验证状态与风险

曾稳定通过的定向验证：

```bash
"C:/Users/yuyu/.cargo/bin/cargo.exe" test -p mindustry-core builder_ai_ -- --nocapture
"C:/Users/yuyu/.cargo/bin/cargo.exe" test -p mindustry-core build_turret -- --nocapture
"C:/Users/yuyu/.cargo/bin/cargo.exe" check -p mindustry-core
```

历史已知全包风险：

- 旧记录中 `cargo test -p mindustry-core` 曾有 `world_stream_with_java_like_payload_is_parsed_and_confirmed` 失败；
- 后续交接记录又显示 workspace 测试曾通过；
- 接手者必须以当前本地实测为准，不要只相信历史记录。

## 11. 接手者开工检查清单

每次新 AI/新会话接手，先执行：

```bash
git -C "D:/MDT/rust-mindustry" status --short
git -C "D:/MDT/rust-mindustry" branch --show-current
git -C "D:/MDT/rust-mindustry" log --oneline -10
git -C "D:/MDT/mindustry-upstream-v157.4" describe --tags --always --dirty
git -C "D:/MDT/mindustry-upstream-v157.4" rev-parse --short HEAD
```

然后阅读：

```text
D:/MDT/rust-mindustry/MIGRATION.md
D:/MDT/rust-mindustry/AI_HANDOFF.md
```

开工前必须确认：

- 工作目录是 `D:/MDT/rust-mindustry`；
- 参考目录是 `D:/MDT/mindustry-upstream-v157.4`；
- 当前分支是 `main`；
- 工作区是否已有未提交改动；
- 不使用 `D:/MDT/mindustry-rust`；
- 乱码先按 UTF-8 处理；
- 每个迁移闭环中文提交并推送 `origin main`。
