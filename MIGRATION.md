# Mindustry Java → Rust 迁移总控文档

**固定 Rust 工作路径（上下文压缩后不要重新找）：`D:\MDT\rust-mindustry`；命令中统一写作 `D:/MDT/rust-mindustry`。**

```text
CONTEXT_BOOTSTRAP_RUST_WORKDIR=D:/MDT/rust-mindustry
CONTEXT_BOOTSTRAP_JAVA_REFERENCE=D:/MDT/mindustry-upstream-v157.4
CONTEXT_BOOTSTRAP_FORBIDDEN_OLD_RUST_DIR=D:/MDT/mindustry-rust
CONTEXT_BOOTSTRAP_GIT_BRANCH=main
```

本文档用于约束后续 AI/开发者持续迁移，目标是防止漏迁移、跑偏目录、把工程做成孤立模块，或忘记最终要交付的是可整合、可联机、可游玩的 Rust 版 Mindustry/MDT。

> **压缩上下文后先读这一行：当前唯一 Rust 工作路径是 `D:\MDT\rust-mindustry`（等价命令路径 `D:/MDT/rust-mindustry`）。不要重新搜索、不要改用 `D:\MDT\mindustry-rust`，后者是废案。**

## 0. 固定路径速记（上下文压缩后优先看）

- Rust 工作仓库：`D:\MDT\rust-mindustry`（命令中可写作 `D:/MDT/rust-mindustry`）
- Java 参考仓库：`D:\MDT\mindustry-upstream-v157.4`（命令中可写作 `D:/MDT/mindustry-upstream-v157.4`）
- 废案目录，禁止参考/写入：`D:\MDT\mindustry-rust`
- Git 远端：`https://github.com/Anon-deisu/mindustry-rust`
- 只推送分支：`main`

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
tag: v158.1
commit: 05b2ecd
```

如果用户再次要求“更新至 158.x / 拉取覆盖本地参考基线”，必须先确认该目录当前 tag/commit，再继续迁移。

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
rust core/src .rs:      353
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
  -c http.proxy=http://127.0.0.1:10809 \
  -c https.proxy=http://127.0.0.1:10809 \
  push origin main
```

如果 10809 失败，再用 10808 混合代理重试；不要推送到 `master`。

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
- `SaveRegion::manifest_for_version(...)` 已对照 Java `SaveVersion` region 顺序和版本门控：
  - v11: meta/content/patches/map/entities/markers/custom；
  - v8-v10: 无 patches，但有 markers/custom；
  - v7: 无 markers，但有 custom；
  - v6 及更旧 manifest 不含 custom。
- `RawSaveEnvelope` 已补 deflated roundtrip 测试，覆盖完整 v11 region 和 v10/v8/v7/v6 门控 region set；
- `RawSaveEnvelope` 已接入 markers/custom 语义桥：
  - `set_markers_from_map_markers(...)`；
  - `markers_as_map_markers(...)`；
  - `set_custom_chunks(...)`；
  - `custom_chunks(...)`；
  - 已用 deflated envelope roundtrip 验证 marker UBJSON 与 custom chunk bytes 可恢复。
- `RawSaveEnvelope` 已接入结构化 region 桥：
  - content header snapshot；
  - content patches；
  - modern chunk map；
  - `SaveEntitiesRegion`（按 Java `writeEntities()` 顺序保留 `entityMapping bytes -> teamBlocks -> worldEntities bytes`）；
  - markers/custom。
- `GameState::apply_network_world_data(...)` 接入部分地图/波次/locales/patcher 状态。
- `GameState::advance_game_update_frame(...)` 已对照 Java `Logic.update()` 的非暂停 game 分支，提供最小帧推进入口：
  - 仅 `state.isGame() && !state.isPaused()` 时推进；
  - `tick += deltaSeconds * 60`，非有限 delta 视为 0；
  - 每个有效 game update frame 执行 `update_id += 1`；
  - 该入口为后续真实 building/entity dispatcher、`RegenProjector.lastUpdateFrame` 等 update-id 门控提供统一帧边界。
- `GameState::apply_legacy_team_blocks(...)` 已把 Java `SaveVersion.readTeamBlocks(...)` 输出落到 runtime `Teams.plans`；
- `Teams::to_legacy_team_blocks(...)` / `GameState::export_legacy_team_blocks(...)` 已补 Java `SaveVersion.writeTeamBlocks(...)` 形态导出：
  - 按 active teams 导出；
  - 可按 Java 行为兜底包含 sharded；
  - block name 由调用者映射为 content id；
  - 当前 runtime `BlockPlan.config` 以 Null/String 形式导出，typed config 保真仍待补。
- `GameRuntime::export_network_map_snapshot(&ContentLoader)` 已成为核心 runtime 的统一网络 map 快照导出入口，`mindustry_server::ServerLauncher::network_world_data_template(...)` 直接调用该入口，不再在服务端保留独立 map/building helper：
  - `export_network_map_snapshot_from_parts(...)` 接收 `World + &[BuildingComp]`，普通 block run 不再跨过 entity tile；
  - center tile 写 `has_entity=true/is_center=true/building=Some(...)`；
  - multi-tile footprint 的非中心 tile 写 `has_entity=true/is_center=false`，对齐 Java map chunk 的中心/非中心 building 记录形态；
  - 当前 building chunk payload 先写 `revision byte + BuildingComp::write_base(..., false)`，已能让 Rust runtime 读回 team/rotation/health/tile_pos/module base；
  - 导出侧已开始追加 block-specific tail：`Door` 写 1 byte open，`AutoDoor/blast-door` 写父/子双 bool，`ShieldWall` 写 shield f32，并通过 content registry 判定 block kind；
  - power/light 导出已覆盖 `PowerGenerator` 系列与 `LightBlock`：generator/reactor/heater 按 Java `version()==1` 写 `productionEfficiency/generateTime` 及子类字段，light 按 revision 0 写 color；
  - effect 导出已覆盖 `MendProjector / OverdriveProjector / ForceProjector / Radar / BaseShield / BuildTurret`：`BaseShield` 按 Java `version()==1` 写 `smoothRadius/broken`，其余按 revision 0 写 Java 子字段；
  - turret 导出已覆盖 `Turret` 基类与第一批子类：liquid/power/laser generic turret 按 Java `version()==1` 写 reload/rotation，`ItemTurret` 按 `version()==2` 追加 item ammo，`PayloadAmmoTurret` 写 payload ammo seq，`ContinuousTurret/ContinuousLiquidTurret` 按 `version()==3` 追加 lastLength，`PointDefenseTurret` 与 `TractorBeamTurret` 按 revision 0 写 rotation；
  - production 导出已覆盖 `Drill / BeamDrill / BurstDrill`：三者均按 Java `version()==1` 写持久化进度字段，读回时 runtime-only 派生字段按默认值恢复；
  - crafting 导出已覆盖 `GenericCrafter / AttributeCrafter / HeatCrafter / Separator / HeatProducer`：generic 系列按 revision 0 写 `progress/warmup`，`Separator` 按 Java `version()==1` 写 `progress/warmup/seed`，`HeatProducer` 按 Java 顺序写 generic 前缀再写 heat；
  - distribution 导出已覆盖第一批核心物流 building tail：`Conveyor/ArmoredConveyor` item slots、`StackConveyor` link/cooldown、`ItemBridge/DuctBridge` link/warmup/incoming/moved、`BufferedItemBridge` bridge+buffer、`MassDriver` link/rotation/state、`Duct` recDir、`DuctRouter/OverflowDuct/StackRouter` sort item、`Junction` 四向 buffer、`DirectionalUnloader` unload item/offset、`Sorter` sort item、`Unloader` sort item、`UnitCargoLoader` read unit id 与 `UnitCargoUnloadPoint` item/stale；其中 conveyor、bridge、buffered bridge、duct、duct router/overflow duct/stack router、junction、unloader 按 Java revision 1 写出，sorter 按 Java revision 2 写出；
  - payload 导出已覆盖 `PayloadConveyor / PayloadRouter / PayloadMassDriver / PayloadLoader / PayloadUnloader / PayloadDeconstructor / PayloadConstructor / PayloadSource / PayloadVoid`：conveyor/router 写 `progress/itemRotation/Payload.write(item)`，router/source/mass-driver/loader/unloader 按 Java `version()==1` 写出对应尾字段；`PayloadUnloader` 在 Java 无独立 `write/read/version`，沿用 `PayloadLoaderBuild` 的 common + exporting tail；constructor/deconstructor/source/void 先写 `PayloadBlockBuild` common 再写 Java 子类字段；非空 payload body 当前保留 raw/最小 exact codec，完整 Unit 实体恢复仍需后续接入；
  - logic 导出已覆盖 `MessageBlock / SwitchBlock / LogicDisplay / MemoryBlock / CanvasBlock / LogicBlock(Processor)`：message/memory/canvas 按 revision 0 写 Java 字段，switch/display 按 Java `version()==1` 写 enabled/transform，processor 按 Java `version()==4` 写 compressed code、变量、no-memory sentinel、tag/iconTag、wait timers 与 accumulator；
  - campaign 导出已覆盖 `LaunchPad / AdvancedLaunchPad / LandingPad / Accelerator`：三类 Java build 均按 `version()==1` 写出 launch counter、landing config/priority/cooldown/arriving/arrival timer/liquid removed 与 accelerator progress，`Accelerator.launching` 作为 runtime-only 状态不写出；
  - unit 导出已覆盖 `UnitFactory / Reconstructor / RepairTurret / UnitAssembler / UnitAssemblerModule`：factory/reconstructor 先写 `PayloadBlockBuild` common payload 再按 Java `version()==3` 写进度、计划/命令；repair tower 按 revision 1 写 rotation；assembler 先写 common payload 再按 revision 1 写 progress/readUnitIds/PayloadSeq blocks/commandPos；assembler module 当前写 Java `PayloadBlockBuild` common payload；
  - liquid 导出已覆盖 Java `LiquidBridge`：`bridge-conduit/phase-conduit` 按 Java `ItemBridgeBuild.version()==1` 写 `link/warmup/incoming/(wasMoved||moved)`；`DirectionLiquidBridge` 在 Java 无自定义 `write/read/version`，导出侧不写额外 tail；
  - storage 导出已覆盖 `CoreBlock`：按 Java `version()==1` 写 `commandPos`，普通 container/vault 没有 Java block-specific tail，导出侧不制造虚假状态；
  - sandbox 导出已覆盖 `ItemSource` 与 `LiquidSource`：`ItemSource` 按 revision 0 写 output item short config，`LiquidSource` 按 Java `version()==1` 写 source liquid short config，`counter/stored_liquid/amount` 等运行时派生字段不写出；
  - legacy 导出已覆盖 `LegacyCommandCenter / LegacyMechPad / LegacyUnitFactory`：command center 写单字节兼容占位，mech pad 写旧版 3 个 float pad 数据，legacy unit factory 按 revision 0 写 buildTime 与 spawnCount，避免旧图/旧块读取偏移；
  - 已补 `game_runtime_exports_network_map_snapshot_with_owned_building_chunks`、`game_runtime_exports_defense_wall_state_tail_in_network_map_snapshot`、`game_runtime_exports_power_and_light_state_tail_in_network_map_snapshot`、`game_runtime_exports_effect_block_state_tail_in_network_map_snapshot`、`game_runtime_exports_turret_state_tail_in_network_map_snapshot`、`game_runtime_exports_production_state_tail_in_network_map_snapshot`、`game_runtime_exports_crafting_state_tail_in_network_map_snapshot`、`game_runtime_exports_distribution_state_tail_in_network_map_snapshot`、`game_runtime_exports_payload_state_tail_in_network_map_snapshot`、`game_runtime_exports_logic_state_tail_in_network_map_snapshot`、`game_runtime_exports_campaign_state_tail_in_network_map_snapshot`、`game_runtime_exports_unit_state_tail_in_network_map_snapshot`、`game_runtime_exports_liquid_bridge_state_tail_in_network_map_snapshot`、`game_runtime_exports_core_storage_state_tail_in_network_map_snapshot`、`game_runtime_exports_sandbox_state_tail_in_network_map_snapshot`、`game_runtime_exports_legacy_state_tail_in_network_map_snapshot` 与 `server_world_data_exports_owned_building_chunks_for_runtime_loader`，验证 core 导出和服务端 world stream 解码后的 `map_snapshot` 均可被 `GameRuntime::load_network_map_with_buildings(...)` 反向加载出 owned building / defense wall / power-light / effect / turret / production / crafting / distribution / payload / logic / campaign / unit / liquid / storage / sandbox / legacy sidecar。

仍需：

- 完整 Java 兼容 `NetworkIO.writeWorld/loadWorld`；
- markers/custom chunks 与完整 save dispatcher 的运行态接入；
- 将 `RawSaveEnvelope` region 层与 runtime world/entities 物化流程接成完整 save read/write dispatcher；
- `teamBlocks` 导出补 typed config 保真与 content header 临时映射写出；
- building chunk 继续接入剩余 block-specific tail writers（炮塔/物流/payload/logic 等），避免只导出 base payload；
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
- `BuildTurretStatsPlan`
- `BuildTurretIconRegion`
- `BuildTurretDrawCommand`
- `BuildTurretDrawPlan`
- `build_turret_stats_plan(...)`
- `build_turret_icons(...)`
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
- `BuildTurretBuild.write/read()` 已新增基于 `ContentLoader` 与 `TypeIO.writePlans/readPlans` 的 typed plans 读写路径；旧 raw 路径保留作兼容兜底，`build_turret_read_child_with_loader(...)` 现在会在 typed plan 解码失败或有尾字节时保留 raw plan bytes，避免旧图/内容映射异常导致整个 building state parse error。
- `BuildTurretState.plans` 与 `UnitComp.builder.plans` 已有双向桥接，typed read/write 后可恢复到单位建造队列。
- `BuildTurretBuild.sense/senseObject()` 对 `buildX/buildY/building/breaking` 的 unit build plan 转发语义已有轻量 helper。
- `BuildTurret.setStats()` 已按 Java `stats.addPercent(Stat.buildSpeed, buildSpeed)` 收口到 `BuildTurretStatsPlan`。
- `BuildTurret.icons()` 已按 Java `{baseRegion, region}` 顺序收口到 `[Base, Main]`。

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
- `DefenseWallKind::Thruster`
- `blocks.rs` 内容注册 `thruster`
- 已对照 `Thruster.java` 锁定：
  - 构造器 `rotate = true`；
  - 构造器 `quickRotate = false`；
  - plan/build 绘制顺序为 base region 后 topRegion；
  - top rotation = `rotation * 90` / `rotdeg()`。
- 已对照 `Blocks.java` 锁定内容注册：
  - 位置跟随 `scrap-wall-gigantic` 后、`beryllium-wall` 前；
  - `requirements(Category.defense, BuildVisibility.sandboxOnly, with(Items.scrap, 96))`；
  - `health = 55 * 16 * wallHealthMultiplier`；
  - `size = 4`；
  - 内容层显式 `rotate = true`。

仍需：

- 接入真实 block rendering adapter；
- 图标/贴图层接入真实 atlas：`icons()` 应返回 `[region, topRegion]`。

### 7.5 Door / AutoDoor

### 7.5a Wall

已推进：

- `WallState`
- `WallStatsPlan`
- `WallIconRegion`
- `WallDestroySound`
- `WallDrawPlan`
- `WallReflectAxis`
- `WallCollisionPlan`
- `wall_stats_plan(...)`
- `wall_init_destroy_sound(...)`
- `wall_icon_region(...)`
- `wall_collision_hit(...)`
- `wall_draw_hit_decay(...)`
- `wall_should_lightning(...)`
- `wall_deflects_bullet(...)`
- `wall_reflect_x(...)`
- `wall_draw_plan(...)`
- `wall_collision_plan(...)`
- 已对照 `Wall.setStats()/init()/icons()/draw()/collision()` 锁定：
  - `chanceDeflect > 0` 时展示 base deflect chance；
  - `lightningChance > 0` 时展示 lightningChance 百分比与 lightningDamage；
  - size=2 且 destroySound unset 时改为 `blockExplodeWall`；
  - icon 优先 atlas 中 `name`，否则 `name + "1"`；
  - collision 总是先将 hit 置 1；
  - flashHit 时绘制 additive 白色方形，alpha=`hit*0.5`，未暂停时按 `delta/10` 衰减；
  - lightning 触发条件为 `lightningChance > 0 && chance(lightningChance)`，角度为 `bullet.rotation()+180`；
  - deflect 需要 `chanceDeflect > 0`、速度大于 `0.1`、reflectable、且随机命中 `chanceDeflect / bullet.damage()`；
  - deflect 命中后按 `penX > penY` 反转 x/y 速度，owner/team 转为 wall，bullet time 加 1，并返回 false 禁用本次 collision。

仍需：

- 接入真实 `Lightning.create(...)`、sound pitch 随机、bullet translate/owner/team/time 写回；
- flash overlay 接入 renderer；
- atlas icon 查询桥接真实 content/atlas。

已推进：

- `DoorState`
- `DoorEffectKind`
- `DoorControlPlan`
- `DoorTappedPlan`
- `AutoDoorTriggerRect`
- `door_check_solid(...)`
- `door_sense_enabled(...)`
- `door_can_toggle(...)`
- `door_tapped_should_configure(...)`
- `door_effect_for_current_open(...)`
- `door_origin_id(...)`
- `door_control_enabled_plan(...)`
- `door_tapped_plan(...)`
- `write_door_state(...)`
- `read_door_state(...)`
- `auto_door_should_open(...)`
- `auto_door_ground_check(...)`
- `auto_door_trigger_size(...)`
- `auto_door_trigger_rect(...)`
- `auto_door_remote_toggle_valid(...)`
- `AutoDoorUpdatePlan`
- `AutoDoorSetOpenPlan`
- `DoorChainNode`
- `DoorChainToggle`
- `DoorChainTogglePlan`
- `DoorChainGraphNode`
- `door_chain_build_plan(...)`
- `door_chain_toggle_plan(...)`
- `DoorRegion`
- `DoorDrawCommand`
- `DoorDrawPlan`
- `door_region_for_open(...)`
- `door_plan_region(...)`
- `door_draw_plan(...)`
- `auto_door_update_plan(...)`
- `auto_door_set_open_plan(...)`
- 已对照 `AutoDoor.updateTile()` 锁定：
  - timer 未到或 net client 时不扫描/不发送 toggle；
  - 触发范围尺寸为 `size * tilesize + triggerMargin * 2`，中心为 building `x/y`；
  - 触发范围内存在 `isGrounded && !allowLegStep` 单位时应打开；
  - open 状态变化时才发送 toggle；
  - remote toggle 仅 tile 存在且 build 是 AutoDoorBuild 时有效；
  - `setOpen` 总是更新 pathfinder，只有 `wasVisible` 时播放 effect/sound。
- 已对照 `Door.control()/tapped()/origin()/effect()` 锁定：
  - `control enabled` 在 net client、目标状态不变、关门且 tile 内有单位、origin timer 未到时均不 configure；
  - `p1` 非零视为 shouldOpen；
  - tapped 在 `open && unitsInTile` 或 origin timer 未到时不 configure，否则切换 `!open`；
  - chained 为空时 origin 为 self，否则为 chained first；
  - 当前 open=false 时 effect 为 openfx，open=true 时 effect 为 closefx。
- 已对照 `Door` 构造器配置闭包锁定：
  - 非生成中先播放 origin doorSound/effect；
  - chained 为空时只处理 base；
  - 关闭时跳过 tile 内有单位的门；
  - 跳过已经是目标 open 状态的门；
  - `chainEffect` 控制链上门是否额外播放 effect；
  - 非生成中每个切换门更新 pathfinder。
- 已对照 `Door.updateChained()` 的纯遍历部分锁定：
  - 从 self 入队，按 `removeLast` / `addFirst` 形成与 Java 队列一致的 chain 顺序；
  - 只串联 door 图中存在的邻接节点；
  - 断开链与非门邻接不会串入当前 chain。
- 已对照 `Door.getPlanRegion()`、`Door.draw()`、`AutoDoor.draw()` 锁定：
  - `config == Boolean.TRUE` 或 `open == true` 时选择 openRegion；
  - 否则选择默认 region。
- 已将 `AutoDoor.autoDoorToggle(Tile, boolean)` 的联机同步接入 Rust 客户端轻量 runtime：
  - `NetClient::apply_auto_door_toggle_packet(...)` 使用 `Tiles` 中的真实 center/proxy `BuildingRef`，只在回调确认 block 为 AutoDoor 时把 `open` 写入中心建筑的 `ClientTileStorageMirror.door_open`；
  - `NetClient::apply_auto_door_toggle_mirror_packet(...)` 已接入 `PacketKind::AutoDoorToggleCallPacket` 收包分支，用现有 `auto_door_set_open_plan(...)` 归一化 Java `setOpen` 的 open 状态；
  - 这把 content 注册的 `DefenseWallKind::AutoDoor`、world tile/build position、network `AutoDoorToggleCallPacket` 与客户端 mirror 连接起来，不再只是单测 helper。

仍需：

- `Door.updateChained()` 的真实 proximity flood-fill adapter；
- AutoDoor update 侧接入真实 Units/tree 扫描与服务端 `Call.autoDoorToggle(...)` 发送；
- AutoDoor 客户端 mirror 继续下沉到真实 pathfinder tile 更新、effect/sound/renderer 调度。

### 7.6 RegenProjector

已推进：

- `regen_projector_heal_amount(...)`
- `RegenProjectorStatsPlan`
- `RegenProjectorRangePlan`
- `RegenProjectorDrawPlan`
- `RegenProjectorApplyPlan`
- `RegenProjectorState`
- `RegenProjectorUpdatePlan`
- `RegenProjectorMendMap`
- `regen_projector_stats_plan(...)`
- `regen_projector_place_plan(...)`
- `regen_projector_select_plan(...)`
- `regen_projector_draw_plan(...)`
- `regen_projector_light_plan(...)`
- `regen_projector_apply_plan(...)`
- `regen_projector_update(...)`
- `regen_projector_heal_amount_from_percent(...)`
- `regen_projector_record_building_mend(...)`
- `regen_projector_target_in_square(...)`
- `regen_projector_record_mend_runtime(...)`
- `regen_projector_apply_mend_map_to_buildings(...)`
- `regen_projector_apply_mend_plan_to_buildings(...)`
- 已对照 `RegenProjector.updateTile()` 锁定：
  - `warmup` 按上一帧 `didRegen` 通过 `approachDelta(..., 1/70)` 更新；
  - `totalTime += warmup * delta`；
  - 每 tick 先清 `didRegen/anyTargets`；
  - suppression 时提前返回且不计算 targets/heal；
  - `anyTargets` 来自 damaged targets；
  - `optionalTimer += edelta * optionalEfficiency`，超过 `optionalUseTime` 后 consume 并归零；
  - heal percent = `lerp(1, optionalMultiplier, optionalEfficiency) * healPercent`；
  - 有可修复目标时标记 `didRegen`。
- 已对照 `mendMap` 行为锁定：
  - 同一建筑同一帧只保留最大修复量，防止叠加；
  - 修复量受 missing health 上限约束；
  - drain 后清空，供统一应用 `heal/recentlyHealed`。
- 已将 `mendMap` 接到真实 `BuildingComp` 治疗运行态：
  - `regen_projector_record_building_mend(...)` 对齐 Java `!build.damaged() || build.isHealSuppressed()` 过滤，按 `tile_pos` 记录待治疗量；
  - `regen_projector_record_mend_runtime(...)` 已按 Java `indexer.eachBlock(team, centered square...)` 对同队、方形范围内目标进行扫描，并复用 `regen_projector_record_building_mend(...)` 写入 `RegenProjectorMendMap`；
  - `regen_projector_apply_mend_plan_to_buildings(...)` 对齐 `lastUpdateFrame != state.updateId` 门控，只有跨 update frame 时才 drain 并应用；
  - `regen_projector_apply_mend_map_to_buildings(...)` 按 `tile_pos` 查找候选 `BuildingComp`，调用真实 `BuildingComp::heal(amount, now)` 并清空 map；
  - healed 目标可通过 `BuildingComp::recently_healed(now)` 验证，当前 Rust `heal(...)` 已负责更新 `last_heal_time`。

- 已对照 `RegenProjector.drawPlace()/drawSelect()/drawLight()/setStats()` 锁定：
  - place/select 均使用 `range * tilesize` 的 dash square；
  - place 坐标来自 `tile * tilesize + offset`，select 坐标来自建筑当前 `x/y`；
  - selected alpha 由 `absin(4f, 1f)` 派生；
  - `DrawDefault` 下 `drawer.drawLight(this)` 无额外光效，当前 light plan 显式记录为无额外绘制；
  - repair time stat 按 Java `(int)(1f / (healPercent / 100f) / 60f)` 截断；
  - booster 使用 `optionalMultiplier`，range boost 为 `0f`。

仍需：

- 将 `regen_projector_record_mend_runtime(...)` / `regen_projector_apply_mend_plan_to_buildings(...)` 挂入真实 world/building dispatcher 的 `world.build(pos)` 查询；
- drawPlace/drawSelect 的目标列表高亮接入真实 indexer / targets。

### 7.7 BaseShield

已推进：

- `BaseShieldState`
- `base_shield_update(...)`
- `base_shield_should_interact(...)`
- `base_shield_unit_overlap(...)`
- `base_shield_unit_action(...)`
- `base_shield_unit_spark_chance_delta(...)`
- `base_shield_should_emit_unit_spark(...)`
- `base_shield_apply_unit_action(...)`
- `write_base_shield_state(...)`
- `read_base_shield_state(...)`
- `BaseShieldDrawCommand`
- `BaseShieldDrawPlan`
- `BaseShieldRangePlan`
- `BaseShieldInteractionPlan`
- `BaseShieldTintPlan`
- `BaseShieldRuntimeReport`
- `base_shield_clip_radius(...)`
- `base_shield_radius(...)`
- `base_shield_in_fog_to(...)`
- `base_shield_should_absorb_bullet(...)`
- `base_shield_apply_absorb_to_bullet(...)`
- `base_shield_within_radius(...)`
- `base_shield_apply_runtime(...)`
- `effect_base_shield_apply_runtime(...)`
- `base_shield_tint_plan(...)`
- `base_shield_place_plan(...)`
- `base_shield_select_plan(...)`
- `base_shield_interaction_plan(...)`
- `base_shield_draw_plan(...)`
- 已对照 `BaseShield.updateTile()` 锁定：
  - `smoothRadius = lerpDelta(smoothRadius, radius * efficiency, 0.05)`；
  - radius > 1 时才需要 bullet/unit 交互；
  - bullet 扫描矩形为 `(x - rad, y - rad, rad * 2, rad * 2)`；
  - unit 扫描半径为 `rad + 10f`；
  - 敌方、可吸收且在半径内的 bullet 被吸收；
  - unit overlap 大于 0 时被 repel，大于 `hitSize * 1.5` 时 kill；
  - repel 分支按 `Mathf.chanceDelta(0.12f * Time.delta)` 计划 spark 副作用。
- 已对照 `init()/drawPlace()/drawSelect()/radius()/inFogTo()` 锁定：
  - `init()` 使用 `updateClipRadius(radius)`；
  - place 圆心为 `tile * tilesize + offset`，半径为 block radius；
  - select 圆心为 building `x/y`，半径为 block radius；
  - building `radius()` 返回 `smoothRadius`；
  - shield building 对任意 viewer 均 `inFogTo=false`。
- 已对照 `drawShield()` 锁定：
  - broken 时不绘制盾体但仍 reset；
  - animateShields 时 fill poly；
  - 非 animateShields 时 stroke 1.5、alpha `0.09 + clamp(0.08 * hit)`、fill+lines poly；
  - `shieldColor == null` 时使用 team color，否则使用 shieldColor；
  - hit 参与与 white 的 color clamp/blend。
- `base_shield_apply_absorb_to_bullet(...)` 已接入 `BulletComp::absorb()`，让 BaseShield 的吸收判定能真实写入 bullet `absorbed/removed` 状态并清空碰撞记录。
- `base_shield_apply_unit_action(...)` 已接入 `UnitComp`，对齐 Java `unit.kill()` 与 repel 分支的 `unit.vel.setZero(); unit.move(...)`：Kill 写入 `HealthComp::kill()`，Repel 清零速度并沿盾中心到单位方向移动 `overlap + 0.01`。
- `base_shield_apply_runtime(...)` 已把 `BaseShieldBuild.updateTile()` 的核心扫描执行收束成可复用 runtime adapter：
  - 先调用 `base_shield_update(...)` 写入 `smoothRadius`；
  - `radius <= 1` 时不扫描，保持 Java `rad > 1` 门控；
  - 对 bullet 侧跳过已移除/已吸收对象，经 `BulletType.absorbable`、敌队和半径内判定后调用真实 `BulletComp::absorb()`；
  - 对 unit 侧按 `rad + 10` 做敌方单位候选过滤，再用 `base_shield_unit_action(...)` 执行真实 `UnitComp` repel/kill，并保留 spark 事件计数给后续 FX dispatcher；
  - 这一步已把 BaseShield 的 bullet/unit 交互从单测直接调用 helper 提升为可挂接真实 `Groups.bullet.intersect(...)` / `Units.nearbyEnemies(...)` 或 world indexer 的统一入口。
- 已新增 content-backed BaseShield runtime dispatcher：
  - `effect_base_shield_apply_runtime(...)` 直接接收 `EffectBlockData`，仅在 `EffectBlockKind::BaseShield` 时执行；
  - 分发时使用 content 中的 `radius`，覆盖 `shield-projector=200` 与 `large-shield-projector=400` 等 Java `Blocks.java` 参数；
  - 调用方只需提供 shield building、bullet/unit 候选、`BulletType` resolver、delta 与 spark 随机源，后续可直接接入真实 building update loop。

仍需：

- 将 `effect_base_shield_apply_runtime(...)` 挂到真实 building update dispatcher 与 Groups/world indexer；
- shieldColor/teamColor 到真实渲染颜色的 adapter；
- drawPlace/drawSelect dash circle helper 接入真实 renderer。

### 7.7a ShockMine

已推进：

- `ShockMineStatsPlan`
- `ShockMineDrawPlan`
- `ShockMineTriggerPlan`
- `ShockMineLightningCreateEvent`
- `ShockMineBulletCreateEvent`
- `ShockMineSideEffectPlan`
- `shock_mine_should_trigger(...)`
- `shock_mine_stats_plan(...)`
- `shock_mine_stats_text(...)`
- `shock_mine_draw_plan(...)`
- `shock_mine_lightning_angles(...)`
- `shock_mine_bullet_angles(...)`
- `shock_mine_trigger_plan(...)`
- `shock_mine_apply_trigger_to_building(...)`
- `shock_mine_side_effect_plan(...)`
- 已对照 `ShockMine.unitOn()/triggered()/draw()/setStats()` 锁定：
  - 触发条件为 `enabled && unit.team != team && timer(timerDamage, cooldown)`；
  - 触发后自身承受 `tileDamage`；
  - lightning 创建次数为 `tendrils`，每条角度来自 `Mathf.random(360f)`，当前由上层注入随机角度数组以保持纯函数可测；
  - bullet 非空时创建 `shots` 枚，角度为 `(360f / shots) * i + Mathf.random(inaccuracy)`，当前由上层注入 inaccuracy offsets；
  - team top 绘制使用 `teamAlpha`；
  - stats damage 文案保留 tendrils 与 damage 的 2 位格式需求。
  - stats 文案按上游 `Core.bundle.format("bullet.lightning", tendrils, Strings.autoFixed(damage, 2)).replace("[stat]", "[white]")` 的英文 bundle 形态收口。
- 已将 `unitOn()` 触发后的 self-damage 接到真实 `BuildingComp`：
  - `shock_mine_apply_trigger_to_building(...)` 在 `ShockMineTriggerPlan.triggered` 为真时调用 `BuildingComp::damage(tileDamage, now)`；
  - `ShockMineTriggerPlan` 继续保留 lightning/bullet 角度、伤害和长度，作为下游副作用事件生成的稳定输入。
- 已将 `triggered()` 的副作用创建提升为统一事件计划：
  - `shock_mine_side_effect_plan(...)` 按 `ShockMineTriggerPlan.lightning_angles` 生成 `ShockMineLightningCreateEvent`，包含 team、`damageLightningGround` 等 lightning bullet type id、颜色、`LightningConfig(seed/x/y/rotation/length/damage)`；
  - 使用 `LightningSeedState` 为每条 lightning 分配可复现 seed，方便后续接入 `create_lightning_plan(...)` 或真实 lightning dispatcher；
  - 当 Java 侧 `bullet != null` 对应的 Rust bullet type/id 存在时，按 `ShockMineTriggerPlan.bullet_angles` 调用 `BulletType::create_plan(...)` 生成 `ShockMineBulletCreateEvent`，保留 source tile、team、x/y、angle/damage/speed/lifetime；
  - 未触发时不推进 seed、不生成事件，保持 Java `unitOn()` 条件门控语义。

仍需：

- 将 `ShockMineSideEffectPlan` 交给真实 `Lightning.create(...)` / `BulletType.create(...)` / Groups entity dispatcher 执行；
- 接入 effect/sound 与触发可视化调度；
- 将 draw plan 连接到 renderer 的 base/teamRegion 绘制；
- setStats 文案与 bundle/localization 的最终桥接。

### 7.7b Radar

已推进：

- `RadarState`
- `RadarIconRegion`
- `RadarRangePlan`
- `RadarDrawCommand`
- `RadarDrawPlan`
- `radar_icons(...)`
- `radar_fog_radius(...)`
- `radar_force_update_needed(...)`
- `radar_progress(...)`
- `radar_can_pickup(...)`
- `radar_place_plan(...)`
- `radar_select_plan(...)`
- `radar_draw_rotation(...)`
- `radar_glow_alpha(...)`
- `radar_draw_plan(...)`
- `radar_update(...)`
- `radar_fog_event(...)`
- `radar_apply_fog_force_update(...)`
- `radar_update_with_fog_control(...)`
- `EffectRadarRuntimeInput`
- `effect_radar_update_runtime(...)`
- `write_radar_state(...)`
- `read_radar_state(...)`
- 已对照 `Radar.updateTile()/drawPlace()/drawSelect()/draw()/icons()/canPickup()` 锁定：
  - `fogRadius() = fogRadius * progress * smoothEfficiency`；
  - `smoothEfficiency = lerpDelta(..., efficiency, 0.05)`；
  - forceUpdate 阈值为 `abs(radius - lastRadius) >= 0.5`，且使用 progress 更新前的半径；
  - `progress += edelta / discoveryTime` 后 clamp 到 `[0,1]`；
  - `totalProgress += efficiency * edelta` 不 clamp；
  - place 半径为 `fogRadius * tilesize`，select 半径为运行态 `fogRadius() * tilesize`；
  - draw 旋转角为 `rotateSpeed * totalProgress`；
  - glow alpha 为 `glowColor.a * (1 - glowMag + absin(glowScl, glowMag))`；
  - icons 顺序为 baseRegion、region；
  - `canPickup()` 恒为 false；
  - Java write/read 只持久化 `progress`。
- 已将 `RadarBuild.updateTile()` 的 fog force-update 分支接入 Rust fog runtime：
  - `radar_update_with_fog_control(...)` 先复用 `radar_update(...)` 完成 Java 顺序的 `smoothEfficiency/lastRadius/progress/totalProgress` 更新；
  - 仅当半径变化达到 `>= 0.5` 时，使用 `state.last_radius` 构造 `FogEvent::get(tileX, tileY, round(fogRadius()), team)`；
  - `radar_apply_fog_force_update(...)` 调用真实 `FogControl::force_update(...)`，遵守 `rules.fog` 与 team fog data 是否已分配；
  - `static_fog=false` 时仍可标记 dynamic 更新，但不会写入 static fog event，保持与 Java `FogControl.forceUpdate(...)` 分支一致。
- 已新增 content-backed Radar runtime dispatcher：
  - `effect_radar_update_runtime(...)` 直接接收 `EffectBlockData`，仅在 `EffectBlockKind::Radar` 时执行；
  - 分发时使用 content 中的 `fog_radius` 与 `discovery_time`，避免调用方继续手写 Java 默认参数；
  - 输入显式携带 team/tile/efficiency/edelta/fog rules，后续可由真实 building update loop 在拆分 `FogControl` 借用后直接调用；
  - 非 Radar effect block 返回 `None`，防止错误 state/block 组合静默推进。

仍需：

- 将 `effect_radar_update_runtime(...)` 挂入真实 building update dispatcher；
- 将 dash circle / baseRegion / rotating region / additive glow 连接到 renderer；
- content atlas 中 base/glow region 的加载与 outline icon 细节。

### 7.7c ShockwaveTower

已推进：

- `ShockwaveTowerState`
- `ShockwaveTowerFire`
- `ShockwaveTowerStatsPlan`
- `ShockwaveTowerRangePlan`
- `ShockwaveTowerDrawCommand`
- `ShockwaveTowerDrawPlan`
- `shockwave_tower_stats_plan(...)`
- `shockwave_tower_place_plan(...)`
- `shockwave_tower_select_plan(...)`
- `shockwave_tower_wave_damage(...)`
- `shockwave_tower_can_target(...)`
- `shockwave_tower_can_fire(...)`
- `shockwave_tower_apply_damage(...)`
- `shockwave_tower_heat_after_cooldown(...)`
- `shockwave_tower_sense(...)`
- `shockwave_tower_warmup(...)`
- `shockwave_tower_draw_plan(...)`
- `shockwave_tower_update(...)`
- `shockwave_tower_bullet_in_scan_square(...)`
- `shockwave_tower_should_scan_targets(...)`
- `shockwave_tower_apply_runtime(...)`
- `effect_shockwave_tower_apply_runtime(...)`
- `shockwave_tower_progress(...)`
- `shockwave_tower_should_consume(...)`
- 已对照 `ShockwaveTower.updateTile()/setStats()/drawPlace()/drawSelect()/draw()/sense()/warmup()/shouldConsume()` 锁定：
  - 仅 `potentialEfficiency > 0` 时累积 `reloadCounter += edelta()`；
  - 开火条件为 `potentialEfficiency > 0 && reloadCounter >= reload && timerReady && targets.size > 0`；
  - 目标过滤为异队且 `bullet.type.hittable`；
  - wave damage = `min(bulletDamage, bulletDamage * falloffCount / targetCount)`；
  - target damage 严格大于 waveDamage 时扣血，否则 remove；
  - 开火后 `heat = 1`、`reloadCounter = 0`，同 tick 末尾按 `delta / reload * cooldownMultiplier` 衰减并 clamp；
  - stats damage/range/reload per second 与 Java 公式一致；
  - draw heat additive alpha = `heat`，shape color lerp = `heat^2`，shape radius = `shapeRadius * potentialEfficiency`，rotation = `Time.time * shapeRotateSpeed`；
  - `sense(progress) = reloadCounter / reload`，`warmup() = heat`，`shouldConsume() = reloadCounter < reload`。
- 已将 ShockwaveTower 开火分支接入真实 `BulletComp` 候选：
  - `shockwave_tower_apply_runtime(...)` 在 `reloadCounter + edelta >= reload && timerReady` 时按 Java `Groups.bullet.intersect(x-range, y-range, range*2, range*2)` 的方形扫描语义筛选候选；
  - 仅处理异队、未 removed 且 `BulletType.hittable` 的 bullet；
  - 开火后将 wave damage 写回 `BulletComp.damage`，不足以剩余的目标写 `removed=true`；
  - `effect_shockwave_tower_apply_runtime(...)` 使用 content 中的 `reload/range/bullet_damage/falloff_count/cooldown_multiplier`，避免调用方手写上游参数。

仍需：

- 接入真实 Groups bullet 存储遍历、hit/wave effect、sound、shake 与 Trigger 事件；
- 将 heatRegion additive 和 effect-layer polygon 接入 renderer；
- timerCheck/checkInterval 连接到真实 building timer。

### 7.8 ShieldWall / ForceProjector / MendProjector / OverdriveProjector

已推进：

- `ShieldWallState`
- `shield_wall_broken(...)`
- `shield_wall_update(...)`
- `shield_wall_damage(...)`
- `shield_wall_pickup(...)`
- `ShieldWallDrawCommand`
- `ShieldWallDrawPlan`
- `shield_wall_draw_plan(...)`
- `write_shield_wall_state(...)`
- `read_shield_wall_state(...)`
- `DirectionalForceProjectorState`
- `DirectionalForceProjectorStatsPlan`
- `DirectionalForceProjectorPlacePlan`
- `DirectionalForceProjectorDeflectPlan`
- `DirectionalForceProjectorDrawCommand`
- `DirectionalForceProjectorDrawPlan`
- `directional_force_projector_clip_radius(...)`
- `directional_force_projector_effective_length(...)`
- `directional_force_projector_stats_plan(...)`
- `directional_force_projector_bar_fraction(...)`
- `directional_force_projector_outputs_items(...)`
- `directional_force_projector_should_ambient_sound(...)`
- `directional_force_projector_place_plan(...)`
- `directional_force_projector_update(...)`
- `directional_force_projector_picked_up(...)`
- `directional_force_projector_segment(...)`
- `directional_force_projector_deflect_plan(...)`
- `directional_force_projector_draw_plan(...)`
- `directional_force_projector_absorb_bullet(...)`
- `directional_force_projector_absorb_bullet_comp(...)`
- `DirectionalForceProjectorAbsorbEvent`
- `DirectionalForceProjectorBreakEvent`
- `directional_force_projector_absorb_event(...)`
- `directional_force_projector_break_event(...)`
- 已对照 `ShieldWall.draw()` 锁定：
  - 总是先绘制 base region；
  - `shieldRadius <= 0` 时只输出 region；
  - radius = `shieldRadius * tilesize * size / 2`；
  - animateShields 时走 fill square；
  - 非 animateShields 时 stroke 1.5、alpha `0.09 + clamp(0.08 * hit)`、fill square + outline square；
  - additive glow alpha = `(1 - glowMag + absin(glowScl, glowMag)) * shieldRadius`。
- 已对照 `DirectionalForceProjector.init()/setBars()/setStats()/drawPlace()/updateTile()/deflectBullets()/draw()/drawShield()` 锁定：
  - clip radius = `width + 3f`；
  - `length < 0` 时转为 `size * tilesize / 2f`；
  - shield bar fraction broken 时为 0，否则 `1 - buildup / shieldHealth`；
  - stats cooldownTime = `(int)(shieldHealth / cooldownBrokenBase / 60f)`；
  - place 使用 tile world 坐标，不加 block offset，并按 `rotation * 90` 旋转两条端点线；
  - `shouldAmbientSound = !broken && shieldRadius > 1f`，`outputsItems=false`；
  - `shieldRadius = lerpDelta(shieldRadius, broken ? 0 : warmup * width, 0.05)`；
  - deflect segment 为 `(length, ±shieldRadius)` 旋转平移，扫描 bounds 在 segment bbox 基础上 grow `padSize`；
  - animated shield 走 rect、edge lines 与 caps，静态 shield 走 fill/stroke rect；
  - top additive alpha = `buildup / shieldHealth * 0.75`。
  - `directional_force_projector_absorb_bullet_comp(...)` 已把几何相交判定接到真实 `BulletComp::absorb()`，并使用 `BulletType::shield_damage(bullet.damage)` 写入 `buildup`。
  - 吸收命中时现在会把 bullet 坐标移动到 shield segment 的真实交点，再执行 `BulletComp::absorb()`，对齐 Java `b.set(intersectOut); b.absorb(); paramEffect.at(b)` 的顺序。
  - `directional_force_projector_absorb_event(...)` 与 `directional_force_projector_break_event(...)` 已将吸收 FX 与破盾 FX 收束为运行态事件计划，等待后续 renderer/effect dispatcher 执行。
- `ForceProjectorState`
- `ForceProjectorUpdate`
- `force_projector_real_radius(...)`
- `force_projector_shield(...)`
- `force_projector_update_with_timer(...)`
- `force_projector_update(...)`
- `force_projector_sense(...)`
- `force_projector_set_shield(...)`
- `force_projector_outputs_items(...)`
- `force_projector_should_ambient_sound(...)`
- `force_projector_in_fog_to(...)`
- `force_projector_picked_up(...)`
- `force_projector_overwrote(...)`
- `force_projector_bar_fraction(...)`
- `force_projector_absorb_bullet(...)`
- `force_projector_apply_absorb_to_bullet(...)`
- `force_projector_absorb_bullet_comp(...)`
- `force_projector_absorb_explosion(...)`
- `ForceProjectorAbsorbEvent`
- `ForceProjectorBreakEvent`
- `force_projector_absorb_event(...)`
- `force_projector_break_event(...)`
- `ForceProjectorBulletAbsorb`
- `ForceProjectorRemovedPlan`
- `ForceProjectorDeflectPlan`
- `ForceProjectorDrawCommand`
- `ForceProjectorDrawPlan`
- `force_projector_on_removed_plan(...)`
- `force_projector_deflect_plan(...)`
- `force_projector_draw_plan(...)`
- `effect_projector_consume_source_items(...)`
- `effect_block_consume_source_items_from_report(...)`
- `effect_force_projector_consume_phase_items(...)`
- `effect_projector_update_building_frame_with_timer_and_consume(...)`
- `effect_force_projector_update_building_frame_with_timer_and_consume(...)`
- `EffectBlockTimerStateStore`
- `effect_projector_update_building_frame_with_timer_store_and_consume(...)`
- `effect_force_projector_update_building_frame_with_timer_store_and_consume(...)`
- `effect_shockwave_tower_update_building_frame_with_timer_store(...)`
- `effect_build_turret_timer_target_ready_from_store(...)`
- `effect_build_turret_timer_target2_ready_from_store(...)`
- `effect_build_turret_update_building_frame(...)`
- `EffectBlockFrameResources`
- `effect_block_update_building_frame_with_stores(...)`
- `EffectBlockFrameBatchResources`
- `EffectBlockFrameBatchReport`
- `effect_block_frame_input_from_game_update(...)`
- `effect_block_update_building_slice_with_stores(...)`
- `GameRuntime`
- `GameRuntime::advance_and_dispatch_effect_blocks(...)`
- `GameRuntime::advance_owned_effect_blocks(...)`
- `BuildingComp::advance_update_timing(...)`
- `BuildingComp::should_update_tile(...)`
- `BlockDef::no_update_disabled(...)`
- `BlockDef::supports_env(...)`
- `GameRuntime::refresh_owned_building_update_permissions(...)`
- `mindustry_server::ServerLauncher.runtime`
- `mindustry_server::ServerLauncher::update_runtime_effect_blocks(...)`
- `desktop::DesktopLauncher.runtime`
- `write_force_projector_state(...)`
- `read_force_projector_state(...)`
- 已对照 `ForceProjector.updateTile()` 推进：
  - `broken / buildup / radscl / warmup / phaseHeat / hit` 的主状态机；
  - `timer(timerUse, phaseUseTime / timeScale)` 已通过 `force_projector_update_with_timer(...)` 与 `effect_force_projector_update_building_frame_with_timer(...)` 接入 `BuildingTimerState` 侧车，返回 `should_consume_phase` 供上层触发 `consume()`；
  - `effect_force_projector_consume_phase_items(...)` 已把 `should_consume_phase` 接到真实 `BuildingComp.items`，按 `EffectBlockData.boost_items` 和 Java `ConsumeItems.trigger(...)` 的 rounded amount 语义扣相位材料；
  - 破盾阈值与冷却；
  - 爆炸吸收；
  - Java write/read 的 5 个持久化字段。
- 已对照 `ForceProjector.deflectBullets()` 锁定：
  - 非同队；
  - bullet type absorbable；
  - 未被吸收；
  - 点在正多边形内；
  - 命中后 `hit = 1`、`buildup += shieldDamage`，并返回 effect/sound plan。
- 已将吸收/破盾副作用提升成运行态事件：
  - `force_projector_absorb_event(...)` 将 `ForceProjectorBulletAbsorb` 与 bullet 坐标收束为可执行 `absorbEffect.at(bullet)` / `hitSound.at(x,y,...)` 事件；
  - `force_projector_break_event(...)` 将 `force_projector_update(...)` 的 `broke_now` 结果收束为 `shieldBreakEffect.at(x,y,realRadius,teamColor)`、`breakSound.at(x,y)` 与 `Trigger.forceProjectorBreak` 事件计划；
  - `fire_force_projector_break_trigger` 保持 Java `team != state.rules.defaultTeam` 门控，后续可由真实 event bus 执行。
- 已将 bullet absorb 结果接到真实 `BulletComp::absorb()`：
  - `BulletComp::absorb()` 对齐 Java `absorbed = true; remove()` 的核心状态，当前设置 `absorbed/removed` 并清空 collision 记录，保持 `hit=false` 以保留 despawn 路径语义；
  - `force_projector_apply_absorb_to_bullet(...)` 只在 `ForceProjectorBulletAbsorb.absorbed` 为真时修改 bullet 状态。
  - `force_projector_absorb_bullet_comp(...)` 已把 `BulletType::shield_damage(bullet.damage)`、`BulletType.absorbable` 和 `BulletComp.absorbed` 接入同一入口，避免调用方手动拼 Java `bullet.type.shieldDamage(bullet)` 语义。
- 已对照 `ForceProjector.setBars()` / `sense(...)` 锁定：
  - shield bar fraction = `1 - buildup / (shieldHealth + phaseShieldBoost * phaseHeat)`；
  - `LAccess.heat` 返回 `buildup`；
  - `LAccess.shield` 返回剩余护盾；
  - `setProp(LAccess.shield)` 反向设置 `buildup`。
- 已对照 `ForceProjector.outputsItems()/shouldAmbientSound()/inFogTo()` 锁定：
  - `outputsItems=false`；
  - ambient sound 条件为 `!broken && realRadius() > 1f`；
  - 对任意 viewer 均 `inFogTo=false`。
- 已对照 `ForceProjector.onRemoved()` 锁定：
  - 先计算 `realRadius()`；
  - `!broken && radius > 1f` 时播放 `Fx.forceShrink`；
  - 始终继续调用 super removed。
- 已对照 `ForceProjector.deflectBullets()` 的范围裁剪入口收口：
  - `realRadius() > 0 && !broken` 时才扫描；
  - 扫描 bounds 为 `x/y ± realRadius()` 的正方形；
  - 真实 `Groups.bullet.intersect(...)` 仍待接入 runtime。
- 已对照 `ForceProjector.pickedUp()` 锁定：
  - pickup 后只清零 `radscl` 与 `warmup`；
  - `broken / buildup / phaseHeat` 保持原状态，序列化仍按 Java 5 字段读写；
  - `hit` 为 transient，read 后恢复为 `0`。
- 已对照 `ForceProjector.overwrote(...)` 锁定：
  - previous 首个 building 是同 block 且为 `ForceBuild` 时只继承 `broken` 与 `buildup`；
  - 不继承 `radscl / warmup / phaseHeat / hit`。
- 已对照 `ForceProjector.draw()` / `drawShield()` 锁定：
  - buildup > 0 时先绘制 topRegion additive；
  - `broken` 或 `realRadius <= 0.001` 时不绘制盾体但仍 final reset；
  - animateShields 时走 shield layer + fill poly；
  - 非 animateShields 时 stroke 1.5、alpha `0.09 + clamp(0.08 * hit)`、fill poly + outline poly；
  - `shieldRotation`、`sides`、`hit` layer offset 已进入 draw plan。
- `MendProjectorState`
- `ProjectorRuntimeSource`
- `EffectProjectorRuntimeState`
- `EffectBlockRuntimeState`
- `EffectProjectorRuntimeInput`
- `EffectProjectorRuntimeReport`
- `EffectBlockRuntimeContext`
- `EffectBlockRuntimeResources`
- `EffectBlockFrameInput`
- `EffectBlockRuntimeReport`
- `EffectBlockRuntimeStateStore`
- `effect_block_building_delta(...)`
- `effect_block_building_edelta(...)`
- `effect_build_turret_timer_target_ready(...)`
- `effect_build_turret_timer_target2_ready(...)`
- `effect_block_runtime_state_for(...)`
- `effect_block_data_for_building(...)`
- `effect_block_runtime_state_for_building(...)`
- `effect_projector_update_runtime(...)`
- `effect_block_update_runtime(...)`
- `effect_block_update_runtime_state(...)`
- `effect_block_update_building_runtime(...)`
- `effect_radar_update_building_frame(...)`
- `effect_projector_update_building_frame(...)`
- `effect_projector_update_building_frame_with_timer(...)`
- `effect_base_shield_update_building_frame(...)`
- `effect_shockwave_tower_update_building_frame(...)`
- `effect_shockwave_tower_update_building_frame_with_timer(...)`
- `projector_runtime_target_in_range(...)`
- `projector_runtime_target_allowed(...)`
- `mend_projector_outputs_items(...)`
- `mend_projector_range(...)`
- `mend_projector_update(...)`
- `mend_projector_building_damaged(...)`
- `mend_projector_try_heal_building(...)`
- `mend_projector_apply_heal_to_buildings(...)`
- `mend_projector_apply_heal_runtime(...)`
- `write_mend_projector_state(...)`
- `read_mend_projector_state(...)`
- `OverdriveProjectorStatsPlan`
- `overdrive_projector_stats_plan(...)`
- `OverdriveProjectorBoostPlan`
- `overdrive_projector_outputs_items(...)`
- `overdrive_projector_range(...)`
- `overdrive_projector_boost_plan(...)`
- `overdrive_projector_can_overdrive_content(...)`
- `overdrive_projector_apply_boost_to_buildings(...)`
- `overdrive_projector_apply_boost_with_content(...)`
- `overdrive_projector_apply_boost_runtime(...)`
- `overdrive_projector_bar_text_percent(...)`
- `OverdriveProjectorState`
- `overdrive_projector_update(...)`
- `write_overdrive_projector_state(...)`
- `read_overdrive_projector_state(...)`
- `overdrive-dome` 变体当前按同类 overdrive 投射器状态机推进。
- 已对照 `MendProjector.MendBuild.updateTile()` 的 heal pulse 分支补充真实建筑组件接线：
  - `charge >= reload && canHeal` 时，`MendProjectorUpdate` 的 `heal_fraction` 现在可直接应用到上层 range/indexer 已筛出的 `BuildingComp` 候选；
  - `mend_projector_try_heal_building(...)` 对齐 Java `b.damaged() && !b.isHealSuppressed()`，只治疗未死亡、血量低于 `maxHealth - 0.001` 且未被治疗抑制的目标；
  - 治疗量按 `target.max_health * heal_fraction` 写入真实 `BuildingComp::heal(...)`，同步更新 `last_heal_time`，可由 `mend_projector_pulse_plan(fired, healed > 0)` 决定是否播放 heal sound；
  - `mend_projector_apply_heal_runtime(...)` 已加入 `ProjectorRuntimeSource` 的同队 + 真实半径过滤，把 Java `indexer.eachBlock(this, realRange, ...)` 的最小运行态入口接到 `BuildingComp` 治疗；
  - `Fx.healBlockFull` 与 sound 调度仍由后续 runtime/renderer 层承接。
- 已对照 `OverdriveProjector.OverdriveBuild.updateTile()` 的 runtime boost 分支补充真实建筑组件接线：
  - `charge >= reload` 后由 `overdrive_projector_boost_plan(...)` 生成 `realRange / canOverdrive / realBoost / reload + 1`；
  - `overdrive_projector_apply_boost_to_buildings(...)` 对上层 range/indexer 已筛出的候选 `BuildingComp` 调用真实 `BuildingComp::apply_boost(...)`；
  - `BlockDef::can_overdrive()` 汇总 Java `Block.canOverdrive` 默认值与各类 block overrides，`overdrive_projector_apply_boost_with_content(...)` 已能通过 `ContentLoader` 读取真实 content metadata 过滤目标；
  - `overdrive_projector_apply_boost_runtime(...)` 已同时执行同队 + `realRange` 半径过滤 + content `canOverdrive` 过滤，作为后续 building dispatcher/world indexer 可直接调用的最小入口；
  - 低于目标当前 `time_scale` 的 boost 不会降低已有更高加速，也不会延长较高加速持续时间，行为沿用 `Building.applyBoost(...)`。
- 已新增首个 content-backed effect projector dispatcher：
  - `effect_projector_update_runtime(...)` 直接接收 `EffectBlockData`，按 `EffectBlockKind::MendProjector / OverdriveProjector / RegenProjector` 分发到对应 state 与 runtime adapter；
  - 调用方只需传入 `ProjectorRuntimeSource`、`ContentLoader` 与候选 `BuildingComp` slice，避免同时持有 `&mut BuildingComp` 和 `&mut [BuildingComp]`；
  - Regen 分支已经在 dispatcher 内完成 `damaged_targets` 判定、mendMap 记录、`last_update_frame/update_id` 门控和真实 `BuildingComp::heal(...)` 应用；
  - 这是后续真实 building update loop 接入 `BlockDef::Effect(...)` 的最小入口；`Radar` 因需要额外 `FogControl` 已拆到 `effect_radar_update_runtime(...)`，`BaseShield` 因需要 bullet/unit 输入已拆到 `effect_base_shield_apply_runtime(...)`，后续继续收束成更统一的 effect-block runtime dispatcher。
- 已新增跨 effect block 的轻量统一 runtime dispatcher：
  - `EffectBlockRuntimeState` / `effect_block_runtime_state_for(...)` 为已迁移的 effect block 创建统一状态容器，覆盖 projector family、ForceProjector、Radar、BuildTurret、BaseShield、ShockwaveTower；`ShockMine` 当前无持久运行态 state；
  - `effect_block_data_for_building(...)` / `effect_block_runtime_state_for_building(...)` 已能从 `BuildingComp.block.id` 通过 `ContentLoader` 找到 `BlockDef::Effect(...)` 并创建对应 state，为后续 building store 初始化提供入口；
  - `EffectBlockRuntimeStateStore` 已按 `BuildingComp.tile_pos` 管理 per-building effect runtime state，`ensure_for_building(...)` 会按 content 自动初始化并复用既有 state，非 effect block 不会污染状态表；
  - `EffectBlockRuntimeContext` 目前支持 `Projector / Radar / ForceProjector / BaseShield / ShockwaveTower` 五类上下文；
  - `EffectBlockRuntimeResources` 将“已存储 state”之外的 FogControl、content、building/bullet/unit 候选等资源单独传入，方便后续 building store 只保存 `EffectBlockRuntimeState`；
  - `EffectBlockFrameInput` 开始承接 `GameState::advance_game_update_frame(...)` 输出的 `delta/update_id/tick` 以及 fog/tileSize 等帧级参数；
  - `effect_block_building_delta(...)` / `effect_block_building_edelta(...)` 对齐 Java `BuildingComp.delta() = Time.delta * timeScale` 与 `edelta() = efficiency * delta()` 的核心公式；
  - `effect_block_update_runtime(...)` 按传入上下文复用 `effect_projector_update_runtime(...)`、`effect_radar_update_runtime(...)`、`effect_base_shield_apply_runtime(...)` 与 `effect_shockwave_tower_apply_runtime(...)`；
  - `effect_block_update_runtime_state(...)` 已能从统一 `EffectBlockRuntimeState` 自动拆出具体 state 并调用对应 dispatcher，block/state/resource 不匹配时返回 `None`；
  - `effect_block_update_building_runtime(...)` 将 `BuildingComp.block.id -> EffectBlockData -> state store ensure -> runtime dispatch` 串成单栋建筑的一站式入口，为后续 `update_all_buildings(...)` 遍历打通最小调用链；
  - `effect_radar_update_building_frame(...)` 已能直接从 `BuildingComp.team/tile_pos/efficiency` 与帧输入组装 Radar runtime 资源，测试覆盖了 `GameState::advance_game_update_frame(...) -> RadarState` 的最小帧推进路径；
  - `effect_projector_update_building_frame(...)` 已能从 `BuildingComp` 与 `EffectBlockFrameInput` 组装 projector family runtime 输入，`RegenProjector` 测试覆盖了 `GameState::advance_game_update_frame(...) -> delta/edelta/update_id -> last_update_frame` 的状态门控链路；
  - `EffectBlockData` 已新增 `timer_slots / timer_use_slot / timer_check_slot / timer_target_slot / timer_target2_slot`，用于记录 Java `Block.timerDump` 占 0 号槽后各 effect block 自定义 timer 的 content-backed 槽位；
  - `effect_projector_update_building_frame_with_timer(...)` 已开始把 `BuildingTimerState` 侧车接入 `MendProjector` 的 `timer(timerUse, useTime / timeScale)` 可选消耗门控；当前从 `EffectBlockData.timer_use_slot` 读取槽位，`MEND_PROJECTOR_TIMER_USE_SLOT = 1` 作为 Java 对齐 fallback；
  - `effect_projector_consume_source_items(...)` 已把 projector runtime report 的 `MendProjectorUpdate.should_consume_optional`、`OverdriveProjectorUpdate.consumed` 与 `RegenProjectorUpdatePlan.consume_optional` 接到源 `BuildingComp.items` 扣料；普通 Overdrive 扣 `boost_items`，OverdriveDome 扣 `consume_items`，对齐 Java `Building.consume() -> ConsumeItems.trigger(...)` 的副作用拆分；
  - `effect_block_consume_source_items_from_report(...)` 已提供跨 `EffectBlockRuntimeReport` 的统一源建筑扣料入口，覆盖 projector family 与 ForceProjector；
  - `effect_projector_update_building_frame_with_timer_and_consume(...)` / `effect_force_projector_update_building_frame_with_timer_and_consume(...)` 已把 frame update 与源建筑扣料合并为可直接接入 building lifecycle 的入口，减少上层拿到 report 后漏调 consume helper 的风险；
  - `EffectBlockTimerStateStore` 已按 `BuildingComp.tile_pos` 管理 per-building `BuildingTimerState` 侧车，并以 `max(content.timer_slots, Java TimerComp 默认 6 槽)` 初始化；projector/Force 已有 `*_with_timer_store_and_consume` 入口，ShockwaveTower 与 BuildTurret 的 `timerCheck/timerTarget/timerTarget2` 也已有 store-backed wrapper，后续真实 world/building 遍历可同时持有 runtime store 与 timer store；
  - `effect_build_turret_update_building_frame(...)` 已把 `BuildTurret` 的 `timerTarget/timerTarget2` gate、team plan/following/validation 纯逻辑收束为单栋 building-frame wrapper；外层仍需提供 team plan 队列、follower 候选与 validPlace/validBreak 判定，后续再接真实 unit/team/world indexer；
  - `EffectBlockFrameResources` / `effect_block_update_building_frame_with_stores(...)` 已形成单栋 effect block 的最小帧级总调度入口：外部遍历提供 `BuildingComp`、frame、runtime store、timer store 与 family-specific 资源后，可按 `EffectBlockKind` 路由到 projector/Radar/Force/BaseShield/ShockwaveTower wrapper；
  - `effect_block_frame_input_from_game_update(...)` 已把 `GameState::advance_game_update_frame(...)` 的 `delta_ticks/tick/update_id` 转为 `EffectBlockFrameInput`，让 effect block batch dispatcher 可以直接由真实游戏帧源驱动；
  - `core::GameRuntime` 已作为最小运行时 facade 接入 `GameState::advance_game_update_frame(...) -> effect_block_frame_input_from_game_update(...) -> effect_block_update_building_slice_with_stores(...)`，并持有 effect block runtime/timer sidecar store；该入口在帧推进前消费 world-load lifecycle 事件并清理 tile_pos keyed sidecar，避免换图复用旧状态；
  - `GameRuntime` 已持有最小 owned building 集合并通过 `add_building(...)` 按 Java `Tile.setBlock(...)` 的 block-size offset 规则同步 `World` footprint tile 的 `block/build` 引用；`advance_owned_effect_blocks(...)` 可直接驱动 runtime 自有建筑集合，向 Java `Groups.build.update()` 入口继续靠拢（真实 EntityGroup/indexer 仍待接入）；
  - `GameRuntime::add_building(...)` 现在会在放置前清除新 footprint 覆盖到的旧中心建筑、proxy tile 引用以及对应 effect runtime/timer sidecar，避免多方块建筑被小建筑覆盖后遗留旧 `build` 引用；该行为对齐 Java `Tile.setBlock(...)` 多方块 first pass `other.setBlock(Blocks.air)` 的最小运行态语义；
  - `GameRuntime::refresh_owned_building_proximity(...)` 已迁移 Java `BuildingComp.updateProximity()` 的最小同队 edge-neighbor 语义：按 `Edges.getEdges(block.size)` 查询 `world.build(...)`，过滤异队/自身/游离引用，并在 `add_building(...)` / `remove_building_by_tile_pos(...)` 后刷新双向 proximity；
  - `GameRuntime::clear_buildings(...)` 已同步清理 `World` build refs 与 effect runtime/timer sidecar，防止后续 world reload 或 Groups.build 清空时复用旧 tile_pos 状态；
  - `GameRuntime::load_network_map_with_buildings(...)` 已把 `LegacyShortChunkMap` 的 center building chunk 按 Java `SaveVersion.writeMap(...)` 的 `revision byte + build.writeAll(...)` 前缀处理，先物化 `BuildingComp::read_base(...)` 可覆盖的基础字段，再同步 owned buildings、world build refs、proximity 与 update permission；当前已对已迁移的 `MendProjector / OverdriveProjector / ForceProjector / Radar / BaseShield / BuildTurret` block-specific state 建立最小 dispatcher，读入后写入 `EffectBlockRuntimeStateStore`，其中 `BuildTurret` 通过 `ContentLoader` 解析 Java `TypeIO.writePlans/readPlans`，解析失败时保留 raw plan bytes 并有 GameRuntime 回归测试锁定；其他 block-specific `read(...)` 状态仍待后续 dispatcher 承接；
  - 内容注册已补齐 Java `Blocks.load()` 在可见环境方块前的内部前缀：`air -> spawn -> remove-wall -> remove-ore -> cliff -> build1..build16 -> deep-water`，锁定 `deep-water` 的 Java 对齐 id 前移风险；`ConstructBlock` 已新增 Java-compatible `progress / previous / current / accumulator / totalAccumulator / itemsLeft(revision>=1)` payload codec，并由 `GameRuntimeConstructBlockState` / `construct_runtime_states` 接入 `load_network_map_with_buildings(...)`，建筑移除、world reload 与清空时会同步清理 sidecar。当前仍未实现真实构造/拆除 world mutation、builder 队列与 renderer 接入；
  - `GameRuntime` 新增 `GameRuntimePayloadBlockState` / `payload_runtime_states`，已把 `PayloadMassDriverBuild` 的 `PayloadBlock` common wire image 与 `link / turretRotation / state / reloadCounter / charge / loaded / charging` 接入 `load_network_map_with_buildings(...)`，并在建筑移除、world reload 与清空时同步清理 sidecar；`PayloadMassDriver` revision 0 旧图路径已用 runtime 回归锁定，只读 `link/turretRotation/state` 且不会要求 revision 1 的 `reloadCounter/charge/loaded/charging` 尾字段；当前也已接入同一 wire image 下的 `PayloadLoader/Unloader exporting` 与 `PayloadSource unit/configBlock/commandPos`，以及 `PayloadConveyor itemRotation`、`PayloadRouter sorted/recDir` 的读回路径；`PayloadBlock` common / `PayloadConveyor` 前缀里的非末尾 `BuildPayload` 已可通过 nested exact 模式递归消费已迁移 block state，避免吞掉后续子类字段；`PayloadDeconstructor progress/accum/deconstructing` 与 `Constructor/BlockProducer progress/recipe` 也已进入同一 dispatcher；当前非 terminal `UnitPayload` 已有最小 exact reader，完整 Unit 实体恢复与尚未迁移 nested block state 仍待补；
  - `PayloadRef` wire tag 已按 Java `Payload.payloadUnit = 0 / payloadBlock = 1` 修正；此前 Rust 常量顺序写反，会影响所有 `Payload.write(...)` 相关保存/网络 payload body 的 Java 互通。当前已用 `payload_ref_presence_and_headers_match_java_payload_write` 锁定精确字节头；
  - 对 Java `Payload.write(...)` 位于 block-specific payload 末尾的场景已先补 raw body 保留路径：`read_payload_ref_to_end(...)` 会按 Java tag 读取 `BuildPayload(block/version/build_bytes)` 或 `UnitPayload(class_id/unit_bytes)`；`PayloadConveyor.item` 与 `PayloadDeconstructor.deconstructing` 已接入 `GameRuntime`，非空 terminal payload 不再导致 map loader parse error。注意：`PayloadBlock` common 后面还有子类字段的场景（如 constructor/loader/router/mass-driver 等）仍需要真正的 block/unit payload codec，不能用 read-to-end 贪婪读取；
  - `read_terminal_payload_block_build_common(...)` 已覆盖 `PayloadBlockBuild.write()` 作为末尾字段的场景；`PayloadVoid` 已进入 `payload_runtime_states::Void(common)`，`UnitAssemblerModule` 已允许 terminal common 中保留 raw `PayloadRef`。非 terminal common 现在走 nested exact `BuildPayload` 路径；当同类 reader 被嵌在外层 `BuildPayload` 中时，`PayloadConveyor / PayloadDeconstructor / PayloadVoid / UnitAssemblerModule` 会改用 exact payload-ref 读取而不是 `read_to_end`，防止吞掉外层字段；
  - `GameRuntimePowerBlockState` / `power_runtime_states` 已接入 Java power/light 自定义 building payload：`PowerGenerator` 基态（consume/thermal/solar）、`NuclearReactor heat`、`ImpactReactor warmup`、`VariableReactor heat/instability/warmup`、`HeaterGenerator heat` 与 `LightBlock color`；`PowerNode/Battery/Diode/BeamNode/LongPowerNode` 暂不接 block-specific 分支，其持久态主要由 `BuildingComp::read_base(...)` 内的 `PowerModule` 覆盖；
  - `GameRuntimeProductionBlockState` / `production_runtime_states` 已接入 Java production 自定义 building payload：`Drill` 的 `progress/warmup`、`BeamDrill` 的 `time/warmup`，以及继承 `DrillBuild.write/read(...)` 的 `BurstDrill progress/warmup`；新增 `BurstDrillState` 的 Java-compatible read/write helper，只持久化继承自 Drill 的两字段，`smooth_progress/invert_time` 仍按 Java 作为运行时派生字段在读入后复位。`SolidPump/Fracker/WallCrafter` 在 v158 Java 中没有 block-specific `write/read` sidecar，当前 map loader 不为它们制造虚假 sidecar，并已有 `cliff-crusher` 空 payload 保护测试；
  - `GameRuntimeCraftingBlockState` / `crafting_runtime_states` 已接入 Java crafting/heat 自定义 building payload：`GenericCrafter / AttributeCrafter / HeatCrafter` 的 `progress/warmup`、`Separator` 的 `progress/warmup/seed(revision==1)`，以及 `HeatProducer` 的复合 wire image（先读 `GenericCrafter` 前缀，再读 `heat` 尾字段）；`HeatConductor/Incinerator/ItemIncinerator` 当前无 Java block-specific sidecar，不在 map loader 中制造状态；
  - `GameRuntimeDistributionBlockState` / `distribution_runtime_states` 已接入物流类自定义 building payload 的第一批高优先级状态：`Conveyor/ArmoredConveyor` item slots、`StackConveyor` link/cooldown、`ItemBridge/DuctBridge` link/warmup/incoming/moved、`BufferedItemBridge` buffer、`MassDriver` link/rotation/state、`DirectionalUnloader` unload item/offset、`Duct` recDir/current、`DuctRouter/OverflowDuct/StackRouter` sort/current、`Sorter` sort item、`Unloader` sort item、`Junction` 四向 buffer；其中 `Duct.recDir` 已按 Java `byte`、`DirectionalUnloader.offset` 已按 Java `short` 修正 wire 宽度，`OverflowGate` 当前版本无新增持久字段但已消费 revision 1/3 legacy payload 以避免旧图读偏；`Router` 当前无额外 block-specific payload；
  - `GameRuntimeStorageBlockState` / `storage_runtime_states` 已接入 `CoreBlock` 的 `commandPos`（revision >= 1），`container/vault` 等普通 storage 暂无额外 block-specific payload，仍由基础 item module 覆盖；
  - `GameRuntimeLiquidBlockState` / `liquid_runtime_states` 已接入 `LiquidBridge / DirectionLiquidBridge` 的 `link / warmup / incoming / moved` 状态；普通 conduit/router/tank/junction 仍主要依赖基础 liquid module 或无额外 block-specific payload；
  - `GameRuntimeLogicBlockState` / `logic_runtime_states` 已接入 logic building payload 的第一批 Java sidecar：`MessageBlock` 的 UTF message 文本、`SwitchBlock` revision 1 的 enabled bool、`LogicDisplay/TileableLogicDisplay` revision 1 的 `arc.math.Mat` 3x3 transform（bool + 9 个 float）、`MemoryBlock` 的 `memory.length + double[]`（读入容量超出时消费但截断，短缺时保留默认 0）、`CanvasBlock` 的 `data.length + byte[]`（长度不匹配时消费并保留默认像素数据），以及 `LogicBlock/Processor` 的 raw+typed payload sidecar（compressed code bytes + 可选 `LogicConfig`、TypeIO boxed 变量缓存、legacy memory skip、privileged ipt、tag/iconTag、wait timers 与 accumulator）；processor revision 0 旧式 `code + link positions` 分支已通过 GameRuntime 回归锁定；processor 变量读取已从 safe object reader 切到 `read_object_boxed(...)`，按 Java `TypeIO.readObjectBoxed(read, true)` 使用非 safe 字符串与 200 项数组上限，并将 building/unit boxed 引用保留为稳定 wire id；TypeIO 普通 `read_object(...)` 的非 safe 数组读取上限同步收紧为 Java 的 200 项，`read_object_safe(...)` 字符串上限同步为 Java 注释与实现中的 1200 chars；processor 写出已按 Java `LogicBuild.write()` 收紧为 memory 固定写 `0` 且 privileged `ipt` clamp 到 `[1,maxInstructionsPerTick]`，并补了 revision 2/3 原始字节 sentinel 边界测试，防止 `ipt` 与 `tag/iconTag` 错位；processor sidecar 当前只保证 Java-compatible 读入与保存，后续仍需把变量/links/waits 延迟恢复到真实 `LExecutor` runtime，避免 stale world reference；
  - `GameRuntimeCampaignBlockState` / `campaign_runtime_states` 已接入 campaign building payload：`LaunchPad/AdvancedLaunchPad` revision 1 的 `launchCounter`、`LandingPad` 的 `config item / priority / cooldown` 与 revision 1 的 `arriving item / arrivingTimer / liquidRemoved`、`Accelerator` revision 1 的 `progress`；`Accelerator.launching` 仍按 Java 作为运行时派生字段不从 save payload 恢复；
  - `GameRuntimeSandboxBlockState` / `sandbox_runtime_states` 已接入 sandbox source building payload：`ItemSource` 的 output item short config 与 `LiquidSource` revision 1 的 source liquid short config（revision 0 兼容 byte id）；`PowerSource/PowerVoid/ItemVoid/LiquidVoid` 当前无额外 Java block-specific payload，`PayloadSource` 继续走 `payload_runtime_states`；
  - `GameRuntimeLegacyBlockState` / `legacy_runtime_states` 已接入 legacy building payload/旧图兼容消费：`LegacyCommandCenter` 的单字节占位、`LegacyMechPad` 的 3 个旧 float pad 数据、`LegacyUnitFactory` 的 buildTime 与 revision 0 spawnCount；这些状态主要用于旧存档/旧地图读入不偏移，后续 removeSelf replacement 仍需接入真实 world mutation；
  - `GameRuntimeUnitBlockState` / `unit_runtime_states` 已接入 `UnitFactory` 的 `progress/currentPlan/commandPos/commandId`、`Reconstructor` 的 `progress/commandPos/commandId`、`UnitRepairTower` 的 `rotation`、`UnitAssembler` 的空 payload common 状态与 `progress/readUnits/PayloadSeq blocks/commandPos`，以及 `UnitAssemblerModule` 的空 payload common 状态；`UnitAssembler.blocks` 与通用 `PayloadSeq::read_java_new(...)` 已补 Java legacy 正数长度 block-only fallback，旧格式会按 `ContentType::Block + blockId(short) + amount(int)` 读入并有 GameRuntime 集成测试锁定；同时通过 `distribution_runtime_states` 接入 `UnitCargoLoader readUnitId` 与 `UnitCargoUnloadPoint item/stale`。内容层已补齐 `additive/multiplicative/exponential/tetrative-reconstructor` 的 `BlockDef::UnitReconstructor` 注册、requirements/consume/upgrades/capacity/cryofluid 字段与 registry accessor，避免 tech tree 引用落空；`DroneCenter` 类在 Java v158 `Blocks.java` 无 vanilla 注册项，当前不凭空制造 block 名。后续仍需补非空 payload body codec 与 assembler module link 的运行时重建入口；
  - `UnitFactory` / `Reconstructor` 的 GameRuntime 读入已修正为先消费 Java `PayloadBlockBuild.write()` 父类前缀（`payVector/payRotation/Payload.write(payload)`），再读自身字段；此前如果按子类字段直接读，会把 `payVector.x` 错当 `progress`。当前 common 支持空 payload 与可递归精确消费的 `BuildPayload`（包括嵌套 payload-conveyor 再携带 BuildPayload 的场景），非 terminal `UnitPayload` 与未迁移 nested block-specific state 仍待补；
  - `GameRuntimeDefenseWallState` / `defense_wall_runtime_states` 已接入防御墙类 building payload：`Door/AutoDoor` 的 `open` 与 `ShieldWall` 的 `shield/shieldRadius/breakTimer/hit`，并在建筑移除、world reload 与清空时同步清理 sidecar；普通 wall 的碰撞/反弹/闪电行为仍走已迁移 helper，后续需要把墙体 runtime update 进一步收束到统一 building dispatcher；
  - `AutoDoor` payload 已按 Java `AutoDoorBuild.write() -> DoorBuild.write(open) + AutoDoor.open` 双 bool 修正；`read_auto_door_state(...)` 消费父/子两段并以子段为最终 open，GameRuntime 对 `blast-door` 走该路径；
  - `GameRuntimeTurretBlockState` / `turret_runtime_states` 已接入炮塔类 building payload：`ItemTurret` 的 `reloadCounter/rotation/ammo entries/totalAmmo`、`ContinuousTurret/ContinuousLiquidTurret` 的 `reloadCounter/rotation/lastLength`、`PointDefenseTurret` 的 `rotation` 与 `TractorBeamTurret` 的 `rotation`；`PayloadAmmoTurret` 已补 Java legacy 正数长度 block-only `PayloadSeq` 读取，旧格式会按 block payload ammo 过滤并更新 `totalAmmo`，同时仍支持新格式 content-type payload seq；`LiquidTurret/PowerTurret/LaserTurret` 作为 generic turret 读入基础 `reloadCounter/rotation`，后续需继续补 turret live sync 专用读法；
  - `BuildingComp::advance_update_timing(...)` 已迁移 Java `BuildingComp.update()` 开头的 `timeScaleDuration -= Time.delta` / `!canOverdrive` 重置语义，并由 `GameRuntime::advance_and_dispatch_effect_blocks(...)` 在 batch dispatcher 前对传入建筑切片统一执行；
  - `BlockDef::no_update_disabled(...)` / `BuildingComp::should_update_tile(...)` 已迁移 Java `if(enabled || !block.noUpdateDisabled) updateTile()` 的通用门控；effect block batch dispatcher 已先执行该门控，避免后续接入 `noUpdateDisabled=true` 的建筑时错误 tick；
  - `BlockDef::supports_env(...)` 已迁移 Java `Block.supportsEnv(env)` 位掩码公式；`GameRuntime::refresh_owned_building_update_permissions(...)` 已对 owned buildings 执行最小 `checkAllowUpdate` 接线，越界或不支持当前 rules env 的建筑会被置为 disabled；
  - `mindustry_server::ServerLauncher` 已持有 `GameRuntime`，并将 `update(...)` 调整为可变 tick 入口；后续服务端 world load、Groups.build 迁移和 effect block owned dispatch 可直接落到 server runtime，而不是停留在 core-only helper；
  - `ServerLauncher::update(...)` 已实际调用 `update_runtime_effect_blocks(...)`，以 base content loader 和空 bullet/unit 资源安全驱动 `GameRuntime::advance_owned_effect_blocks(...)`；当前无建筑时为空 batch，后续服务端建筑集合接入后会自然进入同一主循环；
  - `ServerLauncher` 现在保留 `last_runtime_effect_report`，并已用 server-level 测试证明 `update()` 能驱动真实 owned `mend-projector` 建筑进入 `GameRuntime::advance_owned_effect_blocks(...)`，创建 batch report 并消费 phase item；
  - `ServerLauncher::flush_pending_world_data(...)` 已从最小 bootstrap payload 改为组装 runtime `NetworkWorldData`：写入 base `content_header_snapshot`、空 `content_patches_snapshot`、由 `GameRuntime::export_network_map_snapshot(&ContentLoader)` 生成的当前 world/building 轻量 map snapshot、`GameState::export_legacy_team_blocks(...)` 导出的 `team_blocks_snapshot`、markers/custom chunks，并在每个连接上补 Java `player.id + Player.write` body；这使新连接 world stream 开始携带服务端 runtime `Teams.plans` 与 owned building entity chunk，不再只是静态占位数据。当前 building chunk 已在 `revision byte + BuildingComp::write_base(...)` 后追加 `Door/AutoDoor/ShieldWall`、PowerGenerator/LightBlock、`Mend/Overdrive/Force/Radar/BaseShield/BuildTurret`、turret tail、`Drill/BeamDrill/BurstDrill`、`GenericCrafter/Separator/HeatProducer`、distribution 第一批物流 tail、logic tail、campaign tail、unit factory/assembler tail、`LiquidBridge`、`CoreBlock.commandPos`、`ItemSource/LiquidSource` 与 `LegacyCommandCenter/LegacyMechPad/LegacyUnitFactory` tail，其中 power generator、base shield、turret 基类/子类、separator、production drill 系列、distribution conveyor/bridge/duct/junction/unloader/sorter、logic switch/display/processor、campaign blocks、unit factory/reconstructor/repair tower/assembler、liquid bridge、core block 和 liquid source 按各自 Java `version()` 写出，后续需继续接入完整 `writeMap` 其他 block-specific tail serialization；
  - `desktop::DesktopLauncher` 已持有 `GameRuntime`，并在网络 world data、state snapshot 与 client-loaded state 切换后同步 `runtime.state`；后续客户端世界/建筑/effect runtime 可以从现有 `game_state` 镜像逐步切换到统一 runtime facade；
  - `EffectBlockFrameBatchResources` / `effect_block_update_building_slice_with_stores(...)` 已形成外部 `&mut [BuildingComp]` 集合的最小遍历入口；内部用 source snapshot 避免借用冲突，同时把原始 building slice 作为 projector 目标集合，并在 report 后对源建筑执行物品消耗；
  - `effect_force_projector_update_building_frame(...)` / `effect_force_projector_update_building_frame_with_timer(...)` 已能从 `BuildingComp.efficiency/optional_efficiency/timeScale`、帧 delta 与 content `phaseUseTime/timerUse` 组装 ForceProjector runtime 输入；`FORCE_PROJECTOR_TIMER_USE_SLOT = 1` 作为 Java 对齐 fallback，且 broken/phase invalid/efficiency=0 时不触碰 timer slot；
  - `effect_base_shield_update_building_frame(...)` 已能从 `BuildingComp`、bullet/unit 候选与帧 delta 组装 BaseShield runtime 输入，写回 `BulletComp::absorb()` 与 `BaseShieldState.smooth_radius`；
  - `effect_shockwave_tower_update_building_frame(...)` 已能从 `BuildingComp.potential_efficiency`、building delta/edelta、bullet 候选与 timer gate 组装 ShockwaveTower runtime 输入，写回 bullet damage/remove 与 `ShockwaveTowerState`；
  - `effect_shockwave_tower_update_building_frame_with_timer(...)` 已开始把 `BuildingTimerState` 侧车接入 ShockwaveTower 的 `timer(timerCheck, checkInterval)` 门控；当前从 `EffectBlockData.timer_check_slot` 读取槽位，`SHOCKWAVE_TOWER_TIMER_CHECK_SLOT = 1` 作为 Java 对齐 fallback；
  - `effect_build_turret_timer_target_ready(...)` 已开始把 `BuildingTimerState` 侧车接入 `BuildTurret` 的 `timer(timerTarget, targetInterval)` 队伍计划/跟随搜索门控；当前从 `EffectBlockData.timer_target_slot` 读取槽位，`BUILD_TURRET_TIMER_TARGET_SLOT = 1` 作为 Java 对齐 fallback；
  - `effect_build_turret_timer_target2_ready(...)` 已把 `BuildTurret` 的 `timer.get(timerTarget2, 30f)` 冲突 break-plan 低频扫描门控显式建模；`BUILD_TURRET_TIMER_TARGET2_SLOT = 2`、`BUILD_TURRET_TIMER_TARGET2_INTERVAL = 30.0` 对齐 Java，`build_turret_update_tick(...)` 只有在该 gate ready 且外层扫描发现 `conflicting_breaker` 时才触发 `DropConflictingBreak`；
- 已新增建筑 timer 侧车：
  - `core/src/mindustry/entities/comp/timer.rs` 新增 `BuildingTimerState`，默认 6 槽，对齐 Java `TimerComp` 的 `Interval(6)` 默认形态；
  - `BuildingTimerState::timer(index, time)` 保持 Java `Float.isInfinite(time) -> false` 与 slot-based `Interval.get(...)` 语义；
  - 当前仍作为低侵入 sidecar，尚未直接挂入 `BuildingComp` 字段；后续需逐步把 `MendProjector / ShockwaveTower / BuildTurret` 的外部 `timer_ready` bool gate 回收到该 sidecar。
  - `EffectBlockRuntimeReport` 将 projector report、Radar fog force-update、BaseShield runtime report 与 ShockwaveTower fire report 收束到同一返回类型；
  - 该入口仍保持显式上下文传参，避免把 `FogControl`、建筑 slice、bullet/unit slice 与 content loader 强行揉成一个巨型可变借用。

仍需：

- `ForceProjector.draw()` 接入真实 renderer/Draw dispatcher；
- `ForceProjector.setBars()/sense/setProp` 接入真实 building runtime；
- `DirectionalForceProjector` 接入真实 Groups.bullet.intersect、absorb effect、shield break effect 与 renderer；
- `MendProjector` 真实 range indexer 扫描、content suppressable 细节、heal effect/sound、drawPlace / drawSelect 接入；
- `OverdriveProjector` 与 `OverdriveDome` 的真实 building range 扫描、status/effect、draw/select 接入；
- 上述 helper 继续接入真实 block runtime，避免停留在单测 helper。

### 7.9 Defense 当前迁移进度小结

当前防御类方块迁移已经形成一条可持续推进的主线，重点进度如下：

- `ShieldWall`：已完成状态、受击、拾取/恢复与 draw plan，对照 Java `ShieldWall.java` 的核心护盾展示与半径计算逻辑已锁定；
- `ForceProjector`：已完成状态机、护盾吸收、爆炸处理、sense / setProp / bar 相关语义，以及持久化读写字段；
- `Teams`：已把 `SaveVersion.readTeamBlocks(...)` / `writeTeamBlocks(...)` 的核心语义接到 Rust runtime，`Teams.plans` 与导出路径已能承接 legacy team blocks；
- `Save envelope`：`RawSaveEnvelope` 已覆盖结构化 region 桥、markers/custom、content header、content patches、entities 相关封装，保存/读取外壳已进入可持续扩展阶段。

下一步防御类方块的重点会放在：

- `MendProjector`：纯语义层已补齐 stats、progress sense、timer-ready 消耗门控、place/select/light/draw plan 与 heal pulse plan；下一步接入真实 range/indexer 扫描、world heal 应用和 renderer/runtime；
- `OverdriveProjector`：纯语义层已补齐 stats、boost bar/text、boost application plan、place/select/light/draw plan 和 Java 线框公式；下一步接入真实范围扫描、目标 `applyBoost` 调用、效果/音效与真实 block runtime；
- 继续把当前已完成的 helper 逐步收拢为真实 building 生命周期调用，避免长期停留在只被测试直接调用的辅助层。

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

### 9.2 GameService / Achievement / SStat 事件桥接

参考：

```text
D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/service/GameService.java
D:/MDT/rust-mindustry/core/src/mindustry/service/game_service.rs
D:/MDT/rust-mindustry/core/src/mindustry/service/s_stat.rs
D:/MDT/rust-mindustry/core/src/mindustry/service/achievement.rs
```

已推进：

- `GameServiceState` 已覆盖 Java `registerEvents()` 中大量事件分支的纯计划层；
- `GameServiceEventPlan::apply_to(...)` 已把事件计划真正写入 `StatService` / `AchievementService` / `AchievementState`：
  - `stat_additions` → `SStat::increment(...)`；
  - `stat_amount_additions` → `SStat::add(...)`；
  - `stat_sets` → `SStat::set(...)`；
  - `stat_max_updates` → `SStat::max(...)`；
  - `achievements` → `AchievementState::complete(...)`；
- 这一步把 `MapMakeEvent`、`MapPublishEvent`、`PlayerJoin`、`ClientPreConnect` 等已有 plan 与真实服务写入接口接上，不再只停留在返回 plan 的 helper 层。

仍需：

- 将具体事件源（client/server runtime event bus、launcher、network callbacks）逐步调用对应 plan 并执行 `apply_to(...)`；
- 对 `GameServiceUpdatePlan`、`GameServiceBlockBuildPlan`、`GameServiceUnitCreatePlan` 等专用 plan 增加同类 apply 桥；
- 平台侧如 Steam/桌面服务接入后，应复用 `StatService` / `AchievementService` 接口，不要绕过本桥接层。

### 9.3 继续 NetworkIO / SaveVersion

参考：

```text
D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/net/NetworkIO.java
D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/io/SaveIO.java
D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/io/versions/SaveVersion.java
```

优先补：

- `WorldReloader.begin()/end()` 已新增 `NetServer::begin_world_reload(...)` / `end_world_reload(...)` runtime adapter：
  - begin 侧按 Java 顺序清远端玩家 unit、标记 logic reset、向 reloader 存下的远端连接发送 `WorldDataBeginCallPacket`；
  - end 侧按 Java 顺序 reset player、PVP assign team、调用真实 `send_world_data(...)` 发送 world stream；
  - 这一步把 `world_reloader.rs` 的纯 plan 接到 `net_server.rs` 真实 world-data transport，后续可直接挂到地图重载/换图流程。
- `BlockPlan` 已新增 `config_value: TypeValue`，`SaveVersion.readTeamBlocks(...)` 读入的 typed config 会保留原始 `TypeValue`，同时继续提供字符串化 `config` 给现有 build/runtime helper 使用；导出 `LegacyTeamBlocks` 时优先写回 typed config，避免 content/team/point 等配置在 Java↔Rust save/world-stream 往返中退化成字符串。
- `NetworkWorldData::bootstrap_for_connection(...)` 已开始按 Java `NetworkIO.writeWorld(...)` 的 `stream.writeInt(player.id); player.write(new Writes(stream));` 顺序写入最小 `NetworkPlayerData` body，Rust 客户端收到 bootstrap world stream 后可解析 player body 并发送 connect confirm。
- `mindustry_server::ServerLauncher` 的 pending world data 发送已不再直接调用 `write_minimal_world_data(...)`：现在先从 `runtime.state` 组装 `NetworkWorldData` 模板，并把 runtime `Teams.plans` 经 `GameState::export_legacy_team_blocks(...)` 写入 `team_blocks_snapshot` 后再 `write_world_data(...)` 发送；已用 server-level 测试反解捕获到的 world stream，锁定 sharded build plan 的 `team_id/block_id/config`。
- `desktop::DesktopLauncher::sync_loaded_world_data(...)` 已在应用 `NetworkWorldData.map_snapshot` 后调用 `GameRuntime::load_network_map_with_buildings(...)`，使联机 world stream 中的 center building payload 开始进入真实客户端 runtime owned building 集合，而不再只停留在 `GameState.world` tile snapshot。
- `GameState::apply_network_world_data(...)` 已把 `NetworkWorldData.team_blocks_snapshot` 接到 `apply_legacy_team_blocks(...)`：通过 `content_header_snapshot` 将 Java content id 映射回 block/item 等 content 名称，联机 world stream 的 `SaveVersion.readTeamBlocks(...)` 结果不再只停留在 `NetworkWorldData` sidecar，而会物化为 runtime `Teams` 的 build plans；缺少 team-blocks 时会清空旧 plans，避免换图后复用 stale plan。
- marker/custom chunks 精确拆分：`NetworkWorldData.marker_custom_tail` 会保存 Java `readMarkers -> readCustomChunks` 后半段的完整原始尾部；当 UBJSON marker 与 custom chunks 边界无法精确拆分时，写回优先保留该 opaque tail，避免 split 失败后额外补空 custom chunk 或丢失未知尾部；
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

补充已接入的 world/content 运行态桥：

- `World::wall_solid_with_content(...)` / `wall_solid_full_with_content(...)` / `passable_with_content(...)` 已通过 `ContentLoader` 查询真实 `BlockDef.base().solid/fills_tile`，不再只能用 “非 air 即 solid” 的临时 fallback；
- 这一步让 pathfinder、建筑放置、单位通行等后续 runtime 可以直接复用 content registry 中的 Java block metadata。
- 2026-05-25：`GameRuntime` 已把非 terminal `BuildPayload` 精确消费接入 payload common/conveyor runtime 读取链：
  - 覆盖 `PayloadRouter` 的 `PayloadConveyorBuild` 前缀，以及 `PayloadMassDriver/PayloadLoader/PayloadDeconstructor/PayloadConstructor/PayloadSource/UnitFactory/Reconstructor/UnitAssembler` 的 `PayloadBlockBuild` common 前缀；
  - 读取顺序按 Java `Payload.write -> BuildPayload.write`：presence bool -> payload type -> block id -> build version -> `Building.writeAll`，再通过 `ContentLoader` 找 block、`BuildingComp::read_base` + 已迁移 block-specific reader 递归消费，避免吞掉后续子类字段；
  - 仍未完成：未迁移 block-specific nested payload state；遇到这些仍 Parse 失败，不能用 `read_to_end` 误吞后续字段。
- 2026-05-25：`GameRuntimeMapLoadReport.block_state_bytes_ignored` 现在也会统计“block-specific state 成功读出但 payload 仍有 trailing bytes”的 building 记录；这是为了暴露 Java wire 少消费问题，当前只计数不阻断加载。
- 2026-05-26：`GameRuntimePayloadReadMode::{TopLevel,NestedExact}` 已区分顶层 terminal raw 保留与嵌套非末尾 exact 消费；当 `BuildPayload` 内部 block state 又包含 `PayloadConveyor / PayloadDeconstructor / PayloadVoid / UnitAssemblerModule` 这类原先 terminal reader 时，嵌套路径改用 exact payload-ref，不再 `read_to_end` 吞外层字段；对 Java 无 block-specific `write/read` 的已知 block kind（如 `router`）会只消费 `BuildingComp` 基础段；已用 `UnitFactory.common -> BuildPayload(router) -> factory fields` 和 `UnitFactory.common -> BuildPayload(payload-conveyor -> BuildPayload(door)) -> factory fields` 回归测试锁定。
- 2026-05-26：`GameRuntime` 已补非 terminal `UnitPayload` exact reader 的 v158 最小安全路径：按 Java 生成实体 `unit.write(read)` 的 class id + entity revision 选择当前内建 schema，结构化消费 common/baseRotation/payloads/buildingPayloads/missile/ammo 六类实体字段，`Payloadc` 单位内的 `payloads` 递归走 exact payload-ref；读取后仍以 `PayloadRef::Unit { class_id, unit_bytes }` raw 保存，避免在完整 Unit 实体恢复前丢字段。已用 `UnitFactory.common -> UnitPayload(flare) -> factory fields` 回归测试锁定外层字段不被吞。
- 2026-05-26：`PayloadAmmoTurret` 的状态线已补最小接入：`TurretBlockKind::PayloadAmmoTurret`、`TurretBlockData.payload_ammo`、`payload_ammo_turret_{write,read}_payloads(...)` 与 `GameRuntimeTurretBlockState::PayloadAmmo` 已按 Java `super.write -> payloads.write` / `super.read -> payloads.read -> removeAll(valid ammo)` 消费；当前 Java v158 原版内容未注册该 “Do not use this class” 块，因此先以直接 runtime reader 回归测试锁定字节消费与无效 ammo 过滤。
- 2026-05-26：`GameRuntime::export_network_map_snapshot(&ContentLoader)` 已追加 Payload 系列 building tail 导出：`PayloadConveyor/Router` 写 conveyor 前缀与 router sort/recDir，`PayloadMassDriver/Loader` 按 revision 1 写 common 与各自扩展字段，`PayloadDeconstructor/Constructor` 写 common + progress/accum/deconstructing 或 producer progress/recipe，`PayloadSource/PayloadVoid` 写 common 与 source unit/config/commandPos。已用 `game_runtime_exports_payload_state_tail_in_network_map_snapshot` 验证导出的 map snapshot 可被 `load_network_map_with_buildings(...)` 反向读回 payload sidecar，避免服务端 world stream 只带 base building 而丢载荷建筑状态。
- 2026-05-26：对照 `PayloadUnloader.java` 后确认 Java 端没有独立 active payload tail，`PayloadUnloaderBuild` 继承 `PayloadLoaderBuild.version()==1/write/read`，只新增 `lastOutputPower` 与卸载运行时逻辑；Rust 因此继续让 `payload-unloader` 走 `GameRuntimePayloadBlockState::Loader` / `PayloadLoaderState` / `write_payload_loader_extra` 共用路径，并新增 `game_runtime_loads_payload_unloader_state_from_network_map_building_payload` 与导出 roundtrip 覆盖，明确锁定 unloader 的 Java-compatible loader tail。
- 2026-05-26：对照 `BlockProducer.java` / `Constructor.java` 后确认 active 序列化顺序为 `PayloadBlockBuild` common -> `BlockProducer.progress` -> `Constructor.recipe(short)`，版本字节只存在于嵌套 `BuildPayload` 的 `block id + build.version + build.writeAll` 头中；已扩展 `constructor_recipe_resets_progress_and_roundtrips_short` 与 `block_producer_progress_and_item_acceptance_follow_java_build_shell`，覆盖 recipe 清空写 `-1`、不可生产配置仍按 Java 重置 progress、已有 payload 时 update 不推进生产且 heat/time 衰减；新增 `game_runtime_roundtrips_payload_constructor_block_payload_version`，锁定 constructor common 中嵌套 `BuildPayload` 的 version/build bytes 在 export -> load 后保留。
- 2026-05-26：`GameRuntime` 新增 `advance_owned_payload_constructors_with_recipe_build_time(...)`，把 owned `PayloadConstructor` building 的 `BlockProducer.updateTile()` 最小语义接入真实 game update frame：随 `GameState::advance_game_update_frame(...)` 推进、刷新 building update permission、按 building efficiency/timeScale 计算 delta/edelta，驱动 `BlockProducerState.progress/heat/time`，达到 recipe build time 后生成 base-only `BuildPayload`、清零 `payVector` 并写回 `payload_runtime_states`。已用 `game_runtime_advances_owned_payload_constructor_into_build_payload` 验证 constructor sidecar 可从 progress 生产出嵌套 block payload；新建 payload 的 block-specific 默认 version/tail 仍需继续对齐。
- 2026-05-26：`BlockDef` 新增 `requirements()` / `build_cost_multiplier()` / `explicit_build_time()` / `effective_build_time(items)`，按 Java `Block.init()` 公式计算 `buildTime`：显式 build time 优先，否则 `sum(requirements * item.cost)`，无 requirements 时默认为 `20f`，最后乘 `buildCostMultiplier`。`GameRuntime::advance_owned_payload_constructors(...)` 已改为直接使用 content registry 的 effective build time，不再要求调用方为普通路径传入 recipe build time 回调；保留 `advance_owned_payload_constructors_with_recipe_build_time(...)` 作为测试/特殊映射 seam。已用 `block_def_effective_build_time_matches_java_init_formula` 锁定 air/router/breach 三类默认、requirements+multiplier、显式 build time 路径。
- 2026-05-26：`PayloadConstructorBlockData::can_produce_block(...)` 已对照 Java `Constructor.canProduce(Block)` 接入完整门禁：`BuildVisibility.visible(...)`、尺寸区间、非 `CoreBlock`、rules banned、环境可建造与 filter 命中；`GameRuntime::configure_owned_payload_constructor(...)` 已对照 Java `config(Block.class)` / `configClear` 接入 owned constructor 配置链，合法 recipe 写入 sidecar 与 `BuildingComp.config`，非法 recipe 不覆盖旧 recipe 但在 recipe 变化时重置 `BlockProducerState.progress`，clear 仅清 recipe/config 而不清 progress。已用 `payload_constructor_can_produce_respects_java_filters`、`game_runtime_configures_owned_payload_constructor_recipe`、`game_runtime_rejects_banned_payload_constructor_recipe_and_resets_progress`、`game_runtime_clears_owned_payload_constructor_recipe` 锁定。
- 2026-05-26：`GameRuntime::advance_owned_payload_constructors*` 已接入 Java `BlockProducer` 的 `ConsumeItemDynamic` 最小运行态语义：按 `recipe.requirements` 与 `Rules.buildCostMultiplier` 的 `ceil(amount * multiplier)` 计算材料需求，材料不足时将本帧生产 efficiency 视为 0、不推进 progress、不产出 payload；材料满足并成功产出时从 owned building 的 `ItemModule` 扣除对应物品。已用 `game_runtime_payload_constructor_waits_for_recipe_items` 与 `game_runtime_payload_constructor_consumes_scaled_recipe_items_when_produced` 锁定缺料等待、倍率向上取整和产出后消耗。
- 2026-05-26：`GameState::build_visibility_context()` 已把当前 state/rules 派生为 `BuildVisibilityContext`（game/menu、editor、campaign、infiniteResources、allowEditWorldProcessors、lighting、unitAmmo、fog），`GameRuntime::configure_owned_payload_constructor(...)` 现在用该上下文调用 `PayloadConstructorBlockData::can_produce_block(...)`，不再固定使用默认可见性上下文；core zone 与 legacy launch pad 解锁仍待后续从真实 world/campaign tech 状态接入。
- 2026-05-26：`GameRuntime::advance_owned_payload_voids(...)` 已接入 Java `PayloadVoidBuild.updateTile()` 的最小运行态语义：先按 `PayloadBlock.moveInPayload(false)` 用 `payloadSpeed * delta` 将 `payVector` 拉回中心；当 payload 已到达且 building `efficiency > 0` 时清空 payload，作为 incinerate 行为的状态侧效果。已用 `game_runtime_advances_owned_payload_void_and_incinerates_payload` 与 `game_runtime_payload_void_keeps_arrived_payload_without_efficiency` 锁定到达清空、无效率不清空。
- 2026-05-26：`GameRuntime::advance_owned_payload_sources(...)` 已接入 Java `PayloadSourceBuild.updateTile()` 的 block 分支最小运行态语义：当 source 无 payload 且配置了 `configBlock` 时生成 base-only `BuildPayload`，将 `payVector` 清零、`payRotation` 设为 building `rotdeg()`，并同步 `PayloadSourceState.has_payload/scl`；`moveOutPayload()` 的相邻建筑转移/倾倒仍待统一 payload 传输 runtime 接入。已用 `game_runtime_payload_source_spawns_configured_block_payload` 锁定。
- 2026-05-26：`SandboxBlockData` 已补 `PayloadSource.canProduce(Block/UnitType)` 对应门禁：block 需 visible、尺寸小于 source、非 CoreBlock、未 banned、环境可建造；unit 需非 hidden、未 banned、支持当前 env。`GameRuntime::configure_owned_payload_source(...)` 已接入 Java `config(Block.class)` / `config(UnitType.class)` / `configClear` 的状态侧语义：block/unit 配置互斥、改变配置时清 payload/scl/has_payload、重复同值配置保持现有 payload，clear 清配置与 payload；完整 Unit runtime codec / EntityMapping 仍待后续推进。已用 `payload_source_can_produce_respects_java_filters`、`game_runtime_configures_owned_payload_source_block_and_clears_payload`、`game_runtime_payload_source_repeated_same_block_config_preserves_payload`、`game_runtime_rejects_banned_payload_source_unit`、`game_runtime_clears_owned_payload_source_config` 锁定。
- 2026-05-26：`GameRuntime::advance_owned_payload_sources(...)` 已把 `UnitPayload` 分支从 skipped 推进到最小 Java save-wire 生成：当前按 v158 `UnitEntity` class id `3` / revision `9` 写出 common unit payload body，写入 team、unit type、health、rotation、当前位置、空 items/statuses/plans/mounts，并在 `PayloadSourceState.command_pos` 存在时写入 `CommandController` target position；`GameRuntime::command_owned_payload_source(...)` 已作为 Java `PayloadSourceBuild.onCommand(Vec2)` 的最小 runtime 入口，不再需要测试直接写 sidecar；`source.has_payload`、`payVector=0`、`payRotation=rotdeg()` 与 block 分支保持一致。注意：这仍是临时最小 UnitEntity shell，完整 `EntityMapping` 按 unit 类型选择 class/revision、真实 Unit 实体创建、`UnitCreateEvent` 等副作用仍待后续接入。
- 2026-05-26：`GameRuntime::advance_owned_payload_sources(...)` 已开始接入 Java `PayloadBlockBuild.moveOutPayload()`：source 生成/已有 payload 后会按 `payloadSpeed * delta` 朝 `rotdeg()` 前方 `size * tilesize / 2` 移动并旋转；到达后会按 Java `BuildingComp.movePayload()` 的前方 `size/2+1` tile 查找同队建筑，当前最小闭环只支持转交到空的 `PayloadVoid`（使用 `payload_block_handle_payload(...)` 设置目标 `payVector/payRotation`），再由既有 `advance_owned_payload_voids(...)` 完成 move-in 与 incinerate。`dumpPayload()`、`PayloadConveyor/Loader/MassDriver/Deconstructor` 的完整 accept/handle/runtime 仍待后续按 Java 条件接入。
- 2026-05-26：`GameRuntime::advance_owned_payload_constructors(...)` 已复用同一最小输出链路：`BlockProducer` 产出 `BuildPayload` 后会立即按 constructor 的 `payloadSpeed/payloadRotateSpeed/size/rotation` 执行 move-out，到达后可转交到前方同队空 `PayloadVoid`，并由 void 运行态继续吞噬；当前仍只覆盖 constructor/source -> void 的最小闭环，未把 loader/conveyor/mass-driver/deconstructor 的 Java `acceptPayload/handlePayload` 全量接入。
- 2026-05-26：payload 输出转交目标已从 `PayloadVoid` 扩展到 `PayloadConveyor` 的最小 Java `acceptPayload/handlePayload` 语义：source/constructor 到达前方后，如果目标 conveyor 同队、空槽、payload block 尺寸满足 `payloadLimit` 且 `progress <= 5f`，会调用 `payload_conveyor_handle_payload(...)` 写入 `item/stepAccepted/itemRotation/animation`；当前只接入外部输入的 block payload，`UnitPayload` fits、router/reinforced conveyor、conveyor 自身 updateTile/checkBlocked/dump/next 递归推进仍待后续完整 runtime loop。
- 2026-05-26：`GameRuntime::advance_owned_payload_conveyors(...)` 已开始接入 Java `PayloadConveyorBuild.updateTile()` 的最小推进闭环：每个有效 game update frame 会刷新 owned building update permission、推进 conveyor `progress/step`，并在 `stepAccepted != curStep && item != null` 时按前方 `size/2+1` tile 尝试把 payload 转交给同队 `PayloadVoid` 或下一段空 `PayloadConveyor`；`transfer_payload_output_to_front(...)` 的 source 现在覆盖 `PayloadSource / PayloadConstructor / PayloadConveyor`，target 覆盖 `PayloadVoid / PayloadConveyor`。当前只实现最小 next 转交与状态清空，不包含 Java 的 `checkBlocked()`、`dumpPayload()`、`next.updateTile()` 递归前推、`PayloadRouter`、`ReinforcedPayloadConveyor`、完整 blocked/unit push/渲染插值和全图确定性 conveyor 调度；这些不得标记为完成，后续必须继续迁移并接入服务端/客户端主循环。
- 2026-05-26：`PayloadRouter` 已补最小块侧配置/过滤/runtime 读写接入：`PayloadBlockData::can_sort_block(...)` 现在锁定 Java `canSort(Block)` 的可见性、尺寸、CoreBlock、banned 与 env 条件，`GameRuntime::configure_owned_payload_router(...)` 已能对 owned router 进行 block 配置/清空并把 `sorted` 写回 `BuildingComp.config`，`GameRuntimePayloadBlockState::Router` 也扩展了 `matches/smooth_rot/control_time` 三个 runtime 侧字段，读回时会根据当前 item 与 `invert` 计算初始匹配状态；`payload_router_match_pick_control_and_serialization_follow_java_shell` 与 `game_runtime_configures_owned_payload_router_block_and_clears_payload` 已锁定最小行为。当前 unit sort 的完整匹配/选路仍未接入，`pickNext`/`control`/`moveFailed`/`drawSelect`/`drawPlanRegion`/`next.acceptPayload` 的完整 Java 路由闭环也还没实现，不能假装完成。
- 2026-05-26：`GameRuntime::advance_owned_payload_loaders(...)` 已开始接入 Java `PayloadLoaderBuild.updateTile()` / `PayloadUnloaderBuild.updateTile()` 的最小运行态闭环：按真实 game update frame 刷新 owned building、复用 `PayloadBlockBuild` common 的 move-in/move-out，将 `PayloadRef::Block` 的 `BuildingComp` base 解析出来后搬运 items/liquids/buffered power 并写回 build bytes，`PayloadLoaderState` 新增 runtime-only `last_output_power` 以承载 unloader 电池卸载输出；`transfer_payload_output_to_front(...)` 的 source/target 已扩展到 `PayloadLoader` 共用 state，loader 导出到前方 `PayloadVoid/Conveyor/Router/Loader` 的最小 handle 路径不再丢 payload。当前仍是最小闭环：item/liquid accept 只覆盖通用模块容量，`separateItemCapacity/consumesItem` 之外的复杂 block override、`dumpLiquid/dumpAccumulate`、power graph 真实供需、instant deconstruct effect、loader/unloader 与 router/mass-driver/deconstructor 的完整组合调度仍需继续迁移。
- 2026-05-26：`PayloadLoader` 的 item 装载门控已补 Java `separateItemCapacity` 最小运行态：`PayloadLoaderState` 新增 `payload_item_capacity_blocked`，`refresh_payload_loader_state_from_common(...)` 会在 payload block 使用独立物品容量且 loader 持有同种已满物品时触发导出；`payload_loader_load_inner_building(...)` 不再用 `items.total() >= itemCapacity` 错误阻止独立容量容器装入其他物品，而是按同种物品容量判断，并保留 `separateItemCapacity || consumesItem(item)` 失败时导出的 Java 分支。当前仍未接入所有 block 的复杂 `acceptItem/handleItem` override、`dumpAccumulate` 和完整 consume graph。
- 2026-05-26：`PayloadLoader/PayloadUnloader` 的 item 装卸已接入 Java `timer(timerLoad, loadTime / efficiency)` 批次门控：`PayloadLoaderState` 新增 runtime-only `load_timer`，loader/unloader 在 `efficiency <= 0.01` 时不推进 timer，达到 `loadTime / efficiency` 后才按 `itemsLoaded` 搬运一批物品，并保留 `loadTime <= 0` 时每帧立即触发的 Java `Interval` 语义；liquid 与 buffered power 分支仍按 Java 不受 item timer 约束。新增 `game_runtime_payload_loader_respects_timer_load_and_efficiency_gate` 与 `game_runtime_payload_unloader_respects_timer_load_gate` 锁定 1 tick 等待、2 tick 批量搬运和 efficiency gate。当前真实 power graph 分发仍未接入；`offloadSpeed/dumpAccumulate` 与 `dumpLiquid` 的最小输出闭环见后续条目，完整 override 仍待补。
- 2026-05-26：`PayloadUnloader` 已接入 Java `offloadSpeed` + `dumpAccumulate()` 的最小 item 输出运行态：`BuildingComp` 新增 transient `cdump/dump_accum`，`advance_owned_payload_loaders(...)` 在 unloader tick 结束后按 `offloadSpeed` 重复累积 `delta()`，并从 Java content item id 顺序中挑选库存，向同队 proximity 中的 `Storage/Core` 或 `ItemVoid` 执行最小 `acceptItem/handleItem` 搬运；随后补入 `Conveyor/ArmoredConveyor` 目标的最小 `acceptItem/handleItem` 路径，会按相对方向、rotation 与容量创建/更新 `GameRuntimeDistributionBlockState::Conveyor` 并同步 `BuildingComp.items`；再补入基础 `Router/Distributor` 目标的最小接收语义，按 Java `RouterBuild.acceptItem()` 要求仅允许同队且自身库存为空时接收 1 个物品；随后补入 `ItemIncinerator/slag-incinerator` 目标，按 Java `ItemIncineratorBuild.acceptItem()` 的 `efficiency > 0` 门控把外排物品作为 sink 消耗；再补入 `Duct/ArmoredDuct` 与 `DuctRouter/OverflowDuct` 目标，复用 Rust `duct_accept_item(...)` / `duct_router_accept_item(...)`，成功外排时同步 `DuctState.current/rec_dir` 或 `DuctRouterState.current`；本轮补入 `Sorter/OverflowGate` 作为 Java `instantTransfer` 中继目标的最小穿透路由，复用 `sorter_should_direct(...)`、`choose_side_route(...)` 与 `overflow_gate_route(...)`，允许 unloader 外排物品穿过 sorter/overflow-gate 到后续 `Storage/Core/Router/ItemVoid/ItemIncinerator`。新增 `GameRuntimePayloadLoaderFrameReport.dumped_items`、`game_runtime_payload_unloader_offloads_items_to_adjacent_storage_with_offload_speed`、`game_runtime_payload_unloader_offloads_items_to_adjacent_conveyor`、`game_runtime_payload_unloader_offloads_items_to_adjacent_router`、`game_runtime_payload_unloader_offloads_items_to_adjacent_item_incinerator`、`game_runtime_payload_unloader_offloads_items_to_adjacent_duct`、`game_runtime_payload_unloader_offloads_items_to_adjacent_duct_router`、`game_runtime_payload_unloader_offloads_items_through_sorter_to_item_void` 与 `game_runtime_payload_unloader_offloads_items_through_overflow_gate_to_item_void`，锁定 payload 内 10 铜经 `timerLoad` 卸出 8 个后，同 tick 按 `offloadSpeed=4` 最多填入 3 个到相邻 conveyor、仅填入 1 个到相邻 router/duct/duct-router、消耗 4 个到相邻 item incinerator，或穿过 sorter/overflow-gate 消耗 4 个到 item void。当前仍未接入完整 block-specific `acceptItem/handleItem/canDump` override、item bridge/junction 等运输网络目标、instantTransfer 多段链与全部三链防护、router/duct 的完整 update 转发、conveyor transient `minitem/mid/next` 的完整 update 语义，以及 cdump 在复杂 proximity 变化下的全部 Java 语义。
- 2026-05-26：基础 `Router/Distributor` 已开始接入 Java `RouterBuild.updateTile()` 的普通 item runtime：新增 transient `GameRuntimeItemRouterState { last_item, last_input, time }` 与 `GameRuntime::advance_owned_item_routers(...)`，router 在收到 `PayloadUnloader.dumpAccumulate()` 或 instantTransfer 穿透写入的物品时会记录 `lastInput/time=0`，随后按 `time += delta / speed`、`getTileTarget(...)` 的 proximity 轮转、overflow-gate 回源跳过与 `target.block instanceof Router || target.block.instantTransfer` 延迟规则，调用现有 `dump_item_to_target(...)` 把物品继续转发到真实接收端；`advance_owned_payload_loaders(...)` 在完成 unloader item dump 后会用同一帧 `delta` 驱动一次 item router tick，`game_runtime_item_router_forwards_unloaded_item_to_real_receiver` 已锁定 unloader -> router -> item-void 的整体闭环，避免 router 只接收不转发。当前仍未接入 Router 的 player control/unit aim、完整 instantTransfer 多段链、复杂 `lastInput` 序列化恢复、与普通 conveyor/duct 全图调度的确定性顺序。
- 2026-05-26：基础 `Duct/ArmoredDuct/DuctRouter/OverflowDuct` 已开始接入 Java `updateTile()` 的普通 item runtime：新增 transient `item_duct_progress_states` 与 `GameRuntime::advance_owned_item_ducts(...)`，在 `dump_item_to_duct(...)` 接收物品时按 Java `handleItem()` 设置 `progress=-1`，随后按 `progress += delta / speed * 2` 与 `1 - 1/speed` 阈值推进；`Duct` 走 front target，`DuctRouter` 走 `proximity + cdump + sortItem`，`OverflowDuct` 走 front-first / side-fallback / invert-aware 最小目标选择，并复用 `dump_item_to_target(...)` 继续接入 router/sorter/void/storage 等真实接收端；`advance_owned_payload_loaders(...)` 在同一帧 item router 后继续驱动一次 duct tick。新增 `GameRuntimeItemDuctFrameReport`、`game_runtime_item_duct_forwards_unloaded_item_to_real_receiver`、`game_runtime_item_duct_router_forwards_unloaded_item_to_real_receiver` 与 `game_runtime_overflow_duct_forwards_unloaded_item_to_real_receiver`，锁定 unloader -> duct / duct-router / overflow-duct -> item-void 的端到端流转，避免只验证普通 duct 单一路径。当前仍未完整覆盖 armored duct 的 Java duct-chain 特例、DuctRouter/OverflowDuct 全分支测试、junction/item-bridge 链路、复杂 proximity 变化和全图确定性调度。
- 2026-05-26：`ItemBridge/phase-conveyor` 已接入 Java `ItemBridgeBuild.updateTile()/updateTransport()` 的最小普通 item runtime：新增 transient `item_bridge_transport_counters` 与 `GameRuntime::advance_owned_item_bridges(...)`，有效 link 会按 `transportCounter += edelta` 与 `transportTime` 批量从源桥转交到同队同 block 的 linked bridge，维护 target `incoming`、source `warmup/moved`；link 无效的输出端会复用真实 `dump_one_item_from_building(...)` 向相邻可接收端外排。`BufferedItemBridge/bridge-conveyor` 也已接入同一 bridge pass：源库存先进入 Java `TimeItem` buffer raw layout，再按 `speed / timeScale` 延迟投递到 linked bridge，投递成功后 FIFO 左移；`dump_target_accepts_item(...)` / `dump_item_to_target(...)` 已允许 linked `ItemBridge/BufferedItemBridge` 接收来自源桥或外部上游的 item，`advance_owned_payload_loaders(...)` 在 router/duct 后继续驱动 bridge tick。新增 `GameRuntimeItemBridgeFrameReport`、`game_runtime_item_bridge_transfers_and_output_dumps_to_real_receiver` 与 `game_runtime_buffered_item_bridge_buffers_then_delivers_to_real_receiver`，锁定 source phase-conveyor -> linked phase-conveyor -> item-void，以及 source bridge-conveyor -> buffer -> linked bridge-conveyor -> item-void 的端到端流转。当前仍未接入 `Junction`、`DuctBridge` 的专用 Java runtime，`ItemBridge` 的 `checkIncoming` 周期窗口、`timeSpeed/time` 动画、完整 `checkAccept/checkDump` 边向兼容、`BufferedItemBridge.timerAccept` 的精确 4 tick timer、液体桥接与复杂双向链仍待迁移。
- 2026-05-26：`PayloadUnloader` 已补上 Java `dumpLiquid(liquids.current())` 的最小液体外排运行态：unloader 从 payload 内卸载液体到自身后，会按 Java `dumpLiquid(..., 2f)` 的比例差与 `cdump` 邻接轮转，把当前液体输出到同队 `LiquidBlock`（已用 `liquid-container` 验证）；随后继续接入 `LiquidJunctionBuild.getLiquidDestination(...)` 的最小链式目的地解析，允许 `PayloadUnloader -> liquid-junction -> liquid-container` 按来源方向穿过 junction 并把流量落到真实容器，junction 自身不存液。新增 `GameRuntimePayloadLoaderFrameReport.dumped_liquid_events`、`game_runtime_payload_unloader_dumps_liquid_to_adjacent_liquid_container` 与 `game_runtime_payload_unloader_dumps_liquid_through_liquid_junction`，锁定 payload 内 80 water 经 `liquidsLoaded=40` 卸出后同 tick 向目标容器外排 20。当前仍未接入完整多段 `getLiquidDestination(...)` 特殊块语义、`ArmoredConduit/DirectionLiquidBridge` 特殊 accept 规则、所有 block-specific `acceptLiquid/canDumpLiquid` override 与真实 liquid graph 调度。
- 2026-05-26：`GameRuntime::advance_owned_payload_mass_drivers(...)` 已开始接入 Java `PayloadMassDriverBuild.updateTile()` 的 owned runtime：`PayloadMassDriverState` 新增 runtime-only `pay_length/effect_delay_timer/last_other/waiting_shooters/rec_payload`，每帧按真实 game update frame 刷新 owned building、衰减 charge、推进 reload、维护 linked 双端队列与 accepting/shooting 状态；已覆盖 loaded payload 推进到发射长度、接收端 waitingShooter 队首/角度/reload 条件、charge 达到 `chargeTime` 后把 payload 转交到 linked 同队空 mass driver，并写入接收延迟/recPayload/lastOther。当前仍是最小双端发射闭环：视觉 effect/sound/shake 仅保留状态标记，Java 的 `targetSize/curSize`、`PayloadMassDriverData` 渲染、真实 `transferEffect` 延迟赋值语义、与 conveyor/router/loader/deconstructor 的组合调度、UnitPayload 完整实体恢复仍需继续迁移。
- 2026-05-26：`GameRuntime::advance_owned_payload_conveyors(...)` 已把 `PayloadRouter` 纳入 conveyor 运行态调度，并开始接入 Java `PayloadRouterBuild.pickNext()/moveFailed()` 的最小选路语义：router 持有 payload 且 `controlTime <= 0` 时会按 `matches` 选择 `recDir` 直行，未匹配且存在 sorted 时跳过 `recDir` 并扫描四向候选；候选接收检查复用 Java `next.acceptPayload(next,item)` 的“source is self”语义，实际转交仍走 `transfer_payload_output_to_front(...)` 的正常 source 条件，失败后触发同一套 pick-next。当前仍未完整覆盖 logic `control(LAccess.config)` runtime 入口、`onControlSelect`、普通 conveyor `next.updateTile()` 递归前推、UnitPayload sort key 与 renderer/config UI 行为。
- 2026-05-26：`PayloadRouterBuild.pickNext()` 的候选方向判断已补 Java `if(next instanceof PayloadConveyorBuild && !(next instanceof PayloadRouterBuild)) next.updateTile()` 对应的最小运行态：`GameRuntime::payload_router_candidate_accepts(...)` 在判断某方向能否接收前，会对前方普通 `PayloadConveyor` 执行一次 `pre_advance_plain_payload_conveyor_target(...)`，让该 conveyor 本 tick 先尝试把已有 payload 转交到前方目标，再用 Java “source is self” 接收条件决定 router 是否选择该方向。当前只覆盖 router pickNext 的一层普通 conveyor 预更新；完整 `PayloadConveyorBuild.updateTile()` 递归链、router 之间递归、`checkBlocked()`、`dumpPayload()` 与非 payload-void/conveyor 组合调度仍待继续迁移。
- 2026-05-26：`PayloadRef::Unit` 已接入最小 unit sort key 解析，`payload_ref_sort_key(...)` 现在会按 v158 已支持的 unit payload class/revision schema，从 `unit.write(...)` 尾部读取 `UnitType` id 并返回 `ContentType::Unit` key；因此 `PayloadRouter` 的 unit 配置不再只是配置/序列化可写，运行态 `checkMatch/pickNext` 已可识别当前最小 `UnitEntity` payload source 生成体与已支持 exact UnitPayload raw body。当前仍不解析单位 hitSize，`payload_ref_fits_payload_limit(...)` 对 UnitPayload 仍返回 false，因此 UnitPayload 还不能按 Java `payload.fits(payloadLimit)` 完整进入普通 payload conveyor；后续需接入 UnitType/EntityMapping 后再补完整 fits、draw 与实体恢复。
- 2026-05-26：`GameRuntime::control_owned_payload_router_rotation(...)` 已接入 Java `PayloadRouterBuild.control(LAccess.config, p1, ...)` 的最小 runtime 入口：对 owned `PayloadRouter` 设置 `rotation = p1 mod 4`，并写入 `controlTime = 60 * 6`，在冷却期间 `pickNext` 不会自动把匹配 payload 拉回 `recDir`，从而允许 logic/manual control 指定方向输出。当前仍未接入真实 logic processor 指令分发、`onControlSelect(Unit)`、`super.control(...)` 的通用 LAccess 处理和 `onProximityUpdate()` 的完整邻接刷新副作用。
- 2026-05-26：`GameRuntime::payload_ref_fits_payload_limit(...)` 已把 `UnitPayload` 从固定拒收推进到按 Java `payload.fits(payloadLimit)` 的最小单位尺寸语义：通过 `payload_ref_sort_key(...)` 取出 `UnitType` id，再使用 content registry 中的 `UnitType.hit_size / TILE_SIZE <= payloadLimit` 判断是否能进入普通 `PayloadConveyor/Router`。这让 `PayloadSource` 生成的 `UnitPayload(flare)` 可直接转交到前方 conveyor/router，而不再只能进入 `PayloadVoid`；完整 Unit 实体恢复、动态 hit size、非 v158 已知 class/revision 的 UnitPayload 仍待后续接入。
- 2026-05-26：`PayloadRouter` 的 `smoothRot` 已从每帧直接赋值推进到 Java `Mathf.slerpDelta(smoothRot, rotdeg(), 0.2f)` 风格的 delta 平滑：`GameRuntime::payload_router_smooth_rot_step(...)` 按 360 度最短角差和 `1 - (1 - 0.2)^delta` 插值，`advance_owned_payload_conveyors(...)` 在 router 分支每个有效 update frame 更新 `smooth_rot`，避免 router 顶部渲染状态与 Java 旋转补间脱节。当前这仍只是 runtime sidecar 的数值对齐，完整 draw/topRegion 渲染、UI select/plan-region 与 client 渲染链路仍待后续接入。
- 2026-05-26：`GameRuntime::advance_owned_payload_deconstructors(...)` 已接入 Java `PayloadDeconstructorBuild.updateTile()` 的 block payload 最小运行态：每个有效 game update frame 刷新 owned building、同步 `has_payload/has_deconstructing` 派生状态、无 `deconstructing` 时重置 `progress`，将 `payRotation` 朝 90° 推进；已有 `common.payload` 到位后转入 `deconstructing` 并初始化 `accum`，拆解中使用 `PayloadRef::Block` 的 `BlockDef::requirements()` 与 `effective_build_time(content.items())` 调用 `payload_deconstructor_update_progress(...)`，把 `items_added` 写回真实 `BuildingComp.items`，完成后清空 `deconstructing/accum`。`transfer_payload_output_to_front(...)` 与 `payload_router_candidate_accepts(...)` 也已把 `PayloadDeconstructor` 纳入可接收目标，source/constructor/loader/conveyor/router 输出的 block payload 可进入前方 deconstructor。当前仍未实现 UnitPayload 的 `UnitType.getTotalRequirements()/unitCost(team)`、`dumpRate/dumpAccumulate()` 对外倾倒、BuildPayload 内部液体落地、`Fx.breakBlock`、logic `sense(content)` 与完整 renderer/UI 行为。
- 2026-05-26：`GameRuntime::configure_owned_payload_mass_driver(...)` 已接入 Java `PayloadMassDriver` 的 `config(Point2.class)` / `config(Integer.class)` 最小状态侧入口：支持相对坐标转绝对 packed tile、直接 packed link、`None/-1` 清配置，并把 owned building 的 `config` 写为 Java `config()` 语义的相对 `Point2`。这只补齐配置链和 sidecar `link` 同步，`advance_owned_payload_mass_drivers(...)`、`waitingShooters` 队列、`payLength/effectDelayTimer/lastOther/recPayload` 以及真实双端发射/接收 runtime 仍未实现。

## 10. 已知验证状态与风险

曾稳定通过的定向验证：

```bash
"C:/Users/yuyu/.cargo/bin/cargo.exe" test -p mindustry-core builder_ai_ -- --nocapture
"C:/Users/yuyu/.cargo/bin/cargo.exe" test -p mindustry-core build_turret -- --nocapture
"C:/Users/yuyu/.cargo/bin/cargo.exe" check -p mindustry-core
```

当前已知全包状态：

- `cargo test -p mindustry-core` 已在本地通过：1782 passed / 0 failed；
- `cargo test -p mindustry-desktop` 已在本地通过：12 passed / 0 failed；
- 2026-05-25/26 本轮定向验证：`cargo test -p mindustry-core build_payload` 通过 9/9；`cargo test -p mindustry-core game_runtime_reports_trailing_block_state_bytes_after_successful_read` 通过 1/1；`cargo test -p mindustry-core game_runtime_loads_unit_factory_common_no_state_build_payload_before_factory_fields` 通过 1/1；`cargo test -p mindustry-core game_runtime_loads_unit_factory_common_unit_payload_before_factory_fields` 通过 1/1；`cargo test -p mindustry-core game_runtime_loads_unit_factory_common_nested_payload_conveyor_without_swallowing_factory_fields` 通过 1/1；`cargo test -p mindustry-core game_runtime_reads_payload_ammo_turret_state_and_filters_invalid_payloads` 通过 1/1；`cargo test -p mindustry-core payload` 通过 166/166；`cargo test -p mindustry-core turret` 通过 59/59；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；
- 2026-05-26 constructor 配置闭环验证：`cargo test -p mindustry-core payload_constructor --no-fail-fast` 通过 8/8；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader` 通过 1/1；`rustfmt --check core/src/mindustry/content/blocks.rs core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过；
- 2026-05-26 constructor 动态物品消耗验证：`cargo test -p mindustry-core payload_constructor --no-fail-fast` 通过 10/10；`cargo test -p mindustry-core game_runtime_payload_constructor --no-fail-fast` 通过 2/2；`cargo test -p mindustry-core game_runtime_advances_owned_payload_constructor_into_build_payload` 通过 1/1；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader` 通过 1/1；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过；
- 2026-05-26 visibility context 验证：`cargo test -p mindustry-core build_visibility_context_reflects_state_and_rules` 通过 1/1；`cargo test -p mindustry-core game_runtime_configures_owned_payload_constructor_recipe` 通过 1/1；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`rustfmt --check core/src/mindustry/core/game_state.rs core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过；
- 2026-05-26 payload void 运行态验证：`cargo test -p mindustry-core payload_void --no-fail-fast` 通过 4/4；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader` 通过 1/1；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过；
- 2026-05-26 payload source block 运行态验证：`cargo test -p mindustry-core payload_source --no-fail-fast` 通过 4/4；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader` 通过 1/1；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过；
- 2026-05-26 payload source 配置验证：`cargo test -p mindustry-core payload_source --no-fail-fast` 通过 9/9；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader` 通过 1/1；`rustfmt --check core/src/mindustry/content/blocks.rs core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过；
- 2026-05-26 payload source unit 最小生成验证：`cargo test -p mindustry-core payload_source --no-fail-fast` 通过 9/9；`cargo test -p mindustry-core unit_payload --no-fail-fast` 通过 6/6；新增 `game_runtime_payload_source_spawns_common_unit_payload_with_command_pos` 覆盖 unit payload exact reader 可消费、class id/revision 与 command target position；
- 2026-05-26 payload source 输出移动/转交验证：`cargo test -p mindustry-core payload_source --no-fail-fast` 通过 10/10；`cargo test -p mindustry-core payload_void --no-fail-fast` 通过 5/5；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader` 通过 1/1；新增 `game_runtime_payload_source_moves_payload_into_front_payload_void` 覆盖 source 产出后 move-out 到达、前方 footprint 建筑解析、同队 `PayloadVoid` 接收与 source 清空，并继续调用 `advance_owned_payload_voids(...)` 验证后续 incinerate；
- 2026-05-26 payload constructor 输出移动/转交验证：`cargo test -p mindustry-core payload_constructor --no-fail-fast` 通过 11/11；`cargo test -p mindustry-core payload_source --no-fail-fast` 通过 10/10；`cargo test -p mindustry-core payload_void --no-fail-fast` 通过 6/6；新增 `game_runtime_payload_constructor_moves_output_into_front_payload_void` 覆盖 constructor 产出后 move-out、前方 `PayloadVoid` 接收、constructor 清空和后续 incinerate；
- 2026-05-26 payload conveyor 接收验证：`cargo test -p mindustry-core payload_source --no-fail-fast` 通过 12/12；`cargo test -p mindustry-core payload_constructor --no-fail-fast` 通过 12/12；`cargo test -p mindustry-core payload_conveyor --no-fail-fast` 通过 8/8；新增 source/constructor -> front conveyor 成功转交与 conveyor `progress > 5f` 拒收保留源 payload 的 runtime 回归；
- 2026-05-26 payload conveyor 最小推进验证：`cargo test -p mindustry-core payload_source --no-fail-fast` 通过 12/12；`cargo test -p mindustry-core payload_constructor --no-fail-fast` 通过 12/12；`cargo test -p mindustry-core payload_conveyor --no-fail-fast` 通过 9/9；`cargo test -p mindustry-core payload_void --no-fail-fast` 通过 7/7；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader` 通过 1/1；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。新增 `game_runtime_payload_conveyor_moves_item_into_front_payload_void` 覆盖 conveyor 持有 item 后按 step 推进并转交给前方 `PayloadVoid`。
- 2026-05-26 payload router 最小配置/读写验证：`cargo test -p mindustry-core payload_router --no-fail-fast` 通过 6/6；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；新增 `game_runtime_configures_owned_payload_router_block_and_clears_payload`、`payload_router_keeps_upstream_subset` 中的 `can_sort_block(...)` 锁定 router 的 block 过滤与 owned 配置闭环。当前 unit sort 的完整匹配/路由仍未接入，不得宣称 `PayloadRouter` 已完整可玩。
- 2026-05-26 payload loader/unloader 最小运行态验证：`cargo test -p mindustry-core payload_loader --no-fail-fast` 通过 4/4；`cargo test -p mindustry-core payload_unloader --no-fail-fast` 通过 2/2；`cargo test -p mindustry-core payload --no-fail-fast` 通过 200/200；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader --no-fail-fast` 通过 1/1；`rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/world/blocks/payloads/mod.rs` 与 `git diff --check` 通过。新增 `game_runtime_payload_loader_loads_items_into_payload_building`、`game_runtime_payload_unloader_unloads_items_from_payload_building`、`game_runtime_payload_loader_moves_exporting_payload_into_front_void` 覆盖 loader 资源写回嵌套 `BuildPayload`、unloader 从嵌套 payload 取出物品、loader 输出到前方 void 的最小 runtime 闭环。
- 2026-05-26 payload loader 独立容量验证：`cargo test -p mindustry-core payload_loader --no-fail-fast` 通过 5/5；`cargo test -p mindustry-core payload_unloader --no-fail-fast` 通过 2/2；`cargo test -p mindustry-core payload --no-fail-fast` 通过 212/212；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader --no-fail-fast` 通过 1/1；`rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/world/blocks/payloads/mod.rs` 与 `git diff --check` 通过。新增 `game_runtime_payload_loader_respects_separate_item_capacity_per_item` 与 `payload_loader_should_export(...)` 的 blocked-state 覆盖，锁定 container 已满 copper 时仍可装入 lead，而不会被 total capacity 误判导出。
- 2026-05-26 payload router pickNext 运行态验证：`cargo test -p mindustry-core payload_router --no-fail-fast` 通过 8/8；`cargo test -p mindustry-core payload_conveyor --no-fail-fast` 通过 9/9；`cargo test -p mindustry-core payload --no-fail-fast` 通过 202/202；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader --no-fail-fast` 通过 1/1。新增 `game_runtime_payload_router_sends_matching_payload_to_recorded_direction` 与 `game_runtime_payload_router_skips_recorded_direction_for_unmatched_payload`，覆盖匹配 payload 走 `recDir`、未匹配 sorted payload 跳过记录方向并向可接受目标转交。
- 2026-05-26 payload router UnitPayload sort key 验证：`cargo test -p mindustry-core payload_router --no-fail-fast` 通过 9/9；`cargo test -p mindustry-core unit_payload --no-fail-fast` 通过 7/7；`cargo test -p mindustry-core payload --no-fail-fast` 通过 203/203；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader --no-fail-fast` 通过 1/1；`rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/world/blocks/payloads/mod.rs` 与 `git diff --check` 通过。新增 `payload_unit_sort_key(...)` 纯 helper 覆盖和 `game_runtime_payload_router_matches_unit_payload_sort_key`，锁定 router unit sorted 配置能匹配 `UnitPayload(flare)` 并按 `recDir` 转交到前方 `PayloadVoid`。
- 2026-05-26 payload router logic control 验证：`cargo test -p mindustry-core payload_router --no-fail-fast` 通过 10/10；`cargo test -p mindustry-core payload --no-fail-fast` 通过 204/204；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader --no-fail-fast` 通过 1/1；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。新增 `GameRuntimePayloadRouterControlResult`、`control_owned_payload_router_rotation(...)` 与 `game_runtime_payload_router_logic_control_holds_manual_rotation`，锁定 control 后 360 tick 冷却、下一帧扣减到 300 tick 且匹配 payload 仍按手动方向输出。
- 2026-05-26 UnitPayload fits/conveyor 接收验证：`cargo test -p mindustry-core payload_source --no-fail-fast` 通过 13/13；`cargo test -p mindustry-core payload_conveyor --no-fail-fast` 通过 10/10；`cargo test -p mindustry-core unit_payload --no-fail-fast` 通过 8/8；`cargo test -p mindustry-core payload --no-fail-fast` 通过 205/205；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader --no-fail-fast` 通过 1/1；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。新增 `game_runtime_payload_source_moves_unit_payload_into_front_payload_conveyor`，覆盖 `PayloadSource(unit=flare)` 生成 UnitPayload 后按 hitSize/payloadLimit 被前方 `PayloadConveyor` 接收。
- 2026-05-26 payload router smoothRot 验证：`cargo test -p mindustry-core payload_router --no-fail-fast` 通过 11/11；`cargo test -p mindustry-core payload --no-fail-fast` 通过 206/206；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader --no-fail-fast` 通过 1/1；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。新增 `game_runtime_payload_router_smooth_rot_slerps_toward_rotation`，锁定 router 从 0° 朝 90° 在 1 tick 内平滑到 18°，而不是直接跳到目标角度。
- 2026-05-26 payload router 前方 conveyor 预更新验证：`cargo test -p mindustry-core payload_router --no-fail-fast` 通过 12/12；`cargo test -p mindustry-core payload_conveyor --no-fail-fast` 通过 10/10；`cargo test -p mindustry-core payload --no-fail-fast` 通过 207/207；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader --no-fail-fast` 通过 1/1；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。新增 `game_runtime_payload_router_pre_updates_front_conveyor_before_picking_next`，锁定未匹配 sorted payload 的 router 在候选方向判断前先让前方普通 payload conveyor 腾空，从而选择前方 conveyor 而不是错误改走侧边 void。
- 2026-05-26 payload deconstructor owned runtime 验证：`cargo test -p mindustry-core payload_deconstructor --no-fail-fast` 通过 6/6；`cargo test -p mindustry-core payload --no-fail-fast` 通过 210/210；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader --no-fail-fast` 通过 1/1；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。新增 `game_runtime_payload_deconstructor_accepts_front_block_payload`、`game_runtime_payload_deconstructor_moves_in_payload_and_starts_deconstruction`、`game_runtime_payload_deconstructor_progresses_and_outputs_items`，锁定前方输入接收、到位转入 `deconstructing`、按 block requirements 产出物品并完成清空。
- 2026-05-26 payload mass driver 配置入口验证：`cargo test -p mindustry-core payload_mass_driver --no-fail-fast` 通过 5/5；新增 `GameRuntimePayloadMassDriverConfig`、`GameRuntimePayloadMassDriverConfigureResult`、`configure_owned_payload_mass_driver(...)` 与 `game_runtime_configures_owned_payload_mass_driver_relative_link`，锁定 relative -> packed link、building config 相对 `Point2` 写回与清配置路径。
- 2026-05-26 payload mass driver owned runtime 验证：`cargo test -p mindustry-core payload_mass_driver --no-fail-fast` 通过 6/6；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning。新增 `game_runtime_advances_owned_payload_mass_driver_queues_and_fires_payload`，覆盖 linked 双端质量驱动器的 waiting shooter 队首、发射长度、charge/reload 条件、发射后源端清空重置、目标端接收 payload 与 `effectDelayTimer/lastOther/recPayload` 标记。
- 2026-05-26 item bridge / buffered bridge 最小 runtime 验证：`cargo test game_runtime_item_bridge --lib` 通过 1/1；`cargo test game_runtime_buffered_item_bridge_buffers_then_delivers_to_real_receiver --lib` 通过 1/1；`cargo test game_runtime_payload_unloader --lib` 通过 12/12；`rustfmt core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。新增 `GameRuntime::advance_owned_item_bridges(...)`、`item_bridge_transport_counters` 与 `bridge_forwarded_items`，覆盖 linked `phase-conveyor` 源桥按 `transportTime` 转交到输出桥，以及 `bridge-conveyor` 源桥先进入 `TimeItem` buffer、达到 `speed/timeScale` 后投递到 linked bridge，输出桥 link invalid 后向相邻 `item-void` 外排。
- 2026-05-26 junction 最小 runtime 验证：新增 `GameRuntime::advance_owned_item_junctions(...)`、`GameRuntimeItemJunctionFrameReport`、`item_junction_time_counters` 与 `junction_forwarded_items`，把 Java `JunctionBuild.acceptItem/handleItem/updateTile` 的四向 buffer 语义接入真实 item dump / payload-unloader 分发 tick；后续已升级为 `DirectionalItemBuffer(capacity=6)` FIFO。已新增 `game_runtime_junction_buffers_then_releases_item_to_real_receiver` 与 `game_runtime_payload_unloader_offloads_items_to_junction_then_runtime_releases` 覆盖 west->junction side0->east item-void、payload-unloader dumpAccumulate -> junction -> item-void 两条闭环。当前仍未完整迁移 sorter/overflow gate 组合输出与 renderer/UI 行为。
- 2026-05-26 junction FIFO 升级：上一条中的“每侧 1 item sidecar”限制已解除，`DuctJunctionState` 改为持有 Java-compatible `DirectionalItemBuffer(capacity=6)`，`write/read` 改用 `DirectionalItemBuffer.write/read_with_legacy(revision == 0)`，`dump_target_accepts_item(...)` 改为检查 `buffer.accepts(side)`，相邻 item dump / payload-unloader 分发会按来源 side 入队 FIFO；`advance_owned_item_junctions_ticks(...)` 现在按 Java `JunctionBuild.updateTile()` 每侧检查 FIFO head，达到 `speed/timeScale` 后只释放该侧一个 head 并 `remove(side)`，保留后续队列。新增 `game_runtime_junction_accepts_six_items_per_side_and_rejects_seventh` 覆盖每侧 6 容量与第 7 个拒收；`game_runtime_payload_unloader_offloads_items_to_junction_then_runtime_releases` 已更新为验证 payload-unloader 可填充 FIFO、同帧后续补货与 head 释放。仍未完整迁移完整 sorter/overflow gate 组合输出、renderer/UI 行为与所有 Java update-order 边界；复核 Java `Junction.java` 与 `DirectionalItemBuffer.java` 后确认没有独立 `timerAccept` 字段，后续不要再把它当作 junction 缺口。
- 2026-05-26 duct-bridge / DirectionBridge 最小 runtime 验证：`DuctBridge` 已从错误复用 `ItemBridgeState` 的序列化/联机尾字段中拆出，新增 runtime-only `DuctBridgeState { progress,last_link,occupied[4] }`、`GameRuntime::advance_owned_duct_bridges(...)` 与 `duct_bridge_forwarded_items`，按 Java `DirectionBridgeBuild.findLink()` 向 rotation 方向 range 内寻找首个同队同 block，linked 时用 `progress += edelta` / `progress > speed` 把源桥物品直接 `handleItem` 到 link，link missing 时按 Java `moveForward()` 最小语义向前方真实目标外排；`acceptItem` 已要求目标自身存在输出 link、容量未满、输入方向不是 rotation 且 `occupied[(rel+2)%4]` 未被 incoming bridge 占用。已新增 `game_runtime_duct_bridge_transfers_to_link_then_output_dumps_to_real_receiver`、`game_runtime_duct_bridge_rejects_input_without_output_link` 与 `game_runtime_exports_duct_bridge_without_item_bridge_tail`，锁定 duct-bridge runtime sidecar 不再污染 Java-compatible building payload。当前仍未完整实现 DirectionBridge 计划/选择渲染、`relativeToEdge` 对多格 block 的边缘精确方向、巨大 delta 下 Java 可能的多次 `items.take()` 丢弃边界、完整 occupied 清理时序与 renderer bridge overlay。
- 2026-05-26 stack-conveyor 最小 runtime 验证：新增 `GameRuntime::advance_owned_stack_conveyors(...)`、`GameRuntimeStackConveyorFrameReport` 与 `stack_conveyor_forwarded_items`，复用 Java-compatible `StackConveyorState { link,cooldown,last_item }`，按简化直线邻接推导 `stateLoad/stateMove/stateUnload`，接入 `acceptItem/handleItem` 的 loading dock 入栈、`cooldown -= speed * eff * delta`、满栈从 loading dock 整栈转交到前方空 stack conveyor、unload dock 通过真实 `dump_item_to_target(...)` 向前方/邻接接收方外排。已新增 `game_runtime_stack_conveyor_accepts_input_when_it_is_loading_dock` 与 `game_runtime_stack_conveyor_transfers_full_stack_and_unloads_to_real_receiver`。当前仍是直线最小闭环：`onProximityUpdate/buildBlending/blendprox`、复杂侧向 loading dock、`outputRouter=false`、renderer stack interpolation、power glow/baseEfficiency 组合和 Java `dump(...)` 的完整路由选择仍待继续迁移。
- 2026-05-26 item MassDriver 最小 runtime 验证：新增 `GameRuntime::advance_owned_item_mass_drivers(...)`、`GameRuntimeItemMassDriverFrameReport` 与 runtime-only `item_mass_driver_reload_counters`，复用 Java-compatible `MassDriverState { link,rotation,state }` 读写尾字段，按 Java `MassDriverBuild.linkValid()/acceptItem()/fire(...)` 的最小语义接入真实 item runtime：有有效 linked 同队同 block 目标时从 idle 进入 shooting，满足 `items.total() >= minDistribute`、目标剩余普通容量满足最小分发且 reload 就绪时，将源端最多 `itemCapacity` 个物品转交到 linked receiver，并初始化源/目标 reload counter；idle/accepting 时继续调用通用 `dump_one_item_from_building(...)` 外排。`dump_target_accepts_item(...)` 已加入 MassDriver 分支，锁定普通 dump 输入必须满足目标自身 `linkValid()` 且总库存未满，而不是无 link 也可塞入。已新增 `game_runtime_item_mass_driver_sends_items_in_flight_and_delivers_after_delay` 与 `game_runtime_item_mass_driver_accepts_dump_only_with_valid_output_link`；验证命令：`cargo test mass_driver --lib` 通过 13/13，`cargo test game_runtime_payload_unloader --lib` 通过 13/13，`cargo test game_runtime_exports_distribution_state_tail_in_network_map_snapshot --lib` 通过 1/1，`cargo check -p mindustry-core`、`rustfmt --check core/src/mindustry/core/game_runtime.rs`、`git diff --check` 均通过。后续闭环已补 `waitingShooters`、双端角度门控与延迟到达；当前仍未完整实现真实 `MassDriverBolt` entity、effects/sound/shake 与偏航/目标死亡后的爆炸掉落；`reloadCounter`、`waitingShooters` 与 in-flight shot 均作为 runtime-only sidecar 不写入 Java save tail。
- 2026-05-26 item MassDriver waiting shooter / angle gate 验证：`item_mass_driver_waiting_shooters` 作为 runtime-only sidecar 接入普通 item mass driver，tick 时按 Java `shooterValid()` 清理队列，idle 优先在有 waiting shooter 且目标剩余普通容量达到 `minDistribute` 时进入 accepting；shooting 端会把自己加入目标 waiting queue，只有队首、目标处于 accepting、源端朝 link 与目标端朝 shooter 的角度均进入 2° 误差内且 reload 就绪时才发射。`game_runtime_item_mass_driver_sends_items_in_flight_and_delivers_after_delay` 已更新为先验证首 tick 只排队/旋转不转移，再多 tick 对齐后发射；验证命令：`cargo test -p mindustry-core mass_driver --lib` 通过 13/13。
- 2026-05-26 item MassDriver 配置入口验证：新增 `GameRuntimeItemMassDriverConfig` / `GameRuntimeItemMassDriverConfigureResult` 与 `GameRuntime::configure_owned_item_mass_driver(...)`，对齐 Java `MassDriver` 的 `config(Point2.class)` / `config(Integer.class)`：支持相对坐标转绝对 packed link、直接 packed link、`None/-1` 清配置，并把 `BuildingComp.config` 写成 Java `config()` 语义的相对 `Point2`；配置切换/清除时会从 runtime-only waiting queue 中移除该 shooter，避免旧 link 残留。新增 `game_runtime_configures_owned_item_mass_driver_relative_and_clears_link` 覆盖 link 写入、config 相对值、同 link 保留有效等待队列、清 link、非 mass-driver 与缺失 building 拒绝。验证命令：`cargo test -p mindustry-core mass_driver --lib` 通过 14/14，`cargo check -p mindustry-core` 通过，仅保留既有 unused warning。
- 2026-05-26 item MassDriver 延迟到达验证：新增 runtime-only `GameRuntimeItemMassDriverInFlight` / `item_mass_driver_in_flight`，普通 item MassDriver 发射时现在只从源端扣出 `DriverBulletData.items[]` 等价物并按 `mass_driver_time_to_arrive(distance, bulletSpeed, bulletLifetime)` 入队，不再 fire tick 直接写入目标；后续 game update tick 到达后才按 Java `handlePayload(...)` 的 `itemCapacity * 2` 上限写入目标、初始化目标 reload、移除 waiting shooter 并置目标 idle。`game_runtime_item_mass_driver_sends_items_in_flight_and_delivers_after_delay` 已更新为验证发射后目标库存仍为 0、in-flight 持有 20 copper，经过到达 tick 后才交付 20。验证命令：`cargo test -p mindustry-core mass_driver --lib` 通过 14/14，`cargo test game_runtime_payload_unloader --lib` 通过 13/13，`cargo check -p mindustry-core`、`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前仍不是完整 bullet entity：未模拟 `MassDriverBolt.update()` 的偏航相交/目标死亡继续飞行与 `despawned()/hit()` 随机掉落/动态爆炸，后续需接入真实 entity/bullet runtime。
- 2026-05-26 item MassDriver 目标失效飞行寿命验证：`GameRuntimeItemMassDriverInFlight` 新增 `expire_ticks/target_lost`，建筑移除时不再直接删除相关 in-flight shot；每 tick 会检查目标 tile/block/team 是否仍匹配，若目标在到达前失效则标记 `target_lost`，到达时间后不会把 items 写入新建筑或已移除目标，而是继续保留到 `bulletLifetime` 结束后清理，贴近 Java `MassDriverBolt.update()` 中 `data.to.dead()` 时继续飞行直到 despawn 的语义。新增 `game_runtime_item_mass_driver_keeps_flight_after_target_removed_until_lifetime`，覆盖发射后移除目标、到达 tick 不交付/不重建目标、寿命结束才清理 flight。验证命令：`cargo test -p mindustry-core mass_driver --lib` 通过 15/15，`cargo test game_runtime_payload_unloader --lib` 通过 13/13，`cargo check -p mindustry-core`、`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前仍未实现 despawn/hit 的随机掉落与动态爆炸副作用。
- 2026-05-26 item MassDriver despawn 掉落/爆炸计划验证：新增 runtime-only `GameRuntimeItemMassDriverDespawnEvent` / `item_mass_driver_despawn_events`，target-lost shot 在 `bulletLifetime` 过期清理时会通过 Rust 既有 `MassDriverBolt::despawn_drop_plans(...)` 与 `dynamic_explosion_plan(...)` 生成可观测的掉落/爆炸计划，并在 `GameRuntimeItemMassDriverFrameReport` 中累计 `despawned_shots/dropped_items/explosion_events`。`game_runtime_item_mass_driver_keeps_flight_after_target_removed_until_lifetime` 已扩展为验证过期后记录 1 次 despawn、20 copper 掉落计划与 1 次 explosion event。验证命令：`cargo test -p mindustry-core mass_driver --lib` 通过 15/15，`cargo test game_runtime_payload_unloader --lib` 通过 13/13，`cargo check -p mindustry-core`、`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前掉落数量仍是 deterministic upper-bound 计划，尚未接入 Java `Mathf.random(0, amount)`、真实 `Fx.dropItem` 与 `Damage.dynamicExplosion(...)` world effect。
- 2026-05-26 DirectionalUnloader / duct-unloader 最小 runtime 验证：新增 `GameRuntime::advance_owned_directional_unloaders(...)`、`GameRuntimeDirectionalUnloaderFrameReport` 与 runtime-only `item_directional_unloader_timers`，复用 Java-compatible `DirectionalUnloaderState { unload_item, offset }` 读写尾字段，按 Java `DirectionalUnloaderBuild.updateTile()` 的最小语义接入真实 item runtime：计时达到 `speed` 后解析正前方/背后同队建筑，背后建筑需可卸载且有 items，配置了 `unloadItem` 时只搬运该 item，未配置时从 `offset` 开始按 `content.items()` 轮询，成功后 `offset = item.id + 1`；搬运路径通过通用 `dump_target_accepts_item(...)` / `dump_item_to_target(...)` 进入真实前方目标，并在通用 duct 接收判断中改用 `DistributionBlockData.is_duct`，使 duct-unloader 这类 Java `isDuct=true` 方块能作为 duct 输入来源。已新增 `game_runtime_directional_unloader_unloads_configured_item_to_front_neighbor` 与 `game_runtime_directional_unloader_round_robins_items_from_offset`；验证命令：`cargo test directional_unloader --lib` 通过 4/4，`cargo test game_runtime_payload_unloader --lib` 通过 13/13，`cargo test game_runtime_exports_distribution_state_tail_in_network_map_snapshot --lib` 通过 1/1，`cargo check -p mindustry-core`、`rustfmt --check core/src/mindustry/core/game_runtime.rs`、`git diff --check` 均通过。当前仍是最小 back->front 单件搬运模型，尚未完整迁移 Java `StorageBuild.linkedCore` 判定、`back.itemTaken(item)` 副作用、UI/配置表/plan config draw、与完整全局 building update dispatcher 的时序一致性。
- 2026-05-26 StackRouter / surge-router 最小 runtime 验证：新增 `stack_router_accept_item(...)`、`stack_router_progress_step(...)` 纯 helper、`GameRuntime::advance_owned_stack_routers(...)`、`GameRuntimeStackRouterFrameReport` 与 runtime-only `item_stack_router_unloading`，继续复用 Java-compatible `DuctRouterState { sort_item,current }` 存档尾字段，按 Java `StackRouterBuild.updateTile()/acceptItem()` 的最小语义接入真实 item runtime：未 unloading 时只从 rotation 方向接收同种 current 物品直到 `itemCapacity`，满栈后按 `enabled ? efficiency + baseEfficiency : 0` 推进 progress，达到 `speed` 后进入 unloading，并复用 DuctRouter `target()` 的 sortItem/cdump/背面排除规则持续向真实目标 dump 直到 current 清空；`dump_item_to_target(...)` 已能把输入写入 stack-router 的 duct-router-compatible sidecar 并把 progress 置为 Java `handleItem()` 风格的 `-1`。已新增 `stack_router_accept_item_and_progress_helpers_follow_java_branching`、`game_runtime_stack_router_unloads_full_stack_to_real_receiver` 与 `game_runtime_stack_router_accepts_same_item_only_while_not_unloading`；验证命令：`cargo test -p mindustry-core stack_router --lib` 通过 4/4。当前仍是最小 runtime 闭环，尚未完整迁移 glow/power draw、严格 Java building update 顺序、复杂多邻居目标在同帧内与其它 transport 的交错时序、完整 `DuctRouterBuild.target()` 与所有 block `acceptItem` 细节差异。
- 2026-05-26 regular Unloader 最小 runtime 验证：新增 `GameRuntime::advance_owned_item_unloaders(...)`、`GameRuntimeItemUnloaderFrameReport` 与 runtime-only `item_unloader_timers` / `item_unloader_rotations` / `item_unloader_last_used`，复用 Java-compatible `GameRuntimeDistributionBlockState::Unloader(sortItem)` 读写尾字段，按 Java `UnloaderBuild.updateTile()` 的最小语义接入真实 item runtime：达到 `speed` 且邻接候选不少于 2 后，配置了 `sortItem` 时只尝试该物品，未配置时按 rotations 轮询 `content.items()`；每个候选邻居刷新 canLoad/canUnload/loadFactor/lastUsed，按现有 `unloader_sort_key(...)` 选择 dumpingTo 与 dumpingFrom，满足 `(from.loadFactor != to.loadFactor || !from.canLoad)` 时通过 unloader 自身暂存 1 个物品并调用通用 `dump_item_to_target(...)` 进入真实接收方，再从来源移除 1 个。已新增 `game_runtime_item_unloader_moves_configured_item_from_storage_to_receiver` 与 `game_runtime_item_unloader_round_robins_unconfigured_items`；验证命令：`cargo test -p mindustry-core item_unloader --lib` 通过 2/2。当前仍是最小邻接搬运模型，尚未完整迁移 `possibleBlocks` 的池化生命周期、`StorageBuild.linkedCore` 精确判定、所有 block 的 `getMaximumAccepted/acceptItem/removeStack/itemTaken` 副作用、UI 配置表/plan center draw 与 Java 全局 building update 顺序。
- 2026-05-26 Junction `DirectionalItemBuffer(capacity=6)` 验证：`cargo test -p mindustry-core directional_item_buffer --lib` 通过 5/5；`cargo test -p mindustry-core junction --lib` 通过 8/8；`cargo test game_runtime_payload_unloader --lib` 通过 13/13；`cargo test game_runtime_exports_distribution_state_tail_in_network_map_snapshot --lib` 通过 1/1；`cargo check -p mindustry-core`、`rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/world/blocks/distribution/mod.rs core/src/mindustry/world/directional_item_buffer.rs` 与 `git diff --check` 通过。 本轮补齐 `DuctJunctionState.buffer`、Java revision 1 buffer 尾字段读写、runtime FIFO 入队/释放与每侧 6 容量拒收测试，替代旧的单槽 `times/item_data` sidecar。
- 2026-05-26 armored-duct 接收条件验证：`duct_accept_item(...)` 的 armored 分支不再无条件接收任意邻边输入，改为对齐 Java `DuctBuild.acceptItem()` 的两类入口：来源是 `isDuct` 且自身 front 指向目标，或来源 facing edge 相对方向等于目标 armored duct rotation；普通 duct 分支保持原“非 front 或 duct 输入”最小语义。`dump_target_accepts_item(...)` 已向 helper 传入 `source_front_points_to_target`，新增 `game_runtime_armored_duct_rejects_unaligned_non_duct_input` 与 `game_runtime_armored_duct_accepts_front_facing_duct_input` 覆盖真实 runtime 输入拒收/接收。验证命令：`cargo test -p mindustry-core duct_acceptance_progress_and_router_filters_follow_upstream --lib` 通过 1/1；`cargo test -p mindustry-core armored_duct --lib` 通过 2/2；`cargo test -p mindustry-core game_runtime_item_duct --lib` 通过 2/2；`cargo test game_runtime_payload_unloader --lib` 通过 13/13；`cargo check -p mindustry-core`、`rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/world/blocks/distribution/mod.rs` 与 `git diff --check` 通过。
- 2026-05-26 OverflowDuct/UnderflowDuct `cdump` 交替验证：新增 `overflow_duct_next_cdump(...)` 与 `overflow_duct_side_order(...)` helper，并让 `overflow_duct_target(...)` 使用 Java `cdump == 0 ? left : right` 的侧向优先级；`advance_owned_item_ducts_ticks(...)` 在 `OverflowDuct` 成功输出后将 building `cdump` 在 0/2 间切换，覆盖普通 overflow front blocked 后的 side fallback 顺序与 underflow 双侧可用时的左右轮换。新增 `game_runtime_underflow_duct_alternates_side_outputs_with_cdump`，验证 `underflow-duct` 两次输出分别进入左右真实 router 接收方并切换/恢复 `cdump`。验证命令：`cargo test -p mindustry-core duct_acceptance_progress_and_router_filters_follow_upstream --lib` 通过 1/1；`cargo test -p mindustry-core underflow_duct --lib` 通过 1/1；`cargo test -p mindustry-core overflow_duct --lib` 通过 1/1；`cargo test -p mindustry-core game_runtime_item_duct --lib` 通过 2/2；`cargo test game_runtime_payload_unloader --lib` 通过 13/13；`cargo check -p mindustry-core`、`rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/world/blocks/distribution/mod.rs` 与 `git diff --check` 通过。当前仍未完整迁移 `OverflowDuct.drawPlanRegion/icons` 与 renderer。
- 2026-05-26 StorageBlock `linkedCore` / `itemTaken` 最小 runtime 验证：`GameRuntime` 新增 runtime-only `storage_linked_cores`，在 building proximity 刷新后按 Java `CoreBuild.onProximityUpdate()` 的 `owns(StorageBuild && coreMerge)` 语义把相邻同队 storage 关联到 core；`dump_target_accepts_item(...)` / `dump_item_to_target(...)` / 普通 `Unloader` source 读取现在会通过 linked core 的 item module 处理 linked storage 的收发，`DirectionalUnloader` 会把 `CoreBuild` 或 linked storage 都视为 core source，默认 `allowCoreUnload=false` 时拒绝；成功 directional unload 后新增 `GameRuntimeItemTakenEvent` 记录 Java `back.itemTaken(item)` hook，linked storage 会把事件目标转发到 core tile。新增 `game_runtime_item_unloader_unloads_linked_storage_from_core_items_when_allowed`、`game_runtime_directional_unloader_rejects_linked_storage_without_core_unload`，并扩展 `game_runtime_directional_unloader_unloads_configured_item_to_front_neighbor` 断言 itemTaken。验证命令：`cargo test -p mindustry-core directional_unloader --lib` 通过 5/5；`cargo test -p mindustry-core item_unloader --lib` 通过 3/3；`cargo test -p mindustry-core game_runtime_payload_unloader --lib` 通过 13/13；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前 linked storage 仍是 runtime-only 重建关系，尚未完整共享 Java `ItemModule` 引用语义、campaign `handleCoreItem(...)` 真实副作用、core 移除时按容量比例拆分 storage items 与完整 `CoreBuild.onProximityUpdate()` 多核心容量同步。
- 2026-05-26 CoreBlock removed linked storage 拆分验证：`remove_building_at_index(...)` 现在会在移除 core 时检查 runtime-only `storage_linked_cores`，对仍存在的 linked storage 按 Java `CoreBuild.onRemoved()` 的 `items.get(item) * storage.itemCapacity / totalCapacity` 语义重建各 storage 自有 `ItemModule`，并清除指向被移除 core/storage 的 link 关系；新增 `game_runtime_core_removal_splits_linked_storage_items` 覆盖 core 持有 copper/lead、移除后相邻 container 得到拆分库存且 link map 清空。验证命令：`cargo test -p mindustry-core linked_storage --lib` 通过 3/3；`cargo test -p mindustry-core item_unloader --lib` 通过 3/3；`cargo test -p mindustry-core directional_unloader --lib` 通过 5/5；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前仍未完整实现多核心共享容量同步、campaign sector item delta、真实 `ItemModule` 引用共享与 core placement/upgrade 全流程。
- 2026-05-26 CoreBlock `storageCapacity` linked storage 刷新验证：`refresh_owned_storage_core_links(...)` 现在会同步刷新每队 core 的 runtime `CoreBuildState.storage_capacity`，最小对齐 Java `CoreBuild.onProximityUpdate()` 中 core 自身容量 + linked storage 容量的计算；`dump_target_accepts_item(...)`、`dump_item_to_target(...)` 与普通 `Unloader` load factor 现在通过 `maximum_accepted_for_item_owner(...)` 使用 runtime storage capacity，使 core 已达到自身 `itemCapacity` 时仍可因相邻 linked storage 接收更多物品，向 linked storage 投递也会写入 core item module。新增 `game_runtime_core_capacity_includes_linked_storage_for_acceptance` 覆盖 core 满自身容量后通过 linked container 继续接收 1 个 copper。验证命令：`cargo test -p mindustry-core linked_storage --lib` 通过 4/4；`cargo test -p mindustry-core item_unloader --lib` 通过 3/3；`cargo test -p mindustry-core directional_unloader --lib` 通过 5/5；`cargo test -p mindustry-core game_runtime_payload_unloader --lib` 通过 13/13；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前仍未完整实现多核心间共享同一 `ItemModule` 引用、coreIncinerates/非 buildable 焚毁效果与 campaign sector 统计。
- 2026-05-26 同队多 Core canonical item owner 验证：`item_module_owner_index(...)` 现在会把同队 `CoreBuild` 与 linked storage 的物品读写路由到该队第一个 canonical core，`refresh_owned_storage_core_links(...)` 会把非 owner core / linked storage 上已有 items 合并进 canonical core 并清空来源，最小模拟 Java `CoreBuild.onProximityUpdate()` 中多个 cores 共享同一个 `ItemModule` 的效果；新增 `game_runtime_same_team_cores_share_canonical_item_owner` 覆盖第二个 core 现有 lead 合并到第一个 core，向第二个 core 投递 copper 实际写入第一个 core，且容量为两个 core 容量总和。验证命令：`cargo test -p mindustry-core canonical_item_owner --lib` 通过 1/1；`cargo test -p mindustry-core linked_storage --lib` 通过 4/4；`cargo test -p mindustry-core item_unloader --lib` 通过 3/3；`cargo test -p mindustry-core directional_unloader --lib` 通过 5/5；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前仍是 canonical-owner 近似，不是真正共享同一个 Rust `ItemModule` 引用；core placement/upgrade、campaign sector 统计和完整 team core registry 仍需迁移。
- 2026-05-26 CoreBlock campaign core item delta 验证：普通 item dump 进入 core 或 linked storage 的 canonical core owner 时，`dump_item_to_target(...)` 现在触发 Java `CoreBuild.handleItem(...)` 的最小副作用：默认队伍 `GameStats.core_item_count` 增加，并在 campaign sector 上调用 `SectorInfo.handle_core_item(+1)`；linked storage 经普通 `Unloader` 被 `removeStack` 取走物品时会按 Java `StorageBuild.removeStack(...)` 对 linked core 的 campaign sector 调用 `handle_core_item(-1)`；DirectionalUnloader 的 `itemTaken` hook 也会在目标为 core 时调用 `handle_core_item(-1)`。新增 `game_runtime_core_handle_item_updates_campaign_sector_delta` 与 `game_runtime_linked_storage_unloader_updates_campaign_core_delta` 覆盖 `rules.sector` 与 `state.sector` 镜像、stats 计数和 linked storage 卸货负 delta。验证命令：`cargo test -p mindustry-core core_handle_item --lib` 通过 1/1；`cargo test -p mindustry-core campaign_core_delta --lib` 通过 1/1；`cargo test -p mindustry-core linked_storage --lib` 通过 5/5；`cargo test -p mindustry-core item_unloader --lib` 通过 3/3；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前仍未完整实现 `coreIncinerates` / `incinerateNonBuildable` 的焚毁分支、真实 incinerate effect、net server/client 差异与完整 sector persistence flush。
- 2026-05-26 CoreBlock 焚毁分支验证：`dump_target_accepts_item(...)` 对 core / linked storage owner 改用 `rules.core_incinerates`，满仓 core 在规则允许时会接收输入但不增加库存；`dump_item_to_target(...)` 现在按 Java `CoreBuild.handleItem(...)` 区分普通入库与焚毁，`incinerateNonBuildable && !item.buildable` 或 stored >= `storageCapacity` 时只扣来源、不写入 core items，并把 `CoreBuildState.no_effect` 置为 false；普通入库置为 true。非 buildable 焚毁仍会计入 `GameStats.core_item_count`，但不触发 campaign `handle_core_item(+1)`，满仓焚毁仍触发 campaign delta。新增 `game_runtime_core_incinerates_overflow_when_rules_allow` 与 `game_runtime_core_incinerates_non_buildable_without_campaign_delta` 覆盖 `coreIncinerates` 满仓 copper、`core-bastion.incinerateNonBuildable` 焚毁 sand、stats/sector/noEffect 行为。验证命令：`cargo test -p mindustry-core core_incinerates --lib` 通过 2/2；`cargo test -p mindustry-core core_handle_item --lib` 通过 1/1；`cargo test -p mindustry-core linked_storage --lib` 通过 5/5；`cargo test -p mindustry-core item_unloader --lib` 通过 3/3；`cargo test -p mindustry-core game_runtime_payload_unloader --lib` 通过 13/13；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前仍未接入真实 `StorageBlock.incinerateEffect(...)` 视觉效果、net server/client 条件差异与 non-core storage campaign removeStack 的完整边界。
- 2026-05-26 CoreBlock net/client handleItem 分支验证：新增 `GameRuntimeNetworkContext { active, server }`，默认离线保持既有 server/offline 权威语义；`CoreBuild.handleItem(...)` 的 Rust 最小副作用现在按 Java `if(net.server() || !net.active())` 分支执行，客户端 active 场景下仍增加默认队伍 `GameStats.core_item_count`，但不直接写 core items，也不重复触发 campaign `handle_core_item(+1)`，core 物品由服务端同步路径负责。新增 `game_runtime_core_handle_item_client_active_skips_authoritative_item_write` 覆盖 active client dump 到 core 后来源扣除、core 库存不本地增加、stats 增加且 campaign delta 为空。验证命令：`cargo test -p mindustry-core core_handle_item --lib` 通过 2/2；`cargo test -p mindustry-core core_incinerates --lib` 通过 2/2；`cargo test -p mindustry-core linked_storage --lib` 通过 5/5；`cargo test -p mindustry-core item_unloader --lib` 通过 3/3；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前仍未把真实 net role 自动从 `NetServer/NetClient` 注入所有 runtime tick，`StorageBlock.incinerateEffect(...)` 仍是 noEffect 可观测状态而非视觉效果。
- 2026-05-26 runtime net role 接入验证：`server::ServerLauncher::new(...)` 现在把 runtime 标记为 `GameRuntimeNetworkContext::server()`，desktop 端在 `NetworkWorldData` 被应用并物化 map/buildings 后标记为 `client()`，world data 清空时恢复 `offline()`；这样上一条 CoreBlock handleItem 的 net/server/client 分支不再只是单测字段，而是接入真实 server/desktop launcher 调用链。验证命令：`cargo test -p mindustry-server server_launcher_reads_port_arg_before_opening_network --lib` 通过 1/1；`cargo test -p mindustry-desktop desktop_launcher_materializes_network_map_buildings_into_runtime --lib` 通过 1/1；`cargo test -p mindustry-desktop desktop_launcher_resets_game_state_and_player_when_world_data_clears --lib` 通过 1/1；`cargo check -p mindustry-server` 与 `cargo check -p mindustry-desktop` 通过，仅保留 core 既有 unused warning；`rustfmt --check server/src/lib.rs desktop/src/lib.rs core/src/mindustry/core/mod.rs core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前仍需把更多 owned runtime tick 纳入 server update 主循环，而不是只在单元测试/局部 launcher 中调用。
- 2026-05-26 CoreBlock team registry 生命周期验证：`GameRuntime::add_building(...)` 现在会在建筑 `BlockFlag::Core` 时注册 `CoreInfo` 到 `state.teams.register_core(...)`，`remove_building_at_index(...)` / `clear_buildings(...)` 会执行 `unregister_core(...)`，`load_network_map_with_buildings(...)` 的清理阶段也复用 clear 路径，避免 core 重载后残留旧 team registry。新增 `game_runtime_core_building_lifecycle_updates_team_registry` 覆盖 core add 后 `Teams.cores/closest_core` 可见、remove/clear 后 registry 清空。验证命令：`cargo test -p mindustry-core core_building_lifecycle --lib` 通过 1/1；`cargo test -p mindustry-core clear_buildings --lib` 通过 1/1；`cargo test -p mindustry-core linked_storage --lib` 通过 5/5；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前仍未完整迁移 core placement/upgrade 资源扣除、`beforePlaceBegan/placeBegan` 的 nextItems 转移、真实 player spawn 与 UI/core bars。
- 2026-05-26 CoreBlock upgrade nextItems 验证：新增 `GameRuntime::can_place_owned_core_on(...)` 与 `upgrade_owned_core(...)`，对齐 Java `canPlaceOn/beforePlaceBegan/placeBegan` 的最小核心升级路径：检查目标块是更大的 CoreBlock、按 footprint floor/core-zone 与资源条件判定能否放置，从当前 team canonical core item owner 复制 items，非无限资源时扣除新 core requirements * `rules.buildCostMultiplier`，原地替换为新 core，刷新 world footprint、`state.teams.registerCore`、proximity、linked storage 与 `CoreBuildState.storage_capacity`。新增 `game_runtime_core_upgrade_replaces_core_and_transfers_items_minus_requirements` 覆盖 `core-shard -> core-foundation` 升级、剩余 items 保留为 `old - requirements`、队伍 core registry 和 storage capacity 同步。验证命令：`cargo test -p mindustry-core core_upgrade --lib` 通过 1/1；`cargo test -p mindustry-core core_building_lifecycle --lib` 通过 1/1；`cargo test -p mindustry-core linked_storage --lib` 通过 5/5；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前仍未完整实现普通 core 新建放置、core-zone 所有边界、升级 FX/Event、player spawn 与 UI/core bars。
- 2026-05-26 CoreBlock core-zone 新建放置验证：新增 `GameRuntime::place_owned_core(...)`，非升级路径会复用 `can_place_owned_core_on(...)` 的 Java `canPlaceOn` core-zone 分支，在 footprint 全部 floor `allowCorePlacement` 且不包含 core 时允许无旧 core/无资源直接创建 core，并接入 `add_building(...)`、world refs、`state.teams.registerCore` 与 `CoreBuildState.storage_capacity`。新增 `game_runtime_places_core_on_core_zone_without_existing_core` 覆盖 core-zone 3x3 footprint 上直接放置 `core-shard`、建筑创建、team registry 与 capacity 同步。验证命令：`cargo test -p mindustry-core places_core --lib` 通过 2/2；`cargo test -p mindustry-core core_upgrade --lib` 通过 1/1；`cargo test -p mindustry-core core_building_lifecycle --lib` 通过 1/1；`cargo check -p mindustry-core` 通过，仅保留既有 unused warning；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。当前仍未完整实现非 core-zone 普通 construction flow、升级 FX/Event、builder/BlockBuildEndEvent、player spawn 与 UI/core bars。
- 旧记录中的 `world_stream_with_java_like_payload_is_parsed_and_confirmed` 失败已通过补最小 player body 与测试期望修复；
- 接手者仍必须以当前本地实测为准，不要只相信历史记录。

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

## 12. 最新闭环：服务端 owned runtime 主循环聚合

- 2026-05-26：`GameRuntime` 新增 `GameRuntimeOwnedItemTransportFrameReport` 与 `GameRuntimeOwnedFrameReport`，把已迁移的 owned item transport（router/unloader/directional unloader/duct/duct bridge/stack router/stack conveyor/mass driver/item bridge/junction）按单个 frame 聚合报告输出。
- `advance_owned_item_transport_blocks(...)` 作为 item transport 独立入口保留；内部私有 `advance_owned_item_transport_blocks_ticks(...)` 可被更高层 frame aggregate 复用，避免每类 public `advance_owned_*` 各自调用 `advance_game_update_frame(...)` 导致 tick/update_id 翻倍。
- `advance_owned_runtime_blocks(...)` 已把 item transport 与 effect blocks 接到同一个 runtime frame：每次只调用一次 `GameState::advance_game_update_frame(...)`，随后刷新 owned building update permission / linked storage，再推进 building timing、item transport 和 effect runtime。
- `server::ServerLauncher::update(...)` 已改为调用 `update_runtime_owned_blocks(1/60)`，并缓存 `last_runtime_item_transport_report` 与 `last_runtime_effect_report`；这一步把已迁移的普通 item 物流与 effect building 从单测/局部 helper 接入真实服务端主循环。
- 已新增 server 级测试 `server_update_drives_owned_item_transport_from_launcher_runtime`，构造 `router -> item-void`，通过 `launcher.update()` 验证 router 物品被服务端主循环搬运，且 `runtime.state.update_id == 1`，锁定“不重复推进 frame”的约束。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_item_transport_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_effect_building_from_launcher_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_payload_unloader --lib`
  - `cargo test -p mindustry-core game_runtime_item_router --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：payload source/constructor/conveyor/loader/void 等 payload runtime 还没有统一纳入 `ServerLauncher::update(...)` 的 single-frame aggregate；后续应继续把更多 owned runtime tick 接入同一个 `advance_owned_runtime_blocks(...)`，不能回退为多个 public advance 串行调用。

### 12.1 服务端 PayloadVoid 主循环接入

- 2026-05-26：`GameRuntimeOwnedFrameReport` 新增 `payload: GameRuntimeOwnedPayloadFrameReport`，当前先接入 `PayloadVoid` 子报告，作为 payload 族进入服务端主循环的第一步。
- `advance_owned_payload_voids(...)` 已拆出内部 `advance_owned_payload_voids_ticks(...)`：public 入口仍保持单独 frame 推进能力；`advance_owned_runtime_blocks(...)` 在已经推进过本帧 `advance_game_update_frame(...)` 与 building timing 后复用 ticks 入口，避免 `PayloadVoid` 接入 server update 时重复增加 `update_id`。
- `server::ServerLauncher` 新增 `last_runtime_payload_report`，`update()` 会缓存同一帧的 payload batch；新增测试 `server_update_drives_owned_payload_void_from_launcher_runtime`，构造带 `BuildPayload(router)` 的 `payload-void`，通过 `launcher.update()` 验证 payload 被清空且 `runtime.state.update_id == 1`。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_void_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_item_transport_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_effect_building_from_launcher_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_advances_owned_payload_void_and_incinerates_payload --lib`
  - `cargo test -p mindustry-core game_runtime_payload_void --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：payload source / constructor / conveyor / loader / deconstructor / payload mass driver 还没进入 `advance_owned_runtime_blocks(...)`；后续继续接入时应同样拆 ticks/helper，不能直接串 public advance。

### 12.2 服务端 PayloadSource 主循环接入

- 2026-05-26：`GameRuntimeOwnedPayloadFrameReport` 追加 `source: GameRuntimePayloadSourceFrameReport`，`advance_owned_runtime_blocks(...)` 现在在同一 frame 内先推进 `PayloadSource`，再推进 `PayloadVoid`。
- `advance_owned_payload_sources(...)` 已拆出内部 `advance_owned_payload_sources_ticks(...)`：public 入口保持原有独立推进语义；服务端聚合路径复用 ticks 入口，避免 source 接入时重复推进 `advance_game_update_frame(...)` 或 building timing。
- 新增测试 `server_update_drives_owned_payload_source_from_launcher_runtime`：构造 `payload-source` 配置为生产 `router`，通过 `launcher.update()` 验证服务端主循环生成 `BuildPayload(router)`，并保持 `runtime.state.update_id == 1`。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_source_from_launcher_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_payload_source_spawns_configured_block_payload --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_void_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_item_transport_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_effect_building_from_launcher_runtime --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：PayloadSource 进入主循环后，`PayloadConveyor/Router/Constructor/Loader/Deconstructor/PayloadMassDriver` 仍需按同样方式拆 ticks 并接入；当前 source 生成后能移动自身 payload，但完整 source→conveyor→void 服务端链路还需 conveyor 接入后再验证。

### 12.3 服务端 PayloadConveyor/Router 主循环接入

- 2026-05-26：`GameRuntimeOwnedPayloadFrameReport` 追加 `conveyor: GameRuntimePayloadConveyorFrameReport`，`advance_owned_runtime_blocks(...)` 已在 source 后、void 前推进 `PayloadConveyor` / `PayloadRouter` ticks。
- `advance_owned_payload_conveyors(...)` 已拆出内部 `advance_owned_payload_conveyors_ticks(content, frame_delta, tick)`；public 入口仍独立推进 frame/timing，服务端聚合路径复用 ticks，继续保持单帧只推进一次 `GameState::advance_game_update_frame(...)`。
- 新增测试 `server_update_drives_owned_payload_conveyor_from_launcher_runtime`：构造已携带 `BuildPayload(router)` 的 `payload-conveyor` 指向 `payload-void`，将 runtime tick 对齐到 conveyor step 边界后调用 `launcher.update()`，验证 conveyor item 被转交到 void，且 `runtime.state.update_id == 1`。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_conveyor_from_launcher_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_payload_conveyor_moves_item_into_front_payload_void --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_source_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_void_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_item_transport_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_effect_building_from_launcher_runtime --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：`PayloadConstructor/Loader/Deconstructor/PayloadMassDriver` 还没进入服务端 aggregate；source→conveyor→void 已具备组成服务端链路的必要节点，但还需要后续补一个跨多帧 server smoke test 来验证生成、移动、接收与销毁的完整顺序。

### 12.4 服务端 PayloadConstructor 主循环接入

- 2026-05-26：`GameRuntimeOwnedPayloadFrameReport` 追加 `constructor: GameRuntimePayloadConstructorFrameReport`，`advance_owned_runtime_blocks(...)` 的 payload 顺序变为 constructor → source → conveyor/router → void。
- `advance_owned_payload_constructors_with_recipe_build_time(...)` 已拆出内部 `advance_owned_payload_constructors_ticks(...)`；public 入口仍负责单独推进 frame/timing，服务端聚合路径使用 content registry 的 `BlockDef::effective_build_time(...)` 回调复用 ticks，避免重复推进全局 frame。
- 新增测试 `server_update_drives_owned_payload_constructor_from_launcher_runtime`：服务端 runtime 中构造带 router 材料与 recipe 的 `constructor`，调用 `launcher.update()` 后验证生产 `BuildPayload(router)`、`producer.has_payload=true`，且 `runtime.state.update_id == 1`。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_constructor_from_launcher_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_advances_owned_payload_constructor_into_build_payload --lib`
  - `cargo test -p mindustry-core game_runtime_payload_constructor_moves_output_into_front_payload_void --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_conveyor_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_source_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_void_from_launcher_runtime --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：`PayloadLoader/Unloader`、`PayloadDeconstructor` 与 `PayloadMassDriver` 还没进入 server aggregate；constructor 已进入主循环但完整 constructor→conveyor→void 多帧 smoke 仍待补。

### 12.5 服务端 PayloadDeconstructor 主循环接入

- 2026-05-26：`GameRuntimeOwnedPayloadFrameReport` 追加 `deconstructor: GameRuntimePayloadDeconstructorFrameReport`，`advance_owned_runtime_blocks(...)` 的 payload 顺序目前为 constructor → source → conveyor/router → deconstructor → void。
- `advance_owned_payload_deconstructors(...)` 已拆出内部 `advance_owned_payload_deconstructors_ticks(...)`；public 入口保持独立推进 frame/timing，server aggregate 复用 ticks，使 payload deconstructor 的 move-in/start-deconstruction/progress 逻辑进入服务端主循环。
- 新增测试 `server_update_drives_owned_payload_deconstructor_from_launcher_runtime`：构造带 `BuildPayload(router)` 的 `small-deconstructor`，通过 `launcher.update()` 验证 payload 从 common slot 转入 `deconstructing`，并保持 `runtime.state.update_id == 1`。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_deconstructor_from_launcher_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_payload_deconstructor_moves_in_payload_and_starts_deconstruction --lib`
  - `cargo test -p mindustry-core game_runtime_payload_deconstructor_progresses_and_outputs_items --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_constructor_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_conveyor_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_source_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_void_from_launcher_runtime --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：`PayloadLoader/Unloader` 与 `PayloadMassDriver` 还没进入 server aggregate；deconstructor 的完成回收路径已在 core 测试覆盖，但还缺 server-level 长 delta/progress 输出 items 测试。

### 12.6 服务端 PayloadLoader/Unloader 主循环接入

- 2026-05-26：`GameRuntimeOwnedPayloadFrameReport` 追加 `loader: GameRuntimePayloadLoaderFrameReport`，`advance_owned_runtime_blocks(...)` 的 payload 顺序目前为 constructor → source → conveyor/router → loader/unloader → deconstructor → void。
- `advance_owned_payload_loaders(...)` 已拆出内部 `advance_owned_payload_loaders_ticks(content, frame_delta, run_item_transport)`；public 入口仍独立推进 frame/timing 且保持 `run_item_transport=true` 的旧语义，server aggregate 使用 `run_item_transport=false`，避免在同一帧里把全局 item transport ticks 推进两次。
- 新增测试 `server_update_drives_owned_payload_loader_from_launcher_runtime`：构造带 `BuildPayload(container)` 的 `payload-loader` 与 5 个铜，调用 `launcher.update()` 后验证 loader report 被缓存、payload move-in 与 item 装载进入服务端主循环，且 `runtime.state.update_id == 1`。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_loader_from_launcher_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_payload_loader --lib`
  - `cargo test -p mindustry-core payload_unloader --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_item_transport_from_launcher_runtime --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：`PayloadMassDriver` 尚未接入 server aggregate；loader/unloader 已进入主循环，但真实 power graph、完整 block-specific `acceptItem/handleItem/acceptLiquid` override 与多段 complex transport/liquid graph 仍需继续迁移。

### 12.7 服务端 PayloadMassDriver 主循环接入

- 2026-05-26：`GameRuntimeOwnedPayloadFrameReport` 追加 `mass_driver: GameRuntimePayloadMassDriverFrameReport`，`advance_owned_runtime_blocks(...)` 的 payload 顺序目前为 constructor → source → conveyor/router → loader/unloader → mass-driver → deconstructor → void。
- `advance_owned_payload_mass_drivers(...)` 已拆出内部 `advance_owned_payload_mass_drivers_ticks(content, frame_delta)`；public 入口仍独立推进 frame/timing，server aggregate 复用 ticks，避免把 `GameState::advance_game_update_frame(...)` 与 building timing 重复推进。
- 新增测试 `server_update_drives_owned_payload_mass_driver_from_launcher_runtime`：构造 linked 双端 `payload-mass-driver`，源端预装 `BuildPayload(router)` 并置为已装载/已充能，目标端处于 accepting 且等待源端；调用 `launcher.update()` 后验证 fired/received report、源端清空、目标端收到 payload/effect delay，并保持 `runtime.state.update_id == 1`。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_mass_driver_from_launcher_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_advances_owned_payload_mass_driver_queues_and_fires_payload --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_loader_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_deconstructor_from_launcher_runtime --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：payload family 已全部进入当前 server aggregate 的最小主循环，但还缺跨多帧 end-to-end smoke（constructor/source → conveyor/router/loader/mass-driver/deconstructor/void）、UnitPayload 完整实体恢复、完整 renderer/UI 和更细的 Java 联机兼容验证。

### 12.8 服务端 payload aggregate 跨多帧整体 smoke

- 2026-05-26：新增 `server_update_drives_owned_payload_constructor_conveyor_void_chain`，在同一个 `ServerLauncher::update()` 主循环里构造 `constructor → payload-conveyor → payload-void` 链路，连续推进多个 server frame，验证 constructor 生产 `BuildPayload(router)`、输出到 conveyor、conveyor 再转交到 void 并被 void incinerate。
- 该测试专门锁定“已迁移模块必须接入整体 runtime 而不是孤立 helper”：每帧都断言 `runtime.state.update_id` 只增加 1，并跨帧累计 `constructor.produced_payloads / constructor.transferred_payloads / conveyor.transferred_payloads / void.incinerated_payloads`。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_constructor_conveyor_void_chain --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_mass_driver_from_launcher_runtime --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：还需要把 loader/deconstructor/mass-driver 也纳入更多跨多帧端到端 smoke，并继续补 UnitPayload 完整实体恢复、真实 renderer/UI、网络同步与 Java↔Rust 联机兼容验证。

### 12.9 服务端 PayloadLoader → PayloadDeconstructor 跨多帧 smoke

- 2026-05-26：新增 `server_update_drives_owned_payload_loader_deconstructor_chain`，在同一个 `ServerLauncher::update()` 主循环里构造 `payload-loader → small-deconstructor` 链路，loader 预装 `BuildPayload(router)` 并处于 exporting，连续推进多个 server frame 后验证 payload 被 loader 输出、deconstructor 接收并转入 deconstructing。
- 该测试覆盖 loader/unloader aggregate 与 deconstructor aggregate 的真实串联顺序，继续收紧“不要让迁移模块独立存在”的要求；每帧仍断言 `runtime.state.update_id == frame`，防止 public wrapper 被误串导致 frame/timing 翻倍。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_loader_deconstructor_chain --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_constructor_conveyor_void_chain --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：仍需补 `source/router`、linked `payload-mass-driver` 与 network snapshot 同步的跨多帧/联机 smoke，并继续迁移完整 UnitPayload、renderer/UI 与 Java 客户端互通细节。

### 12.10 服务端 PayloadSource → PayloadRouter → PayloadVoid 跨多帧 smoke

- 2026-05-26：新增 `server_update_drives_owned_payload_source_router_void_chain`，在同一个 server aggregate 中构造 `payload-source → payload-router → payload-void` 链路；source 配置为生成 `router` block payload，payload-router 配置同 block sort key，连续推进多个 `launcher.update()` 后验证 source 生成/转交、router 按匹配方向输出、void 最终 incinerate。
- 该测试覆盖 sandbox source、router sort key / `matches`、payload conveyor aggregate report 与 payload void 的组合运行；由于 payload-source 会持续生成 payload，测试按“至少一次完整流转”断言，避免把持续生产误判为失败。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_source_router_void_chain --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_constructor_conveyor_void_chain --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：linked `payload-mass-driver` 仍需自然多帧 charge/fire smoke；payload runtime 与 `network_world_data_template()`/Java 客户端可见 state 的端到端同步还需继续补。

### 12.11 服务端 world-data payload sidecar 端到端回读

- 2026-05-26：新增 `server_world_data_roundtrips_payload_loader_state_through_runtime_loader`，用 server `CaptureProvider` 捕获真实 `WORLD_STREAM`，经 `read_world_data(...)` 解出 `NetworkWorldData.map_snapshot`，再用全新的 `GameRuntime::load_network_map_with_buildings(...)` 回读，验证 `payload-loader` 的 `PayloadBlockBuild` common payload 与 `PayloadLoaderState.exporting` 能从服务端 runtime sidecar 进入 world stream 并恢复。
- 该闭环证明 payload 状态不只存在于 server runtime 单测，而是能通过现有 `network_world_data_template()` / `write_world_data(...)` / stream chunk / `read_world_data(...)` / map loader 链路成为客户端可见的 world-data 状态。
- 已验证：
  - `cargo test -p mindustry-server server_world_data_roundtrips_payload_loader_state_through_runtime_loader --lib`
  - `cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：还需扩展到 payload mass-driver/router/deconstructor 的 world-data roundtrip、真实 desktop client 接收应用 smoke，以及 Java↔Rust 联机兼容验证。

### 12.12 服务端 world-data 多类 Payload sidecar 回读

- 2026-05-26：新增 `server_world_data_roundtrips_payload_router_mass_driver_and_deconstructor_states`，在同一条 server `WORLD_STREAM` 中同时携带 `PayloadRouter`、`PayloadMassDriver`、`PayloadDeconstructor` 三类 sidecar，并用新 `GameRuntime::load_network_map_with_buildings(...)` 回读验证。
- 该测试覆盖：router 的 conveyor item、sorted key 与 `recDir`；mass-driver 的 `turretRotation/state/reloadCounter/charge/loaded/charging`；deconstructor 的 `progress/accum/deconstructing`。这把 payload 网络可见状态从 loader 单点推进到多类 payload building 组合。
- 已验证：
  - `cargo test -p mindustry-server server_world_data_roundtrips_payload_router_mass_driver_and_deconstructor_states --lib`
  - `cargo test -p mindustry-server server_world_data_roundtrips_payload_loader_state_through_runtime_loader --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：下一步建议接入 desktop/client `apply_network_world_data` 对 payload map snapshot 的恢复 smoke，随后继续补 Java 客户端互通测试。

### 12.13 Desktop/client world-data payload sidecar materialize

- 2026-05-26：新增 `desktop_launcher_materializes_payload_state_from_network_world_data`，让 desktop launcher 从 `NetClientState.last_loaded_world_data` 应用携带 `payload-loader` sidecar 的 `NetworkWorldData.map_snapshot`，并在 `DesktopLauncher::sync_runtime_state_from_world_data(...)` 中把 payload sidecar materialize 到客户端 `GameRuntime`。
- 该测试把上一轮 server world stream 回读继续推进到 desktop/client 应用路径，验证客户端 runtime 进入 `GameRuntimeNetworkContext::client()` 后能恢复 `PayloadBlockBuild` common payload 与 `PayloadLoaderState.exporting`。
- 已验证：
  - `cargo test -p mindustry-desktop desktop_launcher_materializes_payload_state_from_network_world_data --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_materializes_network_map_buildings_into_runtime --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `rustfmt --check desktop/src/lib.rs`
  - `git diff --check`
- 仍未完成：还需从真实 server stream 到 desktop net-client 的联机 smoke、Java 客户端互通验证，以及更多 payload 类型在 desktop materialize 后的运行/渲染状态测试。

### 12.14 真实 ServerLauncher → DesktopLauncher world-stream payload smoke

- 2026-05-26：新增 workspace 集成测试 `real_server_desktop_world_stream_materializes_payload_sidecar`，在 `tests` crate 中同时启动真实 `ServerLauncher` 与 `DesktopLauncher`，通过本地 `ArcNetProvider` TCP/UDP 握手发送真实 `ConnectPacket`，由服务端 `flush_pending_world_data()` 发出真实 `WORLD_STREAM`，再由客户端 `NetClient` 重组 `Streamable`、`read_world_data(...)`、自动发送 `ConnectConfirmCallPacket`，最后由 `DesktopLauncher::sync_runtime_state_from_world_data(...)` materialize payload sidecar。
- 同步修正 `ClientConnectConfig::default()`：默认 `uuid/usid` 不再为空，避免真实 wire path 中 `ConnectPacket::write_to(...)` 生成不可被服务端 Java-like reader/validation 接受的连接包。之前 capture-provider 单测不经真实序列化/反序列化，无法覆盖这个问题。
- 该闭环证明 payload-loader sidecar 已串过：
  - server runtime payload state
  - `network_world_data_template()` / `write_world_data(...)`
  - `ArcNetProvider` stream begin/chunk
  - `NetClient.last_loaded_world_data`
  - desktop runtime map loader
  - `GameRuntimeNetworkContext::client()`
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_world_stream_materializes_payload_sidecar --lib`
  - `cargo test -p mindustry-desktop desktop_run_connect_arg_starts_real_client_handshake --lib`
  - `cargo test -p mindustry-core update_sends_configured_connect_packet_once_after_connect_event --lib`
  - `cargo check -p mindustry-tests`
- 仍未完成：还需扩展到多 payload 类型的真实联机 world-stream smoke、state snapshot/实时增量同步、Java 客户端/服务端互通验证，以及 renderer/UI/输入控制闭环。

### 12.15 真实联机 world-stream 多类 Payload sidecar materialize

- 2026-05-26：新增 `real_server_desktop_world_stream_materializes_multiple_payload_sidecars`，复用真实 `ServerLauncher → ArcNetProvider → DesktopLauncher/NetClient → GameRuntime` 联机链路，在同一条 server world stream 中携带 `PayloadRouter`、`PayloadMassDriver`、`PayloadDeconstructor` 三类 sidecar，并在 desktop runtime 中验证全部 materialize。
- 测试覆盖字段：
  - `payload-router`：conveyor 中的 `BuildPayload(router)`、sorted block key、`recDir=2`、`matches=true`；
  - `payload-mass-driver`：`turretRotation=45`、`state=Shooting`、`reloadCounter=0.25`、`charge=0.5`、`loaded/charging=true`；
  - `small-deconstructor`：`progress=0.5`、`accum=[1,2]`、`deconstructing=BuildPayload(router)`。
- 同步把 `tests/src/lib.rs` 中真实联机 pump 与本地 TCP/UDP 端口探测抽成测试 helper，并将端口探测尝试次数提高到 128，降低 Windows 环境下偶发端口占用造成的 flaky。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_world_stream_materializes_multiple_payload_sidecars --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-tests`
  - `rustfmt --check tests/src/lib.rs`
  - `git diff --check`
- 仍未完成：后续需要继续补 state snapshot/实时增量同步、更多 gameplay runtime 闭环、Java 客户端/服务端互通 smoke、renderer/UI/输入控制与可游玩路径。

### 12.16 真实联机 StateSnapshot 增量同步 smoke

- 2026-05-26：新增 `real_server_desktop_state_snapshot_updates_runtime_after_world_stream`，先通过真实 `ServerLauncher → DesktopLauncher` world stream 完成 join 与 `ConnectConfirmCallPacket`，再由 `NetServer::send_state_snapshot(...)` 通过真实 `ArcNetProvider` 向客户端发送 `StateSnapshotCallPacket`。
- 测试验证客户端 `NetClient` 记录 `last_state_snapshot` / `last_state_snapshot_mirror`，`DesktopLauncher::sync_state_snapshot()` 随后把 wave、waveTime、enemy count、paused/gameOver、server TPS、rand seed、universe network seconds 应用到 `game_state` 与 `runtime.state`。
- 该闭环把迁移范围从初始 world-data load 推进到 world-stream 之后的真实运行态增量同步，继续收紧“整体可游玩客户端/服务端”目标。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_state_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-tests`
  - `rustfmt --check tests/src/lib.rs`
  - `git diff --check`
- 仍未完成：仍需补 entity/block/hidden snapshot 的真实联机增量同步、客户端输入/构建/单位状态回传、Java↔Rust 联机 smoke、renderer/UI 与完整可游玩路径。

### 12.17 真实联机 Entity/Hidden snapshot 增量同步 smoke

- 2026-05-26：新增 `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream`，在真实 world-stream join 后调用 `NetServer::send_entity_sync_snapshot(...)`，按 Java-like 顺序通过真实 `ArcNetProvider` 发送 `StateSnapshotCallPacket`、两个 `EntitySnapshotCallPacket` 和一个 `HiddenSnapshotCallPacket`。
- 测试验证：
  - 服务端记录 `state_snapshot_packets_sent=1`、`entity_snapshot_packets_sent=2`、`hidden_snapshot_packets_sent=1`；
  - 客户端 `NetClientState` 记录 `last_state_snapshot`、`entity_snapshot_packets_seen=2`、`last_entity_snapshot`、`hidden_snapshot_packets_seen=1`、`last_hidden_snapshot`；
  - desktop `game_state/runtime.state` 仍同步 state snapshot 中的 wave 与 TPS，并保持 `GameRuntimeNetworkContext::client()`。
- 该闭环进一步覆盖 world load 之后的低层 entity/visibility 增量包接收路径，为后续把 snapshot 数据真正落到 world/entity mirror 打基础。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-tests`
  - `rustfmt --check tests/src/lib.rs`
  - `git diff --check`
- 仍未完成：`BlockSnapshotCallPacket` 真实联机 smoke、entity snapshot 数据 materialize 到真实 world/entity mirror、客户端输入/构建回传、Java↔Rust 互通、renderer/UI/可游玩闭环仍需继续。

### 12.18 真实联机 BlockSnapshot 增量同步 smoke

- 2026-05-26：新增 `NetServer::send_block_snapshot(...)`，与 state/entity/hidden snapshot 发送 API 对齐，内部通过 `Net::send_to(..., PacketKind::BlockSnapshotCallPacket, false)` 走 Java-like unreliable snapshot 通道，并记录 `NetServerState.last_block_snapshot*` / `block_snapshot_packets_sent`。
- 新增 `real_server_desktop_block_snapshot_updates_net_client_after_world_stream`：先完成真实 world stream join，再由服务端发送 `BlockSnapshotCallPacket { amount, data }`，客户端 `NetClient` 记录 `last_block_snapshot`、`last_block_snapshot_at`、`block_snapshot_packets_seen` 与 `last_server_snapshot_at`。
- 该闭环补齐 state/entity/hidden 之后的 block snapshot 真实联机接收路径，后续可以继续把 block snapshot bytes materialize 到客户端 world/block mirror。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-server -p mindustry-tests`
  - `rustfmt --check core/src/mindustry/core/net_server.rs tests/src/lib.rs`
  - `git diff --check`
- 仍未完成：block/entity snapshot bytes 的实际 world/entity mirror 应用、客户端输入/构建回传、Java↔Rust 互通、renderer/UI/完整可游玩闭环仍需继续。

### 12.19 客户端 snapshot bytes 轻量镜像

- 2026-05-26：在 `NetClientState` 中新增 `last_block_snapshot_mirror`、`entity_snapshot_mirrors`、`last_hidden_snapshot_mirror`，把收到的 `BlockSnapshotCallPacket` / `EntitySnapshotCallPacket` / `HiddenSnapshotCallPacket` 从原始 packet 记录推进到可查询的轻量镜像。
- Block mirror 按 Java `NetServer.writeBlockSnapshots()` / `NetClient.blockSnapshot(...)` 的 header 结构解析：
  - `int tile_pos`
  - `short block_id`
  - 后续 `build.writeSync(...)` 暂存为 opaque `sync_bytes`
- Entity mirror 按 Java `NetServer.writeEntitySnapshot(...)` / `NetClient.readSyncEntity(...)` 的 header 结构解析：
  - `int entity_id`
  - `byte type_id`
  - 后续 `entity.writeSync(...)` 暂存为 opaque `sync_bytes`
- 因 Java snapshot 子记录没有独立长度，本闭环只安全拆分：
  - `amount == 1`：解析 header，剩余全部作为 `sync_bytes`；
  - `amount > 1` 且数据刚好为纯 header 长度：解析多条 header；
  - 其他多记录 opaque sync bytes 场景写入 `parse_error`，避免假装已经完成字段级 `readSync` 回放。
- 已扩展真实联机测试：
  - `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream` 断言 entity snapshot mirror 中的 `entity_id/type_id` 与 hidden ids；
  - `real_server_desktop_block_snapshot_updates_net_client_after_world_stream` 断言 block snapshot mirror 中的 `tile_pos/block_id/sync_bytes`。
- 已验证：
  - `rustfmt --check core/src/mindustry/core/net_client.rs tests/src/lib.rs`
  - `git diff --check`
  - `cargo test -p mindustry-core update_records_server_snapshots_when_client_loaded --lib`
  - `cargo test -p mindustry-core update_records_block_snapshot_metadata_for_later_world_application --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-tests`
- 仍未完成：需要继续把轻量镜像接入真实 client world/entity runtime，按具体 block/entity 类型补 Java `readSync/writeSync` 字段级解析与回放，并推进客户端输入、构建、单位控制回传和 Java↔Rust 互通 smoke。

### 12.20 客户端 snapshot mirror 接入 GameRuntime sidecar

- 2026-05-26：新增 `GameRuntimeClientBlockSnapshotRecord`、`GameRuntimeClientEntitySnapshotRecord` 与 `GameRuntimeClientSnapshotApplyReport`，并在 `GameRuntime` 中加入：
  - `client_block_snapshot_records`
  - `client_entity_snapshot_records`
  - `client_hidden_entity_ids`
- `DesktopLauncher::update()` 新增 `sync_snapshot_mirrors()`，在 world data / state snapshot 同步后，把 `NetClientState.last_block_snapshot_mirror`、`entity_snapshot_mirrors`、`last_hidden_snapshot_mirror` 应用到 `GameRuntime`。这样真实联机收到的 block/entity/hidden snapshot 不再只停留在 `NetClientState`，而是接入 client runtime sidecar。
- Block snapshot 应用按 Java `NetClient.blockSnapshot(...)` 的约束执行：
  - 只在 world 已加载且 `tile.build` 存在时应用；
  - 校验 `tile.build.block.id == block_id`；
  - 不把 snapshot 当成地图拓扑变更，不创建新建筑；
  - 暂存 `sync_bytes`，后续逐类实现 `Building.readSync(...)` 字段级回放。
- Entity/hidden snapshot 应用按 Java `Groups.sync` 语义的最小安全前置落地：
  - 先以 `entity_id -> { type_id, sync_bytes, hidden }` 形式进入 runtime sidecar；
  - hidden ids 会标记已有 entity mirror，并保留缺失 id 集合；
  - 目前不硬造真实 `UnitComp/Player/Bullet` 池，避免在 `EntityMapping` / `Groups.sync` 尚未完整迁移前伪装成已完成实体系统。
- 同步修复 `DesktopLauncher::sync_runtime_state_from_game_state()`：在把 `game_state` 克隆回 `runtime.state` 后，会重新 `sync_world_footprint_refs(...)`，避免 connect confirm 进入 Playing 时抹掉 runtime world 的 `BuildingRef`，导致后续 block snapshot 找不到 `tile.build`。
- 已验证：
  - `cargo test -p mindustry-core game_runtime_applies_client --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 仍未完成：block/entity 的 `sync_bytes` 仍是 opaque；下一步需要按具体 block `readSync(version)` 与 entity `readSync` 逐类解析，并接入真实 client entity pool / world building typed state。

### 12.21 BlockSnapshot 基础 `Building.readSync` 回放

- 2026-05-26：`GameRuntime::apply_client_block_snapshot_record(...)` 不再只是保存 block snapshot raw bytes；在 `tile.build` 存在且 block id 匹配后，会先用当前 `BuildingComp::read_base(...)` 回放 Java `Building.writeSync -> writeAll -> writeBase` 的基础段，更新客户端 runtime building 的 health、rotation、team、enabled/module/efficiency 等基础状态。
- `GameRuntimeClientSnapshotApplyReport` 新增：
  - `block_base_records_applied`
  - `block_base_read_errors`
  - `block_remaining_sync_bytes`
- 仍会保留完整 `sync_bytes` 到 `client_block_snapshot_records`，并把基础段后的剩余字节计入 `block_remaining_sync_bytes`，后续逐类接入 block-specific `read(read, revision)` / override `readSync(...)`。
- 已扩展真实联机测试：`real_server_desktop_block_snapshot_updates_net_client_after_world_stream` 现在让 server world stream 先携带真实 router building，再发送含 `BuildingComp::write_base(...)` 的 matching block snapshot，断言 desktop runtime 中该 building 的 health/rotation 被真实更新。
- 已验证：
  - `cargo test -p mindustry-core game_runtime_applies_client --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars --lib`
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 仍未完成：block-specific tail 仍未按具体类型和 revision 消费；turret 等 override `readSync(...)` 还需保持 Java 的“同步时保留 rotation/reload”特殊语义。

### 12.22 Conveyor BlockSnapshot child tail 回放

- 2026-05-26：新增 `GameRuntime::apply_client_block_snapshot_record_with_content(...)`，`DesktopLauncher::sync_snapshot_mirrors()` 改用该入口，使 block snapshot 在基础 `read_base` 后可以借助 `ContentLoader` 识别 block family。
- 当前 child tail 入口已从 conveyor 专用解析推进为 distribution dispatcher：用 `client_block_snapshot_revision(...)` 选择 Java sync revision，再复用 `read_distribution_runtime_state_from_building_payload(...)` 写入 `GameRuntime.distribution_runtime_states`。已验证范围仍以 `Conveyor | ArmoredConveyor` 为主，其他 distribution 子类沿用既有 reader 但仍需补专门回归。
- `GameRuntimeClientSnapshotApplyReport` 新增：
  - `block_child_records_applied`
  - `block_child_read_errors`
- 已验证：
  - `cargo test -p mindustry-core game_runtime_applies_client --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars --lib`
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 仍未完成：Router/Bridge/Duct/Sorter/Unloader 等 distribution 子类虽然进入 dispatcher，但仍需逐类真实联机/字段回归；storage/payload/turret 等 family 的 child tail 仍需接入；当前不会把未知 family tail 误消费。

### 12.24 Core BlockSnapshot child tail 回放

- 2026-05-26：`client_block_snapshot_revision(...)` 新增 `StorageBlockKind::Core -> revision 1`，`apply_client_block_snapshot_child_tail(...)` 在 distribution dispatcher 未匹配时继续尝试 `read_storage_runtime_state_from_building_payload(...)`。
- 当前 core snapshot child tail 可按 Java `CoreBuild.version()==1` 消费 `CoreBlock.write(...)` 的 `commandPos`，写入 `GameRuntime.storage_runtime_states`。
- 新增 `game_runtime_applies_client_core_snapshot_child_tail_with_content`，用 `BuildingComp::write_base(...) + write_core_state(...)` 构造 snapshot bytes，断言 `GameRuntimeStorageBlockState::Core.command_pos` 恢复。
- 已验证：
  - `cargo test -p mindustry-core game_runtime_applies_client --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 仍未完成：StorageBlock 普通 storage 无 child tail；core 的 shared item module / linked storage 语义仍依赖后续更完整 world/team runtime 同步。

### 12.25 Payload BlockSnapshot child tail 回放

- 2026-05-26：`apply_client_block_snapshot_child_tail(...)` 在 distribution 与 storage dispatcher 未消费 tail 后，继续尝试 payload dispatcher：`read_payload_runtime_state_from_building_payload(content, block, revision, ..., TopLevel)`。
- `client_block_snapshot_revision(...)` 现在为 Java `version()==1` 的 payload snapshot 类型返回 revision `1`：
  - `PayloadRouter`
  - `PayloadMassDriver`
  - `PayloadLoader/PayloadUnloader`
  - `PayloadSource`
  - `PayloadConveyor`、`PayloadDeconstructor`、`PayloadConstructor`、`PayloadVoid` 仍沿用 Java 默认 revision `0`。
- 成功解析时写入 `GameRuntime.payload_runtime_states`，保留完整 raw `sync_bytes` 到 `client_block_snapshot_records`；失败时只记 `block_child_read_errors`，未知类型不误消费 tail。
- 新增 client snapshot 端到端单测：
  - `game_runtime_applies_client_payload_conveyor_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_router_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_mass_driver_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_loader_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_source_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_deconstructor_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_constructor_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_void_snapshot_child_tail_with_content`
- 已验证：
  - `cargo test -p mindustry-core game_runtime_applies_client --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars --lib`
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `cargo check -p mindustry-core`
- 仍未完成：payload BlockSnapshot 的真实联机 smoke 已覆盖 `PayloadRouter`，但还需继续扩展到 `PayloadMassDriver/Loader/Source/Deconstructor/Constructor/Void`；UnitPayload 完整实体恢复仍需后续继续。

### 12.26 真实联机 PayloadRouter BlockSnapshot child tail smoke

- 2026-05-26：新增 `real_server_desktop_payload_block_snapshot_updates_runtime_after_world_stream`，在真实 `ServerLauncher -> ArcNetProvider -> DesktopLauncher/NetClient -> GameRuntime` 链路中先通过 world stream materialize 一个 `payload-router` building，再由服务端发送包含 `BuildingComp::write_base(...) + write_payload_conveyor_extra(...) + write_payload_router_extra(...)` 的 `BlockSnapshotCallPacket`。
- 测试断言：
  - `NetClient.last_block_snapshot_mirror` 正确解析 Java-like `tile_pos/block_id/sync_bytes`；
  - `desktop.runtime.client_block_snapshot_records` 保留完整 payload-router sync bytes；
  - `desktop.runtime.buildings()` 中 payload-router 的 health/rotation 被 `read_base` 更新；
  - `desktop.runtime.payload_runtime_states` 中生成 `GameRuntimePayloadBlockState::Router`，并恢复 `itemRotation/sorted/recDir/matches`。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_payload_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 仍未完成：真实联机 payload snapshot 目前覆盖 `PayloadRouter` 与 `PayloadMassDriver`；还需补 `PayloadLoader/Source/Deconstructor/Constructor/Void` 的真实 snapshot smoke，并继续推进 entity snapshot typed materialize。

### 12.27 真实联机 PayloadMassDriver BlockSnapshot child tail smoke

- 2026-05-26：新增 `real_server_desktop_payload_mass_driver_block_snapshot_updates_runtime_after_world_stream`，在真实 `ServerLauncher -> DesktopLauncher` 联机链路中先通过 world stream materialize 一个 `payload-mass-driver` building，再发送 `BuildingComp::write_base(...) + write_payload_block_build_common(...) + write_payload_mass_driver_extra(...)` 组成的 `BlockSnapshotCallPacket`。
- 测试覆盖 Java `PayloadDriverBuild.version()==1` 的 child tail 字段：`link/turretRotation/state/reloadCounter/charge/loaded/charging`，并同时验证 `PayloadBlockBuild` common 段的 `payVector/payRotation/payload/carried`。
- 测试断言：
  - `NetClient.last_block_snapshot_mirror` 能解析 payload-mass-driver snapshot header；
  - `desktop.runtime.client_block_snapshot_records` 保留完整 sync bytes；
  - 客户端 building 基础 health/rotation 由 `read_base` 更新；
  - `desktop.runtime.payload_runtime_states` 中恢复完整 `GameRuntimePayloadBlockState::MassDriver { common, driver }`。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_payload_mass_driver_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 仍未完成：`PayloadSource/Deconstructor/Constructor/Void` 的真实 BlockSnapshot smoke、entity snapshot typed runtime 与 Java↔Rust 更完整联机互通仍需继续。

### 12.28 真实联机 PayloadLoader BlockSnapshot child tail smoke

- 2026-05-26：新增 `real_server_desktop_payload_loader_block_snapshot_updates_runtime_after_world_stream`，在真实 server→desktop 联机链路中先通过 world stream materialize 一个 `payload-loader` building，再发送 `BuildingComp::write_base(...) + write_payload_block_build_common(...) + write_payload_loader_extra(...)` 组成的 `BlockSnapshotCallPacket`。
- 测试覆盖 Java `PayloadLoaderBuild.version()==1` 的 `exporting` 字段，以及继承自 `PayloadBlockBuild` 的 common tail。
- 测试断言：
  - `NetClient.last_block_snapshot_mirror` 正确解析 payload-loader snapshot header；
  - `client_block_snapshot_records` 保留 raw sync bytes；
  - 客户端 building 基础 health/rotation 被更新；
  - `payload_runtime_states` 中恢复 `GameRuntimePayloadBlockState::Loader { common, loader }`。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_payload_loader_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 仍未完成：真实 payload snapshot 尚缺 `PayloadDeconstructor/Constructor/Void`；后续应继续覆盖 revision 0 terminal payload/ref 分支。

### 12.29 真实联机 PayloadSource BlockSnapshot child tail smoke

- 2026-05-26：新增 `real_server_desktop_payload_source_block_snapshot_updates_runtime_after_world_stream`，在真实 server→desktop 联机链路中先通过 world stream materialize 一个 `payload-source` building，再发送 `BuildingComp::write_base(...) + write_payload_block_build_common(...) + write_payload_source_extra(...)` 组成的 `BlockSnapshotCallPacket`。
- 测试覆盖 Java `PayloadSourceBuild.version()==1` 的 `unit/configBlock/commandPos` 字段，以及继承自 `PayloadBlockBuild` 的 common tail。
- 测试断言：
  - `NetClient.last_block_snapshot_mirror` 正确解析 payload-source snapshot header；
  - `client_block_snapshot_records` 保留 raw sync bytes；
  - 客户端 building 基础 health/rotation 被更新；
  - `payload_runtime_states` 中恢复 `GameRuntimePayloadBlockState::Source { common, source }`，包含 `unit/config_block/command_pos/has_payload`。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_payload_source_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 仍未完成：真实 payload snapshot 尚缺 `PayloadConstructor/Void`；随后继续覆盖 revision 0 terminal common/recipe 分支，或开始 turret `readSync` 特例。

### 12.30 真实联机 PayloadDeconstructor BlockSnapshot child tail smoke

- 2026-05-26：新增 `real_server_desktop_payload_deconstructor_block_snapshot_updates_runtime_after_world_stream`，在真实 server→desktop 联机链路中先通过 world stream materialize 一个 `small-deconstructor` building，再发送 `BuildingComp::write_base(...) + write_payload_block_build_common(...) + write_deconstructor_extra(...) + write_payload_ref(...)` 组成的 `BlockSnapshotCallPacket`。
- 测试覆盖 Java `PayloadDeconstructorBuild` 默认 revision `0` 的 terminal tail：`progress/accum/deconstructing`，其中 `deconstructing` 使用 `PayloadRef::Block(router)` 并保留 build bytes，验证 top-level `read_payload_ref_to_end(...)` 分支。
- 测试断言：
  - `NetClient.last_block_snapshot_mirror` 正确解析 payload-deconstructor snapshot header；
  - `client_block_snapshot_records` 保留 raw sync bytes；
  - 客户端 building 基础 health 被更新；
  - `payload_runtime_states` 中恢复 `GameRuntimePayloadBlockState::Deconstructor { common, deconstructor }`，包含 `progress/accum/deconstructing`。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_payload_deconstructor_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 仍未完成：真实 payload snapshot 尚缺 `PayloadVoid`；entity snapshot typed runtime、turret `readSync` override 与 Java↔Rust 更完整互通仍需继续。

### 12.31 真实联机 PayloadConstructor BlockSnapshot child tail smoke

- 2026-05-26：新增 `real_server_desktop_payload_constructor_block_snapshot_updates_runtime_after_world_stream`，在真实 server→desktop 联机链路中先通过 world stream materialize 一个 `constructor` building，再发送 `BuildingComp::write_base(...) + write_payload_block_build_common(...) + write_block_producer_progress(...) + write_constructor_recipe(...)` 组成的 `BlockSnapshotCallPacket`。
- 测试覆盖 Java `BlockProducerBuild.write(...)` 的 `progress` 与 `ConstructorBuild.write(...)` 的 `recipe`，即 payload constructor revision 0 child tail。
- 测试断言：
  - `NetClient.last_block_snapshot_mirror` 正确解析 constructor snapshot header；
  - `client_block_snapshot_records` 保留 raw sync bytes；
  - 客户端 building 基础 health/rotation 被更新；
  - `payload_runtime_states` 中恢复 `GameRuntimePayloadBlockState::Constructor { common, producer, recipe }`。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_payload_constructor_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 仍未完成：payload family 的真实 BlockSnapshot smoke 当前已覆盖已迁移 reader 的主要类型；之后可转入 turret `readSync` override 或 entity snapshot typed runtime。

### 12.32 真实联机 PayloadVoid BlockSnapshot child tail smoke 与确认包等待修复

- 2026-05-26：新增 `real_server_desktop_payload_void_block_snapshot_updates_runtime_after_world_stream`，在真实 server→desktop 联机链路中先通过 world stream materialize 一个 `payload-void` building，再发送 `BuildingComp::write_base(...) + write_payload_block_build_common(...)` 组成的 `BlockSnapshotCallPacket`。
- 测试覆盖 Java `PayloadVoidBuild` 默认 revision `0` 的 terminal common tail，验证 `PayloadBlockBuild` common state 能恢复为 `GameRuntimePayloadBlockState::Void(common)`。
- 同时修复 `pump_real_server_desktop_until(...)` 的联机测试竞态：旧逻辑在客户端 `connect_confirm_sent` 且 world materialized 后就 break，服务端可能尚未处理 `ConnectConfirmCallPacket`；现在等待 `server.net_server.state().last_connect_confirm_connection_id.is_some()` 后才结束 pump，避免 tests crate 并发运行时偶发失败。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_payload_void_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 仍未完成：下一步建议转入 turret `readSync` override 或 entity snapshot typed runtime；payload UnitPayload 完整实体恢复仍需后续继续。

### 12.33 Turret BlockSnapshot `readSync` rotation/reload 保留

- 2026-05-26：`client_block_snapshot_revision(...)` 新增 turret family revision 映射：
  - `ItemTurret -> 2`
  - `ContinuousTurret/ContinuousLiquidTurret -> 3`
  - `PayloadAmmoTurret/LiquidTurret/PowerTurret/LaserTurret -> 1`
- `apply_client_block_snapshot_child_tail(...)` 在 distribution/storage/payload dispatcher 未消费 tail 后，继续尝试 `read_turret_runtime_state_from_building_payload(...)`，并写入 `GameRuntime.turret_runtime_states`。
- 对齐 Java `TurretBuild.readSync(...)` 特例：Java 会 `readAll(read, revision)` 后恢复旧 `rotation` 与 `reloadCounter`，避免客户端 turret snapping；Rust 现在在已有 turret runtime state 时，通过 `preserve_client_turret_sync_fields(...)` 保留旧 `TurretState.rotation/reload_counter`，同时更新 ammo/continuous 等其他 snapshot 字段。
- 新增测试 `game_runtime_applies_client_item_turret_snapshot_preserving_rotation_reload_with_content`：
  - 构造已有 `duo` turret runtime state，旧 `rotation/reload_counter`；
  - 发送包含新 `rotation/reload_counter` 与 item ammo 的 client snapshot bytes；
  - 断言 building base health/rotation 更新、ammo 更新，但 turret runtime 的 `rotation/reload_counter` 保留旧值。
- 已验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_item_turret_snapshot_preserving_rotation_reload_with_content --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 仍未完成：真实联机 turret BlockSnapshot smoke 尚未补；`Continuous/PayloadAmmo/Generic` turret 的 rotation/reload 保留还需逐类补测试；PointDefense/TractorBeam 不走 `TurretBuild.readSync` 的保留逻辑。

### 12.34 真实联机 ItemTurret BlockSnapshot `readSync` 保留 smoke

- 2026-05-26：新增 `real_server_desktop_item_turret_block_snapshot_preserves_rotation_reload_after_world_stream`，在真实 `ServerLauncher -> DesktopLauncher` 链路中验证 item turret snapshot。
- 测试流程：
  - 服务端 world stream 先 materialize 一个 `duo` building，并通过 map building payload 下发旧 `GameRuntimeTurretBlockState::Item`；
  - desktop runtime 确认已有旧 `TurretState.rotation/reload_counter`；
  - 服务端随后发送包含 `BuildingComp::write_base(...) + turret_write_child(...) + item_turret_write_ammo(...)` 的 `BlockSnapshotCallPacket`；
  - desktop 端断言 mirror/raw sidecar/base building 均更新，同时 `ammo/total_ammo` 接受 snapshot 新值，但 `rotation/reload_counter` 保留 world stream 后的旧值。
- 该 smoke 把 12.33 的 core 行为接入真实 net client/server packet 路径，覆盖 Java `TurretBuild.readSync(...)` 的关键客户端抗抖语义。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_item_turret_block_snapshot_preserves_rotation_reload_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：`Continuous/PayloadAmmo/Generic` turret 的 rotation/reload 保留还需逐类补 core/真实联机测试；PointDefense/TractorBeam 不走 `TurretBuild.readSync` 的保留逻辑。

### 12.35 Turret `readSync` 保留覆盖扩展到 Generic/Continuous/PayloadAmmo

- 2026-05-26：在 core runtime 层把 Java `TurretBuild.readSync(...)` 的旧 `rotation/reloadCounter` 保留语义，从 `ItemTurret` 扩展覆盖到另外三个 Rust runtime 变体：
  - `GameRuntimeTurretBlockState::Generic`：用 `arc`/PowerTurret 走真实 content + client BlockSnapshot reader；
  - `GameRuntimeTurretBlockState::Continuous`：用 `lustre`/ContinuousTurret 走真实 content + `continuous_turret_write_child(...)` reader；
  - `GameRuntimeTurretBlockState::PayloadAmmo`：用自定义 payload ammo turret block 走 payload reader + `preserve_client_turret_sync_fields(...)`，因为当前基础 content 尚未注册正式 `PayloadAmmoTurret`。
- 新增测试：
  - `game_runtime_applies_client_generic_turret_snapshot_preserving_rotation_reload_with_content`
  - `game_runtime_applies_client_continuous_turret_snapshot_preserving_rotation_reload_with_content`
  - `game_runtime_preserves_payload_ammo_turret_snapshot_rotation_reload_after_reading_payloads`
- 已验证：
  - `cargo test -p mindustry-core rotation_reload --lib`
  - `cargo test -p mindustry-core game_runtime_exports_turret_state_tail_in_network_map_snapshot --lib`
  - `cargo check -p mindustry-core`
- 仍未完成：`ContinuousLiquidTurret/LiquidTurret/LaserTurret` 可继续补同类 content-level 单测；真实联机 smoke 当前只覆盖 `ItemTurret`。

### 12.36 Turret `readSync` content-level 覆盖补齐 ContinuousLiquid/Liquid/Laser

- 2026-05-26：继续补齐 Java `TurretBuild.readSync(...)` 保留语义在剩余 content-level turret kind 上的覆盖：
  - `sublimate` / `ContinuousLiquidTurret`：走 `GameRuntimeTurretBlockState::Continuous` + revision 3；
  - `wave` / `LiquidTurret`：走 `GameRuntimeTurretBlockState::Generic` + revision 1；
  - `meltdown` / `LaserTurret`：走 `GameRuntimeTurretBlockState::Generic` + revision 1。
- 新增测试：
  - `game_runtime_applies_client_continuous_liquid_turret_snapshot_preserving_rotation_reload_with_content`
  - `game_runtime_applies_client_liquid_and_laser_turret_snapshots_preserving_rotation_reload_with_content`
- 至此 core content-level 已覆盖 `ItemTurret/PowerTurret/LiquidTurret/LaserTurret/ContinuousTurret/ContinuousLiquidTurret` 的 client BlockSnapshot 保留路径；`PayloadAmmoTurret` 仍使用自定义 block reader 覆盖，因为基础 content 暂无注册项。
- 已验证：
  - `cargo test -p mindustry-core rotation_reload --lib`
  - `cargo check -p mindustry-core`
- 仍未完成：真实联机 smoke 当前只覆盖 `ItemTurret`；如果继续沿 turret 方向推进，可补 `Continuous` 或 `Generic` 的 server→desktop smoke。

### 12.37 EntitySnapshot typed Unit runtime 初步接入

- 2026-05-26：在 raw entity snapshot sidecar 之外，新增 `UnitSyncWire -> UnitComp` 的 typed runtime 接入。
- Java 依据：
  - `NetClient.readSyncEntity(...)`：按 `id + typeID` 找/建 `Syncc`，执行 `entity.readSync(read)`，新建实体 `snapSync()` 后 `add()`；
  - `NetClient.entitySnapshot(...)`：遍历 snapshot records；
  - `NetClient.hiddenSnapshot(...)`：对已有 sync entity 调 `handleSyncHidden()`。
- Rust 行为：
  - `GameRuntime` 新增 `client_unit_snapshot_entities: BTreeMap<i32, UnitComp>`；
  - 新增 `apply_client_entity_snapshot_record_with_content(...)`，保留 raw sidecar，同时尝试用 `type_io::read_unit_sync(...)` 解析 `sync_bytes`；
  - 成功解析且 `UnitSyncWire.type_id` 能映射到 content unit 时，创建/更新 `UnitComp`，调用 `SyncComp::read_sync()`，新建时 `snap_sync()+add()`，之后 `after_sync()`；
  - `apply_client_hidden_snapshot_ids(...)` 对 typed unit 调 `handle_sync_hidden()`；
  - `DesktopLauncher::sync_snapshot_mirrors(...)` 改为传入 `ContentLoader`，使真实 net client mirror 能落到 typed unit runtime。
- 测试：
  - `game_runtime_applies_client_unit_entity_snapshot_to_typed_runtime_with_content`
  - 扩展 `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream`，真实 server→desktop entity snapshot 现在携带 `dagger` 的 Java-like `UnitSyncWire` bytes，并断言 desktop runtime materialize typed unit。
- 已验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_entity_snapshot_to_typed_runtime_with_content --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：entity `type_id` 到 Java `EntityMapping` 的精确 class-id 分发仍未完整建模；PlayerComp typed snapshot 与多 record 变长拆包仍需后续推进。

### 12.38 EntitySnapshot 多 UnitSync record 变长拆包 fallback

- 2026-05-26：继续对齐 Java `NetClient.entitySnapshot(...) -> readSyncEntity(...)` 的“一个 packet 内连续读取多个 `id + typeID + entity.readSync(...)` record”行为。
- 背景：`NetClient::decode_client_entity_snapshot_records(...)` 只有在 `amount == 1` 时能保留 opaque `sync_bytes`；多 record 且带变长 payload 时 mirror 层无法按固定 header 拆分，会给出 parse error。
- Rust 新增：
  - `GameRuntime::apply_client_entity_snapshot_packet_with_content(...)`
    - 输入 `amount + data`；
    - 按 `id(i32) + type_id(u8) + UnitSyncWire` 顺序连续读取；
    - 为每条 record 同时写 raw sidecar 与 typed `UnitComp`；
  - `DesktopLauncher::sync_snapshot_mirrors(...)` 在 mirror parse error 时尝试调用上述 runtime fallback；fallback 成功则不再只记录 parse error。
- 测试：
  - `game_runtime_applies_multi_unit_entity_snapshot_packet_with_content`
  - 扩展 `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream`：额外发送 `amount=2` 且包含两条 `UnitSyncWire` 的 entity snapshot；NetClient mirror 仍显示 parse error，但 Desktop runtime fallback materialize `1004/1005` 两个 typed unit。
- 已验证：
  - `cargo test -p mindustry-core game_runtime_applies_multi_unit_entity_snapshot_packet_with_content --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：fallback 当前只支持可按 `UnitSyncWire` 连续读通的 Unit records；完整 `EntityMapping` class-id registry 与混合实体 record 仍需后续迁移。

### 12.39 EntitySnapshot PlayerComp typed snapshot 初步接入

- 2026-05-26：补上 Java `NetClient.readSyncEntity(...)` 遇到本地 `player.id()` 时将 entity snapshot 读入 `player.readSync(...)` 的 Rust typed 接入。
- 关键纠偏：
  - Java `Player.writeSync/readSync` 不是 revisioned `Player.write/read` body；
  - sync payload 不带 `i16 revision`，也不包含 `@NoSync lastCommand`；
  - `@SyncLocal` 字段对本地玩家只消费 wire bytes，不覆盖本地输入/位置状态。
- Rust 新增：
  - `NetworkPlayerSyncData`：按 Java `Player.writeSync(...)` 顺序读写 `admin/boosting/color/mouse/name/selectedBlock/selectedRotation/shooting/team/typing/unit/x/y`；
  - `PlayerComp::apply_network_player_sync_data(...)`：支持本地玩家 `@SyncLocal` 保留语义，并把 incoming `unit` 交给后续 `after_sync_unit_state(...)`；
  - `GameRuntime.client_player_snapshot_entities`：保留 typed player snapshot sidecar；
  - `DesktopLauncher::sync_snapshot_mirrors(...)`：当 entity snapshot record 的 `entity_id == launcher.player.id` 时，解析 `NetworkPlayerSyncData`，更新真实 `launcher.player`，调用 `after_sync_unit_state(...)`，同时保留 raw sidecar。
- 测试：
  - `network_player_sync_data_round_trips_java_write_sync_shape`
  - `desktop_launcher_applies_local_player_entity_snapshot_to_typed_player_runtime`
- 已验证：
  - `cargo test -p mindustry-core network_player_sync_data_round_trips_java_write_sync_shape --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_local_player_entity_snapshot_to_typed_player_runtime --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：完整 `EntityMapping` class-id registry 与其他 `Syncc` 仍未迁移；`PlayerComp + UnitComp` 混合多 record 已由下一节补上，但还不是通用实体 dispatcher。

### 12.40 EntitySnapshot 混合 PlayerComp + UnitComp 多 record fallback

- 2026-05-26：在 `ClientEntitySnapshotMirror` 因多 record 变长 `sync_bytes` 无法固定拆分而产生 `parse_error` 时，`DesktopLauncher` 现在先尝试 mixed fallback。
- Java 依据补充：`annotations/src/main/resources/classids.properties` 中 `mindustry.entities.comp.PlayerComp=12`；当前 Rust 仍优先用 `entity_id == launcher.player.id` 落本地玩家副作用，后续 class-id registry 应把 `type_id == 12` 纳入通用分发。
- 新 fallback 行为：
  - 按 Java packet 顺序读取 `entity_id(i32) + type_id(u8)`；
  - 如果 `entity_id == launcher.player.id`，按 `NetworkPlayerSyncData::read_from(...)` 消费一条 `Player.writeSync` body，写 raw sidecar，更新 typed player runtime；
  - 其他 record 先按 `read_unit_sync(...)` 消费一条 `UnitSyncWire`，再复用 `GameRuntime::apply_client_entity_snapshot_record_with_content(...)` 写 raw sidecar 并 materialize typed `UnitComp`；
  - mixed fallback 成功时不再把该 packet 降级成纯 parse error。
- 测试：
  - `desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet`
- 已验证：
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-desktop --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：该路径仍是 `PlayerComp` 特判 + “其他先尝试 UnitSyncWire”的过渡方案；后续要迁移 Java `EntityMapping` class-id registry，用 `type_id` 分发所有已迁移 `Syncc`，避免靠 parse-shape 猜测。

### 12.41 真实联机 PlayerComp + UnitComp 混合 EntitySnapshot smoke

- 2026-05-26：扩展真实 `ServerLauncher -> DesktopLauncher` entity snapshot smoke，不再只验证 UnitSyncWire。
- 测试 `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream` 现在让服务端发送一个 `amount=3` 的变长 entity snapshot packet：
  - `connection_id + type_id 12 + NetworkPlayerSyncData`；
  - `1004 + type_id 2 + UnitSyncWire(dagger)`；
  - `1005 + type_id 2 + UnitSyncWire(flare)`。
- Rust 断言：
  - `NetClient` 仍把该多 record 变长 packet 记录为 mirror parse_error；
  - `DesktopLauncher` mixed fallback 能从真实 packet data 中拆出本地 player 与两个 unit；
  - `runtime.client_player_snapshot_entities[connection_id]` 保留 typed player snapshot；
  - `desktop.player` 更新 `name/admin/color/team/unit_ref`；
  - `runtime.client_unit_snapshot_entities[1004/1005]` 仍 materialize typed units；
  - raw `client_entity_snapshot_records` 同时保留 player 与 unit sync bytes。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：真实 smoke 仍只覆盖本地 player 与两种 vanilla unit；完整 Java `EntityMapping`、远端玩家实体组、其他 `Syncc` 仍待迁移。

### 12.42 Entity class-id registry 基线迁移

- 2026-05-26：将 Java `annotations/src/main/resources/classids.properties` 的 49 条实体 class-id 基线迁入 Rust。
- Rust 新增于现有文件 `core/src/mindustry/entities/mod.rs`：
  - `EntityClassIdEntry`
  - `ENTITY_CLASS_IDS`
  - `PLAYER_CLASS_ID`
  - `entity_class_id(name)`
  - `entity_class_name(id)`
- 已接入：
  - `DesktopLauncher` mixed fallback 对本地 player record 的解析现在要求 `type_id == PLAYER_CLASS_ID`；
  - desktop/unit/real smoke 测试里的 PlayerComp type id 不再硬编码 `12`，统一引用 `PLAYER_CLASS_ID`。
- 测试：
  - `entity_class_ids_match_upstream_classids_properties_baseline`
- 已验证：
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：registry 目前只是静态查询表；`GameRuntime`/`DesktopLauncher` 尚未全面按 class-id dispatcher 分发所有 `Syncc`。

### 12.43 Entity class-id kind 分类接入 mixed fallback

- 2026-05-26：在 class-id registry 上新增 `EntityClassKind::{Player, Unit, Other}` 与 `entity_class_kind(id)`。
- 分类规则：
  - `PLAYER_CLASS_ID` -> `Player`；
  - class-id 名称不含 `.` 的上游 unit/entity name -> `Unit`；
  - fully-qualified component/state name -> `Other`。
- `DesktopLauncher` mixed fallback 现在：
  - 本地 player 必须同时满足 `entity_id == player.id` 与 `type_id == PLAYER_CLASS_ID`；
  - 非 player record 只有在 `entity_class_kind(type_id) == Unit` 时才尝试 `UnitSyncWire`；
  - 其他 class-id 不再盲目按 UnitSyncWire 猜测，转为 parse error，等待对应 `Syncc` 迁移。
- 已验证：
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：`EntityClassKind::Other` 仍只有部分具体 dispatcher；Bullet/Weather/Effect 等 readSync wire 仍待迁移，FireComp 已由下一节补上。

### 12.44 FireComp EntitySnapshot typed runtime 接入

- 2026-05-26：迁移 Java `FireComp.writeSync/readSync` 的当前 wire 形状并接入 mixed entity snapshot dispatcher。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.FireComp=10`；
  - `annotations/src/main/resources/revisions/FireComp/1.json`：字段顺序 `lifetime, tile, time, x, y`；
  - 生成的 `writeSync/readSync` 不写 revision，读完调用 `afterSync()`，而 Java `FireComp.afterSync()` 调 `Fires.register(self())`。
- Rust 新增/变化：
  - `type_io::FireSyncWire`；
  - `type_io::write_fire_sync(...)` / `read_fire_sync(...)`；
  - `FireComp::apply_sync_wire(...)`：恢复 `lifetime/tile/time/x/y` 并调用 `after_sync()`；
  - `GameRuntime.client_fire_snapshot_entities`；
  - `GameRuntime::apply_client_fire_sync_wire(...)`；
  - hidden snapshot 对 typed fire 计为 existing（Java `handleSyncHidden()` 对 Fire 是空实现）；
  - `DesktopLauncher` 正常 single-record 与 mixed fallback 都能在 `EntityClassKind::Fire` 下 materialize typed fire。
- 测试：
  - `fire_sync_wire_roundtrips_java_write_sync_shape`
  - `fire_component_applies_sync_wire_and_registers_like_after_sync`
  - `game_runtime_applies_client_fire_entity_snapshot_to_typed_runtime`
  - 扩展 `desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet`，同 packet 现在覆盖 Player + Unit + Fire。
- 已验证：
  - `cargo test -p mindustry-core fire_sync --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_fire_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：Fire 目前落在 runtime typed sidecar，尚未与 `Fires` tile-indexed collection 完全统一；真实 server→desktop smoke 已由下一节补上。

### 12.45 真实联机 Fire EntitySnapshot smoke

- 2026-05-26：扩展 `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream`，真实 `ServerLauncher -> DesktopLauncher` mixed entity snapshot packet 现在包含 Fire record。
- 第三个 entity snapshot packet 当时为 `amount=4`：
  - 本地 player `NetworkPlayerSyncData`；
  - `1004` dagger `UnitSyncWire`；
  - `1005` flare `UnitSyncWire`；
  - `1006` Fire `FireSyncWire`。
- 测试断言：
  - `NetClient` mirror 层仍保留多 record 变长 packet 的 parse error；
  - `DesktopLauncher` mixed fallback 能在真实 packet data 中拆出 Fire；
  - `runtime.client_fire_snapshot_entities[1006]` materialize typed fire；
  - raw `client_entity_snapshot_records[1006]` 保留 `FIRE_CLASS_ID + fire_bytes`。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：Fire typed sidecar 尚未与 `Fires` 集合统一；Puddle 已由下一节继续补上，Weather/Effect/Bullet 等其他 entity class-id 仍待迁移。

### 12.46 PuddleComp EntitySnapshot typed runtime 与真实联机 smoke

- 2026-05-26：迁移 Java `PuddleComp.writeSync/readSync` 的当前 wire 形状并接入 single-record / mixed entity snapshot dispatcher。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.PuddleComp=13`；
  - `annotations/src/main/resources/revisions/PuddleComp/1.json`：字段顺序 `amount, liquid, tile, x, y`；
  - 生成的 `writeSync/readSync` 不写 revision；`afterSync()` 只有 `liquid != null` 时调用 `Puddles.register(self())`。
- Rust 新增/变化：
  - `type_io::PuddleSyncWire`；
  - `type_io::write_puddle_sync(...)` / `read_puddle_sync(...)`；
  - `EntityClassKind::Puddle` 与 `PUDDLE_CLASS_ID = 13`；
  - `PuddleComp::apply_sync_wire(...)`：恢复 `amount/liquid/tile/x/y` 并保持 Java 的非空 liquid 注册语义；
  - `GameRuntime.client_puddle_snapshot_entities`；
  - `GameRuntime::apply_client_puddle_sync_wire(...)`，通过 `ContentLoader::liquid(...)` 把 wire liquid id 映射为 `PuddleLiquidInfo`/`PuddleLiquid`；
  - hidden snapshot 对 typed puddle 计为 existing；
  - `DesktopLauncher` 正常 single-record 与 mixed fallback 都能在 `EntityClassKind::Puddle` 下 materialize typed puddle。
- 测试/真实 smoke：
  - `puddle_sync_wire_roundtrips_java_write_sync_shape`
  - `puddle_component_applies_sync_wire_and_registers_when_liquid_present`
  - `game_runtime_applies_client_puddle_entity_snapshot_to_typed_runtime`
  - 扩展 `desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet`，同 packet 现在覆盖 Player + Unit + Fire + Puddle；
  - 扩展 `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream`，真实 mixed packet 现在为 `amount=5`，新增 `1007 + PUDDLE_CLASS_ID + PuddleSyncWire`，断言 raw sidecar 与 `runtime.client_puddle_snapshot_entities[1007]`。
- 已验证：
  - `cargo test -p mindustry-core puddle_sync --lib`
  - `cargo test -p mindustry-core puddle_component_applies_sync_wire_and_registers_when_liquid_present --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_puddle_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：Puddle 目前落在 runtime typed sidecar，尚未与 `Puddles` tile-indexed collection 完全统一；`WeatherStateComp`、`EffectStateComp`、`BulletComp` 等其他 entity snapshot wire 仍待迁移。

### 12.47 WeatherStateComp EntitySnapshot typed runtime 与真实联机 smoke

- 2026-05-26：迁移 Java `Weather.WeatherStateComp.writeSync/readSync` 的当前 wire 形状并接入 single-record / mixed entity snapshot dispatcher。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.type.Weather.WeatherStateComp=14`；
  - `annotations/src/main/resources/revisions/WeatherStateComp/2.json`：字段顺序 `effectTimer, intensity, life, opacity, weather, windVector, x, y`；
  - `TypeIO.writeWeather/readWeather` 写 nullable `short` weather id；`TypeIO.writeVec2/readVec2` 写两个 `float`。
- Rust 新增/变化：
  - `type_io::WeatherStateSyncWire`；
  - `type_io::write_weather_state_sync(...)` / `read_weather_state_sync(...)`；
  - `EntityClassKind::Weather` 与 `WEATHER_STATE_CLASS_ID = 14`；
  - `WeatherState` 增加 sync 字段 `x/y` 并提供 `apply_sync_wire(...)`；
  - `ContentLoader::weather(...)` / `weather_by_name(...)` / `weathers(...)`；
  - `GameRuntime.client_weather_snapshot_entities`；
  - `GameRuntime::apply_client_weather_state_sync_wire(...)`，通过 `ContentLoader::weather(...)` 把 wire weather id 映射为 weather name；
  - hidden snapshot 对 typed weather 计为 existing；
  - `DesktopLauncher` 正常 single-record 与 mixed fallback 都能在 `EntityClassKind::Weather` 下 materialize typed weather state。
- 测试/真实 smoke：
  - `weather_state_sync_wire_roundtrips_java_write_sync_shape`
  - `weather_state_applies_sync_wire_and_restores_position_fields`
  - `game_runtime_applies_client_weather_entity_snapshot_to_typed_runtime`
  - `game_runtime_applies_weather_entity_snapshot_packet_with_content`
  - 扩展 `desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet`，同 packet 现在覆盖 Player + Unit + Fire + Puddle + Weather；
  - 扩展 `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream`，真实 mixed packet 现在为 `amount=6`，新增 `1008 + WEATHER_STATE_CLASS_ID + WeatherStateSyncWire`，断言 raw sidecar 与 `runtime.client_weather_snapshot_entities[1008]`。
- 已验证：
  - `cargo test -p mindustry-core weather_state_sync --lib`
  - `cargo test -p mindustry-core weather_state_applies_sync_wire_and_restores_position_fields --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_weather_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_applies_weather_entity_snapshot_packet_with_content --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：Weather 目前落在 runtime typed sidecar，尚未接入完整 `Groups.weather`、renderer/weather update 与客户端真实 weather lifecycle；`EffectStateComp` 已由下一节补上，`BulletComp` 等其他 entity snapshot wire 仍待迁移。

### 12.48 EffectStateComp EntitySnapshot typed runtime 与真实联机 smoke

- 2026-05-27：迁移 Java `EffectStateComp` 最新 revision 6 的 wire 形状并接入 single-record / mixed entity snapshot dispatcher。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.EffectStateComp=9`；
  - `annotations/src/main/resources/revisions/EffectStateComp/6.json`：字段顺序 `color, data, effect, lifetime, offsetPos, offsetRot, offsetX, offsetY, parent, rotWithParent, rotation, time, x, y`；
  - `TypeIO.writeColor/readColor` 写 `int rgba`；`TypeIO.writeObject/readObject` 写动态 object；`TypeIO.writeEffect/readEffect` 写 effect short id；`TypeIO.writePosEntity/readPosEntity` 写 parent entity id。
- Rust 新增/变化：
  - `type_io::EffectStateSyncWire`；
  - `type_io::write_effect_state_sync(...)` / `read_effect_state_sync(...)`；
  - `EntityClassKind::Effect` 与 `EFFECT_STATE_CLASS_ID = 9`；
  - `EffectStateComp` 扩展 `TypeValue data`、`effect_id`、`offset_pos/offset_rot/offset_x/offset_y`、`parent_id`、`rot_with_parent`；
  - `EffectStateComp::apply_sync_wire(...)` 按 Java sync 字段恢复状态；
  - `GameRuntime.client_effect_snapshot_entities`；
  - `GameRuntime::apply_client_effect_state_sync_wire(...)`；
  - hidden snapshot 对 typed effect 计为 existing；
  - `DesktopLauncher` 正常 single-record 与 mixed fallback 都能在 `EntityClassKind::Effect` 下 materialize typed effect state。
- 测试/真实 smoke：
  - `effect_state_sync_wire_roundtrips_java_write_sync_shape`
  - `effect_state_applies_sync_wire_fields_and_preserves_effect_clip`
  - `game_runtime_applies_client_effect_entity_snapshot_to_typed_runtime`
  - `game_runtime_applies_effect_entity_snapshot_packet_with_content`
  - 扩展 `desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet`，同 packet 现在覆盖 Player + Unit + Effect + Fire + Puddle + Weather；
  - 扩展 `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream`，真实 mixed packet 现在为 `amount=7`，新增 `1009 + EFFECT_STATE_CLASS_ID + EffectStateSyncWire`，断言 raw sidecar 与 `runtime.client_effect_snapshot_entities[1009]`。
- 已验证：
  - `cargo test -p mindustry-core effect_state_sync --lib`
  - `cargo test -p mindustry-core effect_state_applies_sync_wire_fields_and_preserves_effect_clip --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_effect_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_applies_effect_entity_snapshot_packet_with_content --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：Effect 目前落在 runtime typed sidecar，尚未接入完整 `EffectRegistry`、renderer/effect lifecycle 与服务端真实 entity group 枚举发包；`BulletComp` 等其他 entity snapshot wire 仍待迁移。

### 12.49 DecalComp EntitySnapshot typed runtime 与真实联机 smoke

- 2026-05-27：迁移 Java `DecalComp.writeSync/readSync` 的当前 wire 形状并接入 single-record / mixed entity snapshot dispatcher。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.DecalComp=8`；
  - `annotations/src/main/resources/revisions/DecalComp/0.json`：字段顺序 `color, lifetime, region, rotation, time, x, y`；
  - `TypeIO.writeColor/readColor` 写 `int rgba`；
  - `region` 类型是 `arc.graphics.g2d.TextureRegion`，上游 annotation serializer 没有对应 TypeIO，因此生成的 Java sync code 会跳过该字段、不产生 wire bytes。
- Rust 新增/变化：
  - `type_io::DecalSyncWire`；
  - `type_io::write_decal_sync(...)` / `read_decal_sync(...)`，严格按实际 wire bytes 写 `color, lifetime, rotation, time, x, y`；
  - `EntityClassKind::Decal` 与 `DECAL_CLASS_ID = 8`；
  - `DecalComp::apply_sync_wire(...)`：恢复 `color/lifetime/rotation/time/x/y`，并保留既有 `DecalRegion`；
  - `GameRuntime.client_decal_snapshot_entities`；
  - `GameRuntime::apply_client_decal_sync_wire(...)`；
  - hidden snapshot 对 typed decal 计为 existing；
  - `DesktopLauncher` 正常 single-record 与 mixed fallback 都能在 `EntityClassKind::Decal` 下 materialize typed decal。
- 测试/真实 smoke：
  - `decal_sync_wire_roundtrips_java_write_sync_shape`
  - `decal_component_applies_sync_wire_and_preserves_region`
  - `game_runtime_applies_client_decal_entity_snapshot_to_typed_runtime`
  - `game_runtime_applies_decal_entity_snapshot_packet_with_content`
  - 扩展 `desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet`，同 packet 现在覆盖 Player + Unit + Decal + Effect + Fire + Puddle + Weather；
  - 扩展 `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream`，真实 mixed packet 现在为 `amount=8`，新增 `1010 + DECAL_CLASS_ID + DecalSyncWire`，断言 raw sidecar 与 `runtime.client_decal_snapshot_entities[1010]`。
- 已验证：
  - `cargo test -p mindustry-core decal_sync --lib`
  - `cargo test -p mindustry-core decal_component --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_decal_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_applies_decal_entity_snapshot_packet_with_content --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：Decal 目前落在 runtime typed sidecar，尚未接入真实 renderer/texture atlas region lifecycle；`region` 因 Java sync 不传输，只能由创建端/渲染端保留或后续生命周期恢复；`BulletComp` 等其他 entity snapshot wire 仍待迁移。

### 12.50 BulletComp EntitySnapshot typed runtime 与真实联机 smoke

- 2026-05-27：迁移 Java `BulletComp.writeSync/readSync` 最新 revision 2 的 wire 形状并接入 single-record / mixed entity snapshot dispatcher。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.BulletComp=7`；
  - `annotations/src/main/resources/revisions/BulletComp/2.json`：字段顺序 `collided, damage, data, fdata, lifetime, owner, rotation, team, time, type, vel, x, y`；
  - `TypeIO.writeIntSeq/readIntSeq` 写 `int length + i32[]`；
  - `TypeIO.writeObject/readObject` 写动态 object；
  - `TypeIO.writeEntity/readEntity` 写 owner entity id；
  - `TypeIO.writeTeam/readTeam` 写 `u8` team id；
  - `TypeIO.writeBulletType/readBulletType` 写 `short` bullet content id；
  - `TypeIO.writeVec2/readVec2` 写两个 `float`。
- Rust 新增/变化：
  - `type_io::BulletSyncWire`；
  - `type_io::write_bullet_sync(...)` / `read_bullet_sync(...)`；
  - `EntityClassKind::Bullet` 与 `BULLET_CLASS_ID = 7`；
  - `BulletComp::apply_sync_wire(...)`：恢复 `collided/damage/data/fdata/lifetime/owner/rotation/team/time/type/vel/x/y`；
  - `GameRuntime.client_bullet_snapshot_entities`；
  - `GameRuntime::apply_client_bullet_sync_wire(...)`，通过 `ContentLoader::get_by_id(ContentType::Bullet, ...)` 校验 bullet content id；
  - hidden snapshot 对 typed bullet 计为 existing；
  - `DesktopLauncher` 正常 single-record 与 mixed fallback 都能在 `EntityClassKind::Bullet` 下 materialize typed bullet。
- 测试/真实 smoke：
  - `bullet_sync_wire_roundtrips_java_revision_2_write_sync_shape`
  - `bullet_component_applies_revision_2_sync_wire_fields`
  - `game_runtime_applies_client_bullet_entity_snapshot_to_typed_runtime`
  - `game_runtime_applies_bullet_entity_snapshot_packet_with_content`
  - 扩展 `desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet`，同 packet 现在覆盖 Player + Unit + Bullet + Decal + Effect + Fire + Puddle + Weather；
  - 扩展 `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream`，真实 mixed packet 现在为 `amount=9`，新增 `1011 + BULLET_CLASS_ID + BulletSyncWire`，断言 raw sidecar 与 `runtime.client_bullet_snapshot_entities[1011]`。
- 已验证：
  - `cargo test -p mindustry-core bullet_sync --lib`
  - `cargo test -p mindustry-core bullet_component_applies_revision_2_sync_wire_fields --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_bullet_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_applies_bullet_entity_snapshot_packet_with_content --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：Bullet 目前落在 runtime typed sidecar，尚未接入完整 `Groups.bullet` lifecycle、碰撞/渲染/服务端真实实体枚举发包；`Mover` 是 Java transient runtime 回调，不属于 snapshot wire，后续应在 bullet runtime 更新路径中单独迁移。

### 12.51 WorldLabelComp EntitySnapshot typed runtime 与真实联机 smoke

- 2026-05-27：迁移 Java `WorldLabelComp.writeSync/readSync` revision 1 的 wire 形状并接入 single-record / mixed entity snapshot dispatcher。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.WorldLabelComp=35`；
  - `annotations/src/main/resources/revisions/WorldLabelComp/1.json`：字段顺序 `flags, fontSize, parent, text, x, y, z`；
  - `flags` 写 `byte`，其中 `FLAG_ONLY_PARENT_VISIBLE = 32`；
  - `fontSize/x/y/z` 写 `float`；
  - `parent` 走 `TypeIO.writePosEntity/readPosEntity`，当前 snapshot wire 表现为 `i32 entity id`，空引用为 `-1`；
  - `text` 走 `TypeIO.writeString/readString`。
- Rust 新增/变化：
  - `type_io::WorldLabelSyncWire`；
  - `type_io::write_world_label_sync(...)` / `read_world_label_sync(...)`；
  - `EntityClassKind::WorldLabel` 与 `WORLD_LABEL_CLASS_ID = 35`；
  - `WorldLabelComp` 新增 `parent_id: Option<i32>` 与 `apply_sync_wire(...)`，恢复 `flags/font_size/parent_id/text/x/y/z`；
  - `GameRuntime.client_world_label_snapshot_entities`；
  - `GameRuntime::apply_client_world_label_sync_wire(...)`；
  - hidden snapshot 对 typed world-label 计为 existing；
  - `DesktopLauncher` 正常 single-record 与 mixed fallback 都能在 `EntityClassKind::WorldLabel` 下 materialize typed world-label。
- 测试/真实 smoke：
  - `world_label_sync_wire_roundtrips_java_revision_1_write_sync_shape`
  - `world_label_applies_revision_1_sync_wire_fields`
  - `game_runtime_applies_client_world_label_entity_snapshot_to_typed_runtime`
  - `game_runtime_applies_world_label_entity_snapshot_packet_with_content`
  - 扩展 `desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet`，同 packet 现在覆盖 Player + Unit + WorldLabel + Bullet + Decal + Effect + Fire + Puddle + Weather；
  - 扩展 `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream`，真实 mixed packet 现在为 `amount=10`，新增 `1012 + WORLD_LABEL_CLASS_ID + WorldLabelSyncWire`，断言 raw sidecar 与 `runtime.client_world_label_snapshot_entities[1012]`。
- 已验证：
  - `cargo test -p mindustry-core world_label_sync --lib`
  - `cargo test -p mindustry-core world_label_applies_revision_1_sync_wire_fields --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_world_label_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_applies_world_label_entity_snapshot_packet_with_content --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 仍未完成：WorldLabel 目前落在 runtime typed sidecar，尚未接入完整 label draw/lifecycle、父实体可见性过滤与服务端真实 entity group 枚举发包；后续继续迁移 `LaunchCoreComp` / `LocationPingComp` 等 entity snapshot wire，并收敛 dispatcher 重复 match。

### 12.52 LaunchCoreComp revision 0 runtime 接入

- 2026-05-27：迁移 Java `LaunchCoreComp` revision 0 字段形状，并把现有 Rust `LaunchCoreComp` helper 接入 `GameRuntime.launch_core_entities`。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.LaunchCoreComp=11`；
  - `annotations/src/main/resources/revisions/LaunchCoreComp/0.json`：字段顺序 `block, lifetime, time, x, y`；
  - `core/src/mindustry/entities/comp/LaunchCoreComp.java`：`@EntityDef(value = LaunchCorec.class, serialize = false)`，不是常规 `Syncc.writeSync/readSync` packet；仍需保留 revision 资源字段顺序供 runtime/import 兼容。
  - `block` 是 `mindustry.world.Block` 内容引用，wire 使用 `short block id`，空为 `-1`；其余字段为 `float`。
- Rust 新增/变化：
  - `type_io::LaunchCoreRevisionWire`；
  - `type_io::write_launch_core_revision(...)` / `read_launch_core_revision(...)`，写入 `short revision=0 + block id + lifetime/time/x/y`；
  - `entities::LAUNCH_CORE_CLASS_ID = 11` 并锁定 class id baseline；
  - `LaunchCoreComp` 新增 `block_id: Option<ContentId>`、`with_block_id(...)`、`apply_revision_wire(...)`、`to_revision_wire(...)`；
  - `LaunchCoreBlock::from_block_def(...)` 可从 content registry 的 `BlockDef` 恢复 size 与 atlas 占位尺寸，避免只停在手写几何数据；
  - `GameRuntime.launch_core_entities` 与 `GameRuntime::apply_launch_core_revision_wire(...)`，通过 `ContentLoader.block(id)` 将 revision 0 payload materialize 为 runtime entity。
- 测试：
  - `launch_core_revision_wire_roundtrips_java_revision_0_shape`
  - `launch_core_applies_revision_zero_wire_fields_and_block_ref`
  - `game_runtime_applies_launch_core_revision_zero_to_runtime_entity`
  - `entity_class_ids_match_upstream_classids_properties_baseline`
- 已验证：
  - `cargo test -p mindustry-core launch_core --lib`
  - `cargo test -p mindustry-core launch_core_revision --lib`
  - `cargo test -p mindustry-core game_runtime_applies_launch_core_revision_zero_to_runtime_entity --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo check -p mindustry-core`
- 仍未完成：LaunchCore 仍需接入真实 launch lifecycle / draw system / effect group；当前 `LaunchCoreBlock::from_block_def(...)` 对 fullIcon atlas 尺寸采用 content size 的占位推导，待 renderer/atlas 完整迁移后替换为真实 `TextureRegion` 尺寸与 `scl()`。

### 12.53 LocationPingComp class-id 与 Player ping runtime 行为

- 2026-05-27：核查 `LocationPingComp` 并推进实际玩家 ping runtime 行为。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.LocationPingComp=48`；
  - 参考仓库中未发现 `core/src/.../LocationPingComp.java`，也没有 `annotations/src/main/resources/revisions/LocationPingComp`；该 class-id 当前表现为保留/生成映射；
  - 实际位置 ping 行为位于 `core/src/mindustry/input/InputHandler.java::pingLocation(...)` 与 `PlayerComp.drawPing()`：
    - 同队可见时设置 `player.pingX/pingY/pingTime=1f/pingText`；
    - 文本为空则 `null`，超过 `Vars.maxPingTextLength` 则截断并追加 `...`；
    - `drawPing()` 使用 `pingDuration = 20f * 60f` 衰减，并按 `pow5Out/pow5In` 计算 alpha/缩放。
- Rust 新增/变化：
  - `entities::LOCATION_PING_CLASS_ID = 48` 并锁定 class-id baseline；
  - `PlayerComp::normalized_ping_text(...)`；
  - `PlayerComp::apply_ping_location(...)`；
  - `PlayerComp::advance_ping(...)` / `ping_alpha(...)` / `ping_draw_plan(...)`；
  - `PlayerPingDrawPlan` 记录 square/triangle/name/text 的 renderer 输入；
  - `input_handler::ping_location(...)` 复用 `PlayerComp` 的文本归一化和 runtime 写入，避免 helper 与实体状态分叉。
- 测试：
  - `ping_location_normalizes_text_and_draw_plan_follows_java_timing`
  - `ping_location_updates_visible_same_team_player_and_truncates_text`
  - `ping_location_rejects_admin_denied_as_validate_error`
  - `client_ping_location_packet_uses_client_payload_shape`
  - `entity_class_ids_match_upstream_classids_properties_baseline`
- 已验证：
  - `cargo test -p mindustry-core ping_location --lib`
  - `cargo test -p mindustry-core player --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo check -p mindustry-core`
- 仍未完成：位置 ping 的真实 renderer 绘制仍需接入 graphics/UI 层；`LocationPingComp` 没有可迁移源文件或 revision，后续若上游生成产物出现实体实现，应再补 typed/revision runtime。

### 12.54 PowerGraph runtime 与 updater 实体闭环

- 2026-05-27：将 `PowerGraphUpdaterComp` 从泛型转发壳推进到可驱动真实 Rust `PowerGraphRuntime` 的闭环。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.PowerGraphComp=41`、`mindustry.entities.comp.PowerGraphUpdaterComp=42`；
  - `core/src/mindustry/entities/comp/PowerGraphUpdaterComp.java`：`@EntityDef(value = PowerGraphUpdaterc.class, serialize = false, genio = false)`，字段 `transient PowerGraph graph`，`update(){ graph.update(); }`；
  - `annotations/src/main/resources/revisions/PowerGraphUpdaterComp/0.json`：`fields: []`；
  - 未发现 checked-in `PowerGraphComp.java`，实际电网行为在 `core/src/mindustry/world/blocks/power/PowerGraph.java`；
  - `PowerGraph.update()` 核心顺序：cheating consumer 快速置满 → 统计 produced/needed → 写 lastScaled/lastCapacity/lastStored/powerBalance → 用电池补差或充电 → distributePower 更新 consumer status。
- Rust 新增/变化：
  - `entities::POWER_GRAPH_CLASS_ID = 41`、`POWER_GRAPH_UPDATER_CLASS_ID = 42`；
  - `PowerProducer`、`PowerConsumer`、`PowerGraphRuntime`；
  - `PowerGraphRuntime::transfer_power(...)`、`power_balance(...)`、`has_power_balance_samples(...)`、`update_with_delta(...)`；
  - `PowerGraphUpdaterComp<PowerGraphRuntime>` 实现 `PowerGraphUpdate`，updater 可直接驱动真实 power graph runtime；
  - 保留既有 power helper 函数，并由 runtime 聚合成接近 Java `PowerGraph.update()` 的状态转移。
- 测试：
  - `power_graph_runtime_update_uses_batteries_and_updates_consumers`
  - `power_graph_runtime_update_charges_batteries_and_handles_cheating_consumers`
  - `power_graph_updater_drives_real_power_graph_runtime`
  - `power_graph_updater_forwards_update_to_graph`
  - `entity_class_ids_match_upstream_classids_properties_baseline`
- 已验证：
  - `cargo test -p mindustry-core power_graph_runtime --lib`
  - `cargo test -p mindustry-core power_graph_updater --lib`
  - `cargo test -p mindustry-core power_graph_beam_and_long_node_helpers_follow_upstream --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo check -p mindustry-core`
- 仍未完成：尚未迁移 Java `PowerGraph.addGraph/add/reflow/remove/clear` 的完整建图/并图/拆图流程，也未把 `BuildingComp.onProximityAdded/updatePowerGraph/powerGraphRemoved/afterPickedUp` 全量接入；当前闭环先覆盖 updater 实体驱动真实 runtime update。

### 12.55 PowerGraph membership/reflow 生命周期

- 2026-05-27：在上一节 `PowerGraphRuntime.update_with_delta(...)` 基础上继续迁移 Java `PowerGraph` 的 membership 管理行为。
- Java 依据：
  - `PowerGraph.add(Building)`：按 `outputsPower/consumesPower/consPower.buffered` 将 building 加入 `producers/consumers/batteries/all`；
  - `PowerGraph.clear()`：清空 `all/producers/consumers/batteries` 并移除 updater entity；
  - `PowerGraph.reflow(Building)`：从起点 BFS 遍历 `getPowerConnections(...)`，避免重复节点；
  - `PowerGraph.removeList(Building)`：测试用，分别从各列表移除 building。
- Rust 新增/变化：
  - `PowerGraphNode`：抽象 Java `Building + Block.consPower` 对 power graph 的输入视图；
  - `PowerGraphRuntime.all`、`producer_nodes`、`consumer_nodes`、`battery_nodes`；
  - `PowerGraphRuntime::add_node(...)`：按 Java 分类规则写入 producer/consumer/battery 列表；
  - `PowerGraphRuntime::remove_list(...)`、`clear(...)`、`add_graph(...)`；
  - `PowerGraphRuntime::reflow_from(...)`：以 caller 提供的 connection callback 执行 BFS membership 重建。
- 测试：
  - `power_graph_runtime_add_node_classifies_and_clear_matches_java_lists`
  - `power_graph_runtime_reflow_and_add_graph_follow_bfs_membership`
- 已验证：
  - `cargo test -p mindustry-core power_graph_runtime --lib`
  - `cargo test -p mindustry-core power_graph_updater --lib`
  - `cargo check -p mindustry-core`
- 仍未完成：`PowerGraph.remove(Building)` 的分支拆图和 `BuildingComp` proximity/pickup/remove/load 钩子尚未接入真实 world building；当前 `reflow_from(...)` 先提供可测试的 BFS membership 核心。

### 12.56 PowerGraph remove 分支拆图核心

- 2026-05-27：迁移 Java `PowerGraph.remove(Building)` 的核心分支拆图语义到纯 runtime。
- Java 依据：
  - `PowerGraph.remove(Building tile)` 遍历被移除 tile 的 power connections；
  - 对每个仍属于旧 graph 的邻接 building 新建 `PowerGraph`；
  - BFS 跳过被移除 tile，并把同一分支中的 connected building 加入新 graph；
  - 每个新 graph 创建后立即 `update()`，旧 graph updater entity 被移除。
- Rust 新增/变化：
  - `PowerGraphRuntime::remove_with_connections(...)`；
  - caller 提供 `connections(node_id) -> Vec<i32>` 与 `lookup(node_id) -> PowerGraphNode`，runtime 负责：
    - 按旧 graph membership 过滤；
    - 分支 BFS 去重；
    - 为每个分支创建新 `PowerGraphRuntime`；
    - 对新分支执行一次 `update_with_delta(1.0)`；
    - 清空旧 graph，表示旧 graph 已失效。
- 测试：
  - `power_graph_runtime_remove_with_connections_splits_branches_and_invalidates_old_graph`
- 已验证：
  - `cargo test -p mindustry-core power_graph_runtime --lib`
  - `cargo check -p mindustry-core`
- 仍未完成：分支拆图仍由 caller 提供连接/节点视图；下一步应把 `BuildingComp.getPowerConnections(...)`、proximity 与 pickup/remove 生命周期接到该方法。

### 12.57 BuildingComp power graph lifecycle 接入点

- 2026-05-27：把 Java `BuildingComp` 中与 power graph 相关的生命周期钩子先落到 Rust `BuildingComp`，为后续 GameRuntime/world 级接线准备稳定入口。
- Java 依据：
  - `BuildingComp.updatePowerGraph()`：遍历 `getPowerConnections(...)` 并合并 `other.power.graph.addGraph(power.graph)`；
  - `BuildingComp.powerGraphRemoved()`：调用 `power.graph.remove(self())`，并从所有 linked building 反向移除自身 link，随后清空 `power.links`；
  - `BuildingComp.afterPickedUp()`：为 power module 换新 graph，清空 links；非 buffered consumer 将 `power.status = 0f`。
- Rust 新增/变化：
  - `BuildingComp::power_graph_node(...)`：把 building/block/power module 状态转换为 `PowerGraphNode` 输入视图；
  - `BuildingComp::power_graph_removed_links(...)`：清空自身 `PowerModule.links` 并返回旧 links，供 GameRuntime 反向解除；
  - `BuildingComp::after_picked_up_power(...)`：清空 links、重置 init；非 buffered consumer 对齐 Java 将 status 置 0；
  - 将 `point2_pack` 移入 test import，减少非测试编译警告。
- 测试：
  - `building_component_exposes_power_graph_node_and_lifecycle_helpers`
- 已验证：
  - `cargo test -p mindustry-core building_component_exposes_power_graph_node_and_lifecycle_helpers --lib`
  - `cargo test -p mindustry-core building_component --lib`
  - `cargo check -p mindustry-core`
- 仍未完成：GameRuntime 还未把 proximity 链、linked building 反向清理和 `PowerGraphRuntime::remove_with_connections(...)` 串成真实 world/building 主流程。

### 12.58 GameRuntime power graph owner 与 building/proximity 主链路

- 2026-05-27：把上一节 `BuildingComp` power lifecycle helper 接入 `GameRuntime` 的真实 owned building/proximity/tick 主链路，避免 power graph 继续作为孤立算法模块存在。
- Java 依据：
  - `BuildingComp.onProximityAdded()` 调 `updatePowerGraph()`；
  - `BuildingComp.powerGraphRemoved()` 在移除前清 graph，并从 linked building 反向删 link；
  - `BuildingComp.getPowerConnections(...)` 合并同队 proximity power building 与 `PowerModule.links`；
  - `PowerGraph.update()` 每帧更新 consumer/battery `power.status`。
- Rust 新增/变化：
  - `GameRuntime` 新增 `power_graphs: Vec<PowerGraphRuntime>` 与 `power_graph_memberships: BTreeMap<i32, usize>`，作为 runtime 级 power graph owner；
  - `refresh_owned_building_proximity()` 收尾自动重建 owned power graph membership；
  - `remove_building_at_index(...)` 先调用 `BuildingComp::power_graph_removed_links()`，并从 linked building 反向移除被删 building 的 link；
  - 新增 `refresh_owned_power_graphs_with_content(...)` / `advance_owned_power_graphs(...)`，按 `BuildingComp::power_graph_node(...)` 物化 node，并把 `PowerGraphRuntime::update_with_delta(...)` 后的 consumer/battery status 回写到真实 `BuildingComp.power.status`；
  - `advance_owned_effect_blocks(...)` 与 `advance_owned_runtime_blocks(...)` 在 update permission 后进入 power graph update phase；
  - `load_network_map_with_buildings(...)` 在 proximity 刷新后用 content-aware power spec 重新 materialize graph；
  - `after_owned_building_picked_up_power(...)` 为后续真实 pickup 流程提供 GameRuntime 级入口。
- 测试：
  - `game_runtime_rebuilds_power_graphs_from_owned_building_proximity`
  - `game_runtime_power_remove_clears_links_and_rebuilds_graph_membership`
- 已验证：
  - `cargo test -p mindustry-core game_runtime_rebuilds_power_graphs_from_owned_building_proximity`
  - `cargo test -p mindustry-core game_runtime_power_remove_clears_links_and_rebuilds_graph_membership`
  - `cargo test -p mindustry-core power_graph`
  - `cargo test -p mindustry-core game_runtime_refreshes_owned_building_proximity_like_java_edges`
  - `cargo check --workspace`
- 注意：本轮额外跑了 `cargo test -p mindustry-core`，当前仍有 2 个既有 core storage state 断言失败（`storage_capacity` 被 runtime refresh 为 `4000`，测试期望默认 `0`）：`game_runtime_exports_core_storage_state_tail_in_network_map_snapshot`、`game_runtime_loads_core_storage_state_from_network_map_building_payload`。该失败与本轮 power graph 接线无直接交叉，后续应单独确认 core storage load 后是否应保留 wire 默认值还是立即刷新真实容量。
- 仍未完成：当前 `GameRuntime` 先采用粗粒度重建 graph，尚未把 Java 增量 `PowerGraph.addGraph/remove_with_connections` 的分支拆图作为默认路径；`PowerNode` autolink/UI config、diode 方向传输、动态 `ConsumePower.requestedPower(...)`、完整 generator 燃料/热/液体消耗与 `Groups.powerGraph` 实体调度仍需继续迁移。

### 12.59 PowerNode 手动连线配置入口

- 2026-05-27：继续迁移 Java `PowerNode.config(Integer.class, ...)` 的最小运行态入口，把“修改 `PowerModule.links`”从测试手写推进到 `GameRuntime` API。
- Java 依据：
  - 已有链接时：从双方 `power.links` 移除，并 reflow 两端图；
  - 未有链接时：经 `linkValid(...)` 校验，源端未满 `maxNodes` 才写入双方 links，并合并 power graph；
  - `linkValid(...)` 校验同队、目标有 power/connectedPower、`sameBlockConnection`、范围与目标 node `maxNodes`。
- Rust 新增/变化：
  - `GameRuntimePowerNodeLinkResult`；
  - `GameRuntime::configure_owned_power_node_link(...)`：支持单条 link toggle（linked/unlinked），写入/删除双方 `PowerModule.links`，并重建 owned power graph；
  - `owned_power_node_link_valid_between(...)` 复用 `power_node_link_valid(...)`，当前范围判定先用 tile-center 距离近似 Java hitbox/range；
  - `two_buildings_mut(...)`、`add_owned_power_link_pair(...)`、`remove_owned_power_link_pair(...)` 作为双向 link 写回工具。
- 测试：
  - `game_runtime_configures_power_node_link_and_reflows_graphs`
- 已验证：
  - `cargo test -p mindustry-core game_runtime_configures_power_node_link_and_reflows_graphs`
  - `cargo test -p mindustry-core power_node`
  - `cargo test -p mindustry-core power_graph`
- 仍未完成：`PowerNode.placed()` 的 autolink 候选扫描、`config(Point2[].class)` 批量重配、insulated raycast、非 center hitbox overlap、`getNodeLinks(...)` 和 client/UI tap 入口仍待迁移；diode 方向传输仍保持下一切片。

### 12.60 PowerNode autolink 候选扫描入口

- 2026-05-27：迁移 Java `PowerNode.placed() -> getPotentialLinks(...) -> configureAny(...)` 的最小 owned runtime 入口。
- Rust 新增/变化：
  - `GameRuntime::autolink_owned_power_node(...)`：按 source `PowerNode/LongPowerNode` 的 `autolink/maxNodes/laserRange` 扫描候选，并复用 `configure_owned_power_node_link(...)` 写回 links；
  - 候选过滤覆盖：同队、目标有 power、目标为 producer/consumer/node、非相邻、非同 graph、目标 node 未满、range 可达；
  - 候选排序按 Java 优先级：PowerNode 优先，再按距离升序；
  - `GameRuntimePowerNodeMetadata` 增加 `autolink` 字段。
- 测试：
  - `game_runtime_autolinks_power_node_candidates_like_java_priority`
- 已验证：
  - `cargo test -p mindustry-core game_runtime_autolinks_power_node_candidates_like_java_priority`
  - `cargo test -p mindustry-core power_node`
- 仍未完成：候选扫描仍使用 center tile 距离近似 Java hitbox overlap，`PowerNode.insulated(...)` 的 world raycast 阻断、`config(Point2[].class)` 批量重配、真实 placement 调用点和 UI tap/client packet 接入仍待迁移。

### 12.61 PowerDiode 图间方向传输

- 2026-05-27：迁移 Java `PowerDiodeBuild.updateTile()` 的最小 runtime 行为，把 diode 从纯公式 helper 接入 `GameRuntime` power phase。
- Java 依据：
  - `front()` / `back()` 两侧都存在、同队且有 power；
  - 两侧 power graph 不同时，按 back/front 电池存量比例计算 `amount`；
  - `backGraph.transferPower(-amount)`，`frontGraph.transferPower(amount)`。
- Rust 新增/变化：
  - `GameRuntime::advance_owned_power_diodes(...)`：扫描 owned `PowerDiode` building，按 rotation 找 front/back 邻居，通过 `power_graph_memberships` 定位两侧 graph；
  - 使用 `power_diode_transfer_amount(...)`、`PowerGraphRuntime::transfer_power(...)` 进行图间转移；
  - 转移后把 graph battery status 重新回写到真实 `BuildingComp.power.status`；
  - `advance_owned_power_graphs(...)` 在 graph update 后自动执行 diode phase。
- 测试：
  - `game_runtime_power_diode_transfers_between_front_and_back_graphs`
- 已验证：
  - `cargo test -p mindustry-core game_runtime_power_diode_transfers_between_front_and_back_graphs`
  - `cargo test -p mindustry-core power_diode`
  - `cargo test -p mindustry-core power_graph`
- 仍未完成：当前 diode 只覆盖相邻 front/back building 与 battery graph 转移；图内多建筑 battery 分配、`PowerDiode.bar(...)` UI、禁用/环境规则细节和与真实 `Groups.powerGraph` 调度顺序仍需继续对照。

### 12.62 PowerNode 批量配置与 placed autolink 入口

- 2026-05-27：继续迁移 Java `PowerNode.config(Point2[].class, ...)` 与 `PowerNodeBuild.placed()` 的运行态入口。
- Java 依据：
  - `config(Point2[].class)` 先逐条清理旧 links，再把相对 `Point2` 转换成绝对 tile pos 逐条走 `Integer` config；
  - `placed()` 在非客户端且当前 links 为空时自动调用 `getPotentialLinks(...)` 并 `configureAny(...)`。
- Rust 新增/变化：
  - `GameRuntimePowerNodeBatchLinkReport`；
  - `GameRuntime::configure_owned_power_node_relative_links(...)`：按 Java 顺序清旧链、应用相对 link 列表、记录 linked/cleared/rejected/missing；
  - `GameRuntime::placed_owned_power_node(...)`：服务端侧、空 links 时调用 `autolink_owned_power_node(...)`；
  - `point2_pack` 进入 runtime 主模块导入，用于相对 link 坐标转绝对 tile pos。
- 测试：
  - `game_runtime_reconfigures_power_node_relative_links_like_java_point_array`
  - `game_runtime_placed_power_node_autolinks_only_when_server_side_and_empty`
- 已验证：
  - `cargo test -p mindustry-core game_runtime_reconfigures_power_node_relative_links_like_java_point_array`
  - `cargo test -p mindustry-core game_runtime_placed_power_node_autolinks_only_when_server_side_and_empty`
  - `cargo test -p mindustry-core power_node`
- 仍未完成：真实 placement 调用点仍需从 build/placement 流程接入 `placed_owned_power_node(...)`；UI tap/client packet 路径、`PowerNode.insulated(...)` raycast、`getNodeLinks(...)` 辅助扫描仍待迁移。

### 12.63 PowerNode autolink 绝缘 raycast 阻断

- 2026-05-27：继续迁移 Java `PowerNode.insulated(...)` 在自动连线候选扫描中的运行态语义，避免 `PowerNode.placed()/getPotentialLinks(...)` 自动跨越绝缘墙/绝缘块连线。
- Java 依据：
  - `PowerNode.getPotentialLinks(...)` 与 `PowerNode.getNodeLinks(...)` 通过 `!PowerNode.insulated(tile, other.tile)` 过滤候选；
  - `PowerNode.insulated(int,int,int,int)` 直接调用 `World.raycast(...)`，上游 raycast 会访问起点、终点和中间点；
  - `BuildingComp.isInsulated()` 返回 `block.insulated`；
  - `PowerNode.config(Integer.class, ...)` / `linkValid(...)` 本身不调用 `insulated(...)`，所以 Rust 手动配置入口继续保持 Java-like 行为，不额外拒绝绝缘线段。
- Rust 新增/变化：
  - `GameRuntime::owned_power_line_insulated(...)`：复用已迁移的 `raycast_until(...)`，按 world footprint/build refs 从 source 到 target 扫描 tile；
  - `GameRuntime::owned_building_is_insulated(...)`：content-aware 读取 `DefenseWallData.insulated` 与 `PowerBlockData.insulated`，覆盖 plastanium wall、diode 等当前已迁移绝缘来源；
  - `autolink_owned_power_node(...)` 在候选排序和配置前过滤被绝缘建筑阻断的线段；
  - 手动 `configure_owned_power_node_link(...)` 未接入该过滤，保持 Java `linkValid(...)` 对照语义。
- 测试：
  - `game_runtime_power_line_insulated_matches_java_raycast_flags`
  - `game_runtime_autolink_skips_insulated_power_lines_but_manual_config_stays_java_like`
- 已验证：
  - `cargo test -p mindustry-core power_node`
  - `cargo test -p mindustry-core game_runtime_power_line_insulated_matches_java_raycast_flags`
  - `cargo test -p mindustry-core game_runtime_autolink_skips_insulated_power_lines_but_manual_config_stays_java_like`
  - `cargo check --workspace`
  - `git diff --check`
- 仍未完成：`PowerNode.getNodeLinks(...)` 尚未作为独立 runtime helper 暴露给非 node block placement/preview；UI tap/client packet 接入仍待迁移。

### 12.64 PowerNode range hitbox overlap 对齐

- 2026-05-27：把 `GameRuntime` 的 PowerNode 范围判定从 tile-center 距离近似改为 Java `PowerNode.overlaps(...)` 的 circle-vs-rect hitbox overlap。
- Java 依据：
  - `PowerNode.overlaps(src, other, range)` 使用 source 建筑坐标作为圆心、`laserRange * tilesize` 作为半径；
  - target 使用 `other.tile.worldx()/worldy() + other.block.offset` 作为矩形中心、`other.block.size * tilesize` 作为矩形宽高；
  - 因此大尺寸 target 即使中心点超过 `laserRange`，只要矩形 hitbox 与范围圆相交仍可连接；
  - Arc/libGDX `Intersector.overlaps(Circle, Rect)` 对最近点距离使用严格 `< radius^2`，刚好相切不算 overlap。
- Rust 新增/变化：
  - `owned_power_node_link_overlaps(...)` 改为双向调用 `owned_power_node_circle_overlaps_block(...)`；
  - 新增 `owned_building_center_tiles(...)` / `owned_building_rect_tiles(...)`，使用 `Block.offset` 和 `Block.size` 在 tile 单位下复刻 Java hitbox；
  - 保持目标也是 PowerNode 时的反向 range 判定。
- 测试：
  - `game_runtime_power_node_range_uses_java_circle_rect_overlap_for_large_targets`
  - `game_runtime_power_node_range_rejects_exact_circle_rect_tangent_like_java`
- 已验证：
  - `cargo test -p mindustry-core game_runtime_power_node_range_uses_java_circle_rect_overlap_for_large_targets`
  - `cargo test -p mindustry-core game_runtime_power_node_range_rejects_exact_circle_rect_tangent_like_java`
  - `cargo test -p mindustry-core power_node`
  - `cargo test -p mindustry-core game_runtime_power_line_insulated_matches_java_raycast_flags`
  - `cargo test -p mindustry-core game_runtime_autolink_skips_insulated_power_lines_but_manual_config_stays_java_like`
  - `cargo check --workspace`
  - `git diff --check`
- 仍未完成：source 建筑坐标当前仍以 owned building tile/offset 计算，未来如引入可移动/非 tile-aligned building 坐标，需要进一步对齐 Java `Building.x/y`；`getNodeLinks(...)`、UI tap/client packet 接入仍待迁移。

### 12.65 BuildingComp.placed 反向 PowerNode 自动连线

- 2026-05-27：迁移 Java `BuildingComp.placed() -> PowerNode.getNodeLinks(...) -> other.configureAny(pos())` 的 owned runtime 入口，让非 PowerNode 的耗电/产电建筑放置后由已有 PowerNode 反向自动连线。
- Java 依据：
  - `BuildingComp.placed()` 非客户端侧，在 `(block.consumesPower || block.outputsPower) && block.hasPower && block.connectedPower` 时调用 `PowerNode.getNodeLinks(...)`；
  - 回调中先检查 `!other.power.links.contains(pos())`，再对已有 PowerNode 调 `configureAny(pos())`，避免 `PowerNode.config(Integer)` 的 toggle 语义误断链；
  - `PowerNode.getNodeLinks(...)` 过滤 autolink、node 未满、范围 hitbox、同队、非重复 graph、非绝缘 raycast、非相邻建筑，并按距离排序。
- Rust 新增/变化：
  - `GameRuntimePlacedBuildingReport` 与 `GameRuntime::add_placed_building(...)`：提供“加入 owned world 后立即执行 placement power hooks”的集成入口，避免 placement hook 只停留为孤立 helper；
  - `GameRuntime::placed_owned_power_building(...)`：server-side 入口，对齐 Java `BuildingComp.placed()` 的 power 条件；
  - `GameRuntime::autolink_owned_power_nodes_to_building(...)`：扫描已有 PowerNode，复用 `configure_owned_power_node_link(...)` 写入双方 links 并重建 graph；
  - 候选去重维护 `used_graphs`，预先加入 target 自身 graph 与相邻 conducting graph，成功链接后加入 node graph；
  - 保留“已有 link 则跳过”，避免自动 placement 调用触发 toggle unlink；
  - 当前 Rust 内容层部分 block 尚未全量设置 Java 默认 `connectedPower=true`，本入口临时按 `has_power` 兼容 Java 默认语义，后续内容默认值总对齐时应回收该兼容。
- 测试：
  - `game_runtime_placed_power_building_autolinks_existing_nodes_like_java_get_node_links`
  - `game_runtime_placed_power_building_skips_adjacent_or_insulated_nodes_like_java_get_node_links`
  - `game_runtime_add_placed_building_runs_power_placement_hooks`
- 已验证：
  - `cargo test -p mindustry-core game_runtime_add_placed_building_runs_power_placement_hooks`
  - `cargo test -p mindustry-core game_runtime_placed_power_building_autolinks_existing_nodes_like_java_get_node_links`
  - `cargo test -p mindustry-core game_runtime_placed_power_building_skips_adjacent_or_insulated_nodes_like_java_get_node_links`
  - `cargo test -p mindustry-core power_node`
  - `cargo check --workspace`
  - `git diff --check`
- 仍未完成：真实 build placement/network packet 路径仍需统一切到 `add_placed_building(...)`；BeamNode 的 `getNodeLinks(...)`、PowerNode UI tap/client config packet、Java 默认 `Block.connectedPower/consumesPower` 内容层总对齐仍待迁移。

### 12.66 PowerNode 点击配置入口

- 2026-05-27：迁移 Java `PowerNodeBuild.onConfigureBuildTapped(...)` 的 runtime 语义，补上“点击合法目标 toggle 连线 / 双击自己自动找线或清线”的 owned `GameRuntime` 入口。
- Java 依据：
  - 点击合法目标时只检查 `linkValid(this, other)`，直接 `configure(other.pos())`，因此保留单条 link toggle 行为，且不额外检查 insulated；
  - 双击自己且 links 为空时走 `getPotentialLinks(...)` 收集候选，再 `configure(Point2[])` 批量配置；
  - 双击自己且 links 非空时 `configure(new Point2[0])` 清空全部链接；
  - 其他情况返回未处理，Java UI 层可继续走其它配置逻辑。
- Rust 新增/变化：
  - `GameRuntimePowerNodeTapResult`；
  - `GameRuntime::configure_tapped_owned_power_node(...)`：先尝试合法目标 toggle，失败且是 self tap 时按当前 links 空/非空分别 autolink 或 clear；
  - 复用 `configure_owned_power_node_link(...)`、`autolink_owned_power_node(...)` 与 `configure_owned_power_node_relative_links(...)`，避免生成独立的 tap-only link 状态。
- 测试：
  - `game_runtime_power_node_tap_toggles_valid_target_like_java_config_tap`
  - `game_runtime_power_node_double_tap_autolinks_when_empty_and_clears_when_linked`
- 已验证：
  - `cargo test -p mindustry-core game_runtime_power_node_tap_toggles_valid_target_like_java_config_tap`
  - `cargo test -p mindustry-core game_runtime_power_node_double_tap_autolinks_when_empty_and_clears_when_linked`
  - `cargo test -p mindustry-core power_node`
  - `cargo check --workspace`
  - `git diff --check`
- 仍未完成：`TileTapCallPacket` / `TileConfigCallPacket` 尚未把真实客户端点击事件切到该入口；返回值目前表达 runtime 结果，尚未映射 Java UI 的 consumed boolean/deselect 行为。

### 12.67 Block 电力默认元数据对齐

- 2026-05-27：对齐 Java `Block` 基类默认电力字段，减少 `GameRuntime` PowerNode 链路里的临时 `has_power` 兼容判断。
- Java 依据：
  - `Block.consumesPower` 默认 `true`；
  - `Block.outputsPower` 默认 `false`；
  - `Block.connectedPower` 默认 `true`。
- Rust 新增/变化：
  - `Block::new(...)` 默认 `consumes_power = true`、`connected_power = true`，保留 `outputs_power = false`；
  - `owned_power_node_link_valid_between(...)` 重新只信任 `target.block.connected_power`；
  - `owned_power_building_should_autolink_to_nodes(...)` 回收 `connected_power || has_power` 兼容分支；
  - `block_config_metadata_matches_upstream_defaults_and_helpers` 增加默认电力字段断言。
- 已验证：
  - `cargo test -p mindustry-core block_config_metadata_matches_upstream_defaults_and_helpers`
  - `cargo test -p mindustry-core block_`
  - `cargo test -p mindustry-core power_node`
  - `cargo test -p mindustry-core game_runtime_placed_power_building_autolinks_existing_nodes_like_java_get_node_links`
  - `cargo test -p mindustry-core game_runtime_add_placed_building_runs_power_placement_hooks`
  - `cargo check --workspace`
  - `git diff --check`
- 仍未完成：需要继续审计各具体 block 构造器中显式改写 `consumes_power/connected_power` 的地方，确认是否存在因早期默认值不一致而遗留的冗余赋值或漏设。

### 12.68 BeamNode 方向链接 runtime

- 2026-05-27：迁移 Java `BeamNodeBuild.updateDirections()` 的最小 owned runtime 链路，让 BeamNode 不再停留在纯公式 helper，而是能实际维护 `PowerModule.links` 并参与 `GameRuntime` power graph。
- Java 依据：
  - `BeamNodeBuild.updateDirections()` 逐方向扫描，距离范围为 `1 + size/2 .. range + size/2`；
  - 扫描遇到 `isInsulated()` building 立即停止；
  - 命中同队、`hasPower && connectedPower` 且不是 `PowerNode` 的 building 时，写入该方向 `links[i]` / `dests[i]`；
  - 方向目标变化时，先移除旧目标双方 `power.links` 并 reflow，再向新目标双方写入 `power.links` 并合并 graph。
- Rust 新增/变化：
  - `GameRuntime` 新增 `beam_node_links: BTreeMap<i32, [Option<i32>; 4]>` sidecar，用于保存每个 BeamNode 四方向上一轮目标，避免错误清理其它 BeamNode/PowerNode 写入的 reciprocal links；
  - `refresh_owned_beam_node_links(...)` 扫描所有 BeamNode，并在链接变化后刷新 proximity 与 content-aware power graph；
  - `advance_owned_power_graphs(...)` 开始时刷新 BeamNode links，使真实 power phase 能看到 BeamNode 方向链接；
  - 移除 building 与 reset sidecars 时同步清理 BeamNode sidecar。
- 测试：
  - `game_runtime_refreshes_beam_node_links_and_power_graphs_like_java_update_directions`
  - `game_runtime_beam_node_unlinks_when_insulated_wall_blocks_previous_direction`
- 已验证：
  - `cargo test -p mindustry-core game_runtime_refreshes_beam_node_links_and_power_graphs_like_java_update_directions`
  - `cargo test -p mindustry-core game_runtime_beam_node_unlinks_when_insulated_wall_blocks_previous_direction`
  - `cargo test -p mindustry-core beam_node`
  - `cargo test -p mindustry-core power_graph`
  - `cargo test -p mindustry-core game_runtime_power_diode_transfers_between_front_and_back_graphs`
  - `cargo check --workspace`
  - `git diff --check`
- 仍未完成：`BeamNode.getNodeLinks(...)` 的 placement preview/反向 placement 链路尚未迁移；BeamNode draw/dests 可见性、`world.tileChanges` 增量刷新条件与真实渲染选择规则仍待继续对照。

### 12.23 真实联机 Conveyor BlockSnapshot child tail smoke

- 2026-05-26：扩展 `real_server_desktop_block_snapshot_updates_net_client_after_world_stream`，真实 `ServerLauncher -> DesktopLauncher` world stream 先 materialize 一个 `conveyor` building，再由服务端发送包含 `BuildingComp::write_base(...) + write_conveyor_state(...)` 的 `BlockSnapshotCallPacket`。
- 测试现在断言：
  - `NetClient` 仍能解析 Java-like `tile_pos/block_id/sync_bytes`；
  - `desktop.runtime.client_block_snapshot_records` 保留完整 sync bytes；
  - `desktop.runtime.buildings()` 中对应 conveyor 的 health/rotation 被 `read_base` 更新；
  - `desktop.runtime.distribution_runtime_states` 中对应 tile 生成 `GameRuntimeDistributionBlockState::Conveyor`，并恢复 conveyor item。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-tests`
  - `git diff --check`
- 仍未完成：真实联机 child tail 目前只覆盖 conveyor；其他 distribution/payload/storage/turret family 仍需后续扩展。

### 12.69 Core storage capacity 网络状态回归对齐

- 2026-05-27：修正 core storage 网络状态 roundtrip 测试期望，明确 `storageCapacity` 是 Java `CoreBuild.onProximityUpdate()` 派生运行态容量，不是 `CoreBuild.write(...)` 持久化字段。
- Java 依据：
  - `CoreBuild.write(...)` 只在 `super.write(...)` 后写 `commandPos`；
  - `CoreBuild.read(...)` 只恢复 `commandPos`；
  - `CoreBuild.onProximityUpdate()` 中按自身 `itemCapacity`、邻接 linked storage 与同队其它 core 重新计算并同步 `storageCapacity`。
- Rust 新增/变化：
  - 保持 `write_core_state(...)` / `read_core_state(...)` 只序列化 `command_pos`；
  - 保持 `GameRuntime::load_network_map_with_buildings(...)` 后刷新 owned storage core links/capacity；
  - 更新 `game_runtime_exports_core_storage_state_tail_in_network_map_snapshot` 与 `game_runtime_loads_core_storage_state_from_network_map_building_payload`，期望加载后 core sidecar 中的 `storage_capacity` 已刷新为该 core block 的 `item_capacity`。
- 验证：
  - `cargo test -p mindustry-core game_runtime_exports_core_storage_state_tail_in_network_map_snapshot`
  - `cargo test -p mindustry-core game_runtime_loads_core_storage_state_from_network_map_building_payload`
  - `cargo test -p mindustry-core storage`
- 仍未完成：多 core + linked storage 的 Java `onProximityUpdate()` 容量广播与 campaign delta 已有局部覆盖，后续仍需扩展到真实 world load/placement/removal 全链路 smoke。

### 12.70 BeamNode 生命周期重建接入

- 2026-05-27：把 BeamNode 派生方向链接从“仅 power graph tick 时刷新”推进到放置/地图载入生命周期，避免新放置或网络地图读入后必须等下一帧才形成 `PowerModule.links`。
- Java 依据：
  - `BeamNodeBuild.updateTile()` 在 `lastChange != world.tileChanges` 时调用 `updateDirections()`；
  - `updateDirections()` 派生 `links[]/dests[]`，不写入 block 自身序列化；
  - `Block.drawPotentialLinks(...)` 会用 `BeamNode.getNodeLinks(...)` 做放置预览，说明 BeamNode 链路是运行时/渲染可见的派生状态。
- Rust 新增/变化：
  - `GameRuntimePlacedBuildingReport` 新增 `beam_node_links`；
  - `GameRuntimeMapLoadReport` 新增 `beam_node_links`；
  - `add_placed_building(...)` 在 PowerNode placed hooks 之后调用 `refresh_owned_beam_node_links(...)`；
  - `load_network_map_with_buildings(...)` 在 proximity 刷新后、最终 power graph/storage 刷新前重建 BeamNode links。
- 测试：
  - `game_runtime_add_placed_building_refreshes_beam_node_links_like_java_update_directions`
  - `game_runtime_load_network_map_rebuilds_beam_node_links_before_first_frame`
- 仍未完成：BeamNode `drawPlace/getNodeLinks` 的纯预览 API、真实绘制层消费 `beam_node_links`、以及 `world.tileChanges` dirty-bit 跳过重复刷新仍待继续迁移。

### 12.71 BeamNode 放置预览候选枚举

- 2026-05-27：迁移 Java `Block.drawPotentialLinks(...) -> BeamNode.getNodeLinks(...) -> BeamNodeBuild.couldConnect(...)` 的非渲染候选枚举部分，为后续真实绘制层消费提供 runtime API。
- Java 依据：
  - `Block.drawPotentialLinks(...)` 对 power block 放置预览同时枚举 `PowerNode` 与 `BeamNode`；
  - `BeamNode.getNodeLinks(...)` 对目标方块四个方向各取最近可连接 BeamNode；
  - `BeamNodeBuild.couldConnect(...)` 从 BeamNode 沿方向扫描，遇到绝缘墙或同队 powered/connected building 即失败，扫到目标 block 矩形则成功。
- Rust 新增/变化：
  - `GameRuntime::owned_beam_node_potential_links_for_block(...)`：按目标方块/队伍返回最多四个 BeamNode 候选 tile pos，顺序对应 Java 方向枚举；
  - `owned_beam_node_could_connect_to_block(...)` 复用 `beam_node_could_connect_scan_range(...)` 与 `beam_node_within_target_rect(...)`，并保留 Java 的 `maxRange = 30` 预览上限。
- 测试：
  - `game_runtime_beam_node_potential_links_match_block_draw_potential_links`
  - `game_runtime_beam_node_potential_links_stop_at_insulated_wall_like_java_could_connect`
- 仍未完成：当前 API 只输出候选，不直接绘制；后续需要让 desktop/render placement preview 与 BeamNode laser draw 消费这些候选/`beam_node_links`。

### 12.72 Desktop 本地 Player entity snapshot 计数去重

- 2026-05-27：修正 `DesktopLauncher::sync_snapshot_mirrors(...)` 在 NetClient 已解析 player typed record 后，又把同一条本地 player snapshot 应用到本地 `PlayerComp` 时重复累计 `entity_typed_records_applied` 的问题。
- Java/语义依据：
  - 本地 `Player.readSync` 仍需消费 `@SyncLocal` 字段但不覆盖本地输入/位置；
  - 同一条 entity snapshot record 应只作为一条 typed entity 记录计数，额外同步到本地 `PlayerComp` 是 desktop local mirror 的消费副作用，不应让 report 变成 2。
- Rust 新增/变化：
  - `sync_snapshot_mirrors(...)` 保留对 `GameRuntime::apply_client_entity_snapshot_record_with_content(...)` 的 typed runtime sidecar 应用；
  - 对 `apply_client_player_entity_snapshot(...)` 的本地玩家应用增加 `!runtime_typed_applied` 计数门控，避免重复计数；
  - 保持 mixed fallback 路径原语义不变：那里先只写 raw sidecar，再由 typed 分支累计 1。
- 验证：
  - `cargo test -p mindustry-desktop desktop_launcher_applies_local_player_entity_snapshot_to_typed_player_runtime`
  - `cargo test -p mindustry-desktop`
  - `cargo test --workspace`
  - `git diff --check`
- 仍未完成：Desktop 快照路径仍需继续扩展到更多真实 Java entity 子类与本地/远端玩家切换 smoke。

### 12.73 其他玩家 preview build plans overlay 消费入口

- 2026-05-27：迁移 Java `PlayerComp.getPreviewPlans()` 与 `InputHandler.drawOtherBuildPlans()` 的非渲染消费语义，提供 Rust input/runtime 可复用的轻量 overlay plan builder，避免 preview build plans 只停留在网络字段或孤立 sidecar。
- Java 依据：
  - `PlayerComp.getPreviewPlans()` 在 preview group delay 后把 assembling plans commit 到 current preview plans；
  - `InputHandler.drawOtherBuildPlans()` 对本地玩家或不同队玩家清空 preview plans；
  - `drawOtherBuildPlans()` 在 `previewPlansDirty` 时重建 preview tree，并在鼠标 hover 其他玩家建造预览时给出玩家名/位置/选中反馈；
  - `BuildPlan.animScale` 按 `Mathf.lerpDelta(animScale, 1f, 0.2f)` 推进。
- Rust 新增/变化：
  - `OtherPlayerPreviewOverlayFrame` 描述本地玩家、队伍、时间、delta 与鼠标世界坐标；
  - `other_player_preview_overlay_plan(...)` 调用 `PlayerComp::get_preview_plans(...)`，按 Java 规则提交 delayed preview group、清理本地/敌队 plans、消费 dirty 标记；
  - 新增 `OtherPlayerPreviewOverlayPlan` / `EntryPlan` / `OverlapPlan`，输出 world position、alpha、hover/overlap 与玩家位置，供后续 desktop/render 层消费；
  - block 几何通过 `block_lookup` 注入，保持与真实 content registry 接线点分离，后续应由 desktop/runtime block registry 提供 `size/offset`。
- 验证：
  - `cargo test -p mindustry-core preview_overlay_plan_commits_only_after_preview_group_delay`
  - `cargo test -p mindustry-core preview_overlay_plan_clears_local_or_enemy_preview_plans_like_java_draw_other_build_plans`
  - `cargo test -p mindustry-core input_handler`
- 仍未完成：当前切片只生成 overlay plan，不执行真实 renderer 绘制；后续需要让 desktop/input 持有远端 `PlayerComp` 列表并由 content registry 驱动 `block_lookup`，再把 overlay entries 接入真实预览/hover UI。

### 12.74 PowerNode TileConfig 网络入口接入 server runtime

- 2026-05-27：把 Java `PowerNode` 的 `config(Integer.class)` / `config(Point2[].class)` 语义从纯 runtime helper 推进到 Rust server 网络事件消费链路，让客户端发来的 `TileConfigCallPacket` 能驱动服务端 owned `PowerModule.links`。
- Java 依据：
  - `PowerNodeBuild.onConfigureBuildTapped(...)` 对有效目标调用 `configure(other.pos())`，最终经 `TileConfigCallPacket` 发送绝对 packed tile pos；
  - 双击自身时，空链接会生成 `Point2[]` 相对链接数组，已有链接则发送空 `Point2[]` 清空；
  - `PowerNode` 的 `Integer` 配置负责 toggle 单条 link，`Point2[]` 配置负责先清旧链接再按相对坐标重建。
- Rust 新增/变化：
  - `GameRuntimePowerNodeConfigResult` 描述 `Int` 单 link、`Point2Array` 批量重建、缺失/非 PowerNode/不支持值等结果；
  - `GameRuntime::configure_owned_power_node_value(...)` 将 `TypeValue::Int` 映射到 `configure_owned_power_node_link(...)`，将 `TypeValue::Point2Array` 映射到 `configure_owned_power_node_relative_links(...)`；
  - `ServerLauncher::update()` 现在会消费新增 `NetServerState.events` 中的 `TileConfigCallPacket`，并把 PowerNode 配置应用到真实 server `GameRuntime`；
  - `ServerLauncher` 保留最近一次 PowerNode config 结果与 seen/changed 计数，便于后续联机 smoke 与调试。
- 验证：
  - `cargo test -p mindustry-core game_runtime_power_node_config_value_accepts_java_integer_and_point_array`
  - `cargo test -p mindustry-server server_update_applies_power_node_tile_config_packets_to_runtime`
- 仍未完成：这里只接入了 `TileConfigCallPacket` 对 PowerNode link 的服务端状态变更；后续还需要把变更广播/rollback、desktop 侧配置 UI、以及 `TileTapCallPacket` 的通用 TapEvent/插件事件语义继续对照 Java。

### 12.75 Desktop 远端玩家 preview overlay cache

- 2026-05-27：把 12.73 的 `other_player_preview_overlay_plan(...)` 接入 desktop 侧真实 `NetClient -> GameRuntime -> PlayerComp` 数据链，开始形成可供 renderer/UI 消费的其他玩家建造预览 overlay cache。
- Java 依据：
  - `Groups.player` 中远端 `Player` 持有 `previewPlans`，`InputHandler.drawOtherBuildPlans()` 每帧遍历同队非本地玩家；
  - `ClientPlanSnapshotReceivedCallPacket` 携带 `player/group/plans`，客户端需要按玩家累积 preview group；
  - player snapshot 提供远端玩家 name/team/position，preview overlay hover 使用这些字段。
- Rust 新增/变化：
  - `NetClientState` 新增 `client_plan_snapshot_received_packets` 历史缓存，避免 desktop 只看到最后一个 preview packet；
  - `DesktopLauncher` 新增 `remote_players: BTreeMap<i32, PlayerComp>`，从 `GameRuntime.client_player_snapshot_entities` materialize 远端玩家镜像，不覆盖本地玩家；
  - `DesktopLauncher::sync_remote_preview_plan_packets(...)` 按 `ClientPlanSnapshotReceivedCallPacket.player_id` 调用 `NetClient::apply_received_preview_plans_to_player(...)`；
  - `DesktopLauncher::rebuild_other_player_preview_overlays_at(...)` 通过真实 content registry 提供 block `size/offset`，生成 `other_player_preview_overlays`；
  - world reset / cursor reset 时同步清理远端玩家、overlay cache 与 preview packet cursor。
- 验证：
  - `cargo test -p mindustry-desktop desktop_launcher_builds_remote_player_preview_overlay_from_snapshot_and_plan_packets`
  - `cargo test -p mindustry-core update_records_received_preview_plan_packets_and_applies_to_player`
- 仍未完成：当前 cache 已可消费但还没有真实 renderer/UI 绘制；远端玩家生命周期仍依赖 snapshot/hidden sidecar 的后续完善，真实多人多 chunk plan 的端到端 server->desktop smoke 仍需继续补。

### 12.76 Server 转发客户端 preview plan snapshot

- 2026-05-27：把 Java `NetServer.clientPlanSnapshot(...)` / `clientPlanSnapshotSend(...)` 的联机转发语义接入 Rust server launcher，使客户端发来的 preview build plan chunk 不再只停留在 `NetServerState` 记录里，而是会转成 `ClientPlanSnapshotReceivedCallPacket` 发给其他连接。
- Java 依据：
  - `clientPlanSnapshot(Player player, int groupId, ClientBuildPlans plans)` 先调用 `player.handlePreviewPlans(groupId, plans)` 维护玩家预览组；
  - `clientPlanSnapshotSend(...)` 遍历同队其他在线玩家，并向非本人、非 local、连接可用的目标发送 `Call.clientPlanSnapshotReceived(...)`；
  - 该 remote call 为 low priority / unreliable，允许按 Java chunk 原样透传。
- Rust 新增/变化：
  - `ServerLauncher::apply_new_network_server_events(...)` 新增 `ClientPlanSnapshotCallPacket` 分支；
  - 当前以 source `connection_id` 作为临时 `player_id`，把 `group_id` 与 `plans` 原样封装为 `ClientPlanSnapshotReceivedCallPacket`；
  - 目标连接来自 `NetServerState.connection_states`，先排除发送者、已踢出和已断开的连接，再复用 `NetServer::send_client_plan_snapshot_received_to_many(...)` 以非可靠方式发送；
  - `plans: None` 也会转发，用于对齐 Java 空预览/清理语义；
  - 转发错误会写入 `ServerLauncher.network_error`，便于后续端到端 smoke 定位。
- 验证：
  - `cargo test -p mindustry-server server_update_forwards_client_plan_snapshot_to_other_connections`
  - `cargo test -p mindustry-server`
- 仍未完成：当前最小闭环尚未按真实 `Player/team` 做同队过滤，也未把 `connection_id -> player entity id` 的绑定替换为真实 player id；下一步应补 server 侧玩家 preview 状态维护（`PlayerComp.handlePreviewPlans`/组延迟/周期广播）与 Rust server->desktop 多客户端 smoke。

### 12.77 Server preview 玩家状态与同队过滤

- 2026-05-27：把 12.76 的无状态粗转发继续推进到 server launcher 的玩家 preview 状态维护与同队在线过滤，避免 preview snapshot 被广播给异队或半连接对象。
- Java 依据：
  - `NetServer.clientPlanSnapshot(...)` 会先让发送方 `Player` 调用 `handlePreviewPlans(groupId, plans)`；
  - `clientPlanSnapshotSend(...)` 只向同队、非本人、非 local、连接可用的玩家发送 `clientPlanSnapshotReceived`；
  - `PlayerComp.handlePreviewPlans(...)` 维护 group 单调递增、assembling/current 分离与延迟 commit。
- Rust 新增/变化：
  - `ServerLauncher` 新增 `server_preview_players: BTreeMap<i32, PlayerComp>`，按连接 id 维护 server 侧 preview player sidecar；
  - `apply_server_preview_plan_packet(...)` 将 `BuildPlanWire` 转为 `BuildPlan`，调用既有 `PlayerComp::handle_preview_plans(...)`，并保留 connection/team/name/locale 到 `PlayerComp.con`/字段；
  - `ClientPlanSnapshotCallPacket` 转发目标现在要求：目标不是发送者、team 与 source 一致、`has_connected && player_added && !kicked && !has_disconnected`；
  - `plans: None` 仍进入玩家状态，用于开启新 group 并清空 assembling，保持 Java 空 preview 清理语义。
- 验证：
  - `cargo test -p mindustry-server server_update_forwards_client_plan_snapshot_to_other_connections`
- 仍未完成：server preview sidecar 仍以 `connection_id` 作为临时 player id；还需要接入真实 player entity id、周期性 `planPreviewSyncTime` 广播、以及多客户端 server->desktop smoke。

### 12.78 Real server -> desktop preview snapshot smoke

- 2026-05-27：补充真实 `ServerLauncher` + 真实 `DesktopLauncher` 的 preview snapshot smoke，确认 server 侧收到 `ClientPlanSnapshotCallPacket` 后，通过真实网络发送的 `ClientPlanSnapshotReceivedCallPacket` 会被 desktop `NetClient` 接收，并进入 `remote_players`/overlay cache。
- Rust 新增/变化：
  - `tests/src/lib.rs` 新增 `real_server_desktop_preview_snapshot_forwarding_updates_remote_player_cache_after_world_stream`；
  - 测试先让真实 desktop 完成 world stream / connect confirm，再在 server state 中插入一个同队、已加入的模拟 source connection；
  - 通过 `Net::handle_server_received_from_connection(...)` 注入 source preview packet，让 `ServerLauncher::update()` 走真实 server event 消费与 `send_client_plan_snapshot_received_to_many(...)`；
  - target desktop 通过真实 `ArcNetProvider` 收包，`DesktopLauncher::update()` 将 packet 应用到 `remote_players`，最后用 content registry 重建 `other_player_preview_overlays`。
- 验证：
  - `cargo test -p mindustry-tests real_server_desktop_preview_snapshot_forwarding_updates_remote_player_cache_after_world_stream -- --test-threads=1`
- 仍未完成：该 smoke 避免了双真实 desktop 同时握手带来的端口/握手不稳定，source 端仍是模拟连接；后续在连接身份配置稳定后，应补双真实 Rust desktop 或 Java↔Rust preview smoke。

### 12.79 Server preview 周期广播

- 2026-05-27：将 12.76/12.77 的“收到客户端 preview snapshot 后立即转发”修正为更接近 Java `NetServer.planPreviewSyncTime` 的 server 周期广播模型。
- Java 依据：
  - `planPreviewSyncTime = Timekeeper.ofSeconds(0.5f)`，server update 中约每 500ms 遍历 `Groups.player`；
  - 每个玩家广播前执行 `++player.lastPreviewPlanGroupServer`，再读取 `player.getPreviewPlans()`；
  - 空 preview 也调用 `clientPlanSnapshotSend(player, id, null)`，用于让同队其他客户端清理旧预览；
  - `clientPlanSnapshotSend(...)` 只发给同队、非本人、非 local、连接在线的玩家，过长 plans 按 chunk 拆分。
- Rust 新增/变化：
  - `ServerLauncher` 新增 `next_server_preview_broadcast_at` 与 `server_preview_broadcasts_sent`；
  - `ServerLauncher::update()` 在网络事件应用后按 `PLAN_PREVIEW_SYNC_INTERVAL_MS` 节流调用 `broadcast_server_preview_plans_if_due(...)`；
  - `ClientPlanSnapshotCallPacket` 现在只更新 server 侧 `PlayerComp` preview sidecar，不再即时转发；
  - `broadcast_server_preview_plans(...)` 会先同步所有 ready `connection_states` 到 `server_preview_players`，再调用 `PlayerComp::get_preview_plans(now_millis)`，通过 `NetServer::broadcast_client_plan_previews(...)` 发送同队 preview/null chunk；
  - ready 但尚未发送过 preview 的玩家也会参与首次周期广播，保持 Java “空 plans 发 null”语义。
- 验证：
  - `cargo test -p mindustry-server server_update_records_client_plan_snapshot_and_broadcasts_preview_to_teammates`
  - `cargo test -p mindustry-server server_preview_due_broadcast_syncs_empty_ready_players`
  - `cargo test -p mindustry-tests real_server_desktop_preview_snapshot_forwarding_updates_remote_player_cache_after_world_stream -- --test-threads=1`
- 仍未完成：当前 server preview sidecar 仍以 `connection_id` 作为临时 player id；真实 source 仍未替换成完整 player entity 绑定；双真实 Rust desktop 与 Java↔Rust preview 联机 smoke 仍需继续补。

### 12.80 NetClient snapshot record 索引

- 2026-05-27：补强 Java `NetClient.entitySnapshot(...)` / `blockSnapshot(...)` 的 Rust 客户端镜像层，把已解析出的 record mirror 继续索引到 `NetClientState`，便于 desktop/runtime 后续按 entity id / tile pos 查询最新 snapshot record。
- Java 依据：
  - entity snapshot 的 `data` 每条记录为 `int entityId`、`byte typeId`、后续 `entity.writeSync(...)` 可变长 payload；
  - block snapshot 的 `data` 每条记录为 `int tilePos`、`short blockId`、后续 `build.writeSync(...)` 可变长 payload；
  - hidden snapshot 会让客户端隐藏/移除对应实体镜像，因此 Rust 的 entity record cache 也应清掉对应 id。
- Rust 新增/变化：
  - `NetClientState` 新增 `block_snapshot_record_mirrors: BTreeMap<i32, ClientBlockSnapshotRecordMirror>`，按 `tile_pos` 保存最新 block snapshot record；
  - `NetClientState` 新增 `entity_snapshot_record_mirrors: BTreeMap<i32, ClientEntitySnapshotRecordMirror>`，按 `entity_id` 保存最新 entity snapshot record；
  - `EntitySnapshotCallPacket` / `BlockSnapshotCallPacket` 的真实收包分支在保留 raw packet 与 packet-level mirror 的同时，写入上述索引；
  - `HiddenSnapshotCallPacket` 会从 entity record 索引中移除对应 id，避免后续 runtime 消费 stale entity。
- 验证：
  - `cargo test -p mindustry-core update_records_block_snapshot_metadata_for_later_world_application`
  - `cargo test -p mindustry-core snapshot_mirrors_split_header_only_multi_records_and_report_opaque_payloads`
  - `cargo test -p mindustry-core update_records_server_snapshots_when_client_loaded`
  - `cargo test -p mindustry-core hidden_snapshot_removes_indexed_entity_snapshot_records`
- 仍未完成：多 record snapshot 中每条 `writeSync(...)` 是变长 payload，当前只有 `amount == 1` 时能无歧义保留 payload；`amount > 1` 且携带 opaque payload 时仍需等完整 entity/building typed decoder 后才能精确切分并回放到 runtime。

### 12.81 Item MassDriver in-flight 几何侧车

- 2026-05-27：补强普通 item `MassDriver` 的 runtime-only in-flight shot，使其不只记录倒计时和物品，还保存 Java `MassDriverBolt` 所需的发射/目标坐标、当前位置、旋转、速度、travel/lifetime 计时。
- Java 依据：
  - `MassDriverBuild.fire(...)` 会从源端扣除 `DriverBulletData.items[]`，按当前旋转和 `translation` 创建 `MassDriverBolt`；
  - `MassDriverBolt.update(...)` 依据 from/to/bullet position 判断命中或继续飞行，`despawned()/hit()` 使用 bullet rotation 做掉落方向和动态爆炸。
- Rust 新增/变化：
  - `GameRuntimeItemMassDriverInFlight` 新增 `from_x/from_y/to_x/to_y/x/y/rotation/speed/travel_ticks/elapsed_ticks/lifetime_ticks`；
  - `item_mass_driver_fire_to_link(...)` 生成 in-flight shot 时使用 mass-driver `translation` 和源/目标建筑坐标初始化几何数据；
  - `advance_item_mass_driver_in_flight_ticks(...)` 每 tick 推进 `elapsed_ticks`、`remaining_ticks` 和线性当前位置，仍保持“到达后才交付”的现有 runtime 语义；
  - target-lost despawn 事件现在把 shot rotation 传给 `MassDriverBolt::despawn_drop_plans(...)`，为后续真实掉落方向/爆炸复现保留接线点；
  - 原测试改名为 `game_runtime_item_mass_driver_sends_items_in_flight_and_delivers_after_delay`，并断言发射时位置在源端 translation、飞行 tick 期间位置推进且未提前交付。
- 验证：
  - `cargo test -p mindustry-core game_runtime_item_mass_driver_sends_items_in_flight_and_delivers_after_delay`
  - `cargo test -p mindustry-core game_runtime_item_mass_driver_keeps_flight_after_target_removed_until_lifetime`
  - `cargo test -p mindustry-core mass_driver_bolt_intersection_drops_and_explosion_stats_are_pure`
- 仍未完成：当前仍是 runtime-only 线性飞行侧车，还未生成真实 ECS bullet entity，也未完全接入 `MassDriverBolt::update_plan(...)` 的相交/越界命中判定、effects/sound/shake 和随机掉落。

### 12.82 Storage linkedCore itemTaken 事件路由

- 2026-05-27：补齐普通 `Unloader` 从 storage/core 取物时的 Java `removeStack(...) -> itemTaken(...)` 等价事件路由，和 `DirectionalUnloader` 的 linked storage core-unload 覆盖测试。
- Java 依据：
  - `storage/Unloader.updateTile()` 成功交易时执行 `dumpingTo.building.handleItem(...)` 后再 `dumpingFrom.building.removeStack(item, 1)`；
  - `StorageBuild.itemTaken(item)` 在 `linkedCore != null` 时转发给 linked core；
  - `DirectionalUnloader.updateTile()` 成功从 back 取物后调用 `back.itemTaken(item)`，linked storage 应把事件落到 core。
- Rust 新增/变化：
  - `GameRuntimeItemUnloaderFrameReport` 新增 `item_taken_events`；
  - `advance_owned_item_unloaders_ticks(...)` 在普通 unloader 成功交易时累计 `item_taken_events`；
  - `item_unloader_transfer_item(...)` 从 source owner 扣物后改为调用统一 `record_item_taken(content, from_index, item_id)`，让普通 storage 事件落在自身 tile，linked storage 事件落到 core tile，并复用 campaign core delta 处理；
  - 移除旧的 `note_linked_storage_remove_stack_side_effects(...)` 专用旁路，避免 itemTaken 语义分散；
  - 新增测试通过临时把 `duct-unloader.allow_core_unload = true`，验证 linked storage 可以被 directional unloader 卸出，且 `GameRuntimeItemTakenEvent { source_tile_pos: storage, event_tile_pos: core }`。
- 验证：
  - `cargo test -p mindustry-core game_runtime_directional_unloader_records_item_taken_for_linked_storage_when_core_unload_allowed`
  - `cargo test -p mindustry-core game_runtime_directional_unloader_rejects_linked_storage_without_core_unload`
  - `cargo test -p mindustry-core game_runtime_item_unloader_moves_configured_item_from_storage_to_receiver`
  - `cargo test -p mindustry-core game_runtime_item_unloader_unloads_linked_storage_from_core_items_when_allowed`
- 仍未完成：`linkedCore` 仍是 runtime 派生表 `storage_linked_cores`，不是 Building 持久字段；后续若更多路径直接“从 storage 取物”，必须继续统一调用 `record_item_taken(...)`，并补 save/load/网络 map 下 linked core 重建 smoke。

### 12.83 Item MassDriver bolt update 接入 runtime

- 2026-05-27：把普通 item `MassDriver` 的 runtime in-flight shot 从“只等 `remaining_ticks == 0` 交付”推进到复用 Java `MassDriverBolt.update(...)` 等价几何命中计划。
- Java 依据：
  - `MassDriverBolt.update(Bullet b)` 在目标存活时按 `baseDst/dst1/dst2` 判断是否进入命中半径或越过目标并相交；
  - 目标死亡时 bullet 不交付，继续飞到 bullet lifetime 后按 `despawned()/hit()` 掉落和爆炸；
  - 低 FPS 越过目标时会把 bullet 位置 snap 到目标边界后调用 `MassDriverBuild.handlePayload(...)`。
- Rust 新增/变化：
  - `advance_item_mass_driver_in_flight_ticks(...)` 每 tick 改为按 shot `speed + rotation + elapsed_ticks` 推进 bullet 当前位置，允许继续飞过目标直到 lifetime；
  - 同一函数接入 `MassDriverBolt::update_plan(...)`，命中时立即调用 `resolve_item_mass_driver_in_flight_shot(...)`，`remaining_ticks` 退化为 travel 上界/观测字段，不再是唯一交付条件；
  - `snap_position` 会回写到 shot 当前位置，后续接真实 bullet/entity 或 hit effect 时可复用；
  - `target_lost`/`keep_flying` 仅表示寿命内继续飞，`expire_ticks <= 0` 时仍走现有 `create_item_mass_driver_despawn_event(...)` 掉落/爆炸路径，避免目标丢失 shot 永久挂起。
- 验证：
  - `cargo test -p mindustry-core game_runtime_item_mass_driver_resolves_on_bolt_hit_plan_before_remaining_ticks_zero`
  - `cargo test -p mindustry-core game_runtime_item_mass_driver`
- 仍未完成：当前仍是 runtime-only in-flight 侧车，还未生成真实 ECS bullet entity，也未播放 `shootEffect/receiveEffect/sound/shake`，随机掉落仍使用确定性全掉落测试路径；后续应继续把 effect/event、真实 bullet mirror 和联机同步接到 desktop/server 链路。

### 12.84 InputHandler takeItems removeStack 侧效计划

- 2026-05-27：补齐 `InputHandler.takeItems(...)` 的 Java `build.removeStack(...)` 语义出口；Rust 纯 input helper 在成功扣物时现在会返回 `ItemRemoveStackPlan`，供上层按 core / linked storage 路由 campaign sector core-item delta。
- Java 依据：
  - `InputHandler.takeItems(Building build, Item item, int amount, Unit to)` 先执行 `build.removeStack(item, Math.min(to.maxAccepted(item), amount))`，再把 removed 加到 unit；
  - `CoreBuild.removeStack(...)` 会在 campaign default team 下 `handleCoreItem(item, -result)`；
  - `StorageBuild.removeStack(...)` 只有在 `linkedCore != null` 时才执行等价 core-item delta；
  - `InputHandler.setItem/setItems/setTileItems` 远程入口是直接 `build.items.set(...)`，不是 `removeStack(...)`，本闭环不为这些 direct-set 路径伪造 removeStack 侧效。
- Rust 新增/变化：
  - 新增 `ItemRemoveStackPlan { build, item, item_id, amount_removed, source_is_core, source_is_storage }`；
  - `TakeItemsOutcome` 新增 `remove_stack: Option<ItemRemoveStackPlan>`；
  - `take_items(...)` 成功扣物时返回 removeStack 计划，失败/无移除时保持 `None`；
  - 新增 core source 覆盖测试，确认 `source_is_core` 会随 block flag 透出。
- 验证：
  - `cargo test -p mindustry-core take_items_`
- 仍未完成：`input_handler` 仍是纯 helper，尚未把 `ItemRemoveStackPlan` 接入完整 `GameRuntime`/campaign sector 状态；linked storage 还需要 runtime 侧根据 `storage_linked_cores` 判定后再应用 `handleCoreItem(-amount)`。

### 12.85 NetClient takeItems 库存镜像扣减

- 2026-05-27：补齐 Java 客户端收到 `TakeItemsCallPacket` 后对本地建筑库存的可见变化；Rust `NetClientState.building_storage_mirrors` 现在会在收包时扣减对应 item。
- Java 依据：
  - `InputHandler.takeItems(...)` 是 `@Remote(called = Loc.server)`，客户端收到后执行 `build.removeStack(...)` 并把物品加入目标 unit；
  - 当前 Rust 客户端已有 `SetItem/SetItems/ClearItems` 对 `building_storage_mirrors` 的应用，`TakeItems` 之前只记录 last packet，导致 UI/runtime 镜像会保留 stale 库存。
- Rust 新增/变化：
  - `NetClient::apply_take_items_packet(...)` 从 `ClientTileStorageMirror.items` 中扣减 `packet.amount.max(0)`，结果下限 clamp 到 0；
  - `NetClient::update()` 的 `PacketKind::TakeItemsCallPacket` 分支调用该应用函数，同时保留原有 packet 计数与 last packet 记录；
  - 更新 forwarded inventory packet 测试，确认 `SetItem(copper=42)` 后收到 `TakeItems(copper,5)` 时客户端镜像变为 37。
- 验证：
  - `cargo test -p mindustry-core net_client::tests::apply_building_item_and_liquid_packets_updates_storage_mirror`
  - `cargo test -p mindustry-core net_client::tests::update_records_server_forwarded_inventory_payload_and_unit_packets`
- 仍未完成：客户端还没有独立 unit item mirror，因此 `to.addItem(...)` 只在 packet/mirror 层留下建筑扣减；后续应把 unit/entity snapshot 或 typed unit runtime 与 `TakeItemsCallPacket.to` 接起来。

### 12.86 GameRuntime removeStack 计划消费

- 2026-05-27：把 12.84 的 `ItemRemoveStackPlan` 接入 `GameRuntime`，形成 `InputHandler.takeItems(...) -> removeStack plan -> campaign core-item delta` 的 runtime 消费点。
- Java 依据：
  - `InputHandler.takeItems(...)` 的核心副作用是 `build.removeStack(...)`；
  - `CoreBuild.removeStack(...)` 在 campaign default team 下 `handleCoreItem(item, -result)`；
  - `StorageBuild.removeStack(...)` 只有 linkedCore 存在时才对 campaign core item 做 `-result`。
- Rust 新增/变化：
  - `GameRuntime::apply_item_remove_stack_plan(content, plan)` 根据 `plan.build.tile_pos` 定位建筑，并刷新 `storage_linked_cores`；
  - 若 source 是 core，则直接对该 core 执行 `note_campaign_core_item_delta(..., -amount_removed)`；
  - 若 source 是 linked storage，则通过 `linked_core_index_for_storage(...)` 路由到 core 后执行同样 delta；
  - 不重复扣 `items`，因为 `take_items(...)` 已经完成内存移除，runtime helper 只消费 Java `removeStack` 的账本副作用。
- 验证：
  - `cargo test -p mindustry-core game_runtime_item_remove_stack_plan_updates_campaign_core_delta_for_core_and_linked_storage`
  - `cargo test -p mindustry-core game_runtime_core_handle_item`
  - `cargo test -p mindustry-core game_runtime_linked_storage_unloader_updates_campaign_core_delta`
- 仍未完成：`RequestItemCallPacket`/`TakeItemsCallPacket` 在 server 权威事件循环中仍未自动调用 `input_handler::take_items(...)` 与 `GameRuntime::apply_item_remove_stack_plan(...)`；下一步可继续把 server packet 消费桥接到该 helper。

### 12.87 Item MassDriver shoot/receive effect 事件侧车

- 2026-05-27：补齐普通 item `MassDriver` 上游默认 effect/sound/shake 字段，并在 runtime 发射/命中交付路径记录 shoot/receive 事件侧车，为后续真实 ECS effect/sound 播放与联机同步留出统一接入点。
- Java 依据：
  - `MassDriver.shootEffect = Fx.shootBig2`、`smokeEffect = Fx.shootBigSmoke2`、`receiveEffect = Fx.mineBig`、`shootSound = Sounds.massdriver`、`receiveSound = Sounds.massdriverReceive`、`shootSoundVolume = 0.5f`、`shake = 3f`；
  - `MassDriverBuild.fire(...)` 在创建 `MassDriverBolt` 后播放 `shootEffect`、`smokeEffect`、`Effect.shake(...)` 和 `shootSound.at(...)`；
  - `MassDriverBuild.handlePayload(...)` 成功接收 payload 后播放 `receiveEffect`、`receiveSound.at(...)` 和 `Effect.shake(...)`。
- Rust 新增/变化：
  - `DistributionBlockData` 新增 `shoot_effect/smoke_effect/receive_effect/shoot_sound/receive_sound`，并给 `DistributionBlockKind::MassDriver` 设置上游默认值；
  - `GameRuntime` 新增 `item_mass_driver_shoot_events` 与 `item_mass_driver_receive_events`，reset/clear 路径会同步清空；
  - `item_mass_driver_fire_to_link(...)` 成功扣物并创建 in-flight shot 时记录发射事件，包含源/目标 tile、发射坐标、shoot/smoke effect、shoot sound、volume 与 shake；
  - `resolve_item_mass_driver_in_flight_shot(...)` 只有目标仍有效且成功进入交付路径时记录接收事件；target-lost/despawn 路径不会伪造 receive event。
- 验证：
  - `cargo test -p mindustry-core game_runtime_item_mass_driver_sends_items_in_flight_and_delivers_after_delay`
  - `cargo test -p mindustry-core game_runtime_item_mass_driver`
  - `cargo test -p mindustry-core unloader_and_mass_driver_keep_upstream_subset`
  - `cargo check --workspace`
- 仍未完成：当前仍只是 runtime event sidecar，还没有在 desktop/server/client 真实播放 effect/sound/shake，也未把这些事件编码进联机协议或 ECS effect entity；后续应把侧车消费到渲染/音频/网络快照链路。

### 12.88 Server RequestItem → TakeItems 权威 runtime 桥接

- 2026-05-27：把客户端发来的 `RequestItemCallPacket` 接入 `ServerLauncher::apply_new_network_server_events()`，形成 `RequestItemCallPacket -> input_handler::request_item(...) -> input_handler::take_items(...) -> GameRuntime::apply_item_remove_stack_plan(...) -> TakeItems/TransferItemEffect outbound packet` 的 server 权威闭环。
- Java 依据：
  - `InputHandler.requestItem(Player player, Building build, Item item, int amount)` 校验可交互、距离、死亡、数量、`Units.canInteract(...)` 与 admin action 后调用 `Call.takeItems(...)`；
  - `InputHandler.takeItems(Building build, Item item, int amount, Unit to)` 执行 `build.removeStack(...)`、`to.addItem(...)` 并按移除数量触发 `transferItemEffect(...)`；
  - `TakeItemsCallPacket` 是 server→client 的 `@Remote(called = Loc.server, unreliable = true)` 出站效果，不应作为普通客户端权威请求直接信任。
- Rust 新增/变化：
  - `ServerLauncher` 新增 `server_units: BTreeMap<i32, UnitComp>`，按连接 id 保存 server-side player unit item mirror，避免 `take_items(...)` 的 `to.addItem(...)` 只作用在临时对象上；
  - `ServerLauncher` 新增 RequestItem/TakeItems 统计与 last-outcome 字段，方便测试与后续调试；
  - `apply_new_network_server_events()` 新增 `PacketKind::RequestItemCallPacket` 分支，使用连接状态而不是信任 client payload 里的 `player` 字段；
  - 在 player/unit 位置同步尚未接通前，server-side player mirror 若仍处于零位 bootstrap，会暂时放行距离检查，避免所有远离地图原点的合法 inventory 请求被误拒；
  - 成功取物后，建筑库存会在 `GameRuntime` 中扣减，unit mirror 会加物，`remove_stack` 计划会继续更新 campaign core item delta，并通过 `Net::send(..., false)` 广播 `TakeItemsCallPacket` 和 `TransferItemEffectCallPacket`。
- 验证：
  - `cargo test -p mindustry-server`
  - `cargo test -p mindustry-core take_items_`
  - `cargo test -p mindustry-core game_runtime_item_remove_stack_plan_updates_campaign_core_delta_for_core_and_linked_storage`
  - `cargo check --workspace`
- 仍未完成：server 端 player/unit 仍是 launcher 侧 mirror，尚未接入真实单位 entity snapshot/出生/死亡/切换流程；`Units.canInteract(...)` 和 admin action 目前用保守占位闭包通过，距离校验在缺少位置同步时有 bootstrap fallback，后续必须接入完整权限、严格距离、单位生命周期与 WithdrawEvent/统计广播。`TransferInventory` 是下一条最自然的对称闭环。

### 12.89 Server TransferInventory → TransferItemTo 权威 runtime 桥接

- 2026-05-27：补齐 `TransferInventoryCallPacket` 的 server 权威消费，把“单位携带物 → 建筑库存”的存货链路接到 `server_units` mirror、`GameRuntime` 建筑库存、core/campaign handleStack 副作用和 `TransferItemToCallPacket` 出站广播。
- Java 依据：
  - `InputHandler.transferInventory(Player player, Building build)` 校验距离、建筑 item module、玩家存活、`build.allowDeposit()`、unit stack、`Units.canInteract(...)`、deposit rate 与 admin action；
  - 校验通过后计算 `accepted = build.acceptStack(item, unit.stack.amount, unit)`，再调用 `Call.transferItemTo(unit, item, accepted, unit.x, unit.y, build)`；
  - `InputHandler.transferItemTo(...)` 会扣 unit stack、播放 item transfer effect，并在 `amount > 0` 时执行 `build.handleStack(item, amount, unit)`。
- Rust 新增/变化：
  - `ServerLauncher::apply_new_network_server_events()` 新增 `PacketKind::TransferInventoryCallPacket` 分支；
  - 新增 transfer-inventory / transfer-item-to 统计与 last-outcome 字段；
  - `apply_server_transfer_inventory_packet(...)` 复用连接状态和 `server_units` mirror，先调用 `input_handler::transfer_inventory(...)` 计算 accepted amount，再调用 `input_handler::transfer_item_to(...)` 真正扣 unit stack、加 building items，并广播 `TransferItemToCallPacket`（unreliable）；
  - `GameRuntime` 新增 `apply_item_handle_stack_side_effects(...)`，deposit 到 core 时复用既有 `note_core_handle_item_side_effects(...)`，把 campaign/core item delta 与 stats 接上；
  - 当前 server `accept_stack` 先按 building item capacity 与当前 total 做通用容量裁剪，后续还需替换为完整 block-specific `acceptStack(...)`。
- 验证：
  - `cargo test -p mindustry-server`
  - `cargo test -p mindustry-core transfer_inventory_`
  - `cargo test -p mindustry-core transfer_item_to_`
  - `cargo check --workspace`
- 仍未完成：`build.allowDeposit()`、`build.acceptStack(...)`、deposit rate、admin action、`Units.canInteract(...)`、真实 player/unit 位置与生命周期仍未完全接入 Java 等价实现；当前 `server_units` 仍是 launcher mirror，不是完整实体组。下一步可继续 payload pickup/drop，或补全 `TransferItemToCallPacket` 客户端 mirror 对 building/unit 的实际状态应用。

### 12.90 Server RequestDropPayload → PayloadDropped 权威 runtime 桥接

- 2026-05-27：优先接入 payload 放下的窄闭环，把 `RequestDropPayloadCallPacket` 从 server 事件流接到 `input_handler::request_drop_payload(...)`、`server_units` 的 `PayloadComp::drop_last_payload(...)` 和 `PayloadDroppedCallPacket` 出站广播。
- Java 依据：
  - `InputHandler.requestDropPayload(Player player, float x, float y)` 检查非 client、玩家存活、单位 payload 非空、admin action，然后把目标坐标限制到单位周围 `tilesize * 4f` 范围；
  - `InputHandler.payloadDropped(Unit unit, float x, float y)` 临时把 payload 单位位置设置为 drop 坐标，执行 `dropLastPayload()`，再还原位置。
- Rust 新增/变化：
  - `ServerLauncher::apply_new_network_server_events()` 新增 `PacketKind::RequestDropPayloadCallPacket` 分支；
  - 新增 request-drop / payload-dropped 统计与 last-outcome 字段；
  - `apply_server_request_drop_payload_packet(...)` 复用连接状态与 `server_units` mirror，调用 `request_drop_payload(...)` 完成 Java margin clamp；
  - `apply_payload_drop_to_server_unit(...)` 对 `UnitRef::Unit` 查找 server unit mirror，临时设置坐标、drop 最后一个 payload、再恢复单位位置；
  - 成功后可靠广播 `PayloadDroppedCallPacket`，客户端可继续按现有 packet 记录/后续 mirror 消费。
- 验证：
  - `cargo test -p mindustry-server`
  - `cargo test -p mindustry-core request_drop_payload_`
  - `cargo check --workspace`
- 仍未完成：drop 出来的 payload 还没有实体化进入完整 world/entity groups，也没有地面碰撞/落点占用判定；`server_units` 仍是 launcher mirror，admin action 仍是占位。下一步可继续 `RequestBuildPayload/PickedBuildPayload`，或把 `PayloadDroppedCallPacket` 客户端 mirror 从“只记录 packet”推进到真实 unit payload mirror mutation。

### 12.91 Server RequestBuildPayload → PickedBuildPayload 权威 runtime 桥接

- 2026-05-27：接入 payload 拾取建筑/建筑内 payload 的 server 权威窄闭环，把 `RequestBuildPayloadCallPacket` 从 server 事件流桥接到 `input_handler::request_build_payload(...)`、`server_units` 的 `PayloadComp::add_payload(...)`、`GameRuntime::payload_runtime_states` / `buildings` 状态变更和 `PickedBuildPayloadCallPacket` 出站广播。
- Java 依据：
  - `InputHandler.requestBuildPayload(Player player, Building build)` 先校验 player/unit payload/build、距离、admin action 与 team interact；然后优先 `build.getPayload()` + `pay.canPickupPayload(current)`，否则尝试 `build.block.buildVisibility != hidden && build.canPickup() && pay.canPickup(build)`；
  - `InputHandler.pickedBuildPayload(Unit unit, Building build, boolean onGround)` 在 `onGround=false` 时 `build.takePayload()` 并 `pay.addPayload(taken)`；在 `onGround=true` 时成功则 `pay.pickup(build)`，二次校验失败也会播放 pickup effect 并 `build.tile.remove()`。
- Rust 新增/变化：
  - `ServerLauncher::apply_new_network_server_events()` 新增 `PacketKind::RequestBuildPayloadCallPacket` 分支；
  - 新增 request-build / picked-build 统计与 last-outcome 字段；
  - `apply_server_request_build_payload_packet(...)` 复用连接状态和 `server_units` mirror，不信任 client payload 里的 `player` 字段；在 player/unit 位置同步未接通前沿用零位 bootstrap 距离 fallback；
  - 新增 `runtime_payload_ref_for_tile(...)` / `take_runtime_payload_ref_for_tile(...)`，支持从 `PayloadBlockBuildState.common.payload`、`PayloadConveyorState.item`、`PayloadRouterState.conveyor.item` 抽取/移除建筑内 payload；
  - 新增 `payload_ref_to_payload_state(...)`，把 `PayloadRef::Block` / `PayloadRef::Unit` 映射到当前 `PayloadComp` 可消费的 `PayloadState`；
  - `apply_stored_build_payload_pickup_to_server_unit(...)` 会二次校验 live payload 与单位容量，然后从 runtime payload state 移除并加入 unit payload；
  - `apply_whole_build_payload_pickup_to_server_unit(...)` 会按 Java 语义二次校验 `BuildVisibility`、`canPickup` 与 payload 容量；成功时移除 runtime building 并加入 unit payload，失败时仍移除 tile 以贴近 Java fallback；
  - 成功后可靠广播 `PickedBuildPayloadCallPacket`，覆盖 `on_ground=false`（建筑内部 payload）和 `on_ground=true`（整栋建筑）两条路径。
- 验证：
  - `cargo test -p mindustry-server server_launcher_applies_request_build_payload_packet`
- 仍未完成：整栋建筑 payload 目前只在 `PayloadComp` 中保存 `PayloadState { kind, size }`，尚未保留完整 `BuildPayload` 的 building bytes/runtime sidecars；`build.canPickup()` 只覆盖 core、linked storage、logic、radar 等已迁移的关键 override；admin action、严格 player/unit 位置同步、完整 entity group、客户端 mirror 对 `PickedBuildPayloadCallPacket` 的真实状态消费仍需后续继续。

### 12.92 Client TransferItemTo 建筑库存 mirror 消费

- 2026-05-27：把 `TransferItemToCallPacket` 从“只记录 last packet/计数”推进到客户端建筑库存 mirror mutation，补上 server `TransferInventory -> TransferItemTo` 权威链路在 client 侧的可观察状态同步。
- Java 依据：
  - `InputHandler.transferItemTo(Unit unit, Item item, int amount, float x, float y, Building build)` 在客户端/服务端都会体现“unit stack 转移到 build”的效果；
  - server 已在 12.89 中把该 packet 作为存货转移成功后的出站包，客户端需要至少更新建筑 item mirror，避免 UI/调试镜像仍停留在旧库存。
- Rust 新增/变化：
  - `NetClient::apply_transfer_item_to_packet(...)` 新增通用 helper：解析 `packet.build.tile_pos` 与 `packet.item`，对 `ClientTileStorageMirror.items` 做 `previous + amount.max(0)`；
  - `handle_client_received(PacketKind::TransferItemToCallPacket)` 调用该 helper 后再记录 last packet 与 timestamp；
  - 既有 building storage mirror 测试扩展到 transfer-in，并更新 client packet 总记录测试，让 secondary build 在收到 `TransferItemToCallPacket(scrap, 9)` 后保留 `scrap=9`。
- 验证：
  - `cargo test -p mindustry-core apply_building_item_and_liquid_packets_updates_storage_mirror`
  - `cargo test -p mindustry-core update_records_server_forwarded_inventory_payload_and_unit_packets`
  - `cargo check --workspace`
- 仍未完成：这里只更新 building storage mirror，尚未同步扣减 client-side unit item mirror，也没有播放 item transfer effect；`TransferItemToUnitCallPacket`、`PayloadDroppedCallPacket`、`PickedBuildPayloadCallPacket` 仍需继续从 last-packet 记录推进到真实 runtime/mirror mutation。

### 12.93 Client payload pickup/drop 单位载荷 mirror 消费

- 2026-05-27：把 `PickedBuildPayloadCallPacket`、`PickedUnitPayloadCallPacket` 和 `PayloadDroppedCallPacket` 从“只记录 last packet/计数”推进到客户端单位载荷 sidecar mirror mutation，补上 server payload pickup/drop 权威链路在 client 侧的最小可观察状态同步。
- Java 依据：
  - `InputHandler.pickedBuildPayload(Unit unit, Building build, boolean onGround)` 成功后会让单位 `PayloadComp` 增加一个建筑 payload；
  - `InputHandler.pickedUnitPayload(Unit unit, Unit target)` 成功后会让承载单位增加一个单位 payload；
  - `InputHandler.payloadDropped(Unit unit, float x, float y)` 会对承载单位执行 `dropLastPayload()`，客户端收到出站包后需要让 payload 计数镜像向 Java 的单位 payload 栈变化靠拢。
- Rust 新增/变化：
  - `NetClientState` 新增 `unit_payload_mirrors: BTreeMap<i32, ClientUnitPayloadMirror>`，按承载单位 id 保存 payload count 与 pickup/drop 观测计数；
  - 新增 `NetClient::apply_picked_build_payload_packet(...)`、`apply_picked_unit_payload_packet(...)` 和 `apply_payload_dropped_packet(...)`，只接受 `UnitRef::Unit`，pickup 递增 `payload_count`，drop 使用 `saturating_sub(1)` 避免 stale/乱序包导致负数；
  - `handle_client_received(...)` 的 `PickedBuildPayloadCallPacket`、`PickedUnitPayloadCallPacket`、`PayloadDroppedCallPacket` 分支现在先更新 unit payload mirror，再保留原有 last packet/timestamp/seen 记录；
  - client 总记录测试现在断言 build-pickup、unit-pickup 和 drop 三条包路径都会更新对应承载单位的 payload sidecar。
- 验证：
  - `cargo test -p mindustry-core apply_payload_packets_updates_unit_payload_mirror`
  - `cargo test -p mindustry-core update_records_server_forwarded_inventory_payload_and_unit_packets`
  - `cargo check --workspace`
- 仍未完成：当前 payload mirror 仍只是 client sidecar 计数与观测统计，尚未保存完整 `BuildPayload` bytes、`UnitPayload` runtime state，也未接入真实 `GameRuntime.client_unit_snapshot_entities` 的 typed `PayloadComp`；后续需要把这些 sidecar 与 entity snapshot / unit runtime / renderer UI 统一起来，避免长期停留在独立镜像层。

### 12.94 Client TakeItems/TransferItemToUnit 单位物品 mirror 消费

- 2026-05-27：继续收敛客户端存货可观察状态，把 `TakeItemsCallPacket`、`TransferItemToCallPacket` 的源单位扣减路径以及 `TransferItemToUnitCallPacket` 从纯 last-packet 记录推进到单位 item sidecar mirror mutation。
- Java 依据：
  - `InputHandler.takeItems(Building build, Item item, int amount, Unit to)` 从建筑扣物后调用 `to.addItem(item, removed)`；
  - `InputHandler.transferItemTo(Unit unit, Item item, int amount, float x, float y, Building build)` 在 `unit.item() == item` 时扣减源单位 stack；
  - `InputHandler.transferItemToUnit(Item item, float x, float y, Itemsc to)` 在 transfer effect 完成后执行 `to.addItem(item)`，上游矿工本地拾矿路径会调用该入口。
- Rust 新增/变化：
  - `NetClientState` 新增 `unit_item_mirrors: BTreeMap<i32, ClientUnitItemMirror>`，按单位/entity id 保存当前 sidecar item、amount 和三类包的观测计数；
  - `NetClient::apply_take_items_to_unit_mirror(...)` 让 `TakeItemsCallPacket.to` 对应单位增加 `amount.max(0)` 的 item；
  - `NetClient::apply_transfer_item_to_source_unit_mirror(...)` 只在已有且 item 匹配的源单位 mirror 上扣减 `amount.max(0)`，避免无 snapshot 基线时凭空伪造未知剩余量；
  - `NetClient::apply_transfer_item_to_unit_mirror(...)` 把 `TransferItemToUnitCallPacket.to` 的 entity id 当作临时 Itemsc/unit mirror key，每包按 Java `addItem(item)` 增加 1；
  - `handle_client_received(...)` 的 TakeItems/TransferItemTo/TransferItemToUnit 分支现在会同时维护建筑库存 mirror 与单位 item mirror。
- 验证：
  - `cargo test -p mindustry-core apply_unit_item_packets_updates_unit_item_mirror`
  - `cargo test -p mindustry-core update_records_server_forwarded_inventory_payload_and_unit_packets`
  - `cargo check --workspace`
- 仍未完成：当前单位 item mirror 仍是 NetClient sidecar，尚未与 `GameRuntime.client_unit_snapshot_entities` 的 typed `UnitComp.items.stack` 双向合并；`TransferItemToUnitCallPacket.to` 是泛型 `EntityRef`/`Itemsc`，后续需要用 entity snapshot type 信息区分 Unit/Building，并把 item transfer effect 的视觉/音效回放接到 desktop renderer。

### 12.95 Desktop/GameRuntime 消费客户端单位物品 mirror

- 2026-05-27：把 12.94 新增的 `NetClientState.unit_item_mirrors` 从独立 sidecar 推进到 desktop/runtime 消费链，开始同步到真实 typed client unit snapshot entity 的 `UnitComp.items.stack`。
- Java 依据：
  - Java 客户端收到 `takeItems`、`transferItemTo`、`transferItemToUnit` 后最终改变的是单位/Itemsc 的实际物品栈，而不是只保留一个调试包记录；
  - Rust desktop 已经通过 entity snapshot 维护 `GameRuntime.client_unit_snapshot_entities`，因此 NetClient sidecar 应在 unit snapshot materialize 后回放到该 typed runtime entity。
- Rust 新增/变化：
  - `GameRuntime::apply_client_unit_item_mirror(...)` 新增最小应用点：按 entity id 查找 `client_unit_snapshot_entities`，将 mirror 中的 item/amount 写入 `UnitComp.items.stack`，并按单位 item capacity clamp；
  - `DesktopLauncher` 新增 `last_applied_unit_item_mirrors` 游标与 `sync_unit_item_mirrors_to_runtime()`，在每帧 snapshot 同步后把有变化的 unit item mirror 应用到 runtime；
  - 如果 sidecar 先于 typed unit snapshot 到达，desktop 不会把它标记为已应用，后续 unit snapshot materialize 后仍会重试；world reset 时会重置游标，避免旧地图 sidecar 污染新 runtime；
  - 新增 core/desktop 测试覆盖 mirror 写入 `UnitComp.items.stack`、capacity clamp、缺失 unit 拒绝以及 desktop update 链路。
- 验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_item_mirror_to_typed_unit`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_unit_item_mirror_to_runtime_unit_snapshot`
  - `cargo check --workspace`
- 仍未完成：payload sidecar 尚未接入 typed `PayloadComp`；unit item mirror 仍依赖 packet sidecar 的“最后可观察值”，尚未用 packet timestamp/snapshot sequence 解决 snapshot 与 remote-call 乱序；`TransferItemToUnitCallPacket.to` 仍需结合 entity class 判断是否是 Unit/Building/其他 Itemsc，视觉 transfer effect 也还没有进入 renderer。

### 12.96 Desktop/GameRuntime 消费客户端单位 payload mirror

- 2026-05-27：把 `NetClientState.unit_payload_mirrors` 继续从独立 sidecar 推进到 desktop/runtime typed unit 链路，最小化同步到 `GameRuntime.client_unit_snapshot_entities[*].payload`。
- Java 依据：
  - `InputHandler.pickedBuildPayload(...)`、`pickedUnitPayload(...)` 和 `payloadDropped(...)` 最终修改的是单位 `PayloadComp.payloads`；
  - Rust 客户端已经有 `PickedBuildPayload/PickedUnitPayload/PayloadDropped` 的 packet sidecar 计数，因此 desktop 应在 unit snapshot materialize 后把该计数回放到 typed `UnitComp.payload`，而不是停留在 NetClient 调试层。
- Rust 新增/变化：
  - `GameRuntime::apply_client_unit_payload_mirror(...)` 新增最小应用点：按 entity id 查找 typed client unit，确保 `PayloadComp` 存在并同步 team/capacity/pickup_units，然后按 sidecar `payload_count` 写入 placeholder `PayloadState` 列表；
  - placeholder 会用 `picked_unit_payloads_seen` 尽量保留 Unit payload 个数，其余按 Build payload 占位，`size` 暂为 `0.0`，用于先接通 count/has_payload/logic sense/UI 调试链；
  - `DesktopLauncher` 新增 `last_applied_unit_payload_mirrors` 游标与 `sync_unit_payload_mirrors_to_runtime()`，与 unit item mirror 一样只在 typed unit 存在且 sidecar 变化时标记已应用；
  - 新增 core/desktop 测试覆盖 payload count、Unit/Build placeholder 区分、drop 后 count 收敛和缺失 unit 拒绝。
- 验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_payload_mirror_to_typed_unit`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_unit_payload_mirror_to_runtime_unit_snapshot`
  - `cargo check --workspace`
- 仍未完成：当前 typed payload 仍是 count/类型占位，尚未保留完整 `BuildPayload` building bytes、`UnitPayload` entity id/sync bytes、真实 payload size 与落地碰撞；packet sidecar 与 entity snapshot 的乱序仍需引入 sequence/timestamp 或 authoritative snapshot 合并策略。

### 12.97 Server RequestUnitPayload → PickedUnitPayload 权威 runtime 桥接

- 2026-05-27：补齐 payload 单位拾取的 server 权威闭环，把 `RequestUnitPayloadCallPacket` 从 server 事件流接到 `input_handler::request_unit_payload(...)`、`server_units` 的 `PayloadComp.add_payload(Unit)` 和 `PickedUnitPayloadCallPacket` 可靠广播。
- Java 依据：
  - `InputHandler.requestUnitPayload(Player player, Unit target)` 要求玩家单位实现 `Payloadc`、目标是 AI、目标 grounded、`pay.canPickup(target)` 且目标在 `unit.type.hitSize * 2 + target.type.hitSize * 2` 范围内；
  - `InputHandler.pickedUnitPayload(Unit unit, Unit target)` 成功时 `pay.pickup(target)`，否则当载体不是 Payloadc 但 target 存在时移除 target。
- Rust 新增/变化：
  - `ServerLauncher::apply_new_network_server_events()` 新增 `PacketKind::RequestUnitPayloadCallPacket` 分支；
  - 新增 request-unit-payload / picked-unit-payload 统计与 last outcome 字段；
  - `apply_server_request_unit_payload_packet(...)` 复用连接状态、server preview player 和 `server_units` mirror，不信任 client payload 内的 `player` 字段，只使用连接 id 绑定当前 player/unit；
  - `unit_payload_target_within_range(...)` 复刻 Java hitSize 范围公式，并保留零位 bootstrap fallback，避免实体位置同步未完成前全量误拒；
  - `apply_picked_unit_payload_to_server_unit(...)` 从 `server_units` 移除目标 unit，并把 `PayloadState { kind: Unit, size: target.hit_size }` 加入载体单位 payload；
  - 新增 server 测试覆盖 target 移除、载体 payload 追加、统计字段和可靠 `PickedUnitPayloadCallPacket` 广播。
- 验证：
  - `cargo test -p mindustry-server server_launcher_applies_request_unit_payload_packet_to_target_unit_and_broadcasts_pickup`
  - `cargo test -p mindustry-server`
  - `cargo check --workspace`
- 仍未完成：`server_units` 仍是 launcher mirror，不是完整 entity group；目标 unit 被移除后尚未同步 entity hidden/remove snapshot，也未保存完整 `UnitPayload` sync bytes；真实 target lookup 应最终接入 world/entity groups 与 Java `Groups.unit`，并移除 bootstrap 距离 fallback。

### 12.98 Server DropItem 权威 unit stack 清空

- 2026-05-27：把 `DropItemCallPacket` 从 input helper 推进到 server 事件循环，形成 `DropItemCallPacket -> input_handler::drop_item(...) -> server_units item stack clear` 的最小权威闭环。
- Java 依据：
  - `InputHandler.dropItem(Player player, float angle)` 在 server 侧要求玩家和单位存在，若单位 stack 为空则抛出 validate；成功时播放 `Fx.dropItem` 并 `unit.clearItem()`；
  - 该 remote 为 `targets = Loc.client, called = Loc.server`：按 upstream 注解生成语义，这是客户端发起、服务端处理的 remote；当前 Rust 不再把同一个 `DropItemCallPacket` 当作 server-to-client 包广播，避免让客户端收包链路依赖错误方向。
- Rust 新增/变化：
  - `ServerLauncher::apply_new_network_server_events()` 新增 `PacketKind::DropItemCallPacket` 分支；
  - 新增 drop-item 统计与 `last_runtime_drop_item_outcome`；
  - `apply_server_drop_item_packet(...)` 用连接 id 绑定 player/unit，不信任客户端隐式 player，调用 `drop_item(DropItemContext { local_player: false }, ...)` 清空 `server_units[connection_id].items.stack.amount`；
  - 成功后只记录 outcome，不可靠广播同一个 C2S packet；后续客户端视觉/typed state 应通过 entity/block snapshot、明确携带 player/unit 的 effect/event，或 Java 等价的同步路径接入；
  - 新增 server 测试覆盖单位物品栈清空、previous item/amount outcome、统计字段，并断言不会错误广播 `DropItemCallPacket`。
- 验证：
  - `cargo test -p mindustry-server server_launcher_applies_drop_item_packet_to_unit_stack`
  - `cargo test -p mindustry-server server_launcher_ -- --test-threads=1`
  - `cargo check --workspace`
- 仍未完成：客户端 typed unit mirror 需要通过后续 authoritative snapshot/明确事件来看到清空结果；`Fx.dropItem` 视觉效果还没有接入 desktop renderer；server 侧仍依赖 `server_units` launcher mirror，后续要接入真实 player unit lifecycle、entity snapshot 和 validate/admin 事件流。

### 12.99 Client UnitEnteredPayload 运行态消费

- 2026-05-27：把 `UnitEnteredPayloadCallPacket` 从 NetClient 记录层推进到 desktop/runtime 消费层，形成 `NetClient last_unit_entered_payload -> DesktopLauncher::sync_unit_entered_payload_to_runtime() -> GameRuntime::apply_client_unit_entered_payload_packet(...)` 的最小客户端运行态闭环。
- Java 依据：
  - `InputHandler.unitEnteredPayload(Unit unit, Building build)`：同队校验后 `unit.remove()`，构造 `UnitPayload`，若 `build.acceptPayload(...)` 成功则 `build.handlePayload(...)`；
  - `CommandAI` 在非 client 侧确认 `unit.type.allowedInPayloads`、`unit.buildOn()` 与 `build.acceptPayload(...)` 后调用 `Call.unitEnteredPayload(unit, build)`。
- Rust 新增/变化：
  - `GameRuntimeClientUnitEnteredPayloadApplyReport` 记录 unit/build 是否存在、队伍是否匹配、unit 是否移除、payload 是否挂载；
  - `GameRuntime::apply_client_unit_entered_payload_packet(...)` 会从 `client_unit_snapshot_entities` 移除目标 unit、标记 raw sidecar hidden、写入 `client_hidden_entity_ids`，并把 `PayloadRef::Unit { class_id, unit_bytes }` 挂到 payload-loader/constructor/deconstructor/mass-driver/source/void 的 runtime common payload；
  - `DesktopLauncher` 增加 `sync_unit_entered_payload_to_runtime()`，按 `unit_entered_payload_packets_seen` 游标去重消费 NetClient 最新包；
  - `UnitEnteredPayloadCallPacket` 增加 wire roundtrip 测试，锁定 `UnitRef -> BuildingRef` 字段顺序。
- 验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_entered_payload_packet_to_payload_building`
  - `cargo test -p mindustry-core unit_entered_payload_packet_uses_java_field_order`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_unit_entered_payload_packet_to_runtime_payload_building`
  - `cargo check --workspace`
- 仍未完成：payload 中的 unit 已从空 bytes 升级到最小同步字节，但尚未写入完整 Java `UnitPayload` full unit serialization；客户端 `Fx.unitDrop`/渲染效果尚未接入；server 侧还需要从真实 CommandAI / unit buildOn runtime 自动触发。

### 12.100 Server UnitEnteredPayload 广播入口

- 2026-05-27：把 `unit_entered_payload(...)` helper 接入 `ServerLauncher`，新增 `ServerLauncher::apply_server_unit_entered_payload(unit_id, build_tile_pos)`，形成 `server_units + runtime building -> UnitEnteredPayloadCallPacket reliable broadcast` 的最小 Rust server 出站闭环。
- Java 依据：
  - `CommandAI` 在服务端判定单位站在目标 building 上且 `build.acceptPayload(build, tmpPayload)` 后调用 `Call.unitEnteredPayload(unit, build)`；
  - `InputHandler.unitEnteredPayload(...)` 在同队校验后移除 unit，并把 `UnitPayload` 写入目标 building。
- Rust 新增/变化：
  - `GameRuntime::attach_unit_payload_to_building(...)` 提取为通用 payload 挂载入口，client/server 可复用；
  - `ServerLauncher::apply_server_unit_entered_payload(...)` 读取 `server_units` 与 `runtime.buildings`，调用 `unit_entered_payload(...)`，成功后通过通用 runtime payload 挂载入口写入目标 payload building，移除 `server_units[unit_id]`，并可靠广播 `UnitEnteredPayloadCallPacket`；
  - 新增 server 测试覆盖 runtime payload-loader 接收 `PayloadRef::Unit`、server unit 移除和 reliable 出站包。
- 验证：
  - `cargo test -p mindustry-server server_launcher_broadcasts_unit_entered_payload_from_runtime_unit_and_building`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_entered_payload_packet_to_payload_building`
  - `cargo check --workspace`
- 仍未完成：该入口已作为手动执行点，后续 12.102 已把它接入 server update 的最小 `enterPayload` tick；payload unit bytes 仍只是 `UnitSyncWire` subset，不是完整 Java `UnitPayload` full serialization。

### 12.101 UnitEnteredPayload 写入 UnitSyncWire payload bytes

- 2026-05-27：把 `UnitEnteredPayload` 运行态 payload 中的 unit body 从空 bytes placeholder 提升为 `UnitSyncWire` 子集字节，避免客户端/服务端 payload sidecar 只能保存 class id 而丢失单位同步字段。
- Java 依据：
  - `InputHandler.unitEnteredPayload(...)` 构造 `new UnitPayload(unit)` 后交给 building `handlePayload(...)`，Java 完整路径最终应保留 payload 内 unit 的实体序列化；
  - 当前 Rust 仍未完成 Java `UnitPayload` full serialization，本节只先用已迁移的 `UnitComp::to_sync_wire()` 与 `type_io::write_unit_sync(...)` 提供可复用的最小单位同步 body。
- Rust 新增/变化：
  - `GameRuntime::unit_payload_ref_from_unit(content, unit)` 现在会把 `UnitComp::to_sync_wire()` 写入 `unit_bytes`，再生成 `PayloadRef::Unit { class_id, unit_bytes }`；
  - client `apply_client_unit_entered_payload_packet(...)` 与 server `apply_server_unit_entered_payload(...)` 共用该路径，因此 desktop/server 断言均改为 payload unit bytes 非空；
  - 这一步仍保持 raw sidecar 过渡语义，不把 `PayloadRef::Unit` 立即 materialize 成完整 unit entity，避免在 full payload serializer 完成前误删字段。
- 验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_entered_payload_packet_to_payload_building`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_unit_entered_payload_packet_to_runtime_payload_building`
  - `cargo test -p mindustry-server server_launcher_broadcasts_unit_entered_payload_from_runtime_unit_and_building`
  - `cargo check --workspace`
- 仍未完成：该节先落的是 `UnitSyncWire` subset；后续 12.103 已把已知 UnitPayload schema 的写入边界升级为 Java `unit.write(...)` 风格 body，但完整实体恢复、渲染和 save/load 仍需继续迁移。

### 12.102 Server update 自动触发 enterPayload 单位进入载荷

- 2026-05-27：把 12.100 的 `ServerLauncher::apply_server_unit_entered_payload(...)` 从手动入口接入真实 `ServerLauncher::update()` tick，形成 `server_units Command(enterPayload) + unit buildOn -> runtime payload building -> UnitEnteredPayloadCallPacket` 的最小自动触发链。
- Java 依据：
  - `CommandAI.updateUnit()` 在非 client 侧检查 `command == UnitCommand.enterPayloadCommand`、`unit.type.allowedInPayloads`、`unit.buildOn() != null`，并在 `targetPos == null || targetPos 对应 building == unit.buildOn()` 时调用 `Call.unitEnteredPayload(unit, build)`；
  - `InputHandler.unitEnteredPayload(...)` 再做同队校验、`unit.remove()`、`UnitPayload` 构造和 `build.handlePayload(...)`。
- Rust 新增/变化：
  - `ServerLauncher::update()` 在网络事件和 world data flush 后调用 `tick_server_unit_entered_payloads()`；
  - `tick_server_unit_entered_payloads()` 扫描 `server_units`，只接受 `UnitControllerState::Command(CommandWire)` 且 `command_id == enterPayload`、`allowed_in_payloads == true`、当前 `build_world/buildOn` 可解析的单位；
  - `target_pos` 非空时要求其指向的 building 与 unit 当前站立 building 一致；命中后复用既有 `apply_server_unit_entered_payload(...)`，因此仍由同一个落点负责 runtime payload 挂载、移除 unit 和 reliable 广播。
- 验证：
  - `cargo test -p mindustry-server server_launcher_update_applies_enter_payload_command_to_payload_building_and_broadcasts_packet`
  - `cargo test -p mindustry-server server_launcher_update_skips_enter_payload_when_target_building_mismatch`
  - `cargo test -p mindustry-server server_launcher_broadcasts_unit_entered_payload_from_runtime_unit_and_building`
  - `cargo check --workspace`
- 仍未完成：当前 tick 仍是 server launcher sidecar 上的最小 `CommandAI` 等价判定，还没有完整迁移 Java `CommandAI` path/finishPath/commandQueue 行为，也没有完整 block-level `acceptUnitPayload/acceptPayload` 精度；后续需要把单位命令、路径与 payload building 接入更完整的 AI/entity lifecycle。

### 12.103 UnitEnteredPayload 写入 Java 风格 UnitPayload body

- 2026-05-27：把 `GameRuntime::unit_payload_ref_from_unit(...)` 的首选写入从 `UnitSyncWire` subset 升级为 Java `UnitPayload.write(...)` 风格的 unit body：`revision + unit.write(...)` 字段序列；仅在 class/revision schema 未知或写入失败时才回退到旧 `UnitSyncWire` body。
- Java 依据：
  - `UnitPayload.write(Writes)` 字段顺序为 `payloadUnit(0) -> unit.classId() -> unit.write(write)`；
  - `Payload.read(Reads)` 在 unit 分支读取 `payloadUnit -> classId -> EntityMapping.map(id).get().read(read)`，因此 `PayloadRef::Unit.unit_bytes` 必须能被 Rust 的 exact unit payload reader 按 Java `unit.write` schema 消费。
- Rust 新增/变化：
  - 新增 `GameRuntime::unit_payload_revision_and_schema(class_id)`，把已迁移 class/revision schema 抽成 writer/reader 共用入口；
  - 新增 `GameRuntime::unit_payload_body_from_unit(...)`，按 schema 写入 revision、abilities、坐标/Ammo 特例、controller、elevation、flag、health、mine tile、mounts、payload seq 占位、plans、rotation、shield、items、statuses、team、unit type、updateBuilding、velocity 与最终 x/y；
  - `game_runtime_applies_client_unit_entered_payload_packet_to_payload_building` 现在断言 flare payload `class_id == 3`、body revision 为 `9`，且 `read_exact_unit_payload_body(...)` 可完整消费。
- 验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_entered_payload_packet_to_payload_building`
  - `cargo test -p mindustry-server server_launcher_update_applies_enter_payload_command_to_payload_building_and_broadcasts_packet`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_unit_entered_payload_packet_to_runtime_payload_building`
  - `cargo check --workspace`
- 仍未完成：`Payloads/BuildingPayloads` schema 的内部 payload seq 当前仍写空占位，`Missile/Ammo` 特化字段也只保留最小同步值；后续需要迁移完整 Unit entity `write/read`、payload nesting materialization 和 Java EntityMapping 恢复，而不是长期依赖 sidecar raw bytes。

### 12.104 同步 UnitPayload sort-key schema 到 v158 revision

- 2026-05-27：把 `payload_unit_sort_key(...)` 使用的 `payload_unit_tail_after_type(class_id, revision)` 表与 `GameRuntime` exact UnitPayload reader/writer 的 v158 class/revision schema 对齐，避免 12.103 写出的 Java 风格 body 在 payload-router/sort 场景下只对 flare 生效、对其他已迁移 unit class revision 失配。
- Rust 新增/变化：
  - `core/src/mindustry/world/blocks/payloads/mod.rs` 中 UnitPayload tail 表改为当前已支持的 class/revision：Common、BaseRotation、Payloads、BuildingPayloads、Ammo、Missile；
  - 修正文档注释：UnitType 后面的 tail 是 `updateBuilding + velocity + last x/y`，Missile 额外插入一个 float；
  - 回归测试锁定 `class_id=4 revision=9` 可识别、旧 `revision=6` 不再误判、Missile `class_id=39 revision=3` 使用额外 tail。
- 验证：
  - `cargo test -p mindustry-core payload_router_match_pick_control_and_serialization_follow_java_shell`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_entered_payload_packet_to_payload_building`
  - `cargo check --workspace`
- 仍未完成：sort-key 仍只从 raw body 尾部恢复 UnitType id，不等于完整 materialize `UnitPayload`；后续需要把 payload-router 的 fits/draw/contentEquals 与完整 UnitType/EntityMapping 恢复继续接上。

### 12.105 收紧 UnitPayload attach 的 Java block-level 门禁

- 2026-05-27：修正 `GameRuntime::attach_unit_payload_to_building(...)` 对 Java `build.acceptPayload(build, new UnitPayload(unit))` 的块级语义：不再把 UnitPayload 塞进 `PayloadLoader/PayloadConstructor/PayloadSource`，并补上 `PayloadConveyor/PayloadRouter` 的真实可承载路径。
- Java 依据：
  - `PayloadLoader.acceptPayload(...)` 明确要求 `payload instanceof BuildPayload`，因此不接收 UnitPayload；
  - `BlockProducer` / `PayloadSource` 的 `acceptPayload(...)` 返回 false；
  - `PayloadConveyor.acceptPayload(...)` 要求目标空且 `payload.fits(payloadLimit)`，`source == this` 时不受 enabled/progress 限制；`PayloadRouter` 继承该逻辑并维护 sorted/matches；
  - `PayloadMassDriver.acceptPayload(...)` 要求 `payload.size() <= maxPayloadSize * tilesize`，`PayloadDeconstructor` 还要求目标空、未在 deconstructing 且 fits。
- Rust 新增/变化：
  - `ensure_payload_state_for_building(...)` 现在会为 `payload-conveyor` / `payload-router` 初始化 sidecar；
  - `attach_unit_payload_to_building_state(...)` 改为按 `BlockDef + GameRuntimePayloadBlockState` 分发：Conveyor/Router 走 `payload_conveyor_accept_payload + payload_conveyor_handle_payload`，MassDriver/Deconstructor/Void 各自校验，Loader/Constructor/Source 对 UnitPayload 显式无匹配分支；
  - desktop/server 的 UnitEnteredPayload smoke 从错误的 `payload-loader` 改为 Java 可接受的 `payload-mass-driver`；
  - 新增 core 回归测试：loader 拒收 UnitPayload、conveyor 可接收 UnitPayload。
- 验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_entered_payload_packet_to_payload_building`
  - `cargo test -p mindustry-core game_runtime_rejects_unit_payload_for_payload_loader_like_java`
  - `cargo test -p mindustry-core game_runtime_attaches_unit_payload_to_payload_conveyor_like_java`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_unit_entered_payload_packet_to_runtime_payload_building`
  - `cargo test -p mindustry-server server_launcher_broadcasts_unit_entered_payload_from_runtime_unit_and_building`
  - `cargo test -p mindustry-server server_launcher_update_applies_enter_payload_command_to_payload_building_and_broadcasts_packet`
  - `cargo check --workspace`
- 仍未完成：Deconstructor 仍缺 Java `unit.type.getTotalRequirements().length > 0` 的真实需求表；后续需要补 UnitType requirements 与更多 block-specific `acceptUnitPayload/acceptPayload`。

### 12.106 Reconstructor 接入 UnitPayload 接收链

- 2026-05-27：对照 Java `ReconstructorBuild.acceptUnitPayload(...)` / `acceptPayload(...)`，把 `UnitReconstructor` 这类非 `PayloadBlock` 的 UnitPayload 接收接入 `GameRuntime`，避免 UnitPayload 只能进入 payload-family sidecar。
- Java 依据：
  - `acceptPayload(source, payload)` 要求当前 payload 为空、`enabled || source == this`、来源不是输出侧、payload 是 `UnitPayload`、存在 `from -> to` upgrade、目标未 banned 且已解锁/AI；
  - `InputHandler.unitEnteredPayload(...)` 调用 `build.acceptPayload(build, new UnitPayload(unit))` 后再 `build.handlePayload(...)`；
  - `PayloadSource/Conveyor` 等外部来源转交给 Reconstructor 时仍必须遵守非输出侧输入限制。
- Rust 新增/变化：
  - 新增 `ensure_unit_state_for_building(...)`，为 `BlockDef::UnitReconstructor` 懒创建 `GameRuntimeUnitBlockState::Reconstructor { common, reconstructor }`；
  - `attach_unit_payload_to_building(...)` 现在会把 `UnitReconstructor` 分流到 `unit_runtime_states`，并复用 `PayloadBlockBuildState` common 保存真实 `PayloadRef::Unit`；
  - `transfer_payload_output_to_front(...)` 新增 Reconstructor 目标，`PayloadSource(unit)` / conveyor/router 输出到前方 Reconstructor 时可进入 unit sidecar；
  - 接收校验复用 `reconstructor_accept_payload(...)`，upgrade 关系来自内容注册表；`rules.banned_units` 会拒收目标升级单位。当前在 `rules.researched` 为空时按“host research mirror 尚未知”临时视作已解锁，后续接入完整 campaign/tech unlock 后需要收紧。
- 新增 core 回归测试：
  - `game_runtime_attaches_unit_payload_to_reconstructor_like_java`
  - `game_runtime_rejects_banned_reconstructor_unit_payload_like_java`
  - `game_runtime_payload_source_moves_unit_payload_into_reconstructor`
- 验证：
  - `cargo test -p mindustry-core reconstructor`
  - `cargo test -p mindustry-core game_runtime_payload_source_moves_unit_payload_into_front_payload_conveyor`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_entered_payload_packet_to_payload_building`
  - `cargo check --workspace`
- 仍未完成：Reconstructor 只完成接收/入仓，尚未在 runtime tick 中执行 `moveInPayload()`、progress 推进、完成后替换为升级后 UnitPayload、恢复 command/defaultCommand、消费资源与 `UnitCreateEvent`；完整 UnitPayload materialize 仍是 raw sidecar。

### 12.107 Reconstructor 最小 updateTile runtime tick

- 2026-05-27：对照 Java `ReconstructorBuild.updateTile()`，把 `UnitReconstructor` 从“只接收 UnitPayload”推进到 owned runtime tick：会移动 payload 入仓、推进 progress、达到 `constructTime` 后把 payload 内 unit type 替换成升级目标，并接入 `advance_owned_runtime_blocks(...)` 聚合报告，避免停留在独立 helper。
- Java 依据：
  - `updateTile()` 中 `payload != null && hasUpgrade(payload.unit.type)` 时先 `moveInPayload()`，到位且 `efficiency > 0` 后 `progress += edelta() * state.rules.unitBuildSpeed(team)`；
  - `progress >= constructTime` 后执行 `payload.unit = upgrade(payload.unit.type).create(payload.unit.team())`、`progress %= 1f`、`consume()`、触发创建音效/烟雾/事件；
  - 若当前 payload 不再有可用 upgrade，则走 `moveOutPayload()`，让完成升级后的单位在下一 tick 从该级重构器输出。
- Rust 新增/变化：
  - `PayloadRef::Unit` 新增 `payload_unit_patch_type_id(...)` / `payload_ref_patch_unit_type(...)`，复用 v158 UnitPayload tail schema，只原地替换 raw unit body 中 `UnitType` 的 2 字节 id，保持 payload envelope、class id、其余同步字段不变，利于 Java wire 兼容；
  - 新增 `GameRuntimeUnitReconstructorFrameReport` 与 `GameRuntimeOwnedUnitFrameReport`，`GameRuntimeOwnedFrameReport` 现在包含 `unit.reconstructor`；
  - 新增 `advance_owned_unit_reconstructors(...)` / ticks 入口，按 `consume_items * rules.unitCost(team)` 检查/消费 building items，并使用 `reconstructor_update(...)` 更新 `progress/speed_scl/time`；
  - `transfer_payload_output_to_front(...)` 现在也能把 `UnitReconstructor` 作为 payload source，支持重构完成后无后续 upgrade 的 UnitPayload 向前方 payload block / reconstructor 转交。
- 新增 core 回归测试：
  - `game_runtime_unit_reconstructor_upgrades_payload_on_tick_like_java`
  - `game_runtime_owned_runtime_blocks_includes_unit_reconstructor_tick`
  - `game_runtime_reconstructor_outputs_upgraded_payload_to_front_conveyor`
  - `payload_router_match_pick_control_and_serialization_follow_java_shell` 扩展验证 UnitPayload type id patch 只改目标 2 字节。
- 验证：
  - `cargo test -p mindustry-core reconstructor`
  - `cargo test -p mindustry-core payload_router_match_pick_control_and_serialization_follow_java_shell`
  - `cargo check --workspace`
- 仍未完成：当前升级完成只 patch raw UnitType id，尚未完整重建 Java `UnitType.create(team)` 后的新 unit health/weapon/mount/controller 默认状态；`command_pos/command_id -> UnitCommand` 写回、`UnitCreateEvent`、create sound、shake、Fx.producesmoke、真实 entity materialize/dump 与 renderer/UI 仍需继续迁移。

### 12.108 Reconstructor 升级后写回 Command controller

- 2026-05-27：继续对照 Java `ReconstructorBuild.updateTile()` 完成升级后的命令继承逻辑：Rust 现在会在 UnitPayload 升级时定位 raw unit body 里的 `controller` 段，允许 `Vec::splice` 变长替换为 `ControllerWire::Command`，写入 `command_pos` 与配置命令；未配置命令时回退到升级目标 unit 的 `default_command`。
- Java 依据：
  - `payload.unit = upgrade(payload.unit.type).create(payload.unit.team())` 后，如果 `payload.unit.isCommandable()`，会先写 `commandPosition(commandPos)`，再写 `command == null && payload.unit.type.defaultCommand != null ? payload.unit.type.defaultCommand : command`；
  - `ReconstructorBuild.write/read` 会保存 `commandPos` 和 `command`；
  - `UnitType.init()` 中只有 `mono/poly/mega` 显式 defaultCommand，且内容名分别对应 `mine/rebuild/repair`。
- Rust 新增/变化：
  - `unit_default_command_id(...)` 支持把目标 `UnitType.default_command` 解析到 `UnitCommand` content id，并保留 `*Command` 后缀兼容映射；
  - `unit_payload_controller_bounds(...)` 按当前 v158 UnitPayload schema 跳过 revision/abilities/schema 特化字段后定位 controller 起止范围；
  - `payload_ref_patch_unit_controller(...)` 用 `type_io::write_controller(...)` 生成新 controller bytes 并替换原 controller 段，避免错误假设 controller 有固定 offset；
  - 修正 `content/unit_types.rs` 的默认命令基线：`mono=mine`、`poly=rebuild`、`mega=repair`，移除 alpha/beta/gamma/evoke/incite/emanate 的误填 default_command。
- 新增 core 回归测试：
  - `game_runtime_reconstructor_applies_default_command_to_upgraded_payload`
  - `unit_core_properties_match_upstream_subset` 增加默认命令断言。
- 验证：
  - `cargo test -p mindustry-core reconstructor`
  - `cargo test -p mindustry-core unit_core_properties_match_upstream_subset`
  - `cargo check --workspace`
- 仍未完成：当前 controller 写回只覆盖 Reconstructor 升级时的 command/defaultCommand 最小分支；Java `isCommandable()`/`commands.contains(command)` 过滤只具备基于当前已迁移 commands 列表的最小能力，尚未完整实现 Payloadc/Weapon 派生 commands、完整 `UnitType.create(team)` 重建 health/mount/weapon 默认值，以及 UnitCreateEvent/effect/sound 联机广播。

### 12.109 UnitFactory 最小 updateTile runtime tick

- 2026-05-27：对照 Java `UnitFactoryBuild.updateTile()`，把 `UnitFactory` 从仅有状态读写/纯 helper 推进到 owned runtime tick，并接入 `advance_owned_runtime_blocks(...).unit.factory` 聚合报告。
- Java 依据：
  - `created()` 会在 `currentPlan == -1` 时自动选择第一个 `plans.indexOf(u -> u.unit.unlockedNow())`，若没有已解锁 plan 则保持 `-1`；
  - `updateTile()` 会在不可配置 factory 上强制 `currentPlan = 0`，修正越界 plan，`efficiency > 0 && currentPlan != -1` 时推进 `progress/time/speedScl`；
  - 每 tick 先 `moveOutPayload()`，然后在 `currentPlan != -1 && payload == null` 时检查 plan unit banned，`progress >= plan.time` 后创建 unit payload、清零 `payVector`、`consume()` 并触发 `UnitCreateEvent`；
  - `shouldConsume()` 要求 `currentPlan != -1 && enabled && payload == null && team.activateUnitFactories()`，动态消耗按当前 plan requirements 和 `rules.unitCost(team)` 缩放。
- Rust 新增/变化：
  - `ensure_unit_state_for_building(...)` 现在会为 `BlockDef::UnitFactory` 懒创建 `GameRuntimeUnitBlockState::Factory { common, factory }`；
  - `add_placed_building(...)` 会在真实放置路径初始化 unit block sidecar；UnitFactory 初始 `current_plan` 优先采用已有 `BuildingComp.config == Int(plan)`，否则按 Java `created()` 选择首个 `unit_unlocked_now(...)` 的 plan，避免新放置 factory 长期停在空配置；
  - 新增 `GameRuntimeUnitFactoryFrameReport`，并把 `GameRuntimeOwnedUnitFrameReport` 拆为 `factory + reconstructor`；
  - 新增 `advance_owned_unit_factories(...)` / ticks：检查当前 plan requirements、调用 `unit_factory_update(...)` 推进进度，完成时用 `UnitComp::new(...).to_sync_wire()` 生成 Java-style `PayloadRef::Unit`，写入 factory command/defaultCommand controller，消耗 items；
  - 2026-05-27 继续补 Java `Team.activateUnitFactories()` 门控：`Rules::unit_factory_active(team_id, tick)` 现在读取 team rule 的 `unit_factory_activation_delay`，`advance_owned_unit_factories_ticks(...)` 在未到激活 tick 时保持 `moveOutPayload()` 类行为但把生产效率置 0，避免敌方/攻击图 factory 在延迟期提前推进进度或产出 payload；
  - 2026-05-27 继续对齐 Java `updateTile()` 顺序：factory payload 到达输出侧后立即尝试转移到前方 payload 目标，若转移成功则同一 tick 继续检查 `progress >= plan.time` 并生产下一份 payload，不再把“moveOut 后再生产”延迟到下一帧；
  - 2026-05-27 继续接入 Java `acceptItem(...)` / `getMaximumAccepted(...)`：`dump_target_accepts_item(...)` 现在识别 `BlockDef::UnitFactory`，按当前 plan、plan requirements、`capacities[item] * rules.unitCost(team)` 和 `UnitFactoryState.current_plan`/`BuildingComp.config` 回退判断是否接收物品，使 conveyor/router 等真实物品流能给 UnitFactory 补料，而不是只靠测试手写 item storage；
  - `transfer_payload_output_to_front(...)` 现在也能把 `UnitFactory` 作为 payload source，允许 factory 输出到前方 payload conveyor/router/reconstructor 等真实目标。
- 新增 core 回归测试：
  - `game_runtime_unit_factory_produces_configured_unit_payload`（通过 `advance_owned_runtime_blocks` 验证聚合入口）
  - `game_runtime_unit_factory_respects_team_activation_delay_like_java`
  - `game_runtime_unit_factory_produces_same_tick_after_payload_moves_out_like_java`
  - `game_runtime_unit_factory_accepts_only_current_plan_items_like_java`
  - `game_runtime_unit_factory_outputs_payload_to_front_conveyor`
  - `game_runtime_unit_factory_created_selects_first_unlocked_plan_like_java`
- 验证：
  - `cargo test -p mindustry-core unit_factory`
  - `cargo check --workspace`
- 仍未完成：当前 UnitFactory runtime 尚未完整实现创建 sound/effect/event 以及真实客户端 UI 渲染按钮接入。

### 12.110 UnitFactory 配置入口接入

- 2026-05-27：对照 Java `UnitFactory` 构造器中的 `config(Integer.class, ...)` 与 `config(UnitType.class, ...)`，在 `GameRuntime` 接入 owned building 的 unit factory 计划配置入口，避免 UnitFactory 只能靠测试手写 `current_plan` 才能运行。
- Java 依据：
  - `config(Integer)` 在 `!configurable` 时直接返回；plan 越界或负数会把 `currentPlan` 置为 `-1`，plan 变化时清零 `progress`；
  - `config(UnitType)` 会按 `plans.indexOf(p -> p.unit == val)` 映射 plan，未命中时等价清空当前 plan；
  - plan/unit 变化后，如果 `command != null` 且新 `unit()` 为空或 `!unit.commands.contains(command)`，Java 会把 `command` 清空，避免旧命令继承到不支持该命令的新造单位；
  - `config()` 返回当前 plan integer，联机/回滚侧需要保存为 Java-like tile config。
- Rust 新增/变化：
  - 新增 `GameRuntimeUnitFactoryConfigureResult`，区分 `Configured / Cleared / MissingBuilding / MissingRuntimeState / NotUnitFactory / NotConfigurable / UnknownUnit / UnknownCommand / UnsupportedValue`，并提供 `changed()` 供 server 分发判断；
  - 新增 `GameRuntime::configure_owned_unit_factory_plan(...)`，按 tile 找 building，懒创建 `GameRuntimeUnitBlockState::Factory`，调用 `unit_factory_configure_plan(...)`，并把 `BuildingComp.config` 写成 `TypeValue::Int(current_plan)` 或清空；
  - 新增 `GameRuntime::configure_owned_unit_factory_unit(...)`，按 `ContentId` 映射到 factory plan；未命中当前 factory plans 时清空 plan，且先检查 block 是否为可配置 UnitFactory，贴近 Java `if(!configurable) return` 的顺序；
  - `configure_owned_unit_factory_plan(...)` 现在会在切换到无效 plan 或切换到已填充 `UnitType.commands` 且不包含当前 `UnitCommand` 的目标 unit 时清空 `command_id`；在当前 unit commands 列表仍未完整迁移/为空时保守保留命令，避免因为内容基线缺口提前清掉合法配置；
  - `content/unit_types.rs` 的低耦合 init 现在按 Java `UnitType.init()` 的默认命令主体规则填充 `commands`：默认 `move`，允许载荷则加 `enterPayload`，可 boost/flying 且能建造则加 `rebuild/assist`，可采矿则加 `mine`，可治疗则加 `repair`；Java `example instanceof Payloadc` 已按 v158 `@EntityDef(... Payloadc.class)` 名单为 `mega/quad/oct/evoke/incite/emanate/quell/disrupt` 补 `loadUnits/loadBlocks/unloadPayload/loopPayload`；显式 `default_command` 会保证也出现在 commands 中，缺省 default 会回退到第一条命令；
  - 新增 `GameRuntime::configure_owned_unit_factory_command(...)`，实现 Java `config(UnitCommand.class, ...)` / `configClear(...)`：只写 `UnitFactoryState.command_id` 或清空命令，不修改 `current_plan`、`progress` 或 `BuildingComp.config`（Java `config()` 仍返回当前 plan）；
  - 新增 `GameRuntime::configure_owned_unit_factory_value(...)`，统一分发 `TypeValue::Int(plan)`、`Content(Unit)`、`Content(UnitCommand)` 与 `Null`，供网络入口直接复用；
  - `ServerLauncher::apply_server_tile_config_packet(...)` 现在会识别 `BlockDef::UnitFactory`，把客户端 `TileConfigCallPacket` 分派到 unit factory value 入口，并把成功变更以 server 形态 `TileConfigCallPacket` 可靠转发给已连接客户端；
  - `DesktopLauncher::sync_tile_config_to_runtime(...)` 现在也会按目标 block kind 分派 tile config：`UnitFactory` 的 `Content(UnitCommand)` / `Null` 会回填客户端 `GameRuntimeUnitBlockState::Factory.command_id`，不再只支持 `UnitCargoUnloadPoint`；
  - `mindustry::input` 新增 `client_unit_factory_plan_config_packet(...)`、`client_unit_factory_unit_config_packet(...)`、`client_unit_factory_command_config_packet(...)` 与 `client_unit_factory_clear_command_packet(...)`，分别生成 Java `UnitFactory.config(Integer.class, ...)`、`config(UnitType.class, ...)`、`config(UnitCommand.class, ...)` 与 `configClear` 对应的客户端 `TileConfigCallPacket` 形态；
  - 2026-05-27 继续补 Java `UnitFactoryBuild.senseObject(LAccess.config)`：`GameRuntime::sense_owned_building_object(...)` 在 `LAccess::Config` 且目标为 `UnitFactory` 时读取当前 `UnitFactoryState.current_plan`，返回对应 `TypeValue::Content(ContentType::Unit, unit_id)`；`current_plan == -1` 返回 `TypeValue::Null`；缺失 sidecar 时会从 `BuildingComp.config == Int(plan)` 恢复同等可感知结果；
  - 同步新增 `GameRuntime::sense_owned_building_logic_object(...)`，把 `TypeValue::Content` 转为逻辑对象名（如 `@mono`），为后续真实 processor/link runtime 接入提供非孤立入口；
  - 继续补 Java `UnitFactoryBuild.sense(LAccess.progress/itemCapacity)`：`GameRuntime::sense_owned_building_number(...)` 对 UnitFactory 返回 clamp 后的 `fraction()` 与 `round(itemCapacity * rules.unitCost(team))`，并在无 plan/无 sidecar 时按 Java 空进度语义回退；
  - 2026-05-27 继续补 Java `UnitFactoryBuild.buildConfiguration()` / `getPlanConfigs()` / `canSetCommand()` 的 runtime 候选计划：新增 `GameRuntimeUnitFactoryConfigurationPlan`、`GameRuntimeUnitFactoryConfigUnitOption`、`GameRuntimeUnitFactoryConfigCommandOption` 与 `GameRuntime::unit_factory_configuration_plan(...)`，同一个 helper 同时输出 UI 可见单位候选（`unlockedNow && !isBanned`）、`getPlanConfigs()` 候选（只排除 banned）、当前输出单位的 command 按钮候选、默认命令选中态、显式命令选中态，以及 Java 默认 `selectionRows/selectionColumns` 与 command columns；
  - 新增 `GameRuntime::unit_factory_can_set_command(...)`，按 Java 规则要求 `commands.len() > 1 && allowChangeCommands`，并隐藏“仅 `move + enterPayload`”的标准单位命令行；该 helper 只负责展示/候选判断，真实配置仍走 `configure_owned_unit_factory_value(...) -> configure_owned_unit_factory_*`，避免做成孤立 UI 数据模块；
  - 该入口直接修改真实 runtime sidecar 与 building config，后续可继续接入 `TileConfigCallPacket`/输入处理，而不是独立 helper。
- 新增 core 回归测试：
  - `game_runtime_configures_unit_factory_plan_and_clears_progress_like_java`
  - `game_runtime_configures_unit_factory_by_unit_id_like_java_unit_config`
  - `game_runtime_configures_unit_factory_command_and_clear_like_java`
  - `game_runtime_configures_unit_factory_value_for_plan_unit_command_and_clear`
  - `game_runtime_senses_unit_factory_config_as_current_plan_unit_like_java`
  - `game_runtime_senses_unit_factory_progress_and_item_capacity_like_java`
  - `game_runtime_unit_factory_configuration_plan_filters_unlocked_unbanned_units_like_java`
  - `game_runtime_unit_factory_configuration_plan_lists_changeable_commands_like_java`
  - `game_runtime_unit_factory_configuration_plan_hides_enter_payload_only_standard_commands_like_java`
  - `game_runtime_clears_incompatible_unit_factory_command_when_plan_changes_like_java`
  - `game_runtime_rejects_unit_factory_config_for_wrong_or_unconfigurable_blocks`
  - `client_unit_factory_config_packets_cover_plan_unit_command_and_clear_values`
  - `unit_core_properties_match_upstream_subset` 现在断言 `mono/poly/mega` 的 commands/default command、Payloadc unit 的 payload command，以及普通 core unit 的 Java-style fallback default；
- 新增 server 回归测试：
  - `server_update_applies_unit_factory_command_tile_config_and_forwards_to_clients`
- 新增 desktop/input 回归测试：
  - `desktop_launcher_syncs_unit_factory_command_tile_config_packet_to_runtime`
  - `client_unit_factory_command_config_packets_use_unit_command_content_and_clear_null`
- 验证：
  - `cargo test -p mindustry-core unit_factory`
  - `cargo test -p mindustry-core unit_types --lib`
  - `cargo test -p mindustry-server unit_factory --lib`
  - `cargo test -p mindustry-desktop unit_factory --lib`
  - `cargo test -p mindustry-core client_unit_factory --lib`
  - `cargo check --workspace`
- 仍未完成：`UnitType.init()` 中 weapons-derived `canHeal` 仍依赖后续完整 Weapon/Bullet 初始化；真实 desktop UI 选择表仍需后续闭环；logic processor 还需要把 `sense_owned_building_logic_object(...)` 接到真实 linked building 对象刷新/执行器生命周期里，避免只靠测试手工注册对象。

### 12.111 UnitAssembler 最小 owned runtime tick

- 2026-05-27：对照 Java `UnitAssemblerBuild.updateTile()`，把 `UnitAssembler` 从内容/状态/序列化 helper 推进到 `advance_owned_runtime_blocks(...).unit.assembler` 的真实 owned runtime 路径。
- Java 依据：
  - `updateTile()` 先处理 tier 变化、payload `moveInPayload()` 后 `yeetPayload(payload)` 累加到 `blocks`，再按当前 tier clamp 到 plan；
  - `progress += edelta() * state.rules.unitBuildSpeed(team) * eff / plan.time`，达到 1 后 `spawned()`，随后 `consume()` 并清空 `blocks`；
  - 动态 consume 来源包括 `ConsumePayloadDynamic`、`ConsumeItemDynamic`、`ConsumeLiquidsDynamic`，并通过 `state.rules.unitCost(team)` 缩放。
- Rust 新增/变化：
  - `ensure_unit_state_for_building(...)` 现在会为 `BlockDef::UnitAssembler` 懒创建 `GameRuntimeUnitBlockState::Assembler { common, assembler }`，并为 `BlockDef::UnitAssemblerModule` 创建 terminal `PayloadBlockBuildState`；
  - 新增 `GameRuntimeUnitAssemblerFrameReport`，并把 `GameRuntimeOwnedUnitFrameReport` 扩展为 `factory + reconstructor + assembler`；
  - 新增 assembler payload/item/liquid requirement 解析与消费：payload requirements 按 `PayloadContentSpec::{Unit,Block}` 映射为 `PayloadKey`，数量按 `rules.unitCost(team)` 缩放；
  - `advance_owned_unit_assemblers_ticks(...)` 已接入 `advance_owned_runtime_blocks`：先把已抵达 `PayloadBlockBuildState.common.payload` 的 payload 转入 `assembler.blocks`，再调用 `unit_assembler_update_progress(...)`，完成时调用 `unit_assembler_spawned(...)`、消费 payload/items/liquids，并上报完成计数；
  - 当前实现是接入主 runtime 的过渡闭环，不是独立 helper；后续 drone/模块/单位实体链路会在同一路径继续补齐。
- 新增 core 回归测试：
  - `game_runtime_owned_runtime_blocks_includes_unit_assembler_tick_like_java`：验证 `tank-assembler` 在 owned runtime 中接收最后一个 `stell` payload、满足 `PayloadSeq` 需求、完成进度、消费 payload requirements 并清零 progress。
- 验证：
  - `cargo test -p mindustry-core unit_assembler`
  - `cargo check --workspace`
- 仍未完成：真实 `AssemblerAI`/`BuildingTether` drone 创建与同步、`UnitAssemblerModule.findLink()`/module→assembler payload 转运、`checkSolid`/spawn rect 占位判定、最终 `UnitType.create(team)` 加入 world/目标 build payload、sound/Fx/Event/Call 联机广播、`shouldConsume()` 与 `team.activateUnitFactories()` 的精确语义仍待后续闭环。

### 12.112 UnitAssembler payload 输入链路接入

- 2026-05-27：对照 Java `UnitAssemblerBuild.acceptPayload(...)`，把 `UnitAssembler` 接入 `transfer_payload_output_to_front(...)` 的真实 payload 目标分支，使 payload source/conveyor 等 runtime 输出可以把需求 payload 送进 assembler common payload，再由 assembler tick 转入 `blocks`。
- Java 依据：
  - `acceptPayload(source, payload)` 要求当前 plan 的 `requirements` 包含该 payload content；
  - 非 module source 要求 assembler 自身 payload slot 为空；module source 可按同类 payload 已占位的情况扣减 1 个 pending payload；
  - stored 数量按 `Mathf.round(requirement.amount * state.rules.unitCost(team))` 判断。
- Rust 新增/变化：
  - `transfer_payload_output_to_front(...)` 新增 `TargetKind::Assembler`，目标为 `BlockDef::UnitAssembler` 时会懒创建 `GameRuntimeUnitBlockState::Assembler`；
  - target accept 侧复用 `unit_assembler_accept_payload(...)`，并将 `PayloadRef` 解析为 `PayloadKey` 后按当前 tier plan requirement、`rules.unitCost(team)` 与 `assembler.blocks` 存量判断；
  - source take/restore 侧补 `GameRuntimeUnitBlockState::AssemblerModule(common)`，为后续 module→assembler 真实转运留出同一条 transfer 路径；
  - 成功转运后通过 `payload_block_handle_payload(...)` 写入 assembler `PayloadBlockBuildState.common`，随后同帧 assembler tick 可 `moveInPayload` 并计入 `blocks`。
- 新增 core 回归测试：
  - `game_runtime_payload_source_feeds_unit_assembler_requirement_in_owned_runtime`：验证 `payload-source` 生成 `stell` UnitPayload 后转入前方 `tank-assembler`，同帧进入 `blocks`、满足 plan payload requirements、完成 unit assembler tick 并消费 payload batch。
- 验证：
  - `cargo test -p mindustry-core unit_assembler`
  - `cargo check --workspace`
- 仍未完成：`UnitAssemblerModule.findLink()` 的空间搜索/链接维护、module 自身每帧 moveIn 后触发 `transfer_payload_output_to_front`、assembler payload slot 非空时 module 同类 payload 的覆盖/合并细节、最终 unit 实体落地与网络 `Call.assemblerUnitSpawned/assemblerDroneSpawned` 仍需补。

### 12.113 UnitAssemblerModule 最小链接与转运 tick

- 2026-05-27：对照 Java `UnitAssemblerModuleBuild.findLink()` / `updateTile()`，把 `basic-assembler-module` 从仅有 terminal payload common 状态推进到 owned runtime 中能寻找相邻 `UnitAssembler` 并转运 payload 的最小闭环。
- Java 依据：
  - `findLink()` 通过 `getLink(team, tile.x, tile.y, rotation)` 找同队 `BlockFlag.unitAssembler`，并调用 assembler 的 `moduleFits(...)`；
  - `updateTile()` 在 `moveInPayload()` 成功、link 仍匹配、`!link.wasOccupied`、`link.acceptPayload(this,payload)` 且 `efficiency > 0` 时，把 payload 交给 link 的 `yeetPayload(payload)` 并清空自身 payload；
  - `moduleFits(...)` 以 assembler spawn rect、module rotation、areaSize 边界为几何约束。
- Rust 新增/变化：
  - 新增 `GameRuntimeUnitAssemblerModuleFrameReport`，并把 `GameRuntimeOwnedUnitFrameReport` 扩展出 `assembler_module`；
  - 新增 `assembler_module_fits(...)` 与 `find_owned_unit_assembler_link_for_module(...)`，按同队 assembler、module rotation、assembler area 边界寻找 link；由于当前 Rust `BuildingComp` 中心坐标模型仍在迁移中，几何判定使用半 tile 容差以匹配现有 world/tile 表示；
  - 新增 `advance_owned_unit_assembler_modules_ticks(...)`，在 `advance_owned_runtime_blocks` 中先于 assembler tick 执行：module payload 到达后复用 `assembler_module_transfer_payload(...)` 与 `transfer_payload_output_to_front(...)` 进入 linked assembler common payload；
  - `transfer_payload_output_to_front(...)` 的 source take/restore 已支持 `GameRuntimeUnitBlockState::AssemblerModule(common)`，确保转运失败时 payload 可回滚回 module。
- 新增 core 回归测试：
  - `game_runtime_assembler_module_transfers_payload_into_linked_assembler`：自动搜索一个 fits 的 module 位置，验证 module 中的 `stell` UnitPayload 同帧进入 linked `tank-assembler`、由 assembler tick 转入 `blocks` 并完成一次组装消费。
- 验证：
  - `cargo test -p mindustry-core assembler_module`
  - `cargo test -p mindustry-core unit_assembler`
  - `cargo check --workspace`
- 仍未完成：`UnitAssemblerBuild.modules` 持久列表、tier 连续性由 module 集合自动驱动 `currentTier`、严格 Java 坐标/offset 级 `moduleFits`、module link 的 world.tileChanges cache、被拆除时 removeModule、assembler payload slot 非空且同类 module payload 的精确合并行为仍需补。

### 12.114 UnitAssembler currentTier 由 linked modules 驱动

- 2026-05-27：继续对照 Java `UnitAssemblerBuild.updateModules/removeModule/checkTier()`，在 owned runtime 中让 linked `UnitAssemblerModule` 的 tier 自动驱动 assembler `current_tier`，避免高阶 plan 只能靠测试或存档手写状态。
- Java 依据：
  - `updateModules(build)` 会 `modules.addUnique(build)` 后 `checkTier()`；
  - `checkTier()` 按 module tier 排序，只有连续 tier（`max` 或 `max+1`）会推进 `currentTier`，有 gap 时停止；
  - `plan()` 使用 `plans.get(Math.min(currentTier, plans.size - 1))`。
- Rust 新增/变化：
  - 新增 `linked_module_tiers_for_assembler(...)`，扫描同队且 `assembler_module_fits(...)` 的 module，收集 `UnitAssemblerModuleBlockData.tier`；
  - `advance_owned_unit_assemblers_ticks(...)` 在选择 plan 前调用 `unit_assembler_current_tier(...)`，同步写回 `assembler.current_tier` 并通过 `GameRuntimeUnitAssemblerFrameReport.tier_updates` 上报变化；
  - `transfer_payload_output_to_front(... TargetKind::Assembler ...)` 在 module source 转运时会按实时 linked modules 计算 effective tier，确保 module payload 能按即将生效的高阶 plan requirement 被接受。
- 新增 core 回归测试：
  - `game_runtime_linked_assembler_module_updates_assembler_tier`
  - 已调整 `game_runtime_assembler_module_transfers_payload_into_linked_assembler` 覆盖 basic module 驱动 `tank-assembler` 使用 tier 1 plan（`conquer` 的 `locus + carbide-wall-large` requirements）。
- 验证：
  - `cargo test -p mindustry-core assembler_module`
  - `cargo test -p mindustry-core unit_assembler`
  - `cargo check --workspace`
- 仍未完成：module 集合本身还未作为 Java `modules` 列表持久保存在 assembler state；当前每 tick 扫描 world building。后续仍需补拆除/旋转/世界 tileChanges cache、严格 offset 几何与多 tier/gap 组合测试。

### 12.115 UnitCargoLoader / UnitCargoUnloadPoint 最小 owned runtime tick

- 2026-05-27：对照 Java `UnitCargoLoader.UnitTransportSourceBuild.updateTile()` 与 `UnitCargoUnloadPointBuild.updateTile()`，把 unit cargo 两个 distribution block 从读写/helper 推进到 `advance_owned_runtime_blocks(...).item_transport` 主链路。
- Java 依据：
  - `UnitCargoLoader`：`warmup/readyness` approach，`unit == null && Units.canCreate(team, unitType)` 时 `buildProgress += edelta()/unitBuildTime`，满 1 后 server 创建 `manifold` tether unit 并 `Call.unitTetherBlockSpawned`；
  - `UnitCargoUnloadPoint`：未满容量或 `dumpAccumulate()` 成功时清 `staleTimer/stale`；满容量且持续 `staleTimeDuration` 后 `stale = true`；
  - loader `write/read` 只同步 tether unit id，unload `write/read` 同步 configured item 与 stale bool。
- Rust 新增/变化：
  - `GameRuntimeOwnedItemTransportFrameReport` 新增 `unit_cargo_loader_built_units`、`unit_cargo_unload_dumped_items` 与 `unit_cargo_unload_stale_points`；
  - `advance_owned_item_transport_blocks_ticks(...)` 末尾调用 `advance_owned_unit_cargo_blocks_ticks(...)`，因此 unit cargo 由 `advance_owned_runtime_blocks` 和服务端 owned runtime 同一路径驱动，不是孤立 helper；
  - loader 分支懒创建/修正 `GameRuntimeDistributionBlockState::UnitCargoLoader`，调用 `unit_cargo_loader_update(...)`，到点后调用 `unit_cargo_loader_spawned(...)` 并把 `has_unit` 置 true（真实 `UnitType.create(team)`/BuildingTether 实体仍待补）；
  - unload 分支懒创建/修正 `GameRuntimeDistributionBlockState::UnitCargoUnload`，复用 runtime 通用 `dump_accumulated_items_from_building(...) -> dump_one_item_from_building(...) -> dump_item_to_target(...)` 链路执行 Java `dumpAccumulate()` 等价邻接输出，再按 dump 后 item total、capacity、真实 `dumped` 与 `stale_time_duration` 调用 `unit_cargo_unload_update(...)`。
- 新增 core 回归测试：
  - `game_runtime_owned_runtime_blocks_advances_unit_cargo_loader_build`
  - `game_runtime_owned_runtime_blocks_marks_unit_cargo_unload_stale`
  - `game_runtime_unit_cargo_unload_point_dumps_item_to_adjacent_router`
- 新增 server-level smoke，证明 unit cargo 不是孤立 helper，而是由真实 `ServerLauncher::update()` 经 `advance_owned_runtime_blocks(...).item_transport` 驱动并缓存到 `last_runtime_item_transport_report`：
  - `server_update_drives_owned_unit_cargo_loader_from_launcher_runtime`
  - `server_update_drives_owned_unit_cargo_unload_stale_from_launcher_runtime`
  - `server_update_drives_owned_unit_cargo_unload_dump_from_launcher_runtime`
- 2026-05-27：继续把 loader spawn 从纯计数推进到最小 server 物化/同步链路。`ServerLauncher::update()` 会在 owned runtime tick 前记录尚未持有 unit 的 `unit-cargo-loader` tile，tick 后按新完成的 loader 创建 server-side `manifold` `UnitComp`，写回 `UnitCargoLoaderState.read_unit_id`，并在网络已开启时可靠广播 `UnitTetherBlockSpawnedCallPacket { tile, id }`：
  - 新增 `server_update_broadcasts_unit_tether_block_spawned_for_owned_unit_cargo_loader`
  - 这仍是最小 server sidecar，尚未把 `BuildingTetherComp` 正式并入 `UnitComp`，也未补客户端 apply packet 后回填 loader state。
- 2026-05-27：补上客户端半链路。`NetClient` 现在专门记录 `UnitTetherBlockSpawnedCallPacket` 与 seen cursor；`DesktopLauncher::update()` 通过 `sync_unit_tether_block_spawned_to_runtime()` 把 packet 桥接到 `GameRuntime::apply_client_unit_tether_block_spawned_packet(...)`，对客户端 runtime 中的 `UnitCargoLoaderState` 执行 Java `spawned(id)` 等价行为（清 `build_progress`、写 `read_unit_id`，不伪造本地 unit 对象）：
  - 新增 `game_runtime_applies_client_unit_tether_block_spawned_packet_to_unit_cargo_loader`
  - 新增 `update_records_unit_tether_block_spawned_packet_for_runtime_bridge`
  - 新增 `desktop_launcher_syncs_unit_tether_block_spawned_packet_to_runtime`
- 2026-05-27：新增真实联机 smoke `real_server_desktop_unit_cargo_loader_tether_spawn_syncs_to_client_runtime`，用真实 `ServerLauncher -> ArcNetProvider -> DesktopLauncher` 链路验证：server owned runtime 里的 `unit-cargo-loader` 完成构建后创建 server-side `manifold`，可靠发出 `UnitTetherBlockSpawnedCallPacket`，desktop `NetClient` 收到并由 `DesktopLauncher` 桥接到客户端 runtime，最终 `UnitCargoLoaderState.read_unit_id` 与 server 侧生成 id 对齐。
- 2026-05-27：补齐 `UnitCargoUnloadPoint.dumpAccumulate()` 的真实运行链路。owned runtime 中 unload point 现在会按 Java `BuildingComp.dumpAccumulate()` 语义累积 `dump_accum`，通过既有邻接输出/acceptItem 路径把物品倒入相邻 router/storage/distribution 等目标；只要成功 dump 就清 `staleTimer/stale`，并通过 `unit_cargo_unload_dumped_items` 上报。server smoke 已验证该行为可由 `ServerLauncher::update()` 驱动，不是孤立 helper。
- 2026-05-27：补齐 `UnitCargoUnloadPoint` 的 item config 最小联机链路。`GameRuntime::configure_owned_unit_cargo_unload_value(...)` 现在接受 `TypeValue::Content(Item)` / `TypeValue::Null`，同步更新 building `config` 与 `UnitCargoUnloadPointState.item_id`；`ServerLauncher` 会把客户端 `TileConfigCallPacket` 分派到该 runtime 入口，并可靠转发 server 形态 `TileConfigCallPacket` 给已连接客户端；`DesktopLauncher` 会消费 `NetClient` 记录的 tile config 包并回填客户端 runtime。
  - 新增 `game_runtime_configures_unit_cargo_unload_point_item_value`
  - 新增 `server_update_applies_unit_cargo_unload_tile_config_and_forwards_to_clients`
  - 新增 `desktop_launcher_syncs_unit_cargo_unload_tile_config_packet_to_runtime`
- 2026-05-27：补齐 `UnitCargoLoader` 构建对真实资源门控的最小接入。loader tick 现在会读取 owned power graph 写回的 `power.status`，并按 `consumeLiquid(nitrogen, 10f / 60f)` 的 Java 配置用现有 liquid consumer helper 计算有效效率；缺电或缺 nitrogen 时不推进 `buildProgress`，资源满足时按 `edelta` 推进并消耗 nitrogen。真实 server->desktop tether spawn smoke 已补充 power-source/nitrogen 基线，避免只靠手写 `building.efficiency` 通过。
  - 新增 `game_runtime_unit_cargo_loader_stalls_without_power_or_nitrogen`
  - 调整 `game_runtime_owned_runtime_blocks_advances_unit_cargo_loader_build`、server unit cargo loader smoke 与真实联机 smoke 使用 power-source + nitrogen 复现 Java consume 链。
- 2026-05-27：开始接入 `CargoAI` 的真实单位往返装卸链路，而不是只保留 loader/unload 两个孤立 block runtime。`UnitComp` 新增 `CargoAiRuntimeState` 与 `UnitControllerState::Cargo`，`ServerLauncher` 在 `unit-cargo-loader` 生成 `manifold` 时写入 tether tile/controller，并在每次 server owned runtime update 后驱动 cargo unit：
  - 空载 cargo unit 从 tether loader 的真实 `ItemModule` 里按数量优先选择物品，目标来自同队、已配置同物品的 `UnitCargoUnloadPoint`，优先非 stale；
  - pickup 复用现有 `take_items(...)`，会真实扣 loader 库存、写入 `UnitComp.items`，并广播 Java 兼容的 `TakeItemsCallPacket` / `TransferItemEffectCallPacket`；
  - 载货 cargo unit 复用 `transfer_item_to(...)` 向匹配 unload point 入库，并广播 `TransferItemToCallPacket`，因此链路接入 server runtime/entity/network，而不是新增孤立 helper；
  - 新增 `server_update_drives_spawned_unit_cargo_ai_between_loader_and_unload_point`，覆盖 loader 库存 -> `manifold` -> unload point 库存的两 tick 闭环，以及对应 packet 发送。
- 2026-05-27：补上客户端 cargo unit 最小物化。`GameRuntime::apply_client_unit_tether_block_spawned_packet(...)` 不再只写 `UnitCargoLoaderState.read_unit_id`，还会用同一 loader building 的 team/坐标在 `client_unit_snapshot_entities` 中创建或更新 `manifold` `UnitComp`，并写入 `UnitControllerState::Cargo` 与 `CargoAiRuntimeState { tether_tile_pos }`；`DesktopLauncher` 的 tether packet 同步测试与真实 server→desktop smoke 均断言该客户端 unit snapshot 存在。这样后续 `TakeItemsCallPacket` / `TransferItemToCallPacket` 经 `NetClient` item mirror 后有真实客户端单位落点，而不是只停在 packet cursor。
- 2026-05-27：继续把 cargo transfer 的客户端接收端接入 runtime。`GameRuntime::apply_client_building_item_storage_mirror(...)` 会把 `NetClient.building_storage_mirrors` 中已知 item 数量写回对应 runtime building 的 `ItemModule`；`DesktopLauncher::update()` 新增 `sync_building_storage_mirrors_to_runtime()`，先于 unit item mirror 应用 building item mirror。新增真实 server→desktop cargo transfer smoke，验证 `unit-cargo-loader` 生成 `manifold` 后通过 `TakeItemsCallPacket` + `TransferItemToCallPacket` 把 copper 从 loader 搬到 unload point，desktop runtime 中的 materialized cargo unit 最终清空，unload point 库存同步为 12。
- 2026-05-27：补齐 cargo tether unit 的最小消失同步。`ServerLauncher` 在 tether loader 缺失、失效或 team 不一致时会清理 `UnitCargoLoaderState`、移除 `server_units` 中的 cargo `manifold`，并在网络开启时广播 `UnitDespawnCallPacket(UnitRef::Unit { id })`；`GameRuntime::apply_client_unit_despawn_packet(...)` 会从 `client_unit_snapshot_entities` 移除物化 unit，`DesktopLauncher::sync_unit_lifecycle_to_runtime()` 消费 `NetClient` 的 unit lifecycle cursor，把 server despawn 接到客户端 runtime。这个闭环接入真实 server entity lifecycle、network packet 与 desktop runtime，而不是只删除测试 sidecar。
- 2026-05-27：继续对照 Java `CargoAI.updateMovement()`，把 cargo unit 的 server-side 状态机从“每 tick 即时找点并转移”推进到带目标记忆、`dropSpacing`、`emptyWaitTime` 与重配置防护的最小等价实现。`CargoAiRuntimeState` 新增 `drop_timer`；`ServerLauncher::tick_runtime_unit_cargo_ai_for_loader(...)` 现在会保留当前 `unload_target_tile_pos`，目标被重新配置/拆除/换队时只清目标不清货物；目标满载时按 `dropSpacing = 90` 累计 `no_dest_timer`，超过 `emptyWaitTime = 120` 后用 `targetIndex` 轮转到下一个同物品 unload point。真实转移仍复用 `transfer_item_to(...)` 与 network packet，因此继续接在 server/runtime/entity/network 链路上。
- 2026-05-27：把 Java `BuildingTetherComp.update()` 从独立 helper 推进到 cargo unit 的正式实体生命周期。`BuildingTetherRef` 现在携带 `tile_pos`，`UnitComp` 新增 `building_tether`；server 生成 cargo `manifold` 与 client materialize tether packet 时都会写入同队有效 building tether。`ServerLauncher::tick_runtime_unit_cargo_ai_for_loader(...)` 每 tick 用 loader live building 刷新 tether ref，并通过 `BuildingTetherComp::update()` 判断 despawn，因此 loader 被拆、失效或换队都会走统一 `UnitDespawnCallPacket` 链路，而不是只靠散落的 team 特判。
- 2026-05-27：继续补 Java `CargoAI.findAnyTarget(...)` 的 stale 优先级语义。Rust 现在把 drop target 枚举抽成 `runtime_unit_cargo_drop_targets(...)`；空载 pickup 规划会先按 loader 库存降序扫描物品，但不会因为第一个物品只有 stale unload 点就立即停下，而是继续寻找后续物品的非 stale 目标，只有所有候选都没有非 stale 目标时才 fallback 到最后一个 stale target。新增回归覆盖 copper 库存更多但 unload stale、lead 库存更少但 unload fresh 时选择 lead。
- 2026-05-27：把 server cargo unit 的位置/状态同步接到现有 Java-like `EntitySnapshotCallPacket` 链路。`ServerLauncher::update()` 在 cargo AI tick 后会把 authoritative cargo `UnitComp::to_sync_wire()` 编入 unit entity snapshot 并非可靠广播；desktop 侧复用既有 `NetClient.entity_snapshot_mirrors -> DesktopLauncher::apply_client_entity_snapshot_packet_with_content(...) -> GameRuntime::apply_client_unit_sync_wire(...)`，真实 server→desktop cargo transfer smoke 现在断言客户端 materialized `manifold` 的位置跟随 server unload 点，而不再只依赖 item mirror。
- 2026-05-27：补 `UnitCargoUnloadPoint.buildConfiguration/configClear` 对应的客户端 packet 构造入口。`mindustry::input` 新增 `client_unit_cargo_unload_item_config_packet(...)` 与 `client_unit_cargo_unload_clear_config_packet(...)`，分别生成 Java `config(Item.class, ...)` 对应的 `TypeValue::Content(ContentRef(Item, id))` 与 `configClear` 对应的 `TypeValue::Null` 客户端 `TileConfigCallPacket`；后续桌面 UI 选择表可直接调用该入口接入既有 server/runtime/network 配置链路。
- 2026-05-27：补 `CargoAI.moveTo(..., moveRange, moveSmoothing)` 的最小 server 运动语义。cargo unit 现在在 pickup/drop 前会通过 `move_runtime_unit_cargo_towards(...)` 向 loader/unload 点推进，只有进入 `transferRange = 20` 后才执行 `take_items`/`transfer_item_to`；移动步长使用 Java 常量 `moveRange = 6`、`moveSmoothing = 20` 的近似过渡实现，不再在装卸前直接瞬移到目标中心。entity snapshot 与真实 server→desktop smoke 也改为断言客户端 unit 处于 transfer range 内，后续仍需补更精确的速度/路径/插值。
- 2026-05-27：补 Java `AIController.retarget()` 在 cargo AI 上的最小 40 tick 节流。`CargoAiRuntimeState` 新增 `retarget_timer`，默认 ready 以保持初次 spawn 行为；空载 pickup 与载货但暂无 unload target 的分支现在会通过 `runtime_unit_cargo_retarget_ready(...)` 按 `target == null ? 40 : 90` 中的空目标间隔近似节流，避免每 tick 扫描/重选目标。新增 server 回归验证 retarget timer 未到时不 pickup，计时到达后才选择 unload target 并装货。
- 2026-05-27：补 Java `UnitCargoLoader.acceptItem(...)` 的真实 item dump 接入。`dump_target_accepts_item(...)` 现在识别 `DistributionBlockKind::UnitCargoLoader`，按 `items.total() < itemCapacity` 与同队判断接收物品；后续 conveyor/router/bridge 等通用物品流可直接给 loader 入库，不再只依赖手写库存或 cargo AI 测试准备。
  - 新增 `game_runtime_unit_cargo_loader_accepts_items_until_capacity_like_java`
- 验证：
  - `cargo test -p mindustry-core game_runtime_unit_cargo_loader_accepts_items_until_capacity_like_java --lib`
  - `cargo test -p mindustry-core unit_cargo_unload_config_packets --lib`（本轮通过 1/1）
  - `cargo test -p mindustry-core unit_cargo`
  - `cargo test -p mindustry-core unit_despawn --lib`（本轮通过 1/1）
  - `cargo test -p mindustry-desktop unit_despawn --lib`（本轮通过 1/1）
  - `cargo test -p mindustry-server cargo_loader_is_missing --lib`（本轮通过 1/1）
  - `cargo test -p mindustry-core building_tether --lib`（本轮通过 1/1）
  - `cargo test -p mindustry-core unit_tether_block_spawned --lib`（本轮通过 2/2）
  - `cargo test -p mindustry-server tethered_unit --lib`（本轮通过 2/2）
  - `cargo test -p mindustry-server unit_cargo --lib`（本轮通过 10/10）
  - `cargo test -p mindustry-tests real_server_desktop_unit_cargo_loader_tether_spawn_syncs_to_client_runtime -- --nocapture`（本轮通过）
  - `cargo test -p mindustry-tests real_server_desktop_unit_cargo_transfer_syncs_item_mirrors_to_client_runtime -- --nocapture`（本轮通过，含 cargo unit entity snapshot moveTo/transfer range 位置同步断言）
  - `cargo test -p mindustry-core unit_tether_block_spawned --lib`（本轮通过 2/2）
  - `cargo test -p mindustry-desktop unit_tether_block_spawned --lib`（本轮通过 1/1）
  - `cargo test -p mindustry-desktop unit_cargo --lib`（本轮通过 1/1）
  - `cargo test -p mindustry-core client_building_item_storage_mirror --lib`（本轮通过 1/1）
  - `cargo test -p mindustry-desktop building_storage_mirror --lib`（本轮通过 1/1）
  - `cargo test -p mindustry-server unit_cargo --lib`（本轮通过 6/6）
  - `cargo test -p mindustry-tests real_server_desktop_unit_cargo_loader_tether_spawn_syncs_to_client_runtime -- --nocapture`（本轮通过）
  - `cargo test -p mindustry-tests real_server_desktop_unit_cargo_transfer_syncs_item_mirrors_to_client_runtime -- --nocapture`（本轮通过；首次遇到临时端口占用重试后通过）
  - `cargo fmt --check`
  - `cargo check --workspace`（本轮通过，仅保留既有 unused warning）
- 仍未完成：`BuildingTetherComp` 已并入 cargo `UnitComp` 与 server/client tether 物化链路，但还未接入通用 `Groups.unit` 生命周期；当前 cargo AI 仍是 server launcher 驱动的最小 authoritative 闭环，尚未完整迁移 Java `retarget()` 的 target-present 90 tick 语义、真实速度/路径/插值、目标选择排序函数与客户端可视化；loader 资源 consumer 的全局 shouldConsume/rollback 权限细节、unload config 的完整 UI 选择表/rollback 权限细节、Java 客户端/服务端更完整联机兼容仍待补。

### 12.116 UnitFactory `CommandBuildingCallPacket` 命令位置联机链路

- 2026-05-27：对照 Java `InputHandler.commandBuilding(...)` 与 `UnitFactoryBuild.onCommand(Vec2 target)`，把建筑命令从“客户端 helper 只生成 packet / UnitFactoryState 只序列化 commandPos”推进到 runtime + server + desktop 的完整最小链路。
- 2026-05-27：继续扩展同一入口，不让 UnitFactory 成为单点孤岛；对照 Java `Reconstructor`、`UnitAssembler`、`PayloadSource`、`CoreBlock` 的 `getCommandPosition()/onCommand(Vec2)`，把 Rust 已有的 `command_pos` sidecar 接到统一 command building runtime。
- Java 依据：
  - `InputHandler.commandBuilding(Player, int[], Vec2)`：server 校验 `ActionType.commandBuilding`，遍历同队且 `build.isCommandable()` 的建筑，执行 `build.onCommand(target)` 与 `build.updateLastAccess(player)`；
  - `UnitFactoryBuild`、`ReconstructorBuild`、`UnitAssemblerBuild`、`PayloadSourceBuild`、`CoreBuild` 的 `onCommand(Vec2 target)`：写入 `commandPos = target`；
  - `UnitFactory.updateTile()` 创建 unit payload 时，后续命令控制器使用该 `commandPos`。
- Rust 新增/变化：
  - `GameRuntimeCommandBuildingReport` 与 `GameRuntime::command_owned_building_positions(...)`：按 team、block 类型与对应 block `commandable` 字段过滤，写入 `UnitFactoryState.command_pos` / `ReconstructorState.command_pos` / `UnitAssemblerState.command_pos` / `PayloadSourceState.command_pos` / `CoreBuildState.command_pos`，并同步 `BuildingComp.last_accessed`；
  - `NetServer` 现在记录并允许/拒绝 `PacketKind::CommandBuildingCallPacket`，使用 `ActionType::CommandBuilding` 填充 `PlayerAction.building_positions`，并生成 server event；
  - `ServerLauncher::apply_new_network_server_events()` 处理 `CommandBuildingCallPacket`，把来源连接转换为 player/team/colored name，调用 runtime 写入 command position，并可靠转发 server 形态 packet（含 `player`）给已连接客户端；
  - `DesktopLauncher::update()` 新增 `sync_command_building_to_runtime()`，消费 `NetClient.last_command_building`，从本地/远端 player 或目标 building 恢复 team，回填客户端 runtime 的 commandable building `command_pos`；
  - `advance_owned_unit_factories(...)` 已有的 payload controller patch 现在可由真实 command building packet 驱动，产出的 unit payload 带 `ControllerWire::Command.target_pos`。
- 新增验证：
  - `cargo test -p mindustry-core game_runtime_commands_unit_factory_position_and_payload_controller_like_java --lib`
  - `cargo test -p mindustry-core game_runtime_commands_other_commandable_building_positions_like_java --lib`
  - `cargo test -p mindustry-server server_update_applies_command_building_packet_to_unit_factory_and_forwards --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_command_building_packet_to_unit_factory_runtime --lib`
- 仍未完成：command position 已接入 factory/reconstructor/assembler/payload-source/core 的 runtime/server/desktop 最小闭环，但客户端 Fx.moveCommand、BuildingCommandEvent、完整 command UI marker/overlay、权限 rollback 以及更多真实 Java↔Rust command smoke 仍需继续补齐。

### 12.117 Reconstructor 命令配置入口接入

- 2026-05-27：对照 Java `Reconstructor` 构造器中的 `config(UnitCommand.class, ...)` / `configClear(...)` 与 `ReconstructorBuild.buildConfiguration(...)`，把重构器命令配置从 sidecar 字段推进到 runtime + input packet + server authoritative 转发 + desktop client 回填闭环。
- Java 依据：
  - `config(UnitCommand.class, (build, command) -> build.command = command)` 只写显式命令；
  - `configClear((build) -> build.command = null)` 清空显式命令；
  - 配置 UI 仅在升级目标单位可命令且 `canSetCommand()` 时展示；按钮选中态为 `command == item || (command == null && unit.defaultCommand == item)`；
  - 升级完成后未显式配置时回退到目标单位 `defaultCommand`，已在前序 Reconstructor controller 写回闭环中覆盖。
- Rust 新增/变化：
  - 新增 `GameRuntimeReconstructorConfigureResult` 与 `GameRuntime::configure_owned_reconstructor_command/value(...)`，按 `BlockDef::UnitReconstructor`、`configurable`、`ContentType::UnitCommand` / `Null` 校验后写入或清空 `ReconstructorState.command_id`，不修改 `progress/constructing/BuildingComp.config`；
  - `mindustry::input` 新增 `client_reconstructor_command_config_packet(...)` 与 `client_reconstructor_clear_command_packet(...)`，生成 Java `UnitCommand` content 与 `configClear` 对应的 `TileConfigCallPacket`；
  - `ServerLauncher::apply_server_tile_config_packet(...)` 现在识别 `BlockDef::UnitReconstructor`，将成功变更可靠转发给已连接客户端；
  - `DesktopLauncher::sync_tile_config_to_runtime(...)` 现在识别 `BlockDef::UnitReconstructor`，把服务端 tile config 回填到本地 `GameRuntimeUnitBlockState::Reconstructor.command_id`；snapshot cursor reset 同步清理 factory/reconstructor 两类 tile config 结果，避免 stale result。
- 新增验证：
  - `cargo test -p mindustry-core game_runtime_configures_reconstructor_command_and_clear_like_java --lib`
  - `cargo test -p mindustry-core client_reconstructor_config_packets_use_unit_command_content_and_clear_null --lib`
  - `cargo test -p mindustry-server reconstructor_command_tile_config --lib`
  - `cargo test -p mindustry-desktop reconstructor_command_tile_config --lib`
- 仍未完成：Reconstructor 的配置 UI 候选表/按钮布局仍未像 UnitFactory 那样完整暴露为 runtime plan；服务端权限 rollback、Java 客户端互通 smoke、升级完成后的完整 Unit 实体重建/effect/sound/event 仍需继续迁移。

### 12.118 Reconstructor 配置候选计划与 config 感知

- 2026-05-27：继续对照 Java `ReconstructorBuild.buildConfiguration(...)` / `canSetCommand()` / `unit()` / `senseObject(LAccess.config)`，把重构器命令按钮候选与当前升级目标单位暴露为 runtime plan，避免上一节的命令配置链路缺少可驱动的 UI 数据。
- Java 依据：
  - `unit()`：仅在已有 UnitPayload、存在 upgrade、且目标单位 `unlockedNowHost() || team.isAI()` 时返回目标单位，否则返回 `null`；
  - `canSetCommand()`：要求 `unit() != null && unit.commands.size > 1 && unit.allowChangeCommands`；
  - `buildConfiguration(...)`：按 `unit.commands` 原顺序生成按钮，固定 `columns = 4`，选中态为显式 `command == item` 或未显式配置时 `unit.defaultCommand == item`；
  - `senseObject(LAccess.config)` 返回 `unit()`。
- Rust 新增/变化：
  - 新增 `GameRuntimeReconstructorConfigurationPlan` 与 `GameRuntimeReconstructorConfigCommandOption`，记录 `current_unit_id/name`、`can_set_command`、命令候选、默认/显式选中态与固定 4 列布局；
  - 新增 `GameRuntime::reconstructor_can_set_command(...)` 与 `GameRuntime::reconstructor_configuration_plan(...)`，从真实 `GameRuntimeUnitBlockState::Reconstructor.common.payload` 推导当前升级目标，复用内容 registry 的 `UnitCommand` 解析与默认命令映射；
  - `sense_owned_building_config_object(...)` 现在除 UnitFactory 外也支持 `BlockDef::UnitReconstructor`，按 Java 返回当前升级目标 `ContentType::Unit`，无 payload/无可用升级时返回 `Null`。
- 新增验证：
  - `cargo test -p mindustry-core reconstructor_configuration_plan --lib`
  - `cargo test -p mindustry-core game_runtime_configures_reconstructor_command_and_clear_like_java --lib`
  - `cargo fmt --check`
  - `cargo check -p mindustry-core`
- 仍未完成：该 plan 仍只是 runtime/UI 数据源，尚未接到真实桌面配置菜单渲染；Java `team.isAI()` 研究绕过、客户端权限 rollback、完整 Java↔Rust UI/联机 smoke 仍需继续迁移。

### 12.119 Reconstructor 物品输入与逻辑感知

- 2026-05-27：对照 Java `ReconstructorBuild.getMaximumAccepted(Item)`、默认 `Building.acceptItem(...)` 与 `sense(LAccess.progress/itemCapacity)`，把重构器消耗物品接入真实 item transport 入口，并补齐逻辑感知值。
- Java 依据：
  - `getMaximumAccepted(item)` 返回 `round(capacities[item.id] * state.rules.unitCost(team))`；
  - Reconstructor 不像 UnitFactory 那样按当前 plan 过滤物品，而是依赖 `consume_items` 初始化出的 `capacities`；
  - `sense(progress)` 返回 `clamp(fraction())`，`sense(itemCapacity)` 返回 `round(itemCapacity * state.rules.unitCost(team))`。
- Rust 新增/变化：
  - `world::blocks::units` 新增 `reconstructor_accept_item(...)` 与 `reconstructor_maximum_accepted(...)`，并扩展既有 Reconstructor 单元测试；
  - `GameRuntime::dump_target_accepts_item(...)` 现在识别 `BlockDef::UnitReconstructor`，按 `capacities[item] * rules.unitCost(team)` 接收 consume item，使 conveyor/router 等真实物品流可给 Reconstructor 补料；
  - `GameRuntime::sense_owned_building_number(...)` 现在支持 `BlockDef::UnitReconstructor` 的 `LAccess::Progress` 与 `LAccess::ItemCapacity`。
- 新增验证：
  - `cargo test -p mindustry-core reconstructor_accepts_consume_items --lib`
  - `cargo test -p mindustry-core senses_reconstructor --lib`
  - `cargo test -p mindustry-core reconstructor_progress_acceptance_and_serialization_follow_upstream --lib`
  - `cargo fmt --check`
  - `cargo check -p mindustry-core`
- 仍未完成：资源 consumer 的 `shouldConsume()` 仍只是 Reconstructor tick 的最小门控，尚未完整迁移 Java consume rollback/efficiency UI；真实 Java↔Rust 服务端物品传输 smoke 仍需补。

### 12.120 UnitBlockSpawn 客户端生命周期回填

- 2026-05-27：对照 Java `UnitBlock.unitBlockSpawn(Tile)` 与 `UnitBuild.spawned()`，把已存在的 `UnitBlockSpawnCallPacket` 从“NetClient 只记录生命周期包”推进到 desktop/runtime 回填，提升 Rust 客户端连接 Java 服务端时的 unit block 状态同步能力。
- Java 依据：
  - `UnitBuild.dumpPayload()` 在 `payload.dump()` 成功后调用 `Call.unitBlockSpawn(tile)`；
  - `UnitBuild.spawned()` 只做 `progress = 0f; payload = null;`；
  - 命令配置、当前 factory plan 等不是 `spawned()` 的清理对象。
- Rust 新增/变化：
  - `GameRuntime::apply_client_unit_block_spawn_packet(...)`：按 packet tile 定位 owned building，懒创建 unit sidecar 后处理 `UnitFactory` / `Reconstructor`，清空 `PayloadBlockBuildState.payload`，调用 `unit_block_spawned(...)` 复位 `progress/has_payload`，Reconstructor 同步清理 `constructing`，保留 `current_plan/command_id/command_pos`；
  - `DesktopLauncher::sync_unit_lifecycle_to_runtime()` 现在除 `UnitDespawnCallPacket` 外也消费 `UnitBlockSpawnCallPacket`；
  - 这让 Java 服务端广播 `UnitBlockSpawnCallPacket` 时，Rust desktop runtime 不再保留 stale unit payload/progress。
- 新增验证：
  - `cargo test -p mindustry-core unit_block_spawn_packet --lib`
  - `cargo test -p mindustry-desktop unit_block_spawn_packet --lib`
  - `cargo fmt --check`
  - `cargo check -p mindustry-core`
- 仍未完成：本节先完成 Java/Rust 服务端到 Rust 客户端的接收端回填；Rust 服务端广播在后续 12.121 继续接入，真实 Java↔Rust smoke 仍需补。

### 12.121 UnitBlockSpawn 服务端广播

- 2026-05-27：继续打通 Java `UnitBuild.dumpPayload() -> Call.unitBlockSpawn(tile)` 方向，把 Rust owned runtime 的 unit block payload 输出结果上报给 server launcher，并广播 `UnitBlockSpawnCallPacket`。
- Java 依据：
  - `UnitBuild.dumpPayload()` 仅在 payload 成功 dump 后触发 `Call.unitBlockSpawn(tile)`；
  - 接收端 `spawned()` 复位 progress/payload，配置与命令字段保持不变。
- Rust 新增/变化：
  - `GameRuntimeUnitFactoryFrameReport` / `GameRuntimeUnitReconstructorFrameReport` 新增 `spawned_tiles`，只在 payload 成功转交到前方目标时记录 tile；
  - `ServerLauncher::update()` 收集 factory/reconstructor `spawned_tiles` 并调用 `broadcast_runtime_unit_block_spawns(...)`；
  - 新增 server 广播 helper，对重复 tile 去重后发送可靠 `UnitBlockSpawnCallPacket`；
  - 这使 Rust server 自身运行 UnitFactory/Reconstructor 输出 payload 时，也能通知 Rust/Java 客户端清理对应 unit block 状态。
- 新增验证：
  - `cargo test -p mindustry-core unit_factory_outputs_payload_to_front_conveyor --lib`
  - `cargo test -p mindustry-core reconstructor_outputs_upgraded_payload_to_front_conveyor --lib`
  - `cargo test -p mindustry-server unit_block_spawn_when_unit_factory_payload_dumps --lib`
  - `cargo fmt --check`
  - `cargo check -p mindustry-core`
- 仍未完成：当前广播只覆盖已迁移的 factory/reconstructor payload transfer 成功路径；尚未覆盖未来完整 world dump/entity materialize、assembler 专属 spawn 包、以及 Java↔Rust 真实联机 smoke。

### 12.122 Reconstructor 队伍激活延迟门控

- 2026-05-27：对照 Java `ReconstructorBuild.status()` / `shouldConsume()`，把重构器生产进度接入 `team.activateUnitFactories()` 等价规则，避免 Rust Reconstructor 在队伍激活延迟内提前推进升级。
- Java 依据：
  - `status()` 在 `!team.activateUnitFactories()` 时返回 inactive；
  - `shouldConsume()` 要求 `constructing && enabled && team.activateUnitFactories()`；
  - payload 无可用 upgrade 时仍可走输出/转交路径，真正被门控的是资源消耗与进度推进。
- Rust 新增/变化：
  - `GameRuntimeUnitReconstructorFrameReport` 新增 `inactive_reconstructors`；
  - `advance_owned_unit_reconstructors_ticks(...)` 现在读取 `Rules::unit_factory_active(team, tick)`，未激活时保持 move-in/out 行为但将 `effective_efficiency` 置零，不推进 `progress`、不升级、不消耗 items；
  - 该行为与已迁移的 UnitFactory 激活延迟口径保持一致。
- 新增验证：
  - `cargo test -p mindustry-core reconstructor_respects_team_activation_delay --lib`
  - `cargo fmt --check`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
- 仍未完成：Reconstructor 的完整消费系统 rollback/UI efficiency、create sound/effect/event 以及真实 Java↔Rust smoke 仍需继续迁移。

### 12.123 UnitAssembler 队伍激活延迟门控

- 2026-05-27：对照 Java `UnitAssemblerBuild.status()` / `shouldConsume()`，把单位组装厂的生产进度接入 `team.activateUnitFactories()` 等价规则，避免 Rust UnitAssembler 在队伍激活延迟内提前完成组装。
- Java 依据：
  - `shouldConsume()` 要求 `enabled && !wasOccupied && Units.canCreate(team, plan().unit) && consPayload.efficiency(this) > 0 && consItem.efficiency(this) > 0 && team.activateUnitFactories()`；
  - `status()` 在 `!team.activateUnitFactories()` 时返回 `BlockStatus.inactive`；
  - 激活延迟影响消耗与进度推进，不应阻断已迁移的 sidecar 保留、payload 状态读取与后续激活后的恢复生产。
- Rust 新增/变化：
  - `GameRuntimeUnitAssemblerFrameReport` 新增 `inactive_assemblers`；
  - `advance_owned_unit_assemblers_ticks(...)` 现在读取 `Rules::unit_factory_active(team, tick)`，队伍未激活时记录 inactive，并将传给 `unit_assembler_update_progress(...)` 的 `effective_efficiency` 置零；
  - 未激活帧不推进 `progress`、不完成单位、不消耗 payload/item requirements；激活后同一 sidecar 可继续推进并完成组装。
- 新增验证：
  - `cargo test -p mindustry-core unit_assembler_respects_team_activation_delay --lib`
  - `cargo test -p mindustry-core game_runtime_owned_runtime_blocks_includes_unit_assembler_tick_like_java --lib`
- 仍未完成：UnitAssembler 的完整 UI status/bar、AssemblerAI/BuildingTether 实体所有权、create effect/sound/event、网络广播与 Java↔Rust 真实联机 smoke 仍需继续迁移。

### 12.124 UnitAssembler 完成事件网络回填

- 2026-05-27：对照 Java `Call.assemblerUnitSpawned(tile)` / `UnitAssemblerBuild.spawned()`，把 Rust UnitAssembler 完成事件从 runtime report 接到 server 广播与 desktop/client 回放，并修正完成后 payload requirement 库存清理语义。
- Java 依据：
  - `updateTile()` 在进度达到 1 后调用 `Call.assemblerUnitSpawned(tile)`；
  - `assemblerUnitSpawned(Tile tile)` 在客户端/服务端定位 `UnitAssemblerBuild` 并调用 `spawned()`；
  - `spawned()` 会 `consume()`、创建/投递 unit、将 `progress = 0f`，并 `blocks.clear()`，不是只扣除本次需求数量。
- Rust 新增/变化：
  - `GameRuntimeUnitAssemblerFrameReport` 新增 `spawned_tiles`，UnitAssembler 完成时记录 tile；
  - `ServerLauncher::update()` 现在把 `report.unit.assembler.spawned_tiles` 广播为可靠 `AssemblerUnitSpawnedCallPacket`；
  - `DesktopLauncher::sync_unit_lifecycle_to_runtime()` 现在消费 `AssemblerUnitSpawnedCallPacket` 并调用 `GameRuntime::apply_client_assembler_unit_spawned_packet(...)`；
  - `unit_assembler_spawned(...)` 现在按 Java 清空 `UnitAssemblerState.blocks`，避免完成后保留 stale/额外 payload 计数。
- 新增验证：
  - `cargo test -p mindustry-core assembler_unit_spawned_packet --lib`
  - `cargo test -p mindustry-desktop syncs_assembler_unit_spawned --lib`
  - `cargo test -p mindustry-server assembler_unit_spawn_packet --lib`
- 仍未完成：完成时真实 materialize `plan.unit` / payload 投递、`commandPos` 应用、`Fx.unitAssemble` / create sound / `UnitCreateEvent`、AssemblerDroneSpawned 以及真实 Java↔Rust smoke 仍需继续迁移。

### 12.125 UnitAssembler 完成后服务端实体产出

- 2026-05-27：继续对照 Java `UnitAssemblerBuild.spawned()`，把上一节的 `AssemblerUnitSpawnedCallPacket` 事件推进到 Rust server 的真实 `server_units` 实体产出，避免 UnitAssembler 只复位 sidecar 而不产生可同步单位。
- Java 依据：
  - `spawned()` 使用当前 plan 的 `unit.create(team)` 创建输出单位；
  - 若 `commandPos != null`，调用 `unit.command().commandPosition(commandPos)`；
  - 输出位置来自 `getUnitSpawn()`，即按 assembler 旋转方向偏移 `(areaSize + size) / 2 * tilesize`。
- Rust 新增/变化：
  - `ServerLauncher::update()` 在收到 `report.unit.assembler.spawned_tiles` 后调用 `apply_runtime_unit_assembler_spawns(...)`；
  - `apply_runtime_unit_assembler_spawns(...)` 按 tile 定位 UnitAssembler、当前 tier plan、输出 `UnitType` 与 `command_pos`，创建新的 server runtime unit；
  - 新单位写入 `server_units`，位置按 Java `getUnitSpawn()` 等价公式计算，旋转使用 building `rotdeg()`，存在 `command_pos` 时设置 `UnitControllerState::Command`；
  - 后续既有 `broadcast_server_unit_entity_snapshots()` 可把该单位同步给客户端。
- 新增验证：
  - `cargo test -p mindustry-server assembler_unit_spawn_packet --lib`
- 仍未完成：Java `spawned()` 中“先包装成 `UnitPayload` 并尝试投递给目标建筑”的分支、`Units.notifyUnitSpawn`、create sound/effect/event、真实 AssemblerDrone/AssemblerAI/BuildingTether 与 Java↔Rust smoke 仍需继续迁移。

### 12.126 AssemblerDroneSpawned 客户端回放

- 2026-05-27：对照 Java `Call.assemblerDroneSpawned(tile, id)` / `UnitAssemblerBuild.droneSpawned(id)`，把 Rust client/desktop 端的 drone spawned 生命周期包从“只记录 last packet”推进到 UnitAssembler runtime sidecar。
- Java 依据：
  - `droneSpawned(int id)` 会播放 spawn effect、将 `droneProgress = 0f`；
  - 客户端收到时把 id 加入 `whenSyncedUnits`，等待后续 `Groups.unit.getByID(id)` 同步成真实 drone；
  - 这是后续真实 `AssemblerAI` / `BuildingTetherComp` 接回 UnitAssembler 进度倍率前的必要同步入口。
- Rust 新增/变化：
  - `unit_assembler_drone_spawned(...)` 复位 `UnitAssemblerState.drone_progress`，并在 client 回放时把有效 id 去重追加到 `read_unit_ids`；
  - `GameRuntime::apply_client_assembler_drone_spawned_packet(...)` 定位 tile 对应 UnitAssembler sidecar 并应用上述状态变更；
  - `DesktopLauncher::sync_unit_lifecycle_to_runtime()` 现在消费 `AssemblerDroneSpawnedCallPacket`。
- 新增验证：
  - `cargo test -p mindustry-core assembler_drone_spawned_packet --lib`
  - `cargo test -p mindustry-desktop syncs_assembler_drone_spawned --lib`
- 仍未完成：Rust server 尚未真实创建 assembly drone 并广播 `AssemblerDroneSpawnedCallPacket`；`read_unit_ids` 仍未接入完整 `client_unit_snapshot_entities` / `AssemblerAI` in-position 计算。

### 12.127 UnitAssembler 服务端 assembly drone 生成

- 2026-05-27：继续对照 Java `UnitAssemblerBuild.updateTile()` 的 drone 构造分支，把 Rust server 端的 assembly drone 实体生成、建筑 tether 与 `AssemblerDroneSpawnedCallPacket` 广播接入真实 update 链路。
- Java 依据：
  - 当 `units.size < dronesCreated && enabled` 且 `droneProgress` 达到 1 时，创建 `droneType` 单位；
  - drone 需要 `AssemblerAI` 控制器与 `BuildingTetherComp` 指向 assembler；
  - 创建后加入 `units`，并广播 `Call.assemblerDroneSpawned(tile, unit.id)`。
- Rust 新增/变化：
  - `unit_assembler_update_progress(...)` 现在使用 `UnitAssemblerBlockData.drone_construct_time` 等价字段，不再硬编码 `60f * 4f`；
  - `GameRuntimeUnitAssemblerFrameReport` 新增 `spawned_drone_tiles`，在 drone progress 达标时上报 tile；
  - `ServerLauncher::apply_runtime_unit_assembler_drone_spawns(...)` 创建 `drone_type` 对应 `UnitComp`，设置 `UnitControllerState::Assembler` 与 `BuildingTetherComp`，写入 `server_units`；
  - server 同步更新 `UnitAssemblerState.read_unit_ids` 并可靠广播 `AssemblerDroneSpawnedCallPacket`，后续实体快照可携带该 drone。
- 新增验证：
  - `cargo test -p mindustry-core assembler_geometry_tiers_acceptance --lib`
  - `cargo test -p mindustry-server assembler_drone_spawn_packet --lib`
- 仍未完成：`AssemblerAI` 目标点/角度与真实 `inPosition()` 尚未参与进度倍率；当前 UnitAssembler 生产进度仍保留“drone 视为到位”的过渡口径以避免破坏既有可玩闭环。

### 12.128 v158.1 基线确认与 assembler drone 客户端实体化

- 2026-05-27：确认上游官方 `Anuken/Mindustry` 已存在 `v158.1` tag，并将本地 Java 参考目录 `D:/MDT/mindustry-upstream-v157.4` 从 `v158` 更新到 `v158.1`（`05b2ecd`）。目录名保持不变，避免上下文压缩后路径漂移。
- 同步更新：
  - `README.md`、`AI_HANDOFF.md`、`core::mindustry::UPSTREAM_BASELINE` 与本节顶部“当前参考基线”均已改为 `v158.1`；
  - 参考目录验证命令：`git -C "D:/MDT/mindustry-upstream-v157.4" describe --tags --always --dirty` 应返回 `v158.1`。
- 继续对照 Java `UnitAssemblerBuild.droneSpawned(id)` / 客户端 `whenSyncedUnits` 行为，`GameRuntime::apply_client_assembler_drone_spawned_packet(...)` 现在除复位 sidecar 外，还会在 `client_unit_snapshot_entities` 中 materialize `assembly-drone` 快照：
  - team/position 来自 assembler building；
  - rotation 为 `90.0`；
  - controller 为 `UnitControllerState::Assembler`；
  - `BuildingTetherComp` 指向来源 assembler tile。
- 新增验证：
  - `cargo test -p mindustry-core assembler_drone_spawned_packet --lib`
  - `cargo test -p mindustry-desktop syncs_assembler_drone_spawned --lib`
- 仍未完成：需要继续对照 v158.1 与 v158 的差异，逐项检查新增/变更 Java 文件；assembler drone 的真实 `AssemblerAI.targetPos/targetAngle/inPosition()` 仍待接入。

### 12.129 v158.1 CoreBlock linked storage item 合并变更

- 2026-05-27：对照 `v158..v158.1` 的 `CoreBlock.java` 变更，修正 Rust linked storage 刷新时的旧 item 处理。
- Java 依据：
  - `CoreBuild.updateProximity()` 在 v158.1 中移除了 `if(t.items != items){ items.add(t.items); }`；
  - linked storage 现在直接 `t.items = items` 并设置 `linkedCore`，不会把被链接 storage 原本持有的 item 累加到 core。
- Rust 新增/变化：
  - `merge_owned_core_and_linked_storage_items(...)` 仍保留同队多个 core 的 canonical owner 合并；
  - 对 `storage_linked_cores` 里的普通 storage，则清空其旧 item module，但不再把旧 item 加到 core，匹配 v158.1 行为；
  - 新测试锁定 linked storage 预存 copper/lead 后刷新链接不会污染 core items。
- 新增验证：
  - `cargo test -p mindustry-core linked_storage_items_are_not_merged --lib`
  - `cargo test -p mindustry-core same_team_cores_share --lib`
- 仍未完成：`LandingPad.java`、`UI.showFollowUpMenu(...)`、`HudFragment` 的 v158.1 UI/显示差异仍需继续对照；其中 UI 类差异不应误改 runtime。

### 12.130 v158.1 LandingPad waiting queue 剪枝与联机事件回放

- 2026-05-27：对照 `v158..v158.1` 的 `LandingPad.java` 变更，把 landing pad waiting queue 从纯 codec/helper 推进到 Rust owned runtime、server broadcast 与 desktop client replay。
- Java 依据：
  - v158.1 将 `pads.removeAll(l -> l.config != item)` 移到 `if(pads.size > 0)` 之前；
  - 旧配置不匹配的 landing pad 必须先从 waiting 队列剪掉，剪枝后为空则不能再 sort/first/Call；
  - 非 fake campaign landing pad 满足 import cooldown 后排入下一帧 waiting；被选中后调用 `Call.landingPadLanded(tile)` 并交换优先级。
- Rust 新增/变化：
  - `GameRuntime::landing_pad_waiting` 按 item id 保存 runtime-only waiting queue，并在 world reset/building remove 时清理；
  - `advance_owned_landing_pads_ticks(...)` 接入真实 building list：更新 import cooldown、先剪枝 stale config，再选择 landing pad、驱动 arrival/liquid removal/item import/dump；
  - `GameRuntimeOwnedFrameReport` 新增 `campaign.landing_pad`；
  - `ServerLauncher::update()` 对 `report.campaign.landing_pad.landed_tiles` 可靠广播 `LandingPadLandedCallPacket`；
  - `DesktopLauncher::sync_world_update_events_to_runtime()` 从 `NetClientState.last_world_update_packet` 消费 `LandingPadLandedCallPacket`，调用 `GameRuntime::apply_client_landing_pad_landed_packet(...)` 回放到本地 landing pad sidecar。
- 新增验证：
  - `cargo test -p mindustry-core landing_pad --lib`
  - `cargo test -p mindustry-server landing_pad --lib`
  - `cargo test -p mindustry-desktop landing_pad --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `cargo check -p mindustry-desktop`
- 仍未完成：LandingPad 的完整 UI 配置表、手动 import 按钮、粒子/音效/shake 仍未迁移；`UI.showFollowUpMenu(...)` 与 `HudFragment` 的 v158.1 UI/显示差异待继续只读确认并记录或接入 UI 层。

### 12.131 v158.1 UI.showFollowUpMenu / HudFragment UI 差异记录

- 2026-05-27：只读对照 `v158..v158.1` 的 `UI.java` 与 `HudFragment.java`，确认变更属于 UI/HUD 表现层，当前 Rust 不应误改 `GameRuntime`。
- Java 依据：
  - `UI.showFollowUpMenu(...)` 在 callback 后，如果 `!state.isGame()`，会 `myself.hide()`；这是 dialog/menu 生命周期语义；
  - `HudFragment.java` 调整 sidebar 背景/颜色、health/shield/payload/ammo bar 绘制、status effect 图标存在性判断与超大持续时间 `∞` tooltip；这是 HUD 渲染语义。
- Rust 当前状态：
  - 现有 UI 主要是 `core/src/mindustry/ui/dialogs/*`、`ui/displayable.rs`、`input/desktop_input.rs` / `input/mobile_input.rs`；
  - 尚无对应 Java `HudFragment` 的完整 HUD fragment 渲染层；
  - 因此暂不把 `HudFragment` 差异落入 runtime，避免把显示层缺口误变成 gameplay 行为。
- 后续接入建议：
  - 若后续补齐 follow-up menu/dialog stack，需在非 game state 回调后自动 hide；
  - 若后续补齐 HUD renderer，再迁移 sidebar bar 绘制、status icon found 判定和无限时长显示。

### 12.132 v158.1 UnitAssembler / AssemblerAI 真实 drone 到位倍率

- 2026-05-27：再次确认 Java 参考目录 `D:/MDT/mindustry-upstream-v157.4` 当前为 `v158.1` / `05b2ecd`；继续对照 `UnitAssembler.java` 与 `AssemblerAI.java`，把 Rust UnitAssembler 从“所有 drone 视为到位”的过渡口径推进到真实 pose 判定。
- Java 依据：
  - `UnitAssemblerBuild.updateTile()` 每帧按 `i * 90f + 45f` 给每个 drone 设置 `AssemblerAI.targetPos`，距离为 `areaSize / 2f * Mathf.sqrt2 * tilesize`，`targetAngle = i * 90f + 45f + 180f`；
  - `AssemblerAI.inPosition()` 要求 `unit.within(targetPos, 10f)` 且 `Angles.within(unit.rotation, targetAngle, 15f)`；
  - `progress += edelta() * unitBuildSpeed * eff / plan.time` 中的 `eff` 来自真实到位 drone 数 / `dronesCreated`。
- Rust 新增/变化：
  - `core/src/mindustry/world/blocks/units/mod.rs`
    - 新增 `UnitAssemblerDroneTarget`；
    - 新增 `unit_assembler_drone_target(...)`；
    - 新增 `unit_assembler_drone_in_position(...)`；
  - `GameRuntime::advance_owned_unit_assemblers_ticks(...)`
    - 移除 `simulated_drones = drones_created`；
    - 从 `UnitAssemblerState.read_unit_ids` 保留顺序去重后，读取 `client_unit_snapshot_entities` 中的 `UnitComp` pose；
    - 只统计 `UnitControllerState::Assembler`、team 匹配且满足 Java 距离/角度阈值的 drone；
    - `GameRuntimeUnitAssemblerFrameReport` 新增 `drones_in_position` 便于验证；
  - `ServerLauncher::update()` 在 owned runtime tick 前调用 `tick_runtime_unit_assembler_ai()`：
    - 依据 assembler building、`read_unit_ids` 与 slot index 计算目标点；
    - 移动 server-side assembler drone 朝目标点靠近；
    - 到近处后转向目标角；
    - 将移动后的 server unit 快照同步到 `runtime.client_unit_snapshot_entities`，让 owned runtime 使用真实 pose 计算生产倍率。
- 新增/更新验证：
  - `cargo test -p mindustry-core assembler_geometry_tiers_acceptance_and_progress_follow_upstream --lib`
  - `cargo test -p mindustry-core unit_assembler --lib`
  - `cargo test -p mindustry-core assembler_module --lib`
  - `cargo test -p mindustry-server assembler --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `cargo check -p mindustry-desktop`
  - `cargo fmt --check`
- 仍未完成：
  - server 侧 drone 移动是最小可玩近似，尚未完整迁移 Java `AIController.moveTo(targetPos, 1f, 3f)` 的完整加速度/避障/朝向细节；
  - `UnitAssembler.spawned()` 中输出 unit 依据 `unit.buildOn()` 投递到出生点建筑的 payload 逻辑仍需继续接入；注意 Java `commandPos` 只写入新 unit 的 command controller，不参与选择 payload 目标建筑；
  - `AssemblerAI` 的完整 controller runtime state 尚未独立实体化，当前以 helper + server movement + snapshot sidecar 实现最小闭环。

### 12.133 UnitAssembler.spawned 输出 unit 的 buildOn payload 投递

- 2026-05-27：继续对照 v158.1 `UnitAssemblerBuild.spawned()`，把组装完成后的输出 unit 从“总是落入 server unit 列表”的过渡实现推进到 Java `UnitPayload` 投递语义。
- Java 依据：
  - `plan.unit.create(team)` 创建输出单位；
  - 若 `unit.isCommandable() && commandPos != null`，只把 `commandPos` 写入新 unit 的 command controller；
  - `unit.set(spawn.x + range, spawn.y + range)` 与 `unit.rotation = rotdeg()` 后，通过 `unit.buildOn()` 查找输出 unit 当前所在建筑；
  - 若该建筑同队且 `acceptPayload(targetBuild, payload)` 成功，则调用 `handlePayload(targetBuild, payload)`；否则非 client 侧才 `unit.add()` 并 `Units.notifyUnitSpawn(unit)`；
  - Java 这里传给 `acceptPayload/handlePayload` 的 source 是 `targetBuild` 本身，即 self-source payload 语义。
- Rust 新增/变化：
  - `ServerLauncher::apply_runtime_unit_assembler_spawns(...)` 在创建输出 `UnitComp` 与 command controller 后，先调用 `try_deliver_runtime_spawned_unit_payload(...)`；
  - `try_deliver_runtime_spawned_unit_payload(...)` 通过 `server_unit_build_on_tile_pos(...)` 查找输出 unit 的 `buildOn` 目标建筑，复用 `unit_entered_payload(...)` 与 `GameRuntime::attach_unit_payload_to_building(...)` 完成 accept/handle 语义，并可靠广播 `UnitEnteredPayloadCallPacket`；
  - 若 payload 投递成功，输出 unit 不再插入 `server_units`；若失败，保持原有 `server_units.insert(...)` 路径，等价 Java fallback `unit.add()`；
  - `server_unit_build_on_tile_pos(...)` 增加 `footprint_tiles(...)` fallback，覆盖多格 payload 建筑 footprint 上的 `unit.buildOn()` 场景。
- 新增验证：
  - `cargo test -p mindustry-server server_launcher_unit_assembler_spawn_delivers_payload_to_build_on_target --lib`
  - `cargo test -p mindustry-server assembler --lib`
  - `cargo check -p mindustry-server`
  - `cargo fmt --check`
- 仍未完成：
  - `UnitAssembler.spawned()` 的 `createSound`、`Fx.unitAssemble` 与 `Events.fire(new UnitCreateEvent(unit, this))` 仍需继续对照 Rust 现有 sound/effect/event/network 机制接入；fallback `Units.notifyUnitSpawn(unit)` 已在下一节接入；
  - output unit 的 `Mathf.range(0.001f)` 微扰目前未建模，后续若碰撞/buildOn 精度需求出现，应迁移为可复现的极小随机偏移；
  - `UnitPayload` 内完整 unit 状态序列化/客户端表现仍需继续沿真实 payload/network/client 链路补齐。

### 12.134 UnitAssembler.spawned fallback `Units.notifyUnitSpawn` 联机同步

- 2026-05-27：继续对照 Java `UnitAssemblerBuild.spawned()` 中 payload 投递失败后的 `unit.add(); Units.notifyUnitSpawn(unit);`，把 Rust server/client 联机侧从“只等下一次 entity snapshot”推进到 Java 的即时 `UnitSpawnCallPacket` 语义。
- Java 依据：
  - `Units.notifyUnitSpawn(Unit unit)` 仅在 `net.server()` 时调用 `Call.unitSpawn(new UnitSyncContainer(unit))`；
  - `unitSpawn` 是 server→client、unreliable、low priority，用于让客户端无需等待 snapshot 就立即看到新 unit；
  - 该调用只发生在输出 unit 未被 `targetBuild.handlePayload(...)` 吃入时；payload 成功时不会通知普通 unit spawn。
- Rust 新增/变化：
  - `ServerLauncher::apply_runtime_unit_assembler_spawns(...)` 在 `try_deliver_runtime_spawned_unit_payload(...)` 失败后，先广播 `UnitSpawnCallPacket`，再把 unit 放入 `server_units`；
  - 新增 `server_unit_spawn_packet(...)`：使用 Java class id (`entity_class_id`) + `UnitComp::to_sync_wire()` 生成 `type_io::UnitSyncContainer`；
  - `GameRuntime::apply_client_unit_spawn_packet(...)` 可从 `UnitSpawnCallPacket` 解码 `UnitSyncWire` 并 materialize/update `client_unit_snapshot_entities`；
  - `NetClientState` 将 `UnitSpawnCallPacket` 单独保存到 `unit_spawn_packets`，不再把它覆盖到 `last_unit_lifecycle_packet`，避免同帧 `AssemblerUnitSpawnedCallPacket -> UnitSpawnCallPacket` 时客户端漏掉 assembler state reset；
  - `DesktopLauncher` 新增 unit spawn packet cursor，先回放 assembler/drone/unit lifecycle，再按顺序回放 unit spawn sync container。
- 新增验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_spawn_packet_sync_container --lib`
  - `cargo test -p mindustry-core update_records_unit_spawn_separately_from_lifecycle_tail --lib`
  - `cargo test -p mindustry-server server_update_broadcasts_assembler_unit_spawn_packet_when_assembler_completes --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_spawn_packet_without_losing_assembler_spawned --lib`
- 仍未完成：
  - `createSound.at(...)` 与 `Fx.unitAssemble.at(...)` 的客户端本地 aftereffect 已在下一节接入 runtime sidecar；实际 renderer/audio backend 播放仍需继续补齐；
  - `Events.fire(new UnitCreateEvent(unit, this))` 目前仍未映射到 Rust campaign stats/event bus；
  - unit spawn 只完成即时 sync container 回放，后续仍需继续检查 Java 客户端 `UnitSyncContainer` 读取时的完整 `add()` 副作用。

### 12.135 UnitAssembler.spawned 客户端本地 sound/effect 副作用

- 2026-05-27：继续对照 Java `UnitAssemblerBuild.spawned()` 最后的 `createSound.at(...)` 与 `Fx.unitAssemble.at(...)`，将 Rust desktop/client 回放 `AssemblerUnitSpawnedCallPacket` 时的本地可见副作用接入 runtime sidecar。
- Java 依据：
  - `Call.assemblerUnitSpawned(tile)` 是 remote call；Java 客户端收到后执行 `build.spawned()`，因此 `createSound.at(...)` 与 `Fx.unitAssemble.at(...)` 是**客户端本地副作用**，不是 server 额外广播 `soundAt/effect` packet；
  - `createSound` 默认 `Sounds.unitCreateBig`，`AssetsProcess.processSounds(...)` 按 `core/assets/sounds` 文件名排序生成 sound id，v158.1 中 `unitCreateBig == 191`；
  - `Fx.unitAssemble` 在 `Fx.java` 的 `Effect.all` 顺序中 id 为 `35`，调用形态为 `Fx.unitAssemble.at(spawn.x, spawn.y, rotdeg() - 90f, plan.unit)`，data 是 output `UnitType`。
- Rust 新增/变化：
  - `audio::standard_sound_id(...)` 先接入 `unitCreate`/`unitCreateBig` 的 Java generated sound id 映射；
  - `entities::FX_UNIT_ASSEMBLE_ID` 记录 v158.1 `Fx.unitAssemble` effect id；
  - `GameRuntime` 新增：
    - `client_local_sound_at_events: Vec<SoundAtCallPacket>`；
    - `client_local_effect_events: Vec<EffectCallPacket2>`；
  - `GameRuntime::apply_client_assembler_unit_spawned_packet(...)` 在重置 assembler state 前，根据 building rotation/area size 计算 Java spawn 点，并排队：
    - `SoundAtCallPacket { sound_id: 191, x, y, volume: create_sound_volume, pitch: 1.0 }`；
    - `EffectCallPacket2 { effect_id: 35, rotation: rotdeg - 90, color: white, data: Content(UnitType) }`；
  - 没有让 Rust server 额外发送 sound/effect packet，避免 Java 客户端在收到 `AssemblerUnitSpawnedCallPacket` 后本地播放一次、又收到额外 packet 再播放一次。
- 新增/更新验证：
  - `cargo test -p mindustry-core standard_sound_ids_follow_upstream_assets_process_order --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_assembler_unit_spawned_packet_like_java --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_assembler_unit_spawned_packet_to_runtime --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_spawn_packet_without_losing_assembler_spawned --lib`
- 仍未完成：
  - `createSound.at(...)` 的 `Mathf.range(0.06f)` pitch 随机目前以确定性 `1.0` 记录，后续接入客户端 RNG/audio backend 时应恢复 Java pitch range；
  - runtime sidecar 已记录 sound/effect 数据，但最终 renderer/audio backend 播放还需继续接 desktop 实际表现层；
  - `Events.fire(new UnitCreateEvent(unit, this))` 仍待迁移到 Rust campaign stats/event bus。

### 12.136 UnitAssembler.spawned UnitCreateEvent 与单位创建统计

- 2026-05-27：继续对照 v158.1 `UnitAssemblerBuild.spawned()` 末尾 `Events.fire(new UnitCreateEvent(unit, this))`，把 Rust UnitAssembler 完成事件从“只 reset/生成 unit/记录 sound-effect”推进到 Java `Logic` 与 `GameService` 的单位创建统计语义。
- Java 依据：
  - `EventType.UnitCreateEvent` 携带 `unit`、可空 `spawner` 与可空 `spawnerUnit`；
  - `Logic` 监听 `UnitCreateEvent`：当 `e.unit.team == state.rules.defaultTeam` 时 `state.stats.unitsCreated++`；若 `state.isCampaign() && !net.client()`，则 `state.getPlanet().stats().unitsProduced.increment(e.unit.type)`；
  - `GameService` 还会在 campaign 中更新 `SStat.unitTypesBuilt`、`buildT5`，并对 `e.unit.team() == player.team()` 增加 `SStat.unitsBuilt`。
- Rust 新增/变化：
  - `GameRuntimeUnitCreateEvent` 记录 `unit_id`、`unit_name`、`team`、`spawner_tile`、`spawner_unit_id`，作为后续事件 bus / service bridge 的 sidecar；
  - `GameRuntime` 新增 `campaign_stats: CampaignStats` 与 `unit_create_events`；
  - `GameRuntime::note_unit_create_event(...)` 统一执行 Java `Logic` 等价副作用：
    - default team unit 增加 `state.stats.units_created`；
    - campaign 且权威端（offline/server，非 net client）增加 `campaign_stats.units_produced[unit]`；
    - 同时保留事件 sidecar；
  - `advance_owned_unit_assemblers_ticks(...)` 在 assembler 完成并 reset 后记录 UnitCreateEvent，覆盖 server/offline owned runtime；
  - `apply_client_assembler_unit_spawned_packet(...)` 在客户端回放 `AssemblerUnitSpawnedCallPacket` 时也记录本地 UnitCreateEvent；客户端只增加 `units_created`，不写 campaign planet stats；
  - world reset 清理 `unit_create_events`，避免跨地图 stale event；
  - `GameServiceUnitCreateSnapshot` 增加 `player_team_unit`，`GameServiceUnitCreatePlan` 增加 `stat_additions`，`unit_create_plan(...)` 现在能表达 Java `SStat.unitsBuilt.add()` 与 default-team `unitTypesBuilt/buildT5` 的分离语义；
  - `server_update_broadcasts_assembler_unit_spawn_packet_when_assembler_completes` 现在同时验证 server runtime 的 UnitCreateEvent sidecar、`units_created` 与 campaign `units_produced`。
- 新增/更新验证：
  - `cargo test -p mindustry-core assembler_unit_spawned --lib`
  - `cargo test -p mindustry-core game_runtime_owned_runtime_blocks_includes_unit_assembler_tick_like_java --lib`
  - `cargo test -p mindustry-core unit_create_plan --lib`
  - `cargo test -p mindustry-server server_update_broadcasts_assembler_unit_spawn_packet_when_assembler_completes --lib`
- 仍未完成：
  - `unit_create_events` 目前是 runtime sidecar，尚未统一 drain/bridge 到正式事件 bus、desktop UI 或 platform service runtime；
  - `UnitFactory`、`Reconstructor`、`PayloadSource`、`UnitSpawnAbility` 的 `UnitCreateEvent` 发射点还需逐个接入同一个 `note_unit_create_event(...)`，避免“只有 assembler 计数”的不一致；
  - `spawner_unit_id` 还未接入 `UnitSpawnAbility` 等 unit-spawner 路径；
  - `GameServiceUnitCreatePlan` 已补齐 player/default team 语义，但还未从真实 runtime event 自动应用到 `DefaultGameService` / achievement backend；
  - Java↔Rust 联机 smoke 仍需后续验证 `AssemblerUnitSpawnedCallPacket`、`UnitSpawnCallPacket`、UnitCreateEvent 统计 sidecar 在同一帧组合下不丢失。

### 12.137 UnitFactory / Reconstructor UnitCreateEvent 接入

- 2026-05-27：继续沿 `UnitCreateEvent` 统一入口迁移，把 Java `UnitFactory` 与 `Reconstructor` 的单位创建/升级完成事件接到 Rust owned runtime，而不是只让 UnitAssembler 计数。
- Java 依据：
  - `UnitFactoryBuild.updateTile()` 在 `progress >= plan.time` 后创建 `plan.unit`、写入 command、生成 `UnitPayload`、`consume()`，随后 `Events.fire(new UnitCreateEvent(payload.unit, this))`；
  - `ReconstructorBuild.updateTile()` 在 `progress >= constructTime` 后把 `payload.unit` 替换为升级后 unit、写入 command、播放效果、`consume()`，随后 `Events.fire(new UnitCreateEvent(payload.unit, this))`。
- Rust 新增/变化：
  - `advance_owned_unit_factories_ticks(...)` 在真实生成 unit payload 且完成 item consume 后调用 `note_unit_create_event(None, unit_name, team, Some(factory_tile), None)`；
  - `advance_owned_unit_reconstructors_ticks(...)` 在 payload unit type/controller patch 成功、完成 consume 后调用同一 `note_unit_create_event(...)`；
  - Reconstructor 的 target upgrade 解析现在同时保留升级后 unit name，避免用 content id 反查时丢失事件统计名称；
  - 现有 UnitFactory/Reconstructor owned runtime 测试扩展断言 `unit_create_events`、`state.stats.units_created` 与 campaign `units_produced`。
- 新增/更新验证：
  - `cargo test -p mindustry-core game_runtime_unit_factory_outputs_payload_to_front_conveyor --lib`
  - `cargo test -p mindustry-core game_runtime_unit_reconstructor_upgrades_payload_on_tick_like_java --lib`
- 仍未完成：
  - `PayloadSource` 的 unit 配置分支仍需接入 UnitCreateEvent；block 配置分支不应发 UnitCreateEvent；
  - `UnitSpawnAbility` 仍需接入 `spawner_unit_id` 非空的 UnitCreateEvent 路径；
  - `UnitFactory` / `Reconstructor` 的 createSound pitch、shake/effect、完整 service/achievement bridge 仍需后续继续补齐。

### 12.138 PayloadSource unit payload UnitCreateEvent 接入

- 2026-05-27：继续对照 v158.1 `PayloadSourceBuild.updateTile()`，把 sandbox `PayloadSource` 配置为 unit 时的 `UnitCreateEvent` 接入 Rust owned runtime；同时锁定配置为 block 时**不**发 UnitCreateEvent。
- Java 依据：
  - `payload == null && unit != null` 时创建 `new UnitPayload(unit.create(team))`，应用可选 `commandPos` 后 `Events.fire(new UnitCreateEvent(p, this))`；
  - `configBlock != null` 分支只创建 `BuildPayload(configBlock, team)`，不触发 `UnitCreateEvent`。
- Rust 新增/变化：
  - `advance_owned_payload_sources_ticks(...)` 在 `PayloadSourceSpawn::Unit` 成功创建 `PayloadRef::Unit` 后记录 `(unit_name, team, tile)`，并在 payload source mutable borrow 结束后统一调用 `note_unit_create_event(...)`；
  - `PayloadSourceSpawn::Block` 分支保持不调用事件入口；
  - `game_runtime_payload_source_spawns_configured_block_payload` 增加断言：即使 default team + campaign，block payload 也不增加 `unit_create_events` / `units_created`；
  - `game_runtime_payload_source_spawns_common_unit_payload_with_command_pos` 增加断言：unit payload 会记录 sidecar、增加 `units_created` 与 campaign `units_produced`，并保留既有 command position payload 编码验证。
- 新增/更新验证：
  - `cargo test -p mindustry-core game_runtime_payload_source_spawns_configured_block_payload --lib`
  - `cargo test -p mindustry-core game_runtime_payload_source_spawns_common_unit_payload_with_command_pos --lib`
- 仍未完成：
  - `UnitSpawnAbility` 的 `UnitCreateEvent(u, null, unit)` 尚未迁移；这条路径需要 `spawner_unit_id` 非空，落点应在 unit/entity ability runtime，而不是 block owned tick；
  - `unit_create_events` 到正式 event bus / `DefaultGameService` / achievement backend 的 bridge 仍未完成；
  - PayloadSource 的其他表现层细节与 Java 完整 sandbox UI 配置仍需后续继续对照。

### 12.139 UnitSpawnAbility 单位产子 runtime / UnitCreateEvent 接入

- 2026-05-27：继续对照 v158.1 `UnitSpawnAbility.update(Unit unit)`，把 Rust 既有的 `UnitSpawnAbility` 纯 plan 从单测层接到真实 `UnitComp` ability slot 与 server update 链路；该闭环不走 block-owned tick，避免把 unit-spawner 做成孤立 helper。
- Java 依据：
  - `timer += Time.delta * state.rules.unitBuildSpeed(unit.team)`；
  - `timer >= spawnTime && Units.canCreate(unit.team, this.unit)` 时，按父单位旋转计算 `spawnX/spawnY` 偏移；
  - 创建子单位、设置位置/旋转，先 `Events.fire(new UnitCreateEvent(u, null, unit))`；
  - 非 client 端再 `u.add(); Units.notifyUnitSpawn(u)`，最后 `timer = 0f`。
- Rust 新增/变化：
  - `UnitSpawnAbility::from_descriptor(...)` 支持 runtime ability 字符串描述（例如 `UnitSpawnAbility:flare:60:0:0` 与 `UnitSpawnAbility(flare,60,0,0)`），用于当前 `UnitType.abilities: Vec<String>` 过渡内容模型；
  - `UnitComp::update_unit_spawn_abilities(...)` 现在从 `AbilityWire.data` 读取/写回 timer，按父单位 transform 调用 `UnitSpawnAbility::update_state(...)`，并在单位 cap 阻止时保留 ready timer，匹配 Java 等待语义；
  - `ServerLauncher::update()` 在同一 playing frame 内调用 `tick_server_unit_spawn_abilities(1.0)`，对 `server_units` 中的父单位逐个 tick ability；
  - 服务端产子时复用现有 `UnitSpawnCallPacket` / `broadcast_server_unit_spawn(...)`，创建的子单位进入 `server_units`，不新增专用网络包；
  - 产子成功时调用 `note_unit_create_event(Some(child_id), unit_name, team, None, Some(parent_id))`，覆盖 Java `new UnitCreateEvent(u, null, unit)` 的 `spawnerUnit` 语义；
  - `Units.canCreate` 对应 Rust `units_can_create(...)`，server 侧按当前 `server_units` 统计同 team/type 数量，并考虑 rules/team cap 与 banned unit。
- 新增/更新验证：
  - `cargo test -p mindustry-core unit_spawn --lib`
  - `cargo test -p mindustry-server unit_spawn_ability --lib`
- 仍未完成：
  - 普通 `UnitType.abilities` 仍是字符串描述，后续需要完整结构化 ability content / mod patcher 支持，避免长期依赖描述字符串；
  - Java client 本地 ability tick / draw 预览 / `spawnEffect.at(...)` 的完整表现层仍未迁移；当前 server 已能即时广播子单位；
  - `unit_create_events` 到正式 event bus / `DefaultGameService` / achievement backend 的 bridge 仍未完成。

### 12.140 EnergyFieldAbility / aegires 单位能力 runtime 接入

- 2026-05-27：继续对照 v158.1 `EnergyFieldAbility.java` 与 `UnitTypes.aegires`，把 Rust 已有的 EnergyField 纯算法接入内容层、`UnitComp` ability slot 与 server units runtime。
- Java 依据：
  - `EnergyFieldAbility.update(Unit unit)`：每帧 `timer += Time.delta`，达到 `reload` 后按 `unit.rotation - 90 + x/y` 计算中心，收集附近 unit/building，按距离排序并限制 `maxTargets`；
  - 同队且受损目标按 `healPercent / 100 * maxHealth` 治疗，同 unit type 再乘 `sameTypeHealMult`；
  - 敌对目标造成 `damage * state.rules.unitDamage(unit.team) * unit.damageMultiplier` 并应用 `status/statusDuration`；
  - `aegires` 参数：`EnergyFieldAbility(40f, 65f, 180f)`、`statusDuration=60f*6f`、`maxTargets=25`、`healPercent=1.5f`、`sameTypeHealMult=0.5f`。
- Rust 新增/变化：
  - `EnergyFieldTarget` 补 `air/targetable`，`EnergyFieldHit` 补 `status_duration`，使 runtime 层能对齐 targetAir/targetGround 与 status 持续时间；
  - `EnergyFieldAbility::from_descriptor(...)` 支持 `EnergyFieldAbility:40:65:180:1.5:0.5:25` 这类参数化描述；
  - `content/unit_types.rs` 为 `aegires` 挂载上述 descriptor，避免只用默认参数；
  - `UnitComp::update_energy_field_abilities(...)` 从 `AbilityWire.data` 读取/写回 timer，调用 `update_targets(...)` 并返回 pulse；
  - `ServerLauncher::tick_server_energy_field_abilities(...)` 在 playing frame 中收集 `server_units` 目标，执行 aegires EnergyField，应用到真实 `HealthComp` / `StatusComp`：
    - 同队受损单位 heal 并标记 `was_healed`；
    - 敌对单位 damage 并应用 `electrified`；
    - server 侧按 v158.1 观察到的 Java `update(...)` 行为不扣 ammo（纯算法仍保留 `unit_ammo_rule` gate 供后续其他版本/规则复用）。
- 新增/更新验证：
  - `cargo test -p mindustry-core energy_field --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server energy_field --lib`
- 仍未完成：
  - `hitBuildings` / building privileged / derelict coreCapture / `Damage.findAbsorber(...)` 尚未接入；当前闭环先覆盖真实 server unit↔unit 目标；
  - EnergyField 的 draw arcs、chain lightning/heal effects、shoot sound 等表现层 sidecar/backend 尚未迁移；
  - 普通 ability content 仍是 descriptor 字符串，后续需要结构化 ability spec / mod patcher 支持。

### 12.141 StatusFieldAbility / oxynoe 同队状态场 runtime 接入

- 2026-05-27：继续对照 v158.1 `StatusFieldAbility.java` 与 `UnitTypes.oxynoe`，把 Rust 状态场能力从纯 pulse 接到 `UnitComp` ability slot、`StatusComp` 与 server same-team unit runtime。
- Java 依据：
  - `StatusFieldAbility.update(Unit unit)`：`timer += Time.delta`，达到 `reload` 且满足 `!onShoot || unit.isShooting` 后，对 `Units.nearby(unit.team, unit.x, unit.y, range, ...)` 内同队单位执行 `other.apply(effect, duration)`；
  - 同时按 `effectX/effectY` 偏移播放 active effect，最后 `timer = 0f`；
  - `oxynoe` 参数：`StatusFieldAbility(StatusEffects.overclock, 60f*6, 60f*6f, 60f)`。
- Rust 新增/变化：
  - `StatusFieldPulse` 增加 `target_ids`，server runtime 可以把 pulse 直接落到实体；
  - `StatusFieldAbility::from_descriptor(...)` 支持 `StatusFieldAbility:overclock:360:360:60`；
  - `content/unit_types.rs` 为 `oxynoe` 挂载上述 descriptor，并在内容测试中锁定；
  - `UnitComp::update_status_field_abilities(...)` 使用 `AbilityWire.data` 存 timer，通过调用方闭包提供目标 id，并保留 `on_shoot`/active effect 计算语义；
  - `ServerLauncher::tick_server_status_field_abilities(...)` 在 playing frame 内收集同队、活着、范围内的 `server_units`（包含自身，匹配 Java 未排除 self 的 `Units.nearby` 路径），对 `pulse.target_ids` 逐个执行 `target.status.apply(effect, duration)`。
- 新增/更新验证：
  - `cargo test -p mindustry-core status_field --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server status_field --lib`
- 仍未完成：
  - `applyEffect` / `activeEffect` 的真实 effect packet 或 desktop 表现层还未接入；当前只保留 active 坐标/参数 pulse 数据；
  - 普通 ability descriptor 仍是过渡模型，后续需要结构化 ability spec / mod patcher；
  - client 本地 ability tick 与 Java↔Rust 视觉 smoke 仍需继续迁移。

### 12.142 SuppressionFieldAbility / navanax-quell-disrupt 治疗抑制 runtime 接入

- 2026-05-27：继续对照 v158.1 `SuppressionFieldAbility.java`、`Damage.applySuppression(...)` 与 `UnitTypes` 中 navanax / quell / disrupt 的使用点，把治疗抑制场从纯 pulse 接到 `UnitType` content、`UnitComp` ability slot、server unit→building runtime。
- Java 依据：
  - `SuppressionFieldAbility.update(Unit unit)`：`active` 为真时累积 `timer += Time.delta`，达到 `maxDelay` 后按 `x/y` 相对 `unit.rotation - 90f` 旋转求场中心，调用 `Damage.applySuppression(unit.team, center, range, reload, maxDelay, ...)`，随后 `timer = 0f`；
  - `Damage.applySuppression(...)` 对范围内敌方 building 调用 `build.applyHealSuppression(reload + 1f, effectColor)`；
  - Java 使用点：`navanax` 默认 `reload/maxDelay/range` 但 `y=-10`；`quell` 设置 `reload=60*8`、`y=1`；`disrupt` 主场 `reload=60*15`、`range=320`、`y=10`，另有两侧 `active=false` 视觉副本。
- Rust 新增/变化：
  - `SuppressionFieldAbility::from_descriptor(...)` 支持 `SuppressionFieldAbility:reload:maxDelay:range:x:y:active:applyParticleChance` 与括号形式；
  - `content/unit_types.rs` 为 `navanax`、`quell`、`disrupt` 挂载 Java 参数对应 descriptor，并保留 disrupt 两个 inactive 视觉副本 descriptor；
  - `UnitComp::update_suppression_field_abilities(...)` 使用 `AbilityWire.data` 存 timer，按单位 transform 产出 `SuppressionFieldPulse`；
  - `ServerLauncher::tick_server_suppression_field_abilities(...)` 在 playing frame 内遍历 `server_units`，对 pulse 范围内敌方 `runtime.buildings` 调用 `apply_heal_suppression(now, reload + 1)`；
  - 新增 server smoke 验证 `quell` 抑制近距离敌方建筑，不影响同队建筑和范围外敌方建筑。
- 新增/更新验证：
  - `cargo test -p mindustry-core suppression_field --lib`
  - `cargo test -p mindustry-core unit_component_ticks_suppression_field --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server suppression_field --lib`
- 仍未完成：
  - `Fx.regenSuppressSeek` 延迟粒子、`effectColor`/`suppress_color_rgba` 表现层与网络 sidecar 尚未接入；
  - building indexer 的精确 footprint/命中半径仍是当前 Rust building center 范围判断，后续需继续对齐 Java `indexer.eachBlock`；
  - client 本地 draw orb/particles、range selection 与结构化 ability spec / mod patcher 仍需继续迁移。

### 12.143 ShieldRegenFieldAbility / scepter-pulsar-bryde 护盾回复场 runtime 接入

- 2026-05-27：继续对照 v158.1 `ShieldRegenFieldAbility.java` 与 `UnitTypes` 中 scepter / pulsar / bryde 的参数，把护盾回复场从纯算法接入 `UnitType` content、`UnitComp` ability slot 与 server same-team unit shield runtime。
- Java 依据：
  - `ShieldRegenFieldAbility.update(Unit unit)`：`timer += Time.delta`，达到 `reload` 后遍历 `Units.nearby(unit.team, unit.x, unit.y, range, ...)`；
  - 对 `other.shield < max` 的同队单位执行 `other.shield = min(other.shield + amount, max)` 且 `other.shieldAlpha = 1f`；
  - 任一目标实际获得护盾时播放 `applyEffect` / `activeEffect` / `sound`，最后 `timer = 0f`；
  - Java 参数：`scepter(25,250,60,60)`、`pulsar(20,40,300,60)`、`bryde(20,40,240,60)`。
- Rust 新增/变化：
  - `ShieldRegenFieldPulse` 增加 `target_ids`，server runtime 可以把按序计算出的 shields 回写到真实单位；
  - `ShieldRegenFieldAbility::from_descriptor(...)` 支持 `ShieldRegenFieldAbility:amount:max:reload:range[:parentizeEffects]` 与括号形式；
  - `content/unit_types.rs` 将 scepter / pulsar / bryde 的裸能力名替换为 Java 参数化 descriptor；
  - `UnitComp::update_shield_regen_field_abilities(...)` 使用 `AbilityWire.data` 存 timer，通过闭包按 ability range 收集目标；
  - `ServerLauncher::tick_server_shield_regen_field_abilities(...)` 在 playing frame 内收集同队、活着、范围内 `server_units`（包含自身），对目标 `ShieldComp.shield` 与 `shield_alpha` 写回，并刷新组件视图。
- 新增/更新验证：
  - `cargo test -p mindustry-core shield_regen_field --lib`
  - `cargo test -p mindustry-core unit_component_ticks_shield_regen_field --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server shield_regen_field --lib`
- 仍未完成：
  - `applyEffect` / `activeEffect` / `sound` 的真实 effect packet、audio 与 desktop 表现层尚未接入；
  - Java `UnitType.shieldColor(other)` 颜色语义当前未进入 effect sidecar；
  - 普通 ability descriptor 仍是过渡模型，后续需结构化 ability spec / mod patcher。

### 12.144 RepairFieldAbility / nova-poly-oct 单位治疗场 runtime 接入

- 2026-05-27：继续对照 v158.1 `RepairFieldAbility.java` 与 `UnitTypes` 中 nova / poly / oct 的参数，把单位治疗场从纯算法接入 `UnitType` content、`UnitComp` ability slot 与 server same-team unit health runtime。
- Java 依据：
  - `RepairFieldAbility.update(Unit unit)`：`timer += Time.delta`，达到 `reload` 后遍历 `Units.nearby(unit.team, unit.x, unit.y, range, ...)`；
  - 若目标 `other.damaged()`，播放 `healEffect` 并设置 `wasHealed = true`；
  - 对范围内同队目标执行 `other.heal((amount + healPercent / 100f * other.maxHealth()) * healMult)`，同类型目标使用 `sameTypeHealMult`；
  - 任一目标受治疗时播放 `activeEffect` / `sound`，最后 `timer = 0f`；
  - Java 参数：`nova(10,240,60)`、`poly(5,480,50)`、`oct(130,120,140)`。
- Rust 新增/变化：
  - `RepairFieldPulse` 增加 `target_ids`，server runtime 可以把 heals 按序回写到真实单位；
  - `RepairFieldAbility::from_descriptor(...)` 支持 `RepairFieldAbility:amount:reload:range[:healPercent[:sameTypeHealMult[:parentizeEffects]]]` 与括号形式；
  - `content/unit_types.rs` 将 nova 的裸能力名替换为参数化 descriptor，并为 poly / oct 补上 Java 里的 RepairFieldAbility；
  - `UnitComp::update_repair_field_abilities(...)` 使用 `AbilityWire.data` 存 timer，通过闭包按 ability range 收集同队目标；
  - `ServerLauncher::tick_server_repair_field_abilities(...)` 在 playing frame 内收集同队、活着、范围内 `server_units`（包含自身），对目标 `HealthComp` 执行 heal 并调用 `heal_mark(...)`。
- 新增/更新验证：
  - `cargo test -p mindustry-core repair_field --lib`
  - `cargo test -p mindustry-core unit_component_ticks_repair_field --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server repair_field --lib`
- 仍未完成：
  - `healEffect` / `activeEffect` / `sound` 的真实 effect packet、audio 与 desktop 表现层尚未接入；
  - building repair / repair command AI 的交互路径不在本闭环内；
  - 普通 ability descriptor 仍是过渡模型，后续需结构化 ability spec / mod patcher。

### 12.145 ForceFieldAbility / quasar-oct 护盾创建与回复 runtime 接入

- 2026-05-27：继续对照 v158.1 `ForceFieldAbility.java` 与 `UnitTypes` 中 quasar / oct 的参数，把力场护盾从纯算法接入 `UnitType` content、`UnitComp` created/update hook 与 server unit shield runtime。
- Java 依据：
  - `created(Unit unit)`：`unit.shield = max`；
  - `update(Unit unit)`：护盾刚破时扣除 `cooldown * regen`，随后按 `Time.delta * regen` 回复到 max 附近；`alpha` 衰减，`radiusScale` 随 active shield 向 1 插值；
  - 护盾为正时扫描敌方 absorbable bullets，命中正多边形内则 `b.absorb()` 并按 `b.type().shieldDamage(b)` 扣盾；
  - Java 参数：`quasar(60,0.4,500,360)`、`oct(140,4,7000,480,8,0)`。
- Rust 新增/变化：
  - `ForceFieldAbility::from_descriptor(...)` 支持 `ForceFieldAbility:radius:regen:max:cooldown[:sides[:rotation]]` 与括号形式；
  - `content/unit_types.rs` 将 quasar 的裸能力名替换为参数化 descriptor，并为 oct 补上 Java ForceField descriptor（保留 RepairField descriptor）；
  - `UnitComp::new(...)` 调用 `apply_created_force_field_abilities()`，对含 ForceField descriptor 的 unit 初始化 `ShieldComp.shield = max`；
  - `UnitComp::update_force_field_abilities(...)` 使用 `AbilityWire.data` 保存 `radius_scale`（负值作为已初始化但破盾/无半径 sentinel），并把 `ForceFieldAbility::update_state(...)` 写回 `ShieldComp.shield`；
  - `ServerLauncher::tick_server_force_field_abilities(...)` 在 playing frame 内 tick `server_units` 的 ForceField 状态，保证 quasar/oct 在 server runtime 中拥有真实护盾值与 regen。
- 新增/更新验证：
  - `cargo test -p mindustry-core force_field --lib`
  - `cargo test -p mindustry-core unit_component_ticks_force_field --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server force_field --lib`
- 仍未完成：
  - server/global bullet list 尚未接入 ForceField 的 absorb/扣盾闭环；当前已保留纯算法 `absorb_bullet(...)` 测试，后续需要接到真实 bullet runtime；
  - `Fx.shieldBreak` / `Fx.absorb`、hit/break sound、shield draw polygon 与 bars 表现层尚未迁移；
  - `AbilityWire.data` 目前只暂存 `radius_scale`/sentinel，完整结构化 ability runtime state 后续需替代该过渡编码。

### 12.146 ShieldArcAbility / tecta 弧形护盾状态 runtime 接入

- 2026-05-27：继续对照 v158.1 `ShieldArcAbility.java` 与 `UnitTypes.tecta`，把弧形护盾从纯算法接入 `UnitType` content、`UnitComp` created/update hook 与 server ability-state tick。
- Java 依据：
  - `created(Unit unit)`：`data = max`；
  - `update(Unit unit)`：`data < max` 时按 `Time.delta * regen` 回复；`active = data > 0 && (unit.isShooting || !whenShooting)`；`widthScale/alpha` 更新；active 时按 `x/y` 相对 `unit.rotation - 90f` 计算弧盾中心；
  - 弧盾 active 时扫描敌方 bullets / units，处理 absorb/deflect、missile unit 安全死亡与普通 unit push；
  - `tecta` 参数：`radius=45`、`angle=82`、`regen=45/60=0.75`、`cooldown=480`、`max=2500`、`y=-20`、`width=8`、`whenShooting=false`、`chanceDeflect=1`。
- Rust 新增/变化：
  - `ShieldArcAbility::from_descriptor(...)` 支持 `ShieldArcAbility:radius:regen:max:cooldown[:angle[:angleOffset[:x[:y[:whenShooting[:width[:chanceDeflect...]]]]]]]`；
  - `content/unit_types.rs` 为 `tecta` 挂载 `ShieldArcAbility:45:0.75:2500:480:82:0:0:-20:false:8:1`；
  - `UnitComp::new(...)` 调用 `apply_created_shield_arc_abilities()`，将对应 `AbilityWire.data` 初始化为 `max`；
  - `UnitComp::update_shield_arc_abilities(...)` 使用 `AbilityWire.data` 存弧盾 data，按单位 transform tick `ShieldArcAbility::update_state(...)` 并写回；
  - `ServerLauncher::tick_server_shield_arc_abilities(...)` 在 playing frame 内 tick `server_units` 的 ShieldArc 状态，使 tecta 在 server runtime 中持续回复弧盾 data。
- 新增/更新验证：
  - `cargo test -p mindustry-core shield_arc --lib`
  - `cargo test -p mindustry-core unit_component_ticks_shield_arc --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server shield_arc --lib`
- 仍未完成：
  - 真实 bullet absorb/deflect、missile unit kill、普通敌方 unit push 尚未接入 server runtime；当前只接状态 tick，纯算法 `apply_bullet_hit(...)` 仍保留；
  - `region=tecta-shield`、arc draw、push/absorb/break effects、sounds 与 shield bar 表现层尚未迁移；
  - `AbilityWire.data` 只能保存弧盾 data，`widthScale/alpha` 仍需结构化 ability runtime state。

### 12.147 SpawnDeathAbility / latum 死亡生成 renale runtime 接入

- 2026-05-27：继续对照 v158.1 `SpawnDeathAbility.java` 与 `UnitTypes.latum`，把死亡生成单位能力从纯 plan 接入 `UnitType` content、`UnitComp` 死亡计划与 server dead-unit removal/spawn runtime。
- Java 依据：
  - `SpawnDeathAbility.death(Unit unit)`：非 client 端计算 `spawned = amount + Mathf.random(randAmount)`；
  - 对每个生成体在 `spread` 范围内随机偏移，调用 `this.unit.spawn(unit.team, unit.x + offset.x, unit.y + offset.y)`；
  - `faceOutwards` 为真时新单位朝向偏移角，否则沿父单位 rotation 加 jitter；
  - `latum` 参数：`new SpawnDeathAbility(renale, 5, 11f)`。
- Rust 新增/变化：
  - `SpawnDeathAbility` 增加 `unit` 字段，并新增 `from_descriptor(...)`，支持 `SpawnDeathAbility:unit:amount:spread[:randAmount[:faceOutwards]]`；
  - `content/unit_types.rs` 为 `latum` 挂载 `SpawnDeathAbility:renale:5:11`；
  - `UnitComp::spawn_death_ability_plans(...)` 从 descriptor 生成死亡产子计划；当前使用确定性等角度 spread，避免 server 测试和回放不可复现；
  - `ServerLauncher::apply_server_unit_death_abilities(...)` 在 playing frame 内移除 dead `server_units`，广播 despawn，并为每个 SpawnDeath plan 创建子 `UnitComp`、`unit.add()`、`broadcast_server_unit_spawn(...)`；
  - 生成子单位时调用 `note_unit_create_event(Some(child_id), unit_name, team, None, Some(parent_id))`，接入统计与 sidecar。
- 新增/更新验证：
  - `cargo test -p mindustry-core spawn_death --lib`
  - `cargo test -p mindustry-core unit_component_plans_spawn_death --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server latum --lib`
- 仍未完成：
  - Java 随机 spread / randAmount 目前用确定性等角度计划替代，后续需要接入可复现 RNG；
  - 普通 unit death/removal 的全局事件总线、death effect、wreck/尸体等完整路径仍需继续迁移；
  - UnitCreateEvent 是否完全等价于 Java `UnitType.spawn(...)` 内部事件需后续对照确认。

### 12.148 MoveEffectAbility / elude 移动特效 ability slot 接入

- 2026-05-27：继续对照 v158.1 `MoveEffectAbility.java` 与 `UnitTypes.elude`，把 elude 的移动尾迹能力从裸内容缺口接入 `UnitType` content 与 `UnitComp` ability slot；本闭环先产出 runtime sidecar plan，避免只在内容表里孤立登记。
- Java 依据：
  - `MoveEffectAbility.update(Unit unit)`：headless 直接返回；否则累加 `counter += Time.delta`；
  - 单位速度满足 `unit.vel.len2() >= minVelocity * minVelocity`，且 `counter >= interval` 或 chance 触发，并且不在玩家队伍雾中时，按 `unit.rotation - 90f` 旋转 `x/y` 偏移并播放 effect；
  - 播放后 `counter %= interval`，按 `amount` 次调用 `effect.at(...)`，颜色使用 `teamColor ? unit.team.color : color`；
  - `elude` 参数：`new MoveEffectAbility(0f, -7f, Pal.sapBulletBack, Fx.missileTrailShort, 4f){{ teamColor = true; }}`。
- Rust 新增/变化：
  - `MoveEffectPlan` 增加 `effect`、`team_color`、`parentize_effects`，从纯坐标计划扩展到能表达 Java effect 调用所需的最小表现层参数；
  - `MoveEffectAbility` 增加 `effect` 字段与 `from_descriptor(...)`，支持 `MoveEffectAbility:x:y:interval:effect[:teamColor[:minVelocity[:amount]]]`；
  - `content/unit_types.rs` 为 `elude` 挂载 `MoveEffectAbility:0:-7:4:missileTrailShort:true`，并在内容覆盖测试中锁定；
  - `UnitComp::update_move_effect_abilities(delta, in_fog)` 使用 `AbilityWire.data` 保存 counter，按单位位置/旋转/速度生成 `MoveEffectPlan`，使该 ability 进入真实 unit runtime slot。
- 新增/更新验证：
  - `cargo test -p mindustry-core move_effect --lib`
  - `cargo test -p mindustry-core unit_component_ticks_move_effect --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
- 仍未完成：
  - Java headless 下不播放视觉；当前 Rust 已保留 plan 入口但尚未把 `MoveEffectPlan` 接入 desktop/client local effect event queue；
  - chance/random offset、rangeLengthMin/Max 与 `inFogTo(Vars.player.team())` 仍是最小参数入口，后续需接入可复现 RNG 与客户端可见性判断；
  - `Pal.sapBulletBack` 颜色、`Fx.missileTrailShort` 真实 effect backend、parentize effects 与本地渲染/音效 sidecar 仍需继续迁移；
  - ability content 仍采用 descriptor 字符串，后续需结构化 ability spec/runtime state，避免复杂 ability 状态继续堆在 `AbilityWire.data`。

### 12.149 RegenAbility / neoplasm 单位自愈 runtime 接入

- 2026-05-27：对照 v158.1 `RegenAbility.java` 与 `NeoplasmUnitType.java`，把 neoplasm preset 的自愈能力从纯公式与 raw marker 推进到 `UnitType` content、`UnitComp` ability slot 与 server unit health runtime。
- Java 依据：
  - `RegenAbility.update(Unit unit)` 每帧执行 `unit.heal((unit.maxHealth * percentAmount / 100f + amount) * Time.delta)`；
  - `NeoplasmUnitType` 为 neoplasm preset 添加 `RegenAbility`，并设置 `percentAmount = 1f / (70f * 60f) * 100f`，即约 70 秒回满；
  - 同 preset 还保留 `LiquidExplodeAbility(neoplasm)` 与 `LiquidRegenAbility(neoplasm, neoplasmHeal)`，本闭环只接自愈 health runtime。
- Rust 新增/变化：
  - `RegenAbility::from_descriptor(...)` 支持 `RegenAbility`、`RegenAbility:percent[:amount]` 与括号形式；
  - `type/unit/neoplasm_unit_type.rs` 将 neoplasm 默认能力从裸 `RegenAbility` 收紧为 `RegenAbility:0.023809524:0`，锁定 Java 70 秒回满参数；
  - `content/unit_types.rs` 内容测试断言 renale 继承该 RegenAbility descriptor；
  - `UnitComp::update_regen_abilities(delta)` 对活着的单位按 descriptor 计算 heal，调用 `heal_mark(...)` 与 `HealthComp::heal(...)`；
  - `ServerLauncher::tick_server_regen_abilities(1.0)` 在 playing frame 中 tick `server_units`，使 renale/latum 自愈进入真实 server unit runtime，并随既有 unit snapshot 同步健康值。
- 新增/更新验证：
  - `cargo test -p mindustry-core regen --lib`
  - `cargo test -p mindustry-core unit_component_ticks_regen --lib`
  - `cargo test -p mindustry-core neoplasm_unit_type_constructor --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server renale_neoplasm_regen --lib`
- 仍未完成：
  - `LiquidExplodeAbility` 死亡洒落 neoplasm puddle 尚未接入 death/world/puddle runtime；
  - `LiquidRegenAbility` 从 neoplasm puddle slurp 回血、`Fx.neoplasmHeal` 与 puddle 消耗尚未接入；
  - RegenAbility 的 UI stat 文案（每秒 flat/percent 展示）与结构化 ability spec 仍需后续迁移。

### 12.150 MoveEffectAbility / elude 客户端本地 effect queue 接入

- 2026-05-27：在 12.148 的 `MoveEffectPlan` 基础上继续补真实客户端表现层入口，把 elude 移动尾迹从 `UnitComp` sidecar plan 推进到 `GameRuntime.client_local_effect_events` 与 `DesktopLauncher::update()`。
- Java 依据：
  - `MoveEffectAbility.update(Unit unit)` 非 headless 时在本地客户端按 interval/chance/速度/雾判断播放 `effect.at(...)`；
  - `elude` 使用 `Fx.missileTrailShort` 与 `teamColor=true`；对照 v158.1 `Fx.java` 静态创建顺序，`Fx.missileTrail = 110`，`Fx.missileTrailShort = 111`（同一计数方式已验证 `Fx.unitAssemble = 35`）。
- Rust 新增/变化：
  - `entities/effect.rs` 增加 `FX_MISSILE_TRAIL_ID=110`、`FX_MISSILE_TRAIL_SHORT_ID=111` 与 `standard_effect_id(...)` 最小映射；
  - `GameRuntime::tick_client_move_effect_abilities(delta, in_fog)` 遍历 `client_unit_snapshot_entities`，调用 `UnitComp::update_move_effect_abilities(...)`，把已知 effect 映射成 `EffectCallPacket2` 写入 `client_local_effect_events`；
  - `DesktopLauncher::update()` 每帧调用该 runtime 入口，使客户端 snapshot 中的 elude 能在真实 desktop update 链路上排队本地 effect；
  - teamColor 通过 `vanilla_teams()` 写入 `EffectCallPacket.color`，普通颜色当前使用白色占位。
- 新增/更新验证：
  - `cargo test -p mindustry-core client_move_effect --lib`
  - `cargo test -p mindustry-desktop elude_move_effect --lib`
- 仍未完成：
  - `chance > 0`、随机 range offset、`rangeLengthMin/Max` 与 fog/player team 判断仍是最小入口，尚未接可复现 RNG 与真实客户端可见性；
  - `parentizeEffects` 的 parent entity 语义尚未通过 effect packet/data 表达；
  - 仅登记 `missileTrail` / `missileTrailShort` 最小 effect id 映射，后续需要系统化迁移 `Fx` registry 与表现参数。

### 12.151 LiquidExplodeAbility / neoplasm 死亡洒液 runtime 接入

- 2026-05-27：对照 v158.1 `LiquidExplodeAbility.java` 与 `NeoplasmUnitType.java`，把 neoplasm preset 的死亡洒落液体从纯 helper 接入 `UnitComp` death plan、server dead-unit lifecycle 与 `Puddles` runtime。
- Java 依据：
  - `LiquidExplodeAbility.death(Unit unit)` 在单位死亡时取 `unit.tileX()/tileY()`；
  - 半径 `rad = max((int)(unit.hitSize / tilesize * radScale), 1)`；
  - 遍历半径内 tile，按距离与 `radAmountScale` 计算 `amount * scaling`，并调用 `Puddles.deposit(tile, liquid, amount * scaling)`；
  - `NeoplasmUnitType` 使用 `liquid = Liquids.neoplasm`，其余参数保持默认：`amount=120`、`radAmountScale=5`、`radScale=1`。
- Rust 新增/变化：
  - `LiquidExplodeAbility::from_descriptor(...)` 支持 `LiquidExplodeAbility:liquid[:amount[:radAmountScale[:radScale[:noiseMag[:noiseScl]]]]]`；
  - `LiquidExplodeAbility::deposit_plans(...)` 生成 tile/amount 计划；当前先使用确定性圆形半径与距离衰减，未接 Java `Simplex.noise2d` 边缘噪声；
  - `UnitComp::liquid_explode_ability_deposit_plans()` 从 unit descriptor 在死亡链路中产出 `LiquidExplodeDepositPlan`；
  - `GameRuntime` 增加 `server_puddles: Puddles`，作为 server-side puddle runtime 的最小承载点；
  - `ServerLauncher::apply_server_unit_death_abilities()` 在移除 dead unit 后调用 `apply_server_liquid_explode_deposits(...)`，通过 `Puddles::deposit_at(...)` 写入真实 puddle runtime。
- 新增/更新验证：
  - `cargo test -p mindustry-core liquid_explode --lib`
  - `cargo test -p mindustry-core unit_component_plans_liquid_explode --lib`
  - `cargo test -p mindustry-server neoplasm_when_renale_dies --lib`
- 仍未完成：
  - Java `Simplex.noise2d` 边缘噪声尚未迁移；当前 deterministic circle 对 renale 最小半径闭环等价到中心 puddle，但 latum 大半径边缘会少噪声细节；
  - server puddle 尚未进入 `EntitySnapshotCallPacket` 广播与 desktop typed runtime 同步；当前先接入 server death lifecycle 与 `Puddles` 数据结构；
  - 液体反应、space/boil 概率分支目前沿 `Puddles` helper 的上下文默认值，后续需接真实 map floor/env 与可复现随机源。

### 12.152 LiquidRegenAbility / neoplasm 吸液回血 runtime 接入

- 2026-05-27：继续对照 v158.1 `LiquidRegenAbility.java` 与 `NeoplasmUnitType.java`，把 neoplasm 单位从同液体 puddle 吸取液体并回血的能力接入 `UnitComp` ability descriptor、server unit health runtime 与 `GameRuntime.server_puddles`。
- Java 依据：
  - `LiquidRegenAbility.update(Unit unit)`：当 `unit.damaged() && !unit.isFlying()` 时，按 `rad = max((int)(unit.hitSize / tilesize * 0.6f), 1)` 扫描附近 tile；
  - 若 tile 上存在同 `liquid` puddle，则每帧取 `fractionTaken = min(puddle.amount, slurpSpeed * Time.delta)`，扣减 puddle 并 `unit.heal(fractionTaken * regenPerSlurp)`；
  - 任意回血后按 `slurpEffectChance` 播放 `slurpEffect`；neoplasm preset 使用 `liquid=neoplasm`、`slurpEffect=Fx.neoplasmHeal`、默认 `slurpSpeed=5`、`regenPerSlurp=6`。
- Rust 新增/变化：
  - `LiquidRegenAbility` 增加 `slurp_effect` 字段与 `from_descriptor(...)`，解析 `LiquidRegenAbility:neoplasm:neoplasmHeal`；
  - `LiquidRegenAbility::slurp_radius(...)` 与 `slurp_tiles(...)` 复现 Java tile 扫描半径；
  - `UnitComp::liquid_regen_abilities()` 从 unit descriptor 暴露 LiquidRegen abilities；
  - `Puddles::slurp_matching_liquid(...)` 对同液体 puddle 扣减 amount 并返回实际取走量；
  - `ServerLauncher::tick_server_liquid_regen_abilities(1.0)` 在 playing frame 中对受伤、非飞行 server unit 执行 slurp + heal，并刷新 unit component views。
- 新增/更新验证：
  - `cargo test -p mindustry-core liquid_regen --lib`
  - `cargo test -p mindustry-core slurp_matching --lib`
  - `cargo test -p mindustry-core unit_component_reads_liquid_regen --lib`
  - `cargo test -p mindustry-server slurps_neoplasm --lib`
- 仍未完成：
  - `slurpEffect` / `Fx.neoplasmHeal` 仅保留 descriptor 字段，尚未接入 effect queue 与随机偏移；
  - `Mathf.chanceDelta(slurpEffectChance)`、`Tmp.v1.rnd(Mathf.random(unit.hitSize/2f))` 尚未接可复现 RNG；
  - `unit.isFlying()` 当前按 Rust `type_info.flying/elevation` 最小判断，后续需与完整 elevation/hover 状态机对齐；
  - server puddle 同步到 desktop snapshot 仍需补齐。

### 12.153 server puddle entity snapshot sync

- 2026-05-27：在 `LiquidExplodeAbility` 与 `LiquidRegenAbility` 已经把 neoplasm puddle 写入 `GameRuntime.server_puddles` 后，继续补齐服务端到客户端的实体快照链路，避免 puddle runtime 只停留在 server sidecar。
- Java / 协议依据：
  - puddle 是实体快照中的 typed entity，外层写 `entity id + class id + sync bytes`；
  - Rust 已有 `PUDDLE_CLASS_ID=13`、`PuddleSyncWire`、`type_io::write_puddle_sync/read_puddle_sync` 与客户端 `EntityClassKind::Puddle` materialize 分发。
- Rust 新增/变化：
  - `Puddles::entries()` 暴露只读迭代器，供服务端 snapshot builder 读取 active puddle entries；
  - `ServerLauncher::server_unit_entity_snapshot_packet()` 在 cargo unit 记录之后继续写入 `runtime.server_puddles`：
    - entity id 使用 `PuddleComp.id`；
    - type id 固定 `PUDDLE_CLASS_ID`；
    - `PuddleSyncWire` 写入 amount、通过 liquid name 反查得到的 content liquid id、`point2_pack(tile.x, tile.y)`、world `x/y`；
    - 已移除或 amount<=0 的 puddle 跳过，content 中找不到 liquid id 的 puddle 跳过。
  - 新增 `server_entity_snapshot_packet_includes_runtime_puddles_for_client_sync`，直接把服务端 packet 喂给 `GameRuntime::apply_client_entity_snapshot_packet_with_content(...)`，断言客户端 `client_puddle_snapshot_entities` materialize 出同 id/amount/tile/liquid 的 puddle。
- 新增/更新验证：
  - `cargo test -p mindustry-server server_entity_snapshot_packet_includes_runtime_puddles_for_client_sync --lib`
- 仍未完成：
  - puddle removal/evaporation 还没有通过 hidden ids 或 delete snapshot 同步给客户端，后续要防止旧 puddle 残影；
  - `server_unit_entity_snapshot_packet` 仍沿用历史函数名，实际已经混合 cargo unit + puddle，后续可在不破坏调用方时重命名为通用 entity snapshot builder；
  - Java `Simplex.noise2d` 边缘噪声、真实 floor/env/space/boil 随机上下文和 `Fx.neoplasmHeal` 表现层仍待迁移。

### 12.154 LiquidRegenAbility / neoplasmHeal effect packet and client queue

- 2026-05-27：继续对照 v158.1 `LiquidRegenAbility.java` 与 `Fx.java`，把 neoplasm 吸液回血后的 `slurpEffect=Fx.neoplasmHeal` 从 descriptor 推进到服务端网络 effect packet 与桌面客户端本地 effect queue。
- Java 依据：
  - `LiquidRegenAbility.update(Unit unit)` 在吸取 puddle 并回血后，`healed && Mathf.chanceDelta(slurpEffectChance)` 时调用 `slurpEffect.at(unit.x + offset, unit.y + offset, unit.rotation, unit)`；
  - `Fx.neoplasmHeal = new Effect(120f, ...)` 并 `.followParent(true).rotWithParent(true)`；
  - 按 v158.1 `Fx.java` effect 声明顺序统计，`Fx.neoplasmHeal` 前有 122 个 effect，故 0-based id 为 `122`；同一计数方式已验证 `missileTrailShort=111`。
- Rust 新增/变化：
  - `entities/effect.rs` 增加 `FX_NEOPLASM_HEAL_ID=122`，`standard_effect_id("neoplasmHeal")` 返回该 id，并从 `entities/mod.rs` re-export；
  - `ServerLauncher::tick_server_liquid_regen_abilities(...)` 在同液体 puddle slurp 后，如果 ability 的 `slurp_effect` 非 `none` 且 `slurp_effect_chance>0`，通过 `broadcast_server_effect_with_data(...)` 发送 `EffectCallPacket2`：
    - effect id 来自 `standard_effect_id("neoplasmHeal")`；
    - x/y/rotation 使用单位当前 transform；
    - data 使用 `TypeValue::Unit(unit_id)`，为后续 parent/follow 语义保留关联；
  - `DesktopLauncher::sync_effect_packets_to_runtime()` 把 `NetClientState` 中收到的 `EffectCallPacket`、`EffectCallPacket2`、`EffectReliableCallPacket` 同步到 `runtime.client_local_effect_events`，避免网络 effect 只停在 net-client telemetry。
- 新增/更新验证：
  - `cargo test -p mindustry-server slurps_neoplasm --lib`
  - `cargo test -p mindustry-desktop syncs_effect_call_packet2 --lib`
- 仍未完成：
  - `Mathf.chanceDelta(slurpEffectChance)` 目前只实现 `chance>0` 的最小门控，尚未接可复现 RNG/delta 概率；
  - Java 的随机 offset `Tmp.v1.rnd(Mathf.random(unit.hitSize/2f))` 暂未实现，当前 effect 位于单位中心；
  - `followParent(true)` / `rotWithParent(true)` 已通过 `TypeValue::Unit(unit_id)` 保留父实体引用线索，但客户端 renderer/EffectState 还未完整解释 parent 跟随语义；
  - `DesktopLauncher::sync_effect_packets_to_runtime()` 目前按 NetClientState 的 last packet/counter 同步，若一个 update 间隔内累积多条同类 effect，后续需要从 net 层保留队列而不是只保留最后一条。

### 12.155 LiquidExplodeAbility / Arc Simplex edge noise

- 2026-05-27：对照 v158.1 `LiquidExplodeAbility.java` 与 Arc `Simplex.noise2d(...)`，把死亡洒液范围从确定性圆形推进到 Java 同款“圆形遍历 + Simplex 噪声削边”。
- Java 依据：
  - `rad = max((int)(unit.hitSize / tilesize * radScale), 1)`；
  - `realNoise = unit.hitSize / noiseMag`；
  - tile 入选条件为 `x*x + y*y <= rad*rad - Simplex.noise2d(0, 2, 0.5f, 1f / noiseScl, x + tx, y + ty) * realNoise * realNoise`；
  - deposit amount 仍保持 `(1f - Mathf.dst(x, y) / rad) * radAmountScale * amount`。
- Rust 新增/变化：
  - `LiquidExplodeAbility::planned_noise_radius(...)` 对 `noise_mag==0` 做安全兜底；
  - `LiquidExplodeAbility::deposit_plans(...)` 使用 `simplex_noise2d(0, 2.0, 0.5, 1.0/noise_scl, center_x+ox, center_y+oy)` 参与 tile 筛选；
  - 新增私有 Arc 兼容 helper：`simplex_noise2d`、`simplex_raw2d`、`simplex_perm`、`simplex_fastfloor`，保留 Arc 的 2D simplex 梯度、归一化与 `fastfloor`/hash 行为；
  - 新增 `liquid_explode_deposit_plans_apply_java_simplex_edge_noise`，用 Java/Arc 稳定样例锁定：
    - `simplex_noise2d(0, 2, 0.5, 1/5, 1, 8) ≈ 0.865014`；
    - `deposit_plans(0,35,10,5)` 从平滑圆的 9 个有效格变为 Java 噪声后的 8 个，剔除 `(1,8)`。
- 新增/更新验证：
  - `cargo test -p mindustry-core liquid_explode --lib`
  - `cargo test -p mindustry-server neoplasm_when_renale_dies --lib`
- 仍未完成：
  - Simplex helper 当前仅迁移本能力需要的 2D 分支，Arc 3D/4D/tiled noise 仍未系统化迁移；
  - floor/env/space/boil 的真实随机上下文仍沿 `Puddles` 默认 helper，后续需要接 map tile 与可复现随机源；
  - death puddle removal/evaporation 的客户端删除同步仍待补齐。

### 12.156 Puddles lifecycle removal and hidden snapshot sync

- 2026-05-27：在 `server_puddles` 已经能由死亡洒液创建、由吸液回血消耗并同步到客户端后，继续补齐 puddle 生命周期删除链路，避免 amount 归零或 update 移除后客户端仍保留旧 puddle typed entity。
- Java 依据：
  - puddle 每帧 `update` 会按 viscosity 蒸发、合并接受量并在 amount 归零/无 liquid/无 tile 时 remove；
  - 网络侧使用 hidden snapshot 类机制通知客户端实体不再可见/存在。
- Rust 新增/变化：
  - `Puddles::update_all(delta, headless)` 遍历当前 puddle，调用 `PuddleComp::update(...)`，删除 removed/empty/no-liquid puddle 并返回对应 entity ids；
  - `ServerLauncher::tick_server_puddles(1.0)` 在 playing frame 中、`LiquidRegenAbility` slurp 后运行，保证被吸干的 puddle 当帧移除；
  - `ServerLauncher::broadcast_server_hidden_snapshot(...)` 将移除的 puddle id 通过 `HiddenSnapshotCallPacket` 广播；
  - `GameRuntime::apply_client_hidden_snapshot_ids(...)` 对 puddle typed runtime 从原先“只标记存在”改为从 `client_puddle_snapshot_entities` 中移除。
- 新增/更新验证：
  - `cargo test -p mindustry-core update_all_removes_empty_puddles --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_puddle --lib`
  - `cargo test -p mindustry-server server_update_slurps_neoplasm --lib`
  - `cargo test -p mindustry-server server_update_hides_puddle --lib`
- 仍未完成：
  - `Puddles::update_all` 当前不做 spread 目标扫描、建筑影响、火焰/particle effect 事件，只先闭合蒸发/empty removal；
  - hidden snapshot 对其他 entity typed maps 仍保持原有 mark/sidecar 语义，只有 puddle 在本轮做实际 remove；
  - 后续需要把 puddle update 的 ripple/steam/fire/particle 事件接到 effect/network 层。

### 12.157 Puddles D4 spread runtime and snapshot

- 2026-05-28：继续对照 Java `PuddleComp.update()`，把 amount 超过 `maxLiquid / 1.5` 时向 `Geometry.d4` 四邻扩散的逻辑接入 `Puddles::update_all` 与 server snapshot。
- Java 依据：
  - `amount >= maxLiquid / 1.5f` 时计算 `deposited = min((amount - maxLiquid / 1.5f) / 4f, 0.3f * Time.delta)`；
  - 遍历 `Geometry.d4`，对存在且可通过的四邻 tile 调用 `Puddles.deposit(other, tile, liquid, deposited, false)`；
  - 原 puddle 扣除 `deposited * targets`。
- Rust 新增/变化：
  - `Puddles::update_all(...)` 不再给 `PuddleComp::update(...)` 固定传 `nearby_spread_targets=0`，而是按 in-bounds D4 neighbors 计算 spread targets；
  - update 后收集 `(target, source, liquid, amount)`，再用 `Puddles::deposit(... initial:false ...)` 创建/合并邻居 puddle，避免遍历时直接二次可变借用；
  - 新增私有 `Puddles::d4_spread_targets(...)`；
  - server 已有 `tick_server_puddles(1.0)` 自动驱动 spread，新邻居 puddle 会通过既有 `EntitySnapshotCallPacket` 同步给客户端。
- 新增/更新验证：
  - `cargo test -p mindustry-core update_all_spreads --lib`
  - `cargo test -p mindustry-server server_update_spreads_overfilled --lib`
- 仍未完成：
  - 当前 spread passability 仅用 `in_bounds` 近似 Java `other != null && (other.block()==air || liquid.moveThroughBlocks)`，尚未接真实 world block/floor passability；
  - spread 引发的 ripple/particle/fire/building puddleOn 事件仍未接入；
  - 液体 update hook `liquid.update(self())` 仍未迁移。

### 12.158 Puddles spread passability from server world/content

- 2026-05-28：继续收紧 12.157 的四向扩散，把 spread target 判定从纯 `in_bounds` 近似推进到 server 真实 world/content solidity。
- Java 依据：
  - `PuddleComp.update()` 扩散时要求 `other != null && (other.block() == Blocks.air || liquid.moveThroughBlocks)`。
- Rust 新增/变化：
  - `Puddles::update_all(...)` 改为委托 `update_all_with_passability(...)`，默认仍允许所有 in-bounds 目标，方便纯 core 测试；
  - `Puddles::update_all_with_passability(delta, headless, passable)` 新增 passability callback，D4 spread target 会同时要求 in-bounds 与 callback 通过；
  - `ServerLauncher::tick_server_puddles(...)` 传入真实 server `World` 与 `ContentLoader`：
    - tile 必须存在；
    - 若 `liquid.move_through_blocks` 为 true 则允许穿过 block；
    - 否则要求 `!world.wall_solid_with_content(x, y, content)`。
  - server spread 测试将东侧邻居设置为 `copper-wall`，验证 water 不向该实体墙扩散，snapshot amount 从 5 收紧为 4。
- 新增/更新验证：
  - `cargo test -p mindustry-core update_all_spread --lib`
  - `cargo test -p mindustry-server server_update_spreads_overfilled --lib`
- 仍未完成：
  - Java 精确条件是 `block()==air || liquid.moveThroughBlocks`；Rust 当前用 content-backed solidity 允许 conveyor 等非 solid block，后续如需 byte-level parity 可改为直接检查 block id 是否 air；
  - floor solid/liquid floor compatibility 已在 deposit helper 中存在，但 spread target 的完整 tile/floor context 仍需从 server world 注入；
  - ripple/particle/fire/building puddleOn/liquid.update hook 仍待接入。

### 12.159 Puddle update side-effect report and server fire snapshot

- 2026-05-28：继续对照 Java `PuddleComp.update()` 的 effects-only 分支，把“高温液体坑接触建筑有概率 `Fires.create(tile)`”从孤立 plan 推进到 server runtime 与 EntitySnapshot。
- Java 依据：
  - `amount >= maxLiquid / 2f && updateTime <= 0f` 时执行 `Units.nearby(...)`、高温液体 + building + `Mathf.chance(0.5)` 起火、`tile.build.puddleOn(self())`；
  - 本轮只闭合 fire 分支，单位 status/ripple 与 building `puddleOn` 继续作为事件报告暴露，后续再接真实 consumer。
- Rust 新增/变化：
  - `PuddleLiquidInfo` 保留 `particle_effect` 名称，避免 `has_particle_effect` 丢失后续 effect packet 所需 id 映射信息；
  - 新增 `PuddleUpdateEvent` / `PuddleUpdateReport`；
  - `Puddles::update_all_with_passability_report(...)` 在原 D4 spread/remove 基础上额外注入：
    - `build_present(x,y)`：server 每 tick 从 world/building ref 刷新 `PuddleTile.build_present`；
    - `fire_chance(x,y,liquid)`：目前 server 使用稳定 hash 近似 Java 0.5 概率，后续应替换为可复现 RNG；
    - 输出 `affect_units/create_fire/puddle_on_building/particle_effect` side-effect event；
  - `GameRuntime` 新增 `server_fires: Fires`，server tick 中按 world 尺寸维护；
  - `ServerLauncher::tick_server_puddles(...)` 消费 `create_fire` event，调用 `Fires::create(...)`；
  - `Fires` 增加 `width/height/entries`，便于 server runtime 维护与 snapshot；
  - `ServerLauncher::server_unit_entity_snapshot_packet()` 将 `server_fires.entries()` 写入 `FIRE_CLASS_ID + FireSyncWire`，使用稳定 tile-derived runtime entity id，客户端既有 Fire typed snapshot 可直接 materialize。
- 新增/更新验证：
  - `cargo test -p mindustry-core update_all_report_exposes_hot_puddle_fire_and_building_events --lib`
  - `cargo test -p mindustry-core create_adds_fire_and_refreshes_existing_lifetime --lib`
  - `cargo test -p mindustry-server server_entity_snapshot_packet_includes_runtime_fires_for_client_sync --lib`
  - `cargo test -p mindustry-server server_update_creates_fire_when_hot_puddle_touches_building --lib`
- 仍未完成：
  - `Units.nearby` 对 grounded/非 hovering 单位施加 liquid status 120 tick，以及移动时 `Fx.ripple`，尚未接入；
  - `tile.build.puddleOn(self())` 还没有对应 Rust building consumer；
  - `liquid.update(self())` 只保留事件链入口，`CellLiquid.update()` 的 neoplasm 扩散/伤害逻辑仍待迁移；
  - 起火概率当前为稳定 hash 近似，后续要接 Java `Mathf.chance(0.5)` 等价 RNG/delta 语义。

### 12.160 Fx.ripple standard effect id

- 2026-05-28：为后续接入 `PuddleComp.update()` 的单位踩液体波纹分支，先补齐 v158.1 `Fx.ripple` 的标准 effect id 映射。
- Java 依据：
  - `Effect` 构造时使用 `this.id = all.size`；
  - `Fx.java` 中 `ripple = new Effect(30, ...)` 前共有 243 个 effect 声明，因此 `Fx.ripple` 的 0-based id 为 `243`。
- Rust 新增/变化：
  - `entities/effect.rs` 增加 `FX_RIPPLE_ID = 243`；
  - `standard_effect_id("ripple")` 返回 `Some(243)`；
  - 顺手补上已有常量 `FX_UNIT_ASSEMBLE_ID` 的 `standard_effect_id("unitAssemble")` 映射；
  - `entities/mod.rs` re-export `FX_RIPPLE_ID`。
- 新增/更新验证：
  - `cargo test -p mindustry-core standard_effect_ids_include_puddle_ripple_dependencies --lib`
- 仍未完成：
  - Puddle 对单位的 `Fx.ripple.at(unit.x, unit.y, unit.type.rippleScale, liquid.color)` 网络/本地 effect 发送尚未接入；
  - 仍需补 `Units.nearby`、单位 hitbox overlap、grounded/hovering 判定与 liquid status apply。

### 12.161 Puddle Units.nearby status/ripple server consumer

- 2026-05-28：继续对照 Java `PuddleComp.update()`，把 effects-only 分支中的 `Units.nearby(...)` 从 report event 接入 server unit runtime 与网络 effect。
- Java 依据：
  - 查询矩形：`rect.setSize(clamp(amount / (maxLiquid / 1.5f)) * 10f).setCenter(x, y)`；
  - 命中单位要求 `unit.isGrounded() && !unit.type.hovering` 且 unit hitbox 与矩形 overlap；
  - 命中后 `unit.apply(liquid.effect, 60 * 2)`；
  - 若 `unit.vel.len2() > 0.1f * 0.1f`，发送 `Fx.ripple.at(unit.x, unit.y, unit.type.rippleScale, liquid.color)`。
- Rust 新增/变化：
  - `ServerLauncher::tick_server_puddles(...)` 消费 `PuddleUpdateEvent.affect_units`：
    - 按 Java 同款矩形尺寸筛选 `server_units`；
    - 跳过 dead、非 grounded、hovering 单位；
    - 用 `ContentLoader::status_effect_by_name(liquid.effect)` 解析并 `unit.status.apply(..., 120.0)`；
    - 移动单位收集 ripple effect，并通过新增 `broadcast_server_effect_colored(...)` 发送 `EffectCallPacket`；
    - ripple rotation 使用 `unit.type_info.ripple_scale`，color 使用 liquid color。
- 新增/更新验证：
  - `cargo test -p mindustry-server server_update_applies_puddle_liquid_status_and_ripple_to_ground_units --lib`
  - `cargo test -p mindustry-server server_update_creates_fire_when_hot_puddle_touches_building --lib`
  - `cargo check -p mindustry-server`
  - `cargo fmt --check`
  - `git diff --check`
- 仍未完成：
  - 目前使用 server-side AABB 近似 Java `Units.nearby` 空间索引，后续可接真实 entity indexer/group；
  - `tile.build.puddleOn(self())` 还未接入；
  - `CellLiquid.update(Puddle)` / neoplasm reaction 仍待迁移；
  - ripple 已发送网络 effect，但 desktop renderer/backend 仍只是进入现有本地 effect sidecar。

### 12.162 CellLiquid.update neighbor building target-liquid absorption

- 2026-05-28：对照 Java `CellLiquid.update(Puddle)`，先把 neoplasm 从周边 building 液体模块吸收 `spreadTarget=water` 并转换沉积 neoplasm 的分支接入 server runtime。
- Java 依据：
  - `CellLiquid` 默认 `maxSpread=0.75f`、`spreadConversion=1.2f`、`spreadDamage=0.11f`、`removeScaling=0.25f`；
  - `Liquids.neoplasm` 设置 `spreadTarget = Liquids.water`，`capPuddles=false`，`moveThroughBlocks=true`，`blockReactive=false`，并可停留在 water/oil/cryofluid/arkycite 上；
  - `update(Puddle)` 对 `Geometry.d4c` 周边 building：若 `build.liquids.get(spreadTarget)>0.0001`，移除 `amount * removeScaling`，并 `Puddles.deposit(tile, this, amount * spreadConversion)`。
- Rust 新增/变化：
  - `type/liquid.rs` 增加 CellLiquid 运行时字段：`cell_spread_target`、`cell_max_spread`、`cell_spread_conversion`、`cell_spread_damage`、`cell_remove_scaling`；
  - `content/liquids.rs` 给 neoplasm 接入 `cell_spread_target=water` 与 `can_stay_on=[water, oil, cryofluid, arkycite]`；
  - `PuddleLiquidInfo::from(&Liquid)` 现在保留 CellLiquid 字段，`reaction_target` 来自 `Liquid.cell_spread_target`；
  - `PuddleUpdateEvent::from_plan(...)` 现在也会为 `liquid_update` 生成事件，保证 Java `liquid.update(self())` 不是只在 effects-only 分支出现；
  - `ServerLauncher::tick_server_puddles(...)` 消费带 `reaction_target` 的 `liquid_update` event：
    - 按 d4+d4 diagonal 扫描邻接 building；
    - 从真实 `BuildingComp.liquids` 中移除目标液体；
    - 将吸收量按 `spreadConversion` 转换为当前 cell liquid 并沉积到目标 tile 的 `server_puddles`；
    - 同步保留 current-building damage/spread 的初步逻辑入口。
- 新增/更新验证：
  - `cargo test -p mindustry-server server_puddle_cell_liquid_update_absorbs_spread_target_from_neighbor_building --lib`
  - `cargo test -p mindustry-core liquid_defaults_match_java_constructor_shape --lib`
  - `cargo test -p mindustry-core liquid_core_properties_match_upstream_subset --lib`
  - `cargo test -p mindustry-core update_all_report_exposes_hot_puddle_fire_and_building_events --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `cargo fmt --check`
  - `git diff --check`
- 仍未完成：
  - `Events.fire(Trigger.neoplasmReact)` 还没有 Rust event bus 等价物；
  - current-building damage/spread 已有入口但仍缺更细的 Java parity 测试；
  - d4c 顺序/集合当前按 d4 + diagonal 显式数组近似，后续应迁移统一 Geometry 常量。

### 12.163 CellLiquid.update nearby puddle absorption/replacement

- 2026-05-28：继续对照 Java `CellLiquid.update(Puddle)`，把 `spread to nearby puddles` 分支接入 `Puddles` 与 server tick，避免 neoplasm 只会吸建筑液体、不会吞噬邻接 water puddle。
- Java 依据：
  - 遍历 `Geometry.d4` 四邻 tile；
  - 若 `Puddles.get(tile)` 存在且 `other.liquid == spreadTarget`：
    - `amount = min(other.amount, max(maxSpread * Time.delta * scaling, other.amount * 0.25f * scaling))`；
    - `other.amount -= amount`，当前 puddle `amount += amount`；
    - 若 `other.amount <= maxLiquid / 3f`，执行 `other.remove()` 并 `Puddles.deposit(tile, puddle.tile, this, max(amount, maxLiquid / 3f))`。
- Rust 新增/变化：
  - `PuddleCellAbsorbReport` 记录吸收次数、替换次数、吸收量和被移除 puddle entity id；
  - `Puddles::absorb_neighbor_target_puddles(...)` 在核心集合内完成四邻 target puddle 扣减、source amount 增量、低残量 remove + replacement deposit，避免 server 层直接多重可变借用 `HashMap`；
  - `ServerLauncher::tick_server_puddles(...)` 在消费 `liquid_update` event 时调用该 helper，并把额外 removed ids 合并到现有 `HiddenSnapshotCallPacket` 广播列表；
  - `CellLiquid.update` 入口遵守 Java `Vars.state.rules.fire` 门控，rules.fire=false 时跳过 neoplasm spread/update 分支；
  - 修正 current-building damage/spread 分支：当前 tile 没有 building 时不再 `continue` 跳过后续 nearby puddle 吸收分支。
- 新增/更新验证：
  - `cargo test -p mindustry-core cell_liquid_absorbs --lib`
  - `cargo test -p mindustry-server server_puddle_cell_liquid_update_absorbs_neighbor_target_puddle_and_hides_removed_id --lib`
- 仍未完成：
  - building 吸收 deposit 与 nearby puddle 吸收之间的执行顺序仍是 Rust 现有批处理近似，后续如出现 parity 差异需进一步收紧到 Java 单 puddle update 顺序；
  - `Geometry.d4/d4c` 仍是局部显式数组，后续应统一迁移 Geometry 常量。

### 12.165 CellLiquid.update neoplasmReact trigger runtime record

- 2026-05-28：对照 Java `Events.fire(Trigger.neoplasmReact)`，把 neoplasm 反应事件从“GameService 映射已存在但没人触发”推进到 server/runtime 事件记录链。
- Java 依据：
  - `EventType.Trigger.neoplasmReact` 是普通 trigger enum；
  - `CellLiquid.update(Puddle)` 在任意分支发生 `reacted` 且当前液体是 `Liquids.neoplasm` 时触发；
  - `GameService` 中 `trigger(Trigger.neoplasmReact, neoplasmWater)` 将其映射为事件型成就 `neoplasmWater`。
- Rust 新增/变化：
  - `GameRuntimeTriggerEvent { trigger, campaign }` 与 `GameRuntime::note_trigger_event(...)`：以现有 runtime event-vector 风格记录 trigger，并保留 `GameState::is_campaign()` 快照；
  - `GameRuntime::clear_runtime_sidecars(...)` 同步清理 `trigger_events`，避免换图/重载后复用旧 trigger；
  - `ServerLauncher::tick_server_puddles(...)` 在 CellLiquid building 吸收、current-building damage/spread、nearby puddle 吸收/替换任一分支发生反应，且 liquid name 为 `neoplasm` 时记录 `Trigger::NeoplasmReact`；
  - 现有 `GameServiceState::trigger_plan(...)` 已验证 `Trigger::NeoplasmReact -> Achievement::NeoplasmWater`，本轮把事件源接到 runtime 记录点。
- 新增/更新验证：
  - 扩展 `server_puddle_cell_liquid_update_absorbs_neighbor_target_puddle_and_hides_removed_id`：设置 campaign sector，触发 neoplasm-water 邻接 puddle 反应后断言 `runtime.trigger_events` 包含 campaign `Trigger::NeoplasmReact`；
  - `cargo test -p mindustry-server server_puddle_cell_liquid_update_absorbs_neighbor_target_puddle_and_hides_removed_id --lib`
  - `cargo test -p mindustry-core trigger_plan_maps_java_game_service_triggers --lib`
- 仍未完成：
  - runtime `trigger_events` 还未自动流入 `DefaultGameService`/平台成就服务；目前先闭合“事件源 → runtime trigger → 已有 GameService trigger_plan 映射”的可验证链路；
  - building 吸收 deposit 与 nearby puddle 吸收之间的执行顺序仍是 Rust 现有批处理近似，后续如出现 parity 差异需进一步收紧到 Java 单 puddle update 顺序；
  - `Geometry.d4/d4c` 仍是局部显式数组，后续应统一迁移 Geometry 常量。

### 12.164 CellLiquid.update current-building damage/spread regression

- 2026-05-28：为上一节已存在的 current-building damage/spread 入口补 server 回归，锁定 Java `CellLiquid.update(Puddle)` 中“液体坑所在 building 含 spreadTarget 时”的行为。
- Java 依据：
  - 条件：`spreadDamage > 0 && puddle.tile.build != null && puddle.tile.build.liquids != null && puddle.tile.build.liquids.get(spreadTarget) > 0.0001f`；
  - `amountSpread = min(build.liquids.get(spreadTarget) * spreadConversion, maxSpread * Time.delta) / 2f`；
  - 对 `Geometry.d4` 执行 `Puddles.deposit(puddle.tile, other, puddle.liquid, amountSpread)`；
  - 对当前 building 造成 `spreadDamage * Time.delta * scaling` 伤害；damage/spread 分支本身不移除 current building 中的 spreadTarget 液体，但前置 `Geometry.d4c` 吸收分支会覆盖 center tile。
- Rust 验证：
  - 新增 `server_puddle_cell_liquid_update_damages_target_liquid_building_and_reaccepts_spread`：
    - 在 neoplasm puddle 所在 tile 放置带 water 的 `liquid-router`；
    - tick 后断言 building health 下降；
    - 断言 building water 会因 `Geometry.d4c` 的 center tile 吸收而下降；
    - 断言 source puddle 通过 same-liquid deposit 的 `accepting` 接收到 `amountSpread`。
- 已跑验证：
  - `cargo test -p mindustry-server server_puddle_cell_liquid_update_damages_target_liquid_building_and_reaccepts_spread --lib`
- 仍未完成：
  - `Events.fire(Trigger.neoplasmReact)` 还没有 Rust event bus 等价物；
  - building 吸收 deposit 与 nearby puddle 吸收之间的执行顺序仍是 Rust 现有批处理近似，后续如出现 parity 差异需进一步收紧到 Java 单 puddle update 顺序；
  - `Geometry.d4/d4c` 仍是局部显式数组，后续应统一迁移 Geometry 常量。

### 12.166 Arc Geometry.d4/d4c neighbor constants

- 2026-05-28：用本地 Gradle Arc jar 验证 `arc.math.geom.Geometry` 方向数组：
  - `d4 = [(1,0), (0,1), (-1,0), (0,-1)]`；
  - `d4c = [(1,0), (0,1), (-1,0), (0,-1), (0,0)]`；
  - 这确认此前把 `d4c` 近似为四邻+四对角是错误的，且漏掉了 center tile。
- Rust 新增/变化：
  - `world::build` 新增并导出 `ORTHOGONAL_WITH_CENTER_NEIGHBORS`，与既有 `ORTHOGONAL_NEIGHBORS` 共同作为 Java `Geometry.d4/d4c` 的共享常量；
  - `Puddles::d4_spread_targets(...)` 与 `Puddles::absorb_neighbor_target_puddles(...)` 改用 `ORTHOGONAL_NEIGHBORS`；
  - `ServerLauncher::tick_server_puddles(...)` 的 CellLiquid 周边 building 吸收改用 `ORTHOGONAL_WITH_CENTER_NEIGHBORS`，因此 current tile building 会先按 Java d4c 参与 target-liquid 吸收，再进入 damage/spread 分支；
  - current-building damage/spread 测试同步收紧：water 应下降，source accepting 至少包含 d4c center 转换沉积。
- 新增/更新验证：
  - `orthogonal_neighbor_constants_match_arc_geometry_d4_and_d4c`
  - `server_puddle_cell_liquid_update_damages_target_liquid_building_and_reaccepts_spread`
- 仍未完成：
  - 其他散落在 AI/pathfinder/block runtime 的私有 D4 常量尚未统一替换；本轮先修正 Puddles/CellLiquid 主链路，避免扩大改动面。

### 12.167 UnitCargoLoader tether unit snapshot preserves local CargoAI sidecar

- 2026-05-28：修复真实 `ServerLauncher -> DesktopLauncher` smoke 中 `UnitTetherBlockSpawnedCallPacket` 已把客户端 `manifold` 物化为 cargo unit，但随后 `EntitySnapshot` 的 `UnitSyncWire.controller=Ground` 覆盖本地 `CargoAI` sidecar 的问题。
- Java/Rust 依据：
  - `CargoAI` / `UnitCargoLoaderBuild.spawned(id)` 关联属于运行态/tether 语义，当前 Rust `UnitControllerState::Cargo` 还没有独立 Java wire controller，`to_sync_wire()` 会按现有兼容路径写成 `ControllerWire::Ground`；
  - 因此客户端在已经由 tether packet 物化出 cargo unit 后，普通 unit snapshot 只能同步血量/位置/物品等 wire 字段，不能抹掉本地 tether/cargo sidecar。
- Rust 新增/变化：
  - `GameRuntime::apply_client_unit_sync_wire(...)` 在已有 unit 是 `Cargo`、且 incoming controller 是 `Ground`、且本地有 `cargo_ai/building_tether` 时，保留 `UnitControllerState::Cargo`、`CargoAiRuntimeState` 与 `BuildingTetherComp`；
  - 新增回归 `game_runtime_preserves_client_cargo_tether_when_unit_snapshot_arrives`；
  - 真实联机 smoke `real_server_desktop_unit_cargo_loader_tether_spawn_syncs_to_client_runtime` 重新通过。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_preserves_client_cargo_tether_when_unit_snapshot_arrives --lib`
  - `cargo test -p mindustry-tests real_server_desktop_unit_cargo_loader_tether_spawn_syncs_to_client_runtime --lib`

### 12.168 Runtime trigger_events -> DefaultGameService trigger achievement bridge

- 2026-05-28：继续补齐 `Events.fire(Trigger.neoplasmReact)` 的后半段消费链，避免事件只停在 `GameRuntime.trigger_events`。
- Java 依据：
  - `GameService.trigger(Trigger.neoplasmReact, neoplasmWater)` 将 campaign 下的 `neoplasmReact` 映射到 `Achievement.neoplasmWater`；
  - `ThoriumReactorOverheat` 等 trigger 仍走同一 `trigger_plan(...)` 统计/成就计划入口。
- Rust 新增/变化：
  - `DefaultGameService` 增加最小 runtime backing store：`stats: BTreeMap<String, i32>`、`achievements: BTreeSet<String>`、`stats_store_count`，并实现 `StatService`/`AchievementService` 的真实读写；
  - `ClientLauncher` 增加 `AchievementState` cache，作为 `GameServiceEventPlan::apply_to(...)` 的去重/缓存状态；
  - `GameRuntime::drain_trigger_events()` 以局部队列 drain 模式消费 trigger，不引入全局 event bus；
  - `DesktopLauncher::sync_runtime_trigger_events_to_service()` 在 update 中把 runtime trigger 转为 `GameServiceTriggerSnapshot`，复用 `trigger_plan -> apply_to` 写入 `DefaultGameService`；
  - 新增回归：
    - `game_runtime_drain_trigger_events_returns_and_clears_local_queue`
    - `default_game_service_platform_methods_persist_runtime_stats_and_achievements`
    - `trigger_plan_apply_to_writes_trigger_stats_and_achievements_into_service_runtime`
    - `desktop_launcher_drains_runtime_trigger_events_into_game_service`
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_drain_trigger_events_returns_and_clears_local_queue --lib`
  - `cargo test -p mindustry-core trigger_plan_apply_to_writes_trigger_stats_and_achievements_into_service_runtime --lib`
  - `cargo test -p mindustry-core default_game_service_platform_methods_persist_runtime_stats_and_achievements --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_drains_runtime_trigger_events_into_game_service --lib`
  - `cargo check --workspace`
  - `cargo test --workspace`
  - `cargo fmt --check`
  - `git diff --check`
- 仍未完成：
  - 目前 bridge 覆盖“持有 `DesktopLauncher.runtime + ClientLauncher.service` 的本地客户端路径”；server runtime 触发到远端客户端平台服务仍需要后续网络事件/packet 或更完整的 Java event bus 迁移；
  - `DefaultGameService` backing store 仍是内存态，后续平台/磁盘持久化要对齐 Java 平台实现。

### 12.169 CellLiquid.update immediate deposit ordering

- 2026-05-28：继续收紧 Java `CellLiquid.update(Puddle)` 顺序。Java 在 `Geometry.d4c` building 吸收和 current-building damage/spread 分支中会立即调用 `Puddles.deposit(...)`，然后才进入 nearby puddle absorption；Rust 之前把这些 deposit 暂存在 `cell_deposits`，等 nearby puddle absorption 后才统一落库。
- 影响：
  - 当邻接 tile 同时有 target-liquid building 和 target-liquid puddle 时，Java 的 building deposit 会先尝试把 neoplasm 倒到现有 water puddle 上，走 `water.react(neoplasm)=0`，不会在 replacement 后变成 same-liquid accepting；
  - Rust 延迟落库会先把 water puddle replacement 成 neoplasm，再把 building deposit 当作 same-liquid accepting 加到 replacement puddle，导致同 tick 状态偏大。
- Rust 新增/变化：
  - `ServerLauncher::tick_server_puddles(...)` 不再维护 `cell_deposits` 延迟队列；
  - d4c building 吸收后立即 `server_puddles.deposit(...)`；
  - current-building damage/spread 分支先计算 `amountSpread`，立即对 d4 方向 deposit，再对 building damage，之后才进入 nearby puddle absorption；
  - 新增回归 `server_puddle_cell_liquid_building_deposit_precedes_neighbor_absorb_like_java`。
- 已跑验证：
  - `cargo test -p mindustry-server server_puddle_cell_liquid_building_deposit_precedes_neighbor_absorb_like_java --lib`
  - `cargo test -p mindustry-server server_puddle_cell_liquid_update --lib`
  - `cargo check --workspace`
  - `cargo fmt --check`
  - `git diff --check`
- 仍未完成：
  - `Puddles::update_all_with_passability_report(...)` 普通 overfilled spread 仍是先收集 `spread_deposits` 后统一落库，后续如要进一步贴近 Java `PuddleComp.update()`，应验证并收紧“同 tick later puddle 是否能看到 earlier puddle spread”的顺序。

### 12.170 PuddleComp.update overfilled spread immediate deposit ordering

- 2026-05-28：继续收紧 Java `PuddleComp.update()` 的普通 overfilled spread 顺序，补上上一节留下的欠账。
- Java 依据：
  - `amount >= maxLiquid / 1.5f` 时计算 `deposited`；
  - 遍历 `Geometry.d4`，每个可通过目标立即执行 `Puddles.deposit(other, tile, liquid, deposited, false)`；
  - 同一 `Groups.puddle.update()` tick 内，后续已经存在的 puddle 会看到前面 puddle 的 same-liquid deposit 写入 `accepting`，并在自己的 `update()` 开头消费；
  - `EntityGroup.update()` 按 `array.items[index].update()` 与动态 `array.size` 顺序迭代，既有实体顺序不能由 `HashMap` 随机迭代决定。
- Rust 新增/变化：
  - `Puddles::update_all_with_passability_report(...)` 先按 `(puddle.id, tile)` 排序当前 tick 的既有 puddle，避免 `HashMap` 顺序影响 Java-like entity update 顺序；
  - 每个 puddle 的 overfilled spread deposits 在该 puddle 更新结束后立即写入 `Puddles::deposit(...)`，不再等整轮 update 结束后统一落库；
  - 这样同 tick 中 id 更靠后的既有 puddle 会在本轮 update 里消费 earlier puddle 写入的 `accepting`。
- 新增验证：
  - `puddle_update_spread_deposits_are_visible_to_later_puddles_same_tick`
    - 构造 id 更小的 overfilled water puddle 与东侧 id 更大的 water puddle；
    - 断言 update 后后者 `accepting == 0`，说明 same-tick spread deposit 已被后者自己的 `PuddleComp.update()` 消费；
    - 断言后者 amount 增大，说明 earlier spread deposit 对 later puddle 可见。
- 已跑验证：
  - `cargo test -p mindustry-core puddle_update_spread_deposits_are_visible_to_later_puddles_same_tick --lib`
  - `cargo test -p mindustry-core update_all_spread --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo test -p mindustry-server server_puddle_cell_liquid_building_deposit_precedes_neighbor_absorb_like_java --lib`
  - `cargo test -p mindustry-server puddle --lib`
- 仍未完成：
  - 新创建的 spread puddle 同 tick 动态更新已在 12.171 继续收紧；
  - `Puddles` remove/registry 与 Java entity group remove 的即时性仍有差异风险，后续应在继续迁移 puddle entity lifecycle 时补严格测试。

### 12.171 Puddles update follows EntityGroup dynamic append for created spread puddles

- 2026-05-28：继续对照 Java `EntityGroup.update()` 的动态 `array.size` 迭代语义，修正上一节仍保留的“tick 开始快照”差异。
- Java 依据：
  - `EntityGroup.update()` 使用 `for(index = 0; index < array.size; index++) array.items[index].update();`；
  - `Puddles.deposit(...)` 在可创建 puddle 时会 `Puddle.create()`、`register(puddle)`、`puddle.add()`，新实体追加到 group；
  - 因为循环条件每轮读取当前 `array.size`，当前 tick 内新 append 的 puddle 也会继续更新。
- Rust 新增/变化：
  - `Puddles::update_all_with_passability_report(...)` 改为 index-based queue；
  - 初始 queue 仍按 `(puddle.id, tile)` 排序，模拟既有 entity group 顺序；
  - 每个 immediate spread deposit 若创建了新 puddle，则把新 puddle 的 `(id, tile)` append 到同一 queue；
  - 用 `processed_ids` 防止重复处理同一 puddle id；
  - `update_all_spreads_overfilled_puddles_to_d4_neighbors` 期望从 `0.3` 收紧为 `0.2`，表示新建邻居在同一 tick 里又执行了一次自身蒸发 update。
- 已跑验证：
  - `cargo test -p mindustry-core update_all_spread --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo test -p mindustry-server puddle --lib`
- 仍未完成：
  - 基础 remove 即时清理已在 12.172 继续收紧，但 Java `EntityGroup.remove` 的 swap/index 修正与 Rust `HashMap` 仍不完全同构；
  - `CellLiquid.update` 仍在 server report 后统一处理，尚未完全 inline 到每个 puddle update 末尾；
  - `puddle_on_building`/`particle_effect` event 的真实 consumer 仍需后续迁移或确认 Java vanilla no-op/headless 条件。

### 12.172 Puddles remove is immediate before later same-tick deposits

- 2026-05-28：继续收紧 Java `PuddleComp.remove()` / `Puddles.remove(tile)` 的即时 tile registry 语义。
- Java 依据：
  - `PuddleComp.update()` 在 `amount <= 0f` 时立即 `remove()` 并返回；
  - `PuddleComp.remove()` 替换为 `Puddles.remove(tile)`，会立即 `world.tiles.setPuddle(tile.array(), null)`；
  - 因此同 tick 后续其他 puddle 的 `Puddles.deposit(...)` 打到该 tile 时应看到 `get(tile) == null`，从而创建新 puddle，而不是写入即将删除的旧 puddle `accepting`。
- Rust 新增/变化：
  - `Puddles::update_all_with_passability_report(...)` 不再把 removed puddle 延迟到整轮末尾清理；
  - 当 `PuddleComp.update(...)` 返回 removed/amount<=0/liquid missing 后，立即从 `self.puddles` 删除对应 tile，并继续处理后续 queue；
  - 新增 `update_all_removes_empty_puddle_before_later_same_tick_deposit`：
    - 先创建低量 water puddle，使其本 tick 先被删除；
    - 再创建后续 overfilled source，使其同 tick spread 到已清空 tile；
    - 断言旧 id 被报告 removed，tile 上出现新 id puddle，且新 puddle 因 12.171 的动态 append 在同 tick 蒸发到 `0.2`。
- 已跑验证：
  - `cargo test -p mindustry-core update_all_removes_empty_puddle_before_later_same_tick_deposit --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo test -p mindustry-server puddle --lib`
- 仍未完成：
  - Rust `HashMap` tile registry 与 Java `EntityGroup.remove` 的 swap/index 修正仍不完全同构，复杂“删除非当前 index + append replacement”场景后续还需更多回归；
  - `CellLiquid.update` inline 时机已在 12.173 继续收紧。

### 12.173 Server CellLiquid.update inline per-puddle event callback

- 2026-05-28：继续收紧 Java `PuddleComp.update()` 末尾 `liquid.update(self())` 的执行时机，避免 Rust server 把所有 puddle base update 跑完后才批量处理 `CellLiquid.update`。
- Java 依据：
  - `PuddleComp.update()` 的末尾直接调用 `liquid.update(self())`；
  - 因此较早 neoplasm puddle 的 `CellLiquid.update` 可以在同一 `Groups.puddle.update()` tick 中影响后续 water puddle 的 base update；
  - 若 neoplasm 先把相邻 water puddle 降到 `maxLiquid / 1.5f` 以下，后续 water puddle 本 tick 不应再执行 overfilled spread。
- Rust 新增/变化：
  - `Puddles::update_all_with_passability_report_and_event_handler(...)`：
    - 保留旧 `update_all_with_passability_report(...)` API；
    - 新增 per-puddle event callback，当前 puddle update + immediate spread deposit 后立即触发；
    - callback 返回 touched tile keys，外层会把新创建/替换且未处理的 puddle 追加到同 tick queue；
    - 增加 current-id mismatch 检查，避免旧 queued key 在 tile 被替换后错误更新新 puddle。
  - `server::ServerLauncher::tick_server_puddles(...)`：
    - 临时取出 `runtime.server_puddles`，用 world/content snapshot 提供 passability/build-present；
    - 通过 per-puddle callback 调用 `process_server_puddle_liquid_update(...)`；
    - 移除旧的整轮后 `liquid_update` 批处理，避免双跑；
    - `CellLiquid.update` 的 building 吸收、current-building damage/spread、nearby puddle absorb/replacement 与 `Trigger::NeoplasmReact` 现在按 puddle 顺序 inline 执行。
  - 调整既有 CellLiquid replacement 测试：
    - replacement neoplasm 现在会按 Java `EntityGroup.update()` 动态 append 在同 tick 继续 update；
    - 因此 replacement amount 会有一次同 tick 蒸发，且在有 building water 时 replacement 自身的 `CellLiquid.update` 可再次把 remaining building water 写入 accepting。
- 新增验证：
  - `server_puddle_cell_liquid_update_runs_before_later_puddle_base_update`
    - neoplasm 先创建，water 后创建；
    - water 初始 amount 设为若未被 neoplasm 先吸收就会外溢；
    - tick 后断言 water 没有向 `(3,2)` 产生多余 spread puddle，且 water amount 已低于 overfilled spread 阈值。
- 已跑验证：
  - `cargo test -p mindustry-server server_puddle_cell_liquid_update_runs_before_later_puddle_base_update --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo test -p mindustry-server puddle --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `cargo fmt --check`
  - `git diff --check`
- 仍未完成：
  - `affect_units/create_fire` 已在 12.174 继续搬进 per-puddle callback；`puddle_on_building/particle_effect` 仍需后续确认/接入；
  - callback 当前用 touched tile keys 重新入队，后续如果更多 liquid/update side-effect 会触达非 D4 tile，需要扩展 touched 范围或由具体 consumer 精确返回。

### 12.174 Server puddle effects-only affect_units/create_fire inline ordering

- 2026-05-28：继续收紧 Java `PuddleComp.update()` effects-only 分支顺序，把 server 侧单位液体状态/ripple 和热液体起火从整轮后批处理搬入 per-puddle callback。
- Java 依据：
  - `PuddleComp.update()` 在 `amount >= maxLiquid / 2f && updateTime <= 0f` 时先执行 `Units.nearby(...)`；
  - 随后若 `liquid.temperature > 0.7f && tile.build != null && Mathf.chance(0.5)`，立即 `Fires.create(tile)`；
  - `tile.build.puddleOn(self())` 之后，最后才进入 `liquid.update(self())`；
  - 因此 `affect_units` 与 `create_fire` 至少应早于同 puddle 的 `CellLiquid.update`。
- Rust 新增/变化：
  - `server::ServerLauncher::process_server_puddle_affect_units(...)`：
    - 从旧 batch loop 提取；
    - 继续按 Java 矩形 overlap、grounded/non-hovering、status duration 120 tick、moving ripple 条件执行；
    - ripple 网络 effect 仍先收集，tick 后统一广播，避免在 callback 内嵌套网络发送。
  - `server::ServerLauncher::process_server_puddle_create_fire(...)`：
    - 从旧 batch `create_fire` loop 提取；
    - 在 per-puddle callback 中先于 `process_server_puddle_liquid_update(...)` 调用。
  - `tick_server_puddles(...)` 现在 per-puddle callback 顺序为：
    1. `affect_units`；
    2. `create_fire`；
    3. `CellLiquid.update` / `liquid_update`。
  - 旧的整轮后 `affect_units` 与 `create_fire` batch loops 已移除，避免重复应用。
- 已跑验证：
  - `cargo test -p mindustry-server puddle --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `cargo fmt --check`
  - `git diff --check`
- 仍未完成：
  - `tile.build.puddleOn(self())` 在 Java vanilla base 中是 no-op 且未发现 core override，但 Rust event consumer 仍未显式建模；
  - server `headless=true` 下 `particle_effect` 不会产生事件；客户端/非 headless puddle particle dispatch 后续仍待接入 renderer/effect runtime；
  - ripple effect 仍在 tick 后统一广播，尚未完全保持 Java effect packet 的逐 puddle 发送时机。

### 12.175 Puddle building hook no-op consumer and particle effect payload

- 2026-05-28：继续收紧 Java `PuddleComp.update()` effects-only 分支剩余事件，先闭合 vanilla `tile.build.puddleOn(self())` 的显式消费边界，并把非 headless `particleEffect.at(...)` 事件从裸 effect name 扩展成可供后续客户端/renderer 消费的 payload。
- Java 依据：
  - `PuddleComp.update()` 中 `tile.build.puddleOn(self())` 位于 `Fires.create(tile)` 之后、`liquid.update(self())` 之前；
  - `BuildingComp.puddleOn(Puddle)` 在 v158.1 vanilla core 中是空实现，当前参考仓库未发现 override；
  - `!headless && liquid.particleEffect != Fx.none` 时，每隔 `liquid.particleSpacing` 执行 `liquid.particleEffect.at(x + Mathf.range(size), y + Mathf.range(size))`，其中 `size = Mathf.clamp(amount / (maxLiquid / 1.5f)) * 4f`。
- Rust 新增/变化：
  - `core::entities::puddles::PuddleParticleEffectEvent`：
    - 记录 `effect`、puddle center `x/y`、以及 Java `Mathf.range(size)` 所需的 symmetric `range`；
    - `PuddleUpdateEvent::particle_effect` 从 `Option<String>` 升级为 `Option<PuddleParticleEffectEvent>`，避免后续 renderer/client 再反推 Java 的 size 公式。
  - `server::ServerLauncher::process_server_puddle_on_building(...)`：
    - 在 per-puddle callback 中显式消费 `puddle_on_building`；
    - 当前保持 vanilla no-op，不增加 updates、不改变 world/fire/unit 状态；
    - 预留 mod/future block-specific hook 接入点，且不改变 `PuddleComp.update()` 顺序。
  - `tick_server_puddles(...)` callback 顺序现在为：
    1. `affect_units`；
    2. `create_fire`；
    3. `puddle_on_building` vanilla no-op；
    4. `CellLiquid.update` / `liquid_update`。
- 新增验证：
  - `update_all_report_carries_particle_effect_spawn_range_for_non_headless_clients`
    - 非 headless 下验证 particle payload 的 effect name、center、Java range 公式；
    - headless 下验证不会产生 particle payload。
  - `server_puddle_on_building_vanilla_hook_is_consumed_as_noop`
    - 有 building 的水 puddle 触发 `puddle_on_building` 事件；
    - server 消费后只保留 puddle update 事件计数，不创建 fire、不移除 puddle，符合 vanilla 空 hook。
- 已跑验证：
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo test -p mindustry-server puddle --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `cargo fmt --check`
  - `git diff --check`
- 仍未完成：
  - `particle_effect` 现在已有 renderer/client 所需 payload，但 desktop/client 非 headless effect runtime 尚未把它真正绘制/播放；
  - ripple effect 仍在 tick 后统一广播，尚未完全保持 Java effect packet 的逐 puddle 发送时机。

### 12.176 Client local dispatch for puddle particle payloads

- 2026-05-28：继续推进 12.175 的 `PuddleParticleEffectEvent`，把非 headless puddle particle payload 接入 `GameRuntime` 的客户端本地 effect 队列，避免事件只停留在 `PuddleUpdateReport` 中。
- Java 依据：
  - `PuddleComp.update()` 在非 headless 下直接调用 `liquid.particleEffect.at(...)`；
  - 这类本地视觉 effect 不需要 server/headless 参与，应进入客户端本地 effect dispatch。
- Rust 新增/变化：
  - `core::core::GameRuntime::queue_client_puddle_particle_effects(...)`：
    - 接收 `PuddleUpdateEvent` 列表；
    - 过滤无 particle payload 或当前标准 effect id 表尚不可解析的 effect；
    - 使用调用方提供的随机 offset，并按 `PuddleParticleEffectEvent::range` clamp 到 Java `Mathf.range(size)` 范围；
    - 将结果写入既有 `client_local_effect_events: Vec<EffectCallPacket2>`，`rotation=0`、`color=-1`、`data=Null` 对齐 Java `Effect.at(x, y)` 的默认调用形态。
- 新增验证：
  - `game_runtime_queues_puddle_particle_payloads_into_client_local_effects`
    - 构造非 headless puddle particle update；
    - 通过 `queue_client_puddle_particle_effects(...)` 写入本地 effect 队列；
    - 验证 effect id、clamped x/y、rotation/color/data。
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core game_runtime_queues_puddle_particle_payloads_into_client_local_effects --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 仍未完成：
  - `standard_effect_id(...)` 当前只覆盖已迁移的少量内置 Fx；完整 vanilla/mod effect registry 仍需后续迁移，否则部分 particle effect payload 会被安全跳过；
  - desktop 侧仍缺真正绘制/消费 `client_local_effect_events` 的 renderer pass；
  - 仍需把未来的非 headless client puddle tick 主循环接到 `queue_client_puddle_particle_effects(...)`。

### 12.177 Desktop local effect render drain seam

- 2026-05-28：为 12.176 接入的 `client_local_effect_events` 增加 desktop 侧显式 drain seam，使本地 effect 队列具备被后续 renderer pass 消费的稳定入口。
- Java 依据：
  - Java `Effect.at(...)` 最终进入客户端本地 effect 显示链，而不是长期滞留在网络/事件队列；
  - 当前 Rust desktop 尚无真实 renderer，因此先提供可测试的 drain 边界，避免本地 effect 队列只能积压。
- Rust 新增/变化：
  - `desktop::DesktopLauncher::drain_local_effect_events_for_render(...)`
    - 使用 `std::mem::take` 取出 `runtime.client_local_effect_events`；
    - 返回 `Vec<EffectCallPacket2>` 给未来 renderer pass；
    - 不在 `update()` 中自动 drain，避免破坏现有网络同步/本地 effect 测试对队列可观察性的假设。
- 新增验证：
  - `desktop_launcher_drains_local_effect_events_for_render`
    - 手动塞入本地 effect；
    - drain 后返回该 effect，并确认 runtime 队列清空；
    - 第二次 drain 返回空。
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-desktop desktop_launcher_drains_local_effect_events_for_render --lib`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - desktop 仍缺真正图形 renderer；当前 drain seam 只保证 effect packet 能从 runtime 队列进入渲染边界；
  - `EffectRegistry` / 完整 Fx id 映射与 `EffectStateComp::draw_with(...)` 的真实绘制链仍待迁移。

### 12.178 Client puddle snapshot particle tick

- 2026-05-28：把 12.176/12.177 的 particle dispatch 继续接入客户端快照 puddle 主循环，让 desktop `update()` 能从已同步的 client puddle snapshot 自动产生本地 particle effect。
- Java 依据：
  - Java 客户端同样会更新 `Groups.puddle`，非 headless 时按 `effectTime += Time.delta` 与 `liquid.particleSpacing` 触发 puddle particle effect；
  - particle effect 依赖 liquid 的 `particleEffect/particleSpacing`，因此客户端快照侧不能只保存 `PuddleComp` 的数值字段，还需要保留 liquid metadata。
- Rust 新增/变化：
  - `GameRuntime` 新增 `client_puddle_snapshot_liquids: BTreeMap<i32, PuddleLiquidInfo>`：
    - `apply_client_puddle_sync_wire(...)` 成功解析 liquid id 时同步写入；
    - hidden snapshot 移除 puddle 时同步删除；
    - runtime clear 时同步清空。
  - 新增 `GameRuntime::tick_client_puddle_snapshot_particle_effects(...)`：
    - 遍历 `client_puddle_snapshot_entities`；
    - 使用 sidecar liquid metadata 判断 particle effect 与 spacing；
    - 更新 `PuddleComp.effect_time`；
    - 到达 spacing 后按 Java size 公式生成 `PuddleParticleEffectEvent` 并写入 `client_local_effect_events`。
  - `desktop::DesktopLauncher::update()` 现在在 move effect ability tick 后调用 `tick_client_puddle_snapshot_particle_effects(1.0, |_| (0.0, 0.0))`：
    - 暂用中心点 offset，后续 renderer/RNG 接入后再替换为 Java `Mathf.range(size)` 随机 offset。
- 新增验证：
  - `game_runtime_ticks_client_puddle_snapshot_particle_effects`
  - `game_runtime_applies_client_puddle_entity_snapshot_to_typed_runtime` 增加 liquid sidecar/hidden cleanup 断言
  - `desktop_launcher_ticks_puddle_particle_snapshots_to_local_effect_queue`
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core game_runtime_ticks_client_puddle_snapshot_particle_effects --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_puddle_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_ticks_puddle_particle_snapshots_to_local_effect_queue --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - desktop puddle particle offset 暂为 `(0,0)`，仍需接入 Java 等价 RNG/range；
  - `standard_effect_id(...)` 仍缺完整 Fx registry；
  - 真正 renderer 仍未消费 drain 出来的 effect packet。

### 12.179 Desktop puddle particle range RNG

- 2026-05-28：把 12.178 中 desktop puddle particle 的临时 `(0,0)` offset 替换为可复现的 range 随机偏移，使非 headless puddle particle 更接近 Java `Mathf.range(size)`。
- Java 依据：
  - `PuddleComp.update()` 调用 `liquid.particleEffect.at(x + Mathf.range(size), y + Mathf.range(size))`；
  - `Mathf.range(size)` 语义为在 `[-size, size]` 范围内取随机偏移，X/Y 各取一次。
- Rust 新增/变化：
  - `desktop::DesktopLauncher` 新增 `puddle_particle_rand_state`；
  - 新增 helper：
    - `mix_puddle_particle_seed(seed0, seed1)`：从 network world 的 `rand_seed0/rand_seed1` 混合出 desktop puddle particle seed；
    - `next_puddle_particle_unit(...)` / `next_puddle_particle_range(...)`：生成 `[0,1)` 与 `[-range, range]` 偏移；
  - `sync_runtime_state_from_world_data(...)` 现在会按 world rand seeds 重置 puddle particle RNG；
  - `DesktopLauncher::update()` 调用 `tick_client_puddle_snapshot_particle_effects(...)` 时为每个 particle 分别生成 X/Y range offset，不再固定在 puddle center。
- 更新验证：
  - `desktop_launcher_ticks_puddle_particle_snapshots_to_local_effect_queue`
    - 现在断言 effect 坐标位于 Java range 范围内；
    - 并断言不会退化为 puddle center。
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-desktop desktop_launcher_ticks_puddle_particle_snapshots_to_local_effect_queue --lib`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - 当前 RNG 是 Rust 侧可复现 LCG，并非 Arc `Rand` 位级同构；
  - `standard_effect_id(...)` 仍缺完整 Fx registry；
  - 真正 renderer 仍未消费 drain 出来的 effect packet。

### 12.180 Standard Fx id mapping expansion

- 2026-05-28：扩展 `standard_effect_id(...)`，补齐当前已迁移内容/运行时直接引用的高频 Fx 名称，减少 Java effect packet 和本地 effect 队列因为无法解析 effect name 而被跳过的情况。
- Java 依据：
  - `core/src/mindustry/content/Fx.java`
    - `none`：line 30，继续保持 Rust `standard_effect_id("none") == None`，避免无效 effect 入队；
    - `smoke`：line 317；
    - `hitLiquid`：line 964；
    - `fire`：line 1414；
    - `fireSmoke`：line 1436；
    - `steam`：line 1453；
    - `vapor`：line 1508；
    - `fireballsmoke`：line 1526；
    - `smokeCloud`：line 2549。
- Rust 新增/变化：
  - `core::entities::effect` 新增常量：
    - `FX_SMOKE_ID = 28`
    - `FX_HIT_LIQUID_ID = 85`
    - `FX_FIRE_ID = 119`
    - `FX_FIRE_SMOKE_ID = 121`
    - `FX_STEAM_ID = 123`
    - `FX_VAPOR_ID = 128`
    - `FX_FIREBALL_SMOKE_ID = 130`
    - `FX_SMOKE_CLOUD_ID = 222`
  - `standard_effect_id(...)` 新增上述名称映射；
  - 保持既有 `unitAssemble/missileTrail/missileTrailShort/neoplasmHeal/ripple` 行为不变。
- 更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 增加高频 Fx 断言。
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include_puddle_ripple_dependencies --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 仍未完成：
  - 这仍不是完整 Fx registry，只是补齐当前迁移链路会直接用到的高频内置 Fx；
  - `Fx.ripple` 继续沿用既有常量 `243`，完整 id 审计后续应统一校验全部 Fx 顺序；
  - 真正 renderer 仍未消费 drain 出来的 effect packet。

### 12.181 Local effect packet materialization into EffectStateComp

- 2026-05-28：将客户端本地 effect 从 `EffectCallPacket2` 队列推进到可 tick/draw 的 `EffectStateComp` 状态层，贴近 Java `Effect.add(...) -> EffectState.create()` 生命周期。
- Java 依据：
  - `core/src/mindustry/entities/Effect.java`
    - `create(...)` / `add(...)` 会创建 `EffectState`，写入 `effect/rotation/data/lifetime/x/y/color/parent`；
    - `render(...)` 返回新的 lifetime。
  - `core/src/mindustry/entities/comp/EffectStateComp.java`
    - `draw()` 调用 `effect.render(id, color, time, lifetime, rotation, x, y, data)`；
    - `clipSize()` 返回 `effect.clip`。
- Rust 新增/变化：
  - `GameRuntime` 新增：
    - `client_local_effect_entities: BTreeMap<i32, EffectStateComp>`；
    - `next_client_local_effect_id: i32`，本地 effect 使用负数 id，避免和 server snapshot entity id 冲突。
  - 新增 `GameRuntime::drain_client_local_effect_events_to_states(...)`：
    - `std::mem::take` 取出 `client_local_effect_events`；
    - 为每个 `EffectCallPacket2` 分配本地负 id；
    - 用 `EffectStateComp::apply_sync_wire(...)` 写入 position/rotation/color/data/effect_id/lifetime/clip；
    - 调用方可传入 effect lookup，缺失时使用默认 `Effect::with_lifetime(id, DEFAULT_EFFECT_LIFETIME, DEFAULT_EFFECT_CLIP)` 作为过渡。
  - runtime clear 会同步清空 `client_local_effect_entities` 并重置负 id 分配器。
- 新增验证：
  - `game_runtime_materializes_local_effect_events_into_effect_states`
    - 验证 packet queue 被清空；
    - 验证本地 `EffectStateComp` 使用负 id；
    - 验证 x/y/rotation/lifetime/clip/data/effect_id 写入正确。
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core game_runtime_materializes_local_effect_events_into_effect_states --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 仍未完成：
  - 本地 effect state 还没有统一 tick/cull/draw pass；
  - 完整 effect lookup/registry 仍未迁移，缺失时仍使用默认 lifetime/clip 过渡；
  - `client_effect_snapshot_entities` 的 server snapshot effect 仍未接真实 effect lookup。

### 12.182 Local EffectState tick/cull/draw pass and desktop render ingress

- 2026-05-28：继续推进 Java `EffectStateComp` 生命周期，把本地 effect 从“packet 队列已物化为 state”推进到可被客户端渲染循环消费的 state pass。
- Java 依据：
  - `core/src/mindustry/entities/Effect.java`
    - `Effect.add(...)` 创建短命 `EffectState`；
    - `Effect.render(...)` 返回新的 lifetime。
  - `core/src/mindustry/entities/comp/EffectStateComp.java`
    - `draw()` 每帧调用 `effect.render(id, color, time, lifetime, rotation, x, y, data)`；
    - `clipSize()` 返回 `effect.clip`。
- Rust 新增/变化：
  - `EffectRenderInput` 增加 `effect_id` 与 `clip`，让 renderer callback 能知道应绘制哪个 Fx 以及裁剪半径；
  - `EffectStateComp` 新增：
    - `tick(delta)`：推进本地 effect time；
    - `is_expired()`：按 `time >= lifetime` 判定回收；
  - `GameRuntime` 新增：
    - `tick_client_local_effect_entities(delta)`：统一 tick 并剔除过期本地 effect；
    - `draw_client_local_effect_entities(render)`：对存活本地 effect 调 `EffectStateComp::draw_with(...)`，并允许 renderer 回写 lifetime；
  - `DesktopLauncher::update()` 在 move-effect 与 puddle particle effect 入队后，立即执行：
    - `materialize_local_effect_events_for_render()`；
    - `tick_local_effect_states_for_render(1.0)`；
  - `DesktopLauncher` 新增 `draw_local_effect_states_for_render(...)`，作为后续真实 renderer 接入点。
- 新增/更新验证：
  - `effect_state_ticks_and_reports_expiry_like_lifetime_entity`
  - `game_runtime_ticks_culls_and_draws_client_local_effect_states`
  - desktop 侧 effect/assembler/move-effect/puddle tests 改为断言 `client_local_effect_events` 被物化清空，`client_local_effect_entities` 持有可渲染状态。
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core effect_state --lib`
  - `cargo test -p mindustry-desktop local_effect --lib`
  - `cargo test -p mindustry-desktop assembler --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - 完整 `EffectRegistry` / `Fx` id→lifetime/clip/renderer 映射仍未迁移，当前 desktop materialize 缺失 lookup 时继续使用默认 lifetime/clip；
  - `client_effect_snapshot_entities` 的服务端 snapshot effect 仍未统一接真实 effect lookup；
  - 真正图形后端还未把 `draw_local_effect_states_for_render(...)` 转成 GPU draw call。

### 12.183 Standard Fx lifetime/clip lookup for local and snapshot EffectState

- 2026-05-28：在上一节本地 `EffectStateComp` 生命周期接线基础上，补入当前已迁移/运行时会直接引用的标准 `Fx` 元数据 lookup，让本地 effect materialize 与服务端 snapshot effect apply 不再全部退回默认 lifetime/clip。
- Java 依据：
  - `core/src/mindustry/entities/Effect.java`
    - `Effect(float life, Cons<EffectContainer>)` 默认 `clip = 50f`；
    - 默认 `followParent = true`，`rotWithParent = false`。
  - `core/src/mindustry/content/Fx.java`
    - `smoke = new Effect(100, ...)`；
    - `hitLiquid = new Effect(16, ...)`；
    - `unitAssemble = new Effect(70, ...).layer(Layer.flyingUnit + 5f)`；
    - `missileTrail = new Effect(50, ...).layer(Layer.bullet - 0.001f)`；
    - `missileTrailShort = new Effect(22, ...).layer(Layer.bullet - 0.001f)`；
    - `fire = new Effect(50f, ...)`；
    - `fireSmoke = new Effect(35f, ...)`；
    - `neoplasmHeal = new Effect(120f, ...).followParent(true).rotWithParent(true).layer(Layer.bullet - 2)`；
    - `steam = new Effect(35f, ...)`；
    - `vapor = new Effect(110f, ...)`；
    - `fireballsmoke = new Effect(25f, ...)`；
    - `smokeCloud = new Effect(70, ...)`；
    - `ripple = new Effect(30, ...).layer(Layer.debris)`，但 Java renderer 内还会 `e.lifetime = 30f * e.rotation`，Rust 目前只保留静态构造 lifetime。
- Rust 新增/变化：
  - `entities::effect` 新增：
    - `standard_effect(effect_id)`；
    - `standard_effect_by_name(name)`；
  - 上述 lookup 返回 `Effect { lifetime, clip, layer, follow_parent, rot_with_parent }`，覆盖当前 `standard_effect_id(...)` 已知的高频 Fx；
  - `GameRuntime::apply_client_effect_state_sync_wire(...)` 使用 `standard_effect(sync.effect_id)` 填充 snapshot effect 的 `effect_clip`；
  - `GameRuntime::drain_client_local_effect_events_to_states(...)` 会把 lookup 到的 `rot_with_parent` 写入本地 `EffectStateComp`；
  - `DesktopLauncher::materialize_local_effect_events_for_render()` 使用同一 lookup，将本地 packet 转成带 Java lifetime/clip/rot metadata 的 `EffectStateComp`。
- 新增/更新验证：
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers`
  - `game_runtime_applies_client_effect_entity_snapshot_to_typed_runtime` 增加标准 Fx clip 断言；
  - desktop local effect tests 增加 `neoplasmHeal` / `missileTrailShort` / `ripple` 的 lifetime/clip 断言。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_effect_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-desktop local_effect --lib`
- 仍未完成：
  - `ripple` 的动态 lifetime `30f * rotation` 还未挂到真实 renderer callback；
  - 这仍是覆盖当前高频 Fx 的过渡表，不是完整 `Fx.java` registry；
  - 真实 GPU renderer 仍未消费 `draw_local_effect_states_for_render(...)`。

### 12.184 EffectState parent-follow transform pass

- 2026-05-28：补齐 `EffectStateComp` 的父实体跟随更新，使服务端 snapshot 下发的 `parent_id/offset/rot_with_parent` 不再只是保存在 state 中，而是能按 Java `ChildComp` 语义更新 effect 坐标与旋转。
- Java 依据：
  - `core/src/mindustry/entities/comp/EffectStateComp.java` 继承/组合 generated entity 中的 child/lifetime 行为；
  - `core/src/mindustry/entities/Effect.java` 创建 effect 时会写入 `parent` 与 `rotWithParent`。
- Rust 新增/变化：
  - `EffectStateComp::update_parent_transform(parent)` 复用 Rust `ChildComp` 的 offset/rotation 公式；
  - `GameRuntime::update_client_effect_snapshot_parent_transforms()` 从 `client_unit_snapshot_entities` 与 `client_bullet_snapshot_entities` 收集父实体 transform，并更新 `client_effect_snapshot_entities`；
  - `DesktopLauncher::update()` 在 snapshot mirror 同步后调用该 pass，让客户端 effect snapshot 在渲染前跟随父实体。
- 新增验证：
  - `effect_state_updates_parent_transform_like_child_component`
  - `game_runtime_updates_effect_snapshot_parent_transform_from_unit_parent`
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core effect_state --lib`
  - `cargo test -p mindustry-core game_runtime_updates_effect_snapshot_parent_transform_from_unit_parent --lib`
  - `cargo test -p mindustry-desktop --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - 本地 `EffectCallPacket2` 当前没有 parent id 字段，仍只能对 snapshot effect 做 parent-follow；
  - 还未覆盖 player/building 作为 parent 的通用 resolver；
  - 真正 renderer 仍未消费更新后的 effect state。

### 12.185 Ripple render-time lifetime rule

- 2026-05-28：补齐当前标准 Fx 表中最明显的 render-time lifetime 差异：Java `Fx.ripple` 在 renderer 内执行 `e.lifetime = 30f * e.rotation`。
- Rust 新增/变化：
  - `standard_effect_render_lifetime(effect_id, rotation, current)`：
    - `Fx.ripple` 返回 `30.0 * rotation`；
    - 其他效果返回当前 lifetime；
  - `DesktopLauncher::draw_standard_local_effect_states_for_render()` 使用上述 helper 作为标准本地 effect draw callback，继续复用 `EffectStateComp::draw_with(...)` 的 lifetime 回写路径。
- 新增验证：
  - `standard_effect_render_lifetime_applies_ripple_dynamic_rotation_rule`
  - `desktop_launcher_standard_effect_draw_updates_ripple_lifetime`
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_render_lifetime --lib`
  - `cargo test -p mindustry-desktop standard_effect_draw --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `cargo test -p mindustry-core --lib`
  - `cargo test -p mindustry-desktop --lib`
  - `git diff --check`
- 仍未完成：
  - 这仍只更新 lifetime，没有产生真实 GPU draw command；
  - 其他 `Fx.java` renderer 内的视觉几何/颜色/随机粒子仍未完整迁移。

### 12.186 Standard local effect draw plans

- 2026-05-28：在 `EffectStateComp::draw_with(...)` 与 `DesktopLauncher::draw_standard_local_effect_states_for_render()` 基础上，开始把高频标准 Fx 的 renderer 公式转成可供桌面渲染层消费的 draw plan，而不是只执行 lifetime 回写。
- Java 依据：
  - `Fx.smoke`：
    - `color(Color.gray, Pal.darkishGray, e.fin())`；
    - `Fill.circle(e.x, e.y, (7f - e.fin() * 7f)/2f)`。
  - `Fx.missileTrail` / `Fx.missileTrailShort`：
    - `color(e.color)`；
    - `Fill.circle(e.x, e.y, e.rotation * e.fout())`。
  - `Fx.ripple`：
    - `e.lifetime = 30f * e.rotation`；
    - `color(e.color * 1.5f)`；
    - `stroke(e.fout() * 1.4f)`；
    - `Lines.circle(e.x, e.y, (2f + e.fin() * 4f) * e.rotation)`。
- Rust 新增/变化：
  - `StandardEffectDrawKind::{FilledCircle, StrokedCircle}`；
  - `StandardEffectDrawPlan`，携带 `effect_id/layer/kind/center/color_from/color_to/color_mix/input_color/color_mul/radius/stroke`；
  - `standard_effect_draw_plan(...)` 覆盖 `smoke`、`missileTrail`、`missileTrailShort`、`ripple`；
  - `DesktopLauncher::collect_standard_local_effect_draw_plans_for_render()` 在 draw callback 中收集 draw plan，同时继续执行 Java 风格 lifetime 回写。
- 新增验证：
  - `standard_effect_draw_plan_covers_smoke_trails_and_ripple`
  - `desktop_launcher_standard_effect_draw_updates_ripple_lifetime` 增加 draw plan 断言。
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo test -p mindustry-desktop standard_effect_draw --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - draw plan 仍需接到真实 GPU/2D backend；
  - 当前只覆盖 4 个高频 Fx，完整 `Fx.java` renderer 仍待逐项迁移；
  - 颜色名如 `Pal.darkishGray` 仍是符号计划，后续 renderer/asset layer 需要解析成实际颜色。

### 12.187 Desktop frame cache for standard local effect draw plans

- 2026-05-28：将上一节手动收集的标准本地 effect draw plan 接入 `DesktopLauncher::update()`，让桌面客户端每帧自动生成可供后续 2D/GPU backend 消费的 effect render frame 数据。
- Rust 新增/变化：
  - `DesktopLauncher` 新增公开帧缓存：
    - `standard_local_effect_draw_plans: Vec<StandardEffectDrawPlan>`；
  - `update()` 在 local effect materialize + tick 后调用 `collect_standard_local_effect_draw_plans_for_render()` 并写入该缓存；
  - 世界卸载 / snapshot cursor 清理时同步清空该缓存；
  - move-effect 测试现在断言 `missileTrailShort` 会在 update 后生成 `FilledCircle` draw plan。
- 新增/更新验证：
  - `desktop_launcher_ticks_elude_move_effect_to_local_effect_queue` 增加帧级 draw plan 断言。
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-desktop standard_effect_draw --lib`
  - `cargo test -p mindustry-desktop ticks_elude --lib`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - 该缓存还没有交给真实窗口/图形 backend 绘制；
  - server snapshot effect 与更多 Fx 的 draw plan 仍需扩展。

### 12.188 Standard Fx seeded particle/cloud draw plans

- 2026-05-28：继续对照 `core/src/mindustry/content/Fx.java`，把下一批已经进入标准 Fx lookup 的火焰/烟雾/蒸汽类 renderer 从“无 draw plan”推进到可由桌面帧缓存携带的 seeded particle/cloud plan。
- Java 依据：
  - `Fx.fire`：`color(Pal.lightFlame, Pal.darkFlame, e.fin())`；`randLenVectors(e.id, 2, 2f + e.fin() * 9f, ...)`；圆半径 `0.2f + e.fslope() * 1.5f`；并调用 `Drawf.light(..., 20f * e.fslope(), Pal.lightFlame, 0.5f)`；
  - `Fx.fireSmoke`：`Color.gray`，`randLenVectors(e.id, 1, 2f + e.fin() * 7f, ...)`，圆半径 `0.2f + e.fslope() * 1.5f`；
  - `Fx.steam`：`Color.lightGray`，`randLenVectors(e.id, 2, 2f + e.fin() * 7f, ...)`，圆半径 `0.2f + e.fslope() * 1.5f`；
  - `Fx.vapor`：`color(e.color)`，`alpha(e.fout())`，`randLenVectors(e.id, 3, 2f + e.finpow() * 11f, ...)`，圆半径 `0.6f + e.fin() * 5f`；
  - `Fx.fireballsmoke`：`Color.gray`，`randLenVectors(e.id, 1, 2f + e.fin() * 7f, ...)`，圆半径 `0.2f + e.fout() * 1.5f`；
  - `Fx.smokeCloud`：`randLenVectors(e.id, e.fin(), 30, 30f, ...)`，局部 alpha `(0.5f - abs(fin - 0.5f)) * 2f`，圆半径 `0.5f + fout * 4f`。
- Rust 新增/变化：
  - `StandardEffectDrawKind` 新增 `SeededCircleParticles`；
  - 新增 `StandardEffectParticleSpec`，记录 Java `randLenVectors(...)` 所需的 deterministic seed、粒子数量、进度、扩散长度、半径曲线和局部 alpha 语义；
  - `StandardEffectDrawPlan` 增加 `alpha`、`particles`、`light_color/light_radius/light_opacity` 字段，用于表达 `vapor` 的全局透明度和 `fire` 的光照；
  - `standard_effect_draw_plan(...)` 现在接收 `EffectStateComp.id` 作为 Java `e.id` 等价 seed，并覆盖 `fire/fireSmoke/steam/vapor/fireballsmoke/smokeCloud`；
  - `DesktopLauncher::collect_standard_local_effect_draw_plans_for_render()` 把 `EffectRenderInput.id` 传入标准 draw plan，使桌面帧缓存保留 seeded particle 所需的运行时 id。
- 新增验证：
  - `standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles`
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo test -p mindustry-desktop standard_effect_draw --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - `StandardEffectParticleSpec` 仍是语义级 seeded particle/cloud plan，还没有在真实 2D/GPU backend 中展开为具体 circle primitives；
  - Java `Angles.randLenVectors` 的完全一致 RNG/角度分布还需要在渲染 backend 或共享 primitive 展开层中复刻；
  - 完整 `Fx.java` renderer 仍待继续逐项迁移。

### 12.189 Standard Fx particle plan circle primitive expansion

- 2026-05-28：在上一节 seeded particle/cloud plan 语义表达基础上，增加一层不依赖窗口库/GPU backend 的 circle primitive 展开接口，使后续桌面 2D backend 能直接消费已计算好的粒子圆心、半径与 alpha，而不是只能理解高层 `randLenVectors(...)` 语义。
- Rust 新增/变化：
  - `StandardEffectParticleSpec` 记录当前 effect 的 `fin/fout/fslope`，用于固定 `randLenVectors(e.id, count, length, ...)` 公式；
  - 新增 `StandardEffectParticleVector { x, y, fin, fout }`，用于由后续 deterministic RNG/Angles 展开层传入 Java 等价随机向量；
  - 新增 `StandardEffectCirclePrimitive { center, radius, alpha }`；
  - `StandardEffectDrawPlan::expand_seeded_particle_circles(...)` 将粒子 plan + 向量输入展开为实际圆 primitive：
    - 普通 `fire/fireSmoke/steam/vapor/fireballsmoke` 使用 effect 全局 `fin/fout/fslope` 计算半径；
    - `smokeCloud` 使用每个向量携带的局部 `fin/fout` 计算 `0.5f + fout * 4f` 和 midpoint alpha。
- 新增验证：
  - `standard_effect_particle_plan_expands_to_circle_primitives`
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_particle --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 仍未完成：
  - 还缺 Java `Angles.randLenVectors` 的 deterministic vector 生成器，因此当前展开层需要调用方传入向量；
  - 桌面端仍未建立真实窗口/2D backend 来消费这些 circle primitives；
  - 光照 primitive 仍保留在 `StandardEffectDrawPlan` 上，尚未统一拆成可排序的 backend draw command。

### 12.190 Arc Rand / Angles.randLenVectors compatible particle vectors

- 2026-05-28：补齐上一节留下的最大缺口：在 Rust effect 层实现与 Arc `Rand` / `Angles.randLenVectors(...)` 对齐的 deterministic seeded particle vector 生成器，使标准 Fx 粒子 plan 能自己生成 Java 等价随机向量并展开为 circle primitives。
- Java/Arc 依据：
  - Mindustry v158.1 使用 Arc artifact：`com.github.Anuken.Arc:arc-core:4d9760e264`；
  - 通过本地 Gradle cache 中 `arc-core-4d9760e264.jar` 的 `javap` 反汇编确认：
    - `Angles.randLenVectors(long, int, float, Floatc2)`：`rand.setSeed(seed)` 后每个粒子取 `rand.random(360f)` 和 `rand.random(length)`；
    - `Angles.randLenVectors(long, float fin, int amount, float length, ParticleConsumer)`：每点先取 `rand.nextFloat()`，长度为 `length * local * fin`，回调局部 `fin = fin * local`、`fout = (1f - fin) * local`；
    - `Rand` 使用 murmurHash3 seed 初始化、xorshift128+ 风格 `nextLong()`，`nextFloat()` 取 `nextLong() >>> 40`；
    - `Vec2.trns` 走 `Mathf.sin/cos` 查表而不是标准库精确三角函数。
- Rust 新增/变化：
  - `StandardEffectParticleSpec::seeded_vectors()`：根据 `seed/count/progress/length` 生成 Java 等价 `StandardEffectParticleVector`；
  - `StandardEffectDrawPlan::seeded_particle_vectors()` 与 `expand_seeded_particle_circles_from_seed()`：让 plan 可从自身 seed 直接生成并展开 circle primitives；
  - 私有 `ArcRand` 复刻 Arc `Rand` 的 murmurHash3 + `nextLong/nextFloat/random(float)`；
  - 私有 `mathf_sin/cos` 复刻 Arc `Mathf` 16384 长度 sin table 采样，用于匹配 `Vec2.trns`；
  - 测试使用本地 Arc jar probe 得到的 Java 输出作为固定期望值，覆盖 seed `42` 的固定 `randLenVectors` 与 seed `47` 的 progressive `smokeCloud` overload。
- 新增/更新验证：
  - `standard_effect_particle_plan_expands_to_circle_primitives` 增加 Java-derived seeded vector 断言。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_particle --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 仍未完成：
  - 还需要把 circle/light primitives 接入桌面真实 2D backend；
  - 当前只覆盖已迁移的标准 Fx 子集，完整 `Fx.java` renderer 仍需继续逐项迁移；
  - `Angles.randLenVectors` 其他 overload（cone/random offset 等）待后续迁移到更多 Fx 时继续补齐。

### 12.191 Unified circle render primitives for standard Fx plans

- 2026-05-28：在 seeded vector 与 circle expansion 已可用后，补一层统一的后端消费接口，把单圆、描边圆与 seeded particle 圆云都解析为同一种 circle render primitive 列表，减少桌面 2D backend 后续需要理解的高层 `Fx` 分支。
- Rust 新增/变化：
  - 新增 `StandardEffectCircleRenderPrimitive { kind, center, radius, stroke, alpha }`；
  - `StandardEffectDrawPlan::circle_render_primitives_from_seed()`：
    - `FilledCircle` / `StrokedCircle` 直接生成单个 primitive；
    - `SeededCircleParticles` 先按 Arc `Rand` / `Angles.randLenVectors` 生成 seeded vectors，再展开为多个 `FilledCircle` primitive；
  - 该接口不引入窗口库或 GPU 依赖，保持 core 层只输出可排序/可消费的几何 primitive。
- 新增验证：
  - `standard_effect_plan_resolves_circle_render_primitives_from_seed`
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_plan_resolves --lib`
  - `cargo test -p mindustry-core standard_effect_particle --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 仍未完成：
  - 颜色解析仍保留为 `color_from/color_to/input_color/color_mix/color_mul` 元数据，尚未统一压成最终 RGBA；
  - `fire` 的 light 仍是 plan 字段，尚未统一成 light render primitive；
  - desktop 仍需建立真实帧渲染/窗口 backend 来消费这些 primitive。

### 12.192 Desktop frame cache for standard effect circle primitives

- 2026-05-28：将上一节 core 层统一 circle render primitive 接入 `DesktopLauncher` 帧缓存，使桌面端每帧不只持有高层 `StandardEffectDrawPlan`，还持有后续 2D backend 可直接消费的圆形 primitive 列表。
- Rust 新增/变化：
  - `entities::mod` 导出 `StandardEffectCircleRenderPrimitive`；
  - `DesktopLauncher` 新增公开帧缓存：
    - `standard_local_effect_circle_primitives: Vec<StandardEffectCircleRenderPrimitive>`；
  - `update()` 生成 `standard_local_effect_draw_plans` 后，立即调用 `StandardEffectDrawPlan::circle_render_primitives_from_seed()` 展开并缓存 primitive；
  - `collect_standard_local_effect_circle_primitives_for_render()` 可从当前帧 draw plan 重新生成 primitive；
  - 世界卸载 / snapshot cursor 清理时同步清空 primitive 缓存。
- 新增/更新验证：
  - `desktop_launcher_standard_effect_draw_updates_ripple_lifetime` 增加 primitive 缓存清理断言；
  - `desktop_launcher_ticks_elude_move_effect_to_local_effect_queue` 增加 `missileTrailShort` 由 draw plan 到 `FilledCircle` primitive 的帧级断言。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-desktop standard_effect_draw --lib`
  - `cargo test -p mindustry-desktop ticks_elude --lib`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - primitive 仍停在内存帧缓存，还未被真实窗口/2D backend present；
  - circle primitive 仍依赖 plan 的颜色元数据，颜色解析和 light primitive 仍待后续统一。

### 12.193 Standard effect light primitives and desktop cache

- 2026-05-28：补齐 `Fx.fire` renderer 中 `Drawf.light(...)` 的过渡渲染数据，把光照从 `StandardEffectDrawPlan` 字段统一拆成 light render primitive，并接入桌面帧缓存。
- Java 依据：
  - `Fx.fire`：`Drawf.light(e.x, e.y, 20f * e.fslope(), Pal.lightFlame, 0.5f)`。
- Rust 新增/变化：
  - 新增 `StandardEffectLightRenderPrimitive { center, radius, color, opacity }`；
  - `StandardEffectDrawPlan::light_render_primitives()` 将 `light_color/light_radius/light_opacity` 转为 light primitive；
  - `entities::mod` 导出 `StandardEffectLightRenderPrimitive`；
  - `DesktopLauncher` 新增公开帧缓存：
    - `standard_local_effect_light_primitives: Vec<StandardEffectLightRenderPrimitive>`；
  - `update()` 每帧从标准 effect draw plan 同步生成 light primitive；
  - 世界卸载 / snapshot cursor 清理时同步清空 light primitive 缓存。
- 新增/更新验证：
  - `standard_effect_plan_resolves_circle_render_primitives_from_seed` 增加 `fire` light primitive 断言；
  - `desktop_launcher_caches_fire_light_primitives_for_render` 覆盖本地 `Fx.fire` packet → effect state → draw plan → circle primitives → light primitive 帧缓存；
  - `desktop_launcher_standard_effect_draw_updates_ripple_lifetime` 增加 light primitive 清理断言。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_plan_resolves --lib`
  - `cargo test -p mindustry-desktop fire_light --lib`
  - `cargo test -p mindustry-desktop standard_effect_draw --lib`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - light primitive 仍是内存帧数据，还未提交给真实 2D/GPU backend；
  - 颜色仍是符号名 `Pal.lightFlame`，尚未接入实际 RGBA palette 解析。

### 12.194 Standard effect color symbol resolution

- 2026-05-28：补齐当前标准 effect draw/light primitive 的基础颜色解析，把 `Color.*` / `Pal.*` 符号色转成 `DecalColor` RGBA，并把解析结果随 circle/light primitive 一起输出给后续 2D backend。
- Java/Arc 依据：
  - Arc `Color.gray = 0x7f7f7fff`，`Color.lightGray = 0xbfbfbfff`，`Color.darkGray = 0x3f3f3fff`；
  - `Pal.darkishGray = new Color(0.3f, 0.3f, 0.3f, 1f)`；
  - `Pal.lightFlame = Color.valueOf("ffdd55")`；
  - `Pal.darkFlame = Color.valueOf("db401c")`。
- Rust 新增/变化：
  - 新增 `standard_effect_color_symbol(name)`，覆盖当前标准 Fx draw plan 已引用的 `Color.white/gray/lightGray/darkGray` 与 `Pal.darkishGray/lightFlame/darkFlame`；
  - `StandardEffectDrawPlan::resolved_draw_color()` 根据 `input_color`、`color_from/color_to/color_mix`、`color_mul` 与 `alpha` 解析出当前 primitive 颜色；
  - `StandardEffectCircleRenderPrimitive` 新增 `color: Option<DecalColor>`；
  - `StandardEffectLightRenderPrimitive` 新增 `color_rgba: Option<DecalColor>`，保留符号名同时提供已解析 RGBA。
- 新增/更新验证：
  - `standard_effect_plan_resolves_circle_render_primitives_from_seed` 增加：
    - `Fx.smoke` 的 `Color.gray -> Pal.darkishGray` 插值 RGBA；
    - `Fx.fire` 的 `Pal.lightFlame -> Pal.darkFlame` 插值 RGBA；
    - `Fx.fire` light primitive 的 `Pal.lightFlame` RGBA。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_plan_resolves --lib`
  - `cargo test -p mindustry-desktop fire_light --lib`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - 颜色解析表只覆盖当前已迁移标准 Fx 所需符号色，不是完整 `Pal.java`/Arc `Color` registry；
  - renderer 后端还未真正消费这些 RGBA primitive。

### 12.195 Desktop standard effect render frame data boundary

- 2026-05-28：在桌面端已有 draw/circle/light 三组标准 effect 帧缓存后，补一个明确的帧级 render data 边界，方便后续真实 2D/GPU backend 在 `launcher.update()` 之后一次性消费当前帧 effect 渲染数据。
- Rust 新增/变化：
  - `desktop/src/lib.rs` 新增：
    - `DesktopStandardEffectRenderFrame { draw_plans, circle_primitives, light_primitives }`；
    - `DesktopLauncher::standard_effect_render_frame()`，克隆当前帧三组缓存并返回统一 frame data；
  - 暂不引入 `winit/wgpu/pixels/sdl2` 等外部依赖，保持当前闭环为无依赖内存帧数据边界。
- 更新验证：
  - `desktop_launcher_caches_fire_light_primitives_for_render` 增加 `standard_effect_render_frame()` 断言，确认 frame 中 draw/circle/light 与 launcher 当前帧缓存一致。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-desktop fire_light --lib`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - `desktop/src/main.rs` 仍未把 frame data 提交给真实窗口/2D backend；
  - 真实 backend 需要用户确认外部依赖后再接入，或继续先迁移无依赖 software primitive/backend trait。

### 12.196 More simple standard Fx draw plans

- 2026-05-28：继续对照 `Fx.java`，扩展一批不需要新增 draw kind 的简单标准 Fx，使它们直接复用现有 `FilledCircle` / `SeededCircleParticles` / circle primitive 链路。
- Java 依据：
  - `fallSmoke = new Effect(110, ...)`：`color(Color.gray, Color.darkGray, e.rotation)`，`Fill.circle(..., e.fout() * 3.5f)`；
  - `rocketSmoke = new Effect(120, ...)`：`Color.gray`，`alpha(clamp(e.fout()*1.6f - pow3In(e.rotation)*1.2f))`，半径 `(1f + 6f * e.rotation) - e.fin()*2f`；
  - `rocketSmokeLarge = new Effect(220, ...)`：同 `rocketSmoke`，半径 `(1f + 6f * e.rotation * 1.3f) - e.fin()*2f`；
  - `magmasmoke = new Effect(110, ...)`：`Color.gray`，半径 `e.fslope() * 6f`；
  - `burning = new Effect(35f, ...)`：`Pal.lightFlame -> Pal.darkFlame`，`randLenVectors(e.id, 3, 2f + e.fin()*7f, ...)`，半径 `0.1f + e.fout()*1.4f`；
  - `fireHit = new Effect(35f, ...)`：`Pal.lightFlame -> Pal.darkFlame`，`randLenVectors(e.id, 3, 2f + e.fin()*10f, ...)`，半径 `0.2f + e.fout()*1.6f`；
  - `blastsmoke = new Effect(26, ...)`：`Color.lightGray -> Color.darkGray`，`randLenVectors(e.id, 12, 1f + e.fin()*23f, ...)`，半径 `1f + e.fout()*3f`。
- Rust 新增/变化：
  - 新增标准 Fx id 常量并接入 `standard_effect_id(...)`：
    - `fallSmoke=29`
    - `rocketSmoke=31`
    - `rocketSmokeLarge=32`
    - `magmasmoke=33`
    - `burning=117`
    - `fireHit=120`
    - `blastsmoke=226`
  - `standard_effect(...)` 增加上述 lifetime/clip 默认值；
  - `standard_effect_draw_plan(...)` 增加上述 7 个 Fx 的 `FilledCircle` 或 `SeededCircleParticles` plan；
  - `blastsmoke` 现在可通过 seeded vector 展开为 12 个 circle primitives。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies`
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers`
  - `standard_effect_draw_plan_covers_simple_smoke_and_fire_variants`
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 仍未完成：
  - 这仍不是完整 `Fx.java` registry；
  - `rocketSmoke` 的 `Interp.pow3In` 当前按 `rotation.powi(3)` 等价处理，后续如抽象完整 Interp registry 可统一替换；
  - `Fx.ripple` id 仍沿用既有 `243`，完整 Fx id 审计时需要与 content registry 初始化顺序统一确认。

### 12.197 Desktop unit item/payload mirror clear regressions

- 2026-05-28：对照当前 `NetClient` packet sidecar → `DesktopLauncher` sync → `GameRuntime` typed unit runtime 链路，补齐桌面端镜像归零/清空回归断言，确保单位物品/载荷镜像不仅能增加和变更，也能清回空状态。
- Rust 现状确认：
  - `GameRuntime::apply_client_unit_item_mirror(...)` 已在 `item=None` 时清空 `UnitComp.items.stack`；
  - `GameRuntime::apply_client_unit_payload_mirror(...)` 已在 `payload_count=0` 时保留 payload comp 但清空 payload 列表；
  - `DesktopLauncher::sync_unit_item_mirrors_to_runtime()` / `sync_unit_payload_mirrors_to_runtime()` 已每帧从 `NetClientState` mirror 同步到 typed runtime。
- Rust 更新验证：
  - `desktop_launcher_applies_unit_item_mirror_to_runtime_unit_snapshot` 增加：
    - mirror 从 `lead x5` 改为 `item=None, amount=99` 后，typed `UnitComp.items.stack.item=None` 且 `amount=0`；
  - `desktop_launcher_applies_unit_payload_mirror_to_runtime_unit_snapshot` 增加：
    - mirror 从 `payload_count=1` 改为 `payload_count=0` 后，typed `UnitComp.payload.payloads.len()==0`。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-desktop unit_item_mirror --lib`
  - `cargo test -p mindustry-desktop unit_payload_mirror --lib`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - 当前 payload mirror 仍只保留 kind/count 近似，未携带真实 build/unit payload 内容；
  - item/payload mirror 移除 map entry 时是否应同步清 typed runtime 仍需结合 Java 真实 packet 生命周期继续确认。

### 12.198 Fx.smokePuff 双圆粒子绘制计划

- 2026-05-28：继续对照 `Fx.java`，迁移 `smokePuff` 标准特效；这是首个“每个 `randLenVectors` 向量展开为两个圆”的标准 Fx，因此同步扩展了现有 `SeededCircleParticles` primitive 展开能力。
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:1814` 附近：
    - `smokePuff = new Effect(30, e -> { ... })`；
    - `color(e.color)`；
    - `randLenVectors(e.id, 6, 4f + 30f * e.finpow(), ...)`；
    - 每个向量绘制两枚 `Fill.circle`：主圆 `e.x + x, e.y + y, e.fout() * 3f`，副圆 `e.x + x / 2f, e.y + y / 2f, e.fout()`。
  - 本地按 `new Effect` 声明顺序计数，`smokePuff` 为 `id=154`；`Fx.ripple` 的完整 id 审计仍保留既有待办。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SMOKE_PUFF_ID = 154`；
    - `standard_effect_id("smokePuff")`、`standard_effect(FX_SMOKE_PUFF_ID)` 接入，lifetime 为 `30.0`、clip 沿用 `DEFAULT_EFFECT_CLIP`、layer 沿用默认层；
    - `StandardEffectParticleSpec` 新增 secondary circle 参数：
      - `secondary_vector_scale`
      - `secondary_radius_base`
      - `secondary_radius_fin_scale`
      - `secondary_radius_fout_scale`
      - `secondary_radius_fslope_scale`
    - `StandardEffectDrawPlan::expand_seeded_particle_circles(...)` 现在可在每个 seeded vector 后追加副圆 primitive；
    - `standard_effect_draw_plan(...)` 新增 `smokePuff`：
      - `input_color = Some(color)`；
      - `count = 6`；
      - `length = 4.0 + 30.0 * finpow`；
      - 主圆半径 `3.0 * fout`；
      - 副圆位置缩放 `0.5`，半径 `1.0 * fout`。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖 `smokePuff` name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime；
  - `standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles` 覆盖 draw plan 字段；
  - `standard_effect_particle_plan_expands_to_circle_primitives` 覆盖双圆展开与 seeded render primitive 数量。
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo test -p mindustry-core standard_effect_particle --lib`
  - `cargo test -p mindustry-core standard_effect_plan_resolves --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 仍未完成：
  - `Fx.java` 仍未完整迁移；
  - `smokePuff` 已进入 primitive/data 边界，但真实 desktop 2D/GPU 绘制 backend 尚未接入；
  - 下一批适合继续迁移：`shootSmallSmoke`、`smokeAoeCloud`、`missileTrailSmokeSmall`、`missileTrailSmoke`、`neoplasmSplat`。

### 12.199 Fx.shootSmallSmoke 方向扇区粒子绘制计划

- 2026-05-28：继续对照 `Fx.java`，迁移 `shootSmallSmoke`；这条效果补齐了当前标准 effect 粒子链路缺失的 `Angles.randLenVectors(seed, amount, length, angle, range, Floatc2)` 方向扇区重载，并补了 Java probe 数值回归。
- Java/Arc 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:1850` 附近：
    - `shootSmallSmoke = new Effect(20f, e -> { ... })`；
    - `color(Pal.lighterOrange, Color.lightGray, Color.gray, e.fin())`；
    - `randLenVectors(e.id, 5, e.finpow() * 6f, e.rotation, 20f, ...)`；
    - 每个向量绘制一枚 `Fill.circle(e.x + x, e.y + y, e.fout() * 1.5f)`。
  - 本地 `arc-core-4d9760e264.jar` 字节码确认该重载等价于：
    - `angle = baseAngle + rand.range(range)`；
    - `length = rand.random(length)`；
    - `rand.range(x) = nextFloat() * 2x - x`。
  - Java probe 固定输入 `seed=159, count=5, length=1.5, angle=45, range=20` 的前两组输出：
    - `(0.09767128, 0.17498657)`
    - `(0.43052074, 0.30730063)`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SHOOT_SMALL_SMOKE_ID = 159`；
    - `standard_effect_id("shootSmallSmoke")`、`standard_effect(FX_SHOOT_SMALL_SMOKE_ID)` 接入，lifetime 为 `20.0`；
    - `StandardEffectParticleSpec` 新增：
      - `angle: Option<f32>`
      - `angle_range: f32`
    - `ArcRand` 新增 `range(float)` 等价实现；
    - `seeded_vectors()` 在非 progressive 粒子模式下支持 `angle + rand.range(angle_range)`；
    - `StandardEffectDrawPlan` 新增 `color_mid`，`resolved_draw_color()` 支持 Java/Arc 三段颜色插值；
    - `standard_effect_color_symbol(...)` 新增 `Pal.lighterOrange = f6e096`；
    - `standard_effect_draw_plan(...)` 新增 `shootSmallSmoke`：
      - `count=5`
      - `length=finpow * 6.0`
      - `angle=Some(rotation)`
      - `angle_range=20.0`
      - `radius_fout_scale=1.5`
      - 颜色 `Pal.lighterOrange -> Color.lightGray -> Color.gray`。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖 `shootSmallSmoke` name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime；
  - `standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles` 覆盖三段颜色和方向粒子字段；
  - `standard_effect_particle_plan_expands_to_circle_primitives` 覆盖 Java probe 前两组 seeded vector 与圆半径。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo test -p mindustry-core standard_effect_particle --lib`
  - `cargo test -p mindustry-core standard_effect_plan_resolves --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 仍未完成：
  - 带 `ParticleConsumer` 的方向/progress 重载仍未迁移；当前只覆盖 `Floatc2` 方向扇区重载；
  - `shootSmall/shootBig` 等三角形 primitive 类 Fx 仍未接入；
  - 真实 desktop 2D/GPU backend 尚未消费这些 primitive。

### 12.200 Fx.smokeAoeCloud 高数量烟云粒子绘制计划

- 2026-05-28：继续对照 `Fx.java`，迁移 `smokeAoeCloud`；该效果不需要新增 primitive 类型，但覆盖了高 count 圆粒子和非默认 clip。
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:604` 附近：
    - `smokeAoeCloud = new Effect(60f * 3f, 250f, e -> { ... })`；
    - `color(e.color, 0.65f)`；
    - `randLenVectors(e.id, 80, 90f, ...)`；
    - `Fill.circle(..., 6f * Mathf.clamp(e.fin() / 0.1f) * Mathf.clamp(e.fout() / 0.1f))`。
  - 本地按 `new Effect` 声明顺序计数，`smokeAoeCloud` 为 `id=55`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SMOKE_AOE_CLOUD_ID = 55`；
    - `standard_effect_id("smokeAoeCloud")`、`standard_effect(FX_SMOKE_AOE_CLOUD_ID)` 接入；
    - lifetime 为 `180.0`，clip 为 `250.0`；
    - `standard_effect_draw_plan(...)` 新增 `smokeAoeCloud`：
      - `input_color = Some(color)`；
      - `alpha = 0.65`；
      - `count = 80`；
      - `length = 90.0`；
      - 半径在 plan 阶段计算为 `6.0 * clamp(fin / 0.1) * clamp(fout / 0.1)`，作为每个粒子的固定 `radius_base`。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖 name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime/clip；
  - `standard_effect_draw_plan_covers_simple_smoke_and_fire_variants` 覆盖 draw plan、80 个 seeded circle primitives。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 仍未完成：
  - `smokeAoeCloud` 仍只进入标准 effect primitive/frame data 边界；
  - 真实 desktop renderer 未消费；
  - 完整 `Fx.java` registry 仍待继续迁移。

### 12.201 Arc Scaled.finpow 对齐修正

- 2026-05-28：在继续迁移 `steamCoolSmoke` 前复核 Arc `Scaled.finpow()`，发现此前 Rust 侧按 `fin * fin` 近似，和 Java/Arc 实际语义不一致；本轮先纠正该基础公式，避免后续所有依赖 `e.finpow()` 的 Fx 持续偏移。
- Java/Arc 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/entities/Effect.java` 中 `EffectContainer` 实现 `Scaled.fin()`；
  - 本地 `arc-core-4d9760e264.jar` 字节码确认 `arc.math.Scaled.finpow()` 为 `Interp.pow3Out.apply(fin())`；
  - 等价公式为 `1.0 - (1.0 - fin)^3`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `effect_finpow_from_fin(fin)`；
    - `standard_effect_draw_plan(...)` 中的 `finpow` 改为 pow3Out 等价公式；
    - `EffectContainer::finpow()` 改为复用同一 helper；
    - 更新受影响测试期望：
      - `Fx.vapor` 在 `fin=0.5` 时 length 从旧近似 `4.75` 改为 Java 等价 `11.625`；
      - `Fx.smokePuff` 在 `fin=0.5` 时 length 从旧近似 `11.5` 改为 `30.25`；
      - `Fx.shootSmallSmoke` 在 `fin=0.5` 时 length 从旧近似 `1.5` 改为 `5.25`，并更新 Java probe seeded vector 期望。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo test -p mindustry-core standard_effect_particle --lib`
  - `cargo test -p mindustry-core standard_effect_plan_resolves --lib`
  - `cargo test -p mindustry-core effect_container_fin --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 仍未完成：
  - 其它 Arc `Interp` helper 仍需按使用点逐步补齐；
  - 历史已迁移 Fx 中如还有 `finpow` 派生常量，需要以后继续结合 Java probe 抽样回归。

### 12.202 Fx.steamCoolSmoke 方向冷却烟迁移

- 2026-05-28：继续对照 `Fx.java`，迁移 `steamCoolSmoke`；该效果复用方向扇区粒子能力，并补齐当前所需的 `Interp.pow2Out` 与 `fout(Interp.pow3Out)` 公式。
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:1804` 附近：
    - `steamCoolSmoke = new Effect(35f, e -> { ... })`；
    - `color(Pal.water, Color.lightGray, e.fin(Interp.pow2Out))`；
    - `alpha(e.fout(Interp.pow3Out))`；
    - `randLenVectors(e.id, 4, e.finpow() * 7f, e.rotation, 30f, ...)`；
    - 半径 `Math.max(e.fout(), Math.min(1f, e.fin() * 8f)) * 2.8f`。
  - 本地按 `new Effect` 声明顺序计数，`steamCoolSmoke` 为 `id=153`。
  - `Pal.water` 来自 `Pal.java`：`596ab8`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_STEAM_COOL_SMOKE_ID = 153`；
    - `standard_effect_id("steamCoolSmoke")`、`standard_effect(FX_STEAM_COOL_SMOKE_ID)` 接入，lifetime 为 `35.0`；
    - 新增/复用插值 helper：
      - `interp_pow2_out(...)`
      - `interp_pow3_out(...)`
      - `effect_finpow_from_fin(...)` 继续复用 pow3Out；
    - `standard_effect_color_symbol(...)` 新增 `Pal.water = 0x596ab8ff`；
    - `standard_effect_draw_plan(...)` 新增 `steamCoolSmoke`：
      - `color_from = Pal.water`
      - `color_to = Color.lightGray`
      - `color_mix = pow2Out(fin)`
      - `alpha = pow3Out(fout)`
      - `count = 4`
      - `length = finpow * 7.0`
      - `angle = Some(rotation)`
      - `angle_range = 30.0`
      - `radius_base = max(fout, min(1, fin * 8)) * 2.8`
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖 name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime；
  - `standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles` 覆盖颜色插值、alpha、方向粒子字段、半径和 primitive 数量。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo test -p mindustry-core standard_effect_particle --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - 真实 desktop renderer 仍未消费这些 primitive；
  - `missileTrailSmokeSmall/missileTrailSmoke` 的多 pass 烟轨和 per-particle light 仍未迁移。

### 12.203 Fx.shootBigSmoke 系列方向烟雾迁移

- 2026-05-28：核对 `artilleryTrailSmoke` 后确认其需要每粒子独立 lifetime/random alpha/条件跳过，当前标准粒子 spec 暂不能精确表达；本轮改为迁移同一区段内可精确复用方向扇区粒子和三段颜色插值的 `shootBigSmoke`、`shootBigSmoke2`、`shootSmokeDisperse`。
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:1967` 附近：
    - `shootBigSmoke = new Effect(17f, ...)`
    - 颜色 `Pal.lighterOrange -> Color.lightGray -> Color.gray`
    - `randLenVectors(e.id, 8, e.finpow() * 19f, e.rotation, 10f, ...)`
    - 半径 `e.fout() * 2f + 0.2f`
  - `Fx.java:1975` 附近：
    - `shootBigSmoke2 = new Effect(18f, ...)`
    - 颜色 `Pal.lightOrange -> Color.lightGray -> Color.gray`
    - `randLenVectors(e.id, 9, e.finpow() * 23f, e.rotation, 20f, ...)`
    - 半径 `e.fout() * 2.4f + 0.2f`
  - `Fx.java:1983` 附近：
    - `shootSmokeDisperse = new Effect(25f, ...)`
    - 颜色 `Pal.lightOrange -> Color.white -> Color.gray`
    - `randLenVectors(e.id, 9, e.finpow() * 29f, e.rotation, 18f, ...)`
    - 半径 `e.fout() * 2.2f + 0.1f`
  - 本地按 `new Effect` 声明顺序计数：
    - `shootBigSmoke=166`
    - `shootBigSmoke2=167`
    - `shootSmokeDisperse=168`
  - `Pal.lightOrange` 来自 `Pal.java`：`f68021`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增：
      - `FX_SHOOT_BIG_SMOKE_ID = 166`
      - `FX_SHOOT_BIG_SMOKE2_ID = 167`
      - `FX_SHOOT_SMOKE_DISPERSE_ID = 168`
    - 接入 `standard_effect_id(...)`、`standard_effect(...)`；
    - `standard_effect_color_symbol(...)` 新增 `Pal.lightOrange = 0xf68021ff`；
    - `standard_effect_draw_plan(...)` 以一个共享分支迁移三者，分别参数化颜色、count、`finpow` length scale、angle range、radius base/fout scale。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖三个 name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖三者 lifetime；
  - `standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles` 覆盖三者颜色、count、length、angle_range、radius 字段。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - `artilleryTrailSmoke` 仍需新增更细粒度 per-particle lifetime/alpha 表达；
  - `shootSmokeSquare` 等 poly/square 形状仍需新增 polygon primitive；
  - 真实 renderer backend 仍未接入。

### 12.204 Fx.hitLiquid 方向液体命中特效绘制计划

- 2026-05-28：继续对照 `Fx.java`，补齐已有 metadata 但缺 draw plan 的 `hitLiquid`；同时核对 `missileTrailSmoke*`，确认它们需要多 pass / scaled lifetime / per-particle light，暂不做近似硬塞。
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:963` 附近：
    - `hitLiquid = new Effect(16, e -> { ... })`；
    - `color(e.color)`；
    - `randLenVectors(e.id, 5, 1f + e.fin() * 15f, e.rotation, 60f, ...)`；
    - `Fill.circle(e.x + x, e.y + y, e.fout() * 2f)`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - `standard_effect_draw_plan(...)` 新增 `FX_HIT_LIQUID_ID` 分支；
    - 使用 `SeededCircleParticles`：
      - `input_color = Some(color)`
      - `count = 5`
      - `length = 1.0 + fin * 15.0`
      - `angle = Some(rotation)`
      - `angle_range = 60.0`
      - `radius_fout_scale = 2.0`
    - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 明确覆盖 `hitLiquid` lifetime `16.0`。
- 新增/更新验证：
  - `standard_effect_draw_plan_covers_smoke_trails_and_ripple` 覆盖 `hitLiquid` draw plan、方向粒子参数与 5 个 circle primitives。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - `missileTrailSmoke/missileTrailSmokeSmall` 需要多 pass 粒子、`Scaled.scaled(...)` 局部 lifetime、`Interp.pow10Out/pow5Out` 和 per-particle light；
  - 真实 renderer backend 仍未接入。

### 12.205 Fx.corrosionVapor / Fx.vaporSmall 迁移

- 2026-05-28：继续对照 `Fx.java`，迁移与既有 `vapor` 同构的 `corrosionVapor` 和 `vaporSmall`；两者均可复用当前 `SeededCircleParticles`、`finpow` 与 alpha helper，无需新增 primitive 类型。
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:1498` 附近：
    - `corrosionVapor = new Effect(50f, ...)`
    - `color(e.color)`
    - `alpha(Interp.pow2Out.apply(e.fslope()) * 0.5f)`
    - `randLenVectors(e.id, 2, 8f + e.finpow() * 3f, ...)`
    - 半径 `3f`
  - `Fx.java:1516` 附近：
    - `vaporSmall = new Effect(50f, ...)`
    - `color(e.color)`
    - `alpha(e.fout())`
    - `randLenVectors(e.id, 4, 2f + e.finpow() * 5f, ...)`
    - 半径 `1f + e.fin() * 4f`
  - 本地按 `new Effect` 声明顺序计数：
    - `corrosionVapor=127`
    - `vapor=128`
    - `vaporSmall=129`
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_CORROSION_VAPOR_ID = 127`、`FX_VAPOR_SMALL_ID = 129`；
    - 接入 `standard_effect_id(...)` 与 `standard_effect(...)`，lifetime 均为 `50.0`；
    - `standard_effect_draw_plan(...)` 新增：
      - `corrosionVapor`：`alpha=pow2Out(fslope)*0.5`、`count=2`、`length=8+finpow*3`、`radius_base=3`；
      - `vaporSmall`：`alpha=fout`、`count=4`、`length=2+finpow*5`、`radius_base=1`、`radius_fin_scale=4`。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖两个 name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime；
  - `standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles` 覆盖两个 draw plan。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - `corrosionVapor/vaporSmall` 仍只到 primitive data 边界，真实 renderer backend 未接入；
  - 完整 vapor/corrosion 相关 Fx 仍需继续对照 `Fx.java`。

### 12.206 Fx.blockExplosionSmoke 双圆烟迁移

- 2026-05-28：继续对照 `Fx.java`，迁移 `blockExplosionSmoke`；该效果与已迁移的 `smokePuff` 结构相同，区别是固定使用 `Color.gray` 而不是输入色。
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:1795` 附近：
    - `blockExplosionSmoke = new Effect(30, e -> { ... })`；
    - `color(Color.gray)`；
    - `randLenVectors(e.id, 6, 4f + 30f * e.finpow(), ...)`；
    - 每个向量绘制两枚圆：
      - 主圆 `e.fout() * 3f`
      - 副圆位移 `x/2,y/2`，半径 `e.fout()`。
  - 本地按 `new Effect` 声明顺序计数，`blockExplosionSmoke=152`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_BLOCK_EXPLOSION_SMOKE_ID = 152`；
    - 接入 `standard_effect_id("blockExplosionSmoke")` 与 `standard_effect(...)`，lifetime 为 `30.0`；
    - `standard_effect_draw_plan(...)` 新增 `blockExplosionSmoke`，复用 `SeededCircleParticles` 的 secondary circle 能力：
      - `color_from = Some("Color.gray")`
      - `count = 6`
      - `length = 4 + 30 * finpow`
      - 主圆 `radius_fout_scale=3`
      - 副圆 `secondary_vector_scale=0.5`、`secondary_radius_fout_scale=1`。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖 name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime；
  - `standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles` 覆盖 draw plan 与 12 个 circle primitives。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - 真实 renderer backend 仍未接入；
  - `blockExplosionSmoke` 之前的复杂 explosion/smoke line/light 组合仍需后续新增 line/light per-particle 表达。

### 12.207 Debris/unit dust circle Fx batch

- 2026-05-28：继续扫描 `Fx.java` 中无需新增 line/poly/light primitive 的圆形粒子效果，批量迁移 debris/unit dust 一组：
  - `breakProp`
  - `unitDrop`
  - `unitLand`
  - `unitDust`
  - `unitLandSmall`
  - `crawlDust`
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:378` 附近：
    - `breakProp = new Effect(23, ...)`，`Layer.debris`，输入色 `*1.1`，`count=6`，`length=19*finpow*max(rotation,1)`，半径 `0.3+fout*3.5*scl`；
    - `unitDrop = new Effect(30, ...)`，`Layer.debris`，`Pal.lightishGray`，`count=9`，`length=3+20*finpow`，半径 `0.4+fout*4`；
    - `unitLand = new Effect(30, ...)`，`Layer.debris`，输入色 `*1.1`，`count=6`，`length=17*finpow`，半径 `0.3+fout*4`；
    - `unitDust = new Effect(30, ...)`，`Layer.debris`，输入色 `*1.3`，`count=3`，`length=8*finpow`，`angle=rotation`、`range=30`，半径 `0.3+fout*3`；
    - `unitLandSmall = new Effect(30, ...)`，`Layer.debris`，输入色 `*1.1`，`count=(int)(6*rotation)`，`length=12*finpow*rotation`，半径 `0.1+fout*3`；
    - `crawlDust = new Effect(35, ...)`，`Layer.debris`，输入色 `*1.6`，`count=2`，`length=10*finpow`，半径 `0.3+fslope*4`。
  - 本地按 `new Effect` 声明顺序计数：
    - `breakProp=37`
    - `unitDrop=38`
    - `unitLand=39`
    - `unitDust=40`
    - `unitLandSmall=41`
    - `crawlDust=43`
  - `Pal.lightishGray` 来自 `Pal.java`：`a2a2a2`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增上述 6 个 `FX_*` 常量；
    - 接入 `standard_effect_id(...)` 与 `standard_effect(...)`，并设置 `Layer::DEBRIS`；
    - `standard_effect_color_symbol(...)` 新增 `Pal.lightishGray`；
    - `standard_effect_draw_plan(...)` 新增共享分支，按 effect id 参数化 color、color_mul、count、length、angle/range、radius_base/fout/fslope scale。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖 6 个 name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime/layer；
  - `standard_effect_draw_plan_covers_simple_smoke_and_fire_variants` 覆盖关键 draw plan 参数。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - 真实 renderer backend 仍未接入；
  - `unitPickup/landShock` 等线框/poly 类 debris Fx 需要新增线/多边形 primitive 后再迁移。

### 12.208 Fire/liquid/status simple circle Fx batch

- 2026-05-28：继续迁移 `Fx.java` 中无需新增 primitive 的 fire/liquid/status 简单圆形效果：
  - `ballfire`
  - `freezing`
  - `wet`
  - `muddy`
  - `sporeSlowed`
  - `oily`
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:1533` 附近：
    - `ballfire = new Effect(25f, ...)`，`Pal.lightFlame -> Pal.darkFlame`，`count=2`，`length=2+fin*7`，半径 `0.2+fout*1.5`；
    - `freezing = new Effect(40f, ...)`，`Liquids.cryofluid.color`，`count=2`，`length=1+fin*2`，半径 `fout*1.2`；
    - `wet = new Effect(80f, ...)`，`Liquids.water.color`，`alpha=clamp(fin*2)`，中心圆半径 `fout`；
    - `muddy = new Effect(80f, ...)`，`Pal.muddy`，`alpha=clamp(fin*2)`，中心圆半径 `fout`；
    - `sporeSlowed = new Effect(40f, ...)`，`Pal.spore`，中心圆半径 `fslope*1.1`；
    - `oily = new Effect(42f, ...)`，`Liquids.oil.color`，`count=2`，`length=1+fin*2`，半径 `fout`。
  - 本地按 `new Effect` 声明顺序计数：
    - `ballfire=131`
    - `freezing=132`
    - `wet=134`
    - `muddy=135`
    - `sporeSlowed=138`
    - `oily=139`
  - 颜色依据：
    - `Liquids.water.color = 596ab8`
    - `Liquids.cryofluid.color = 6ecdec`
    - `Liquids.oil.color = 313131`
    - `Pal.muddy = 432722`
    - `Pal.spore = 7457ce`
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增上述 6 个 `FX_*` 常量；
    - 接入 `standard_effect_id(...)` 与 `standard_effect(...)`；
    - `standard_effect_color_symbol(...)` 新增 liquid/status 颜色符号；
    - `standard_effect_draw_plan(...)` 新增：
      - `ballfire/freezing/oily` 的 `SeededCircleParticles`；
      - `wet/muddy/sporeSlowed` 的 `FilledCircle`。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖 6 个 name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 6 个 lifetime；
  - `standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles` 覆盖颜色、alpha、count、length、radius。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 仍未完成：
  - `melting` 需要 `Mathf.randomSeedRange(...)` 颜色扰动后再迁移；
  - `sapped/electrified/overdriven/overclocked` 等 square/poly 类效果需要新增 polygon/square primitive；
  - `sporeSlowed` 的 StatusEffect wiring 需后续核对 Java 侧实际使用的是 `Fx.sapped` 还是 `Fx.sporeSlowed`。

### 12.209 Fx.melting 熔融圆形粒子迁移

- 2026-05-28：继续对照 `Fx.java`，迁移 `melting` 标准特效；该效果仍可复用当前 `SeededCircleParticles`，但颜色插值需要先复刻 Arc `Mathf.randomSeedRange(...)` 的 seeded 抖动语义。
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:1550` 附近：
    - `melting = new Effect(40f, ...)`；
    - `color(Liquids.slag.color, Color.white, e.fout() / 5f + Mathf.randomSeedRange(e.id, 0.12f))`；
    - `randLenVectors(e.id, 2, 1f + e.fin() * 3f, ...)`；
    - `Fill.circle(..., .2f + e.fout() * 1.2f)`。
  - 本地 `javap`/`jshell` 核对 Arc：
    - `Mathf.randomSeedRange(seed, range)` 会使用 `seed * 99999` 调用 `Rand.setSeed(...)`；
    - 返回 `(nextFloat() - 0.5f) * range * 2f`；
    - golden：`Mathf.randomSeedRange(133L, 0.12f) = -0.085423604`。
  - `Liquids.slag.color` 来自 `Liquids.java`：`Color.valueOf("ffa166")`，Rust RGBA 为 `0xffa166ff`。
  - 本地按 `new Effect` 声明顺序计数：`melting=133`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_MELTING_ID = 133`；
    - 接入 `standard_effect_id("melting")` 与 `standard_effect(...)`，lifetime `40.0`；
    - 新增 `mathf_random_seed_range(seed, range)`，复用 `ArcRand` 且对齐 Arc 的 `seed * 99999`；
    - 新增颜色符号 `Liquids.slag.color`；
    - `standard_effect_draw_plan(...)` 新增 `melting`：
      - `color_from = Liquids.slag.color`
      - `color_to = Color.white`
      - `color_mix = fout / 5 + randomSeedRange(id, 0.12)`
      - `count = 2`
      - `length = 1 + fin * 3`
      - `radius = 0.2 + fout * 1.2`。
- 新增/更新验证：
  - `mathf_random_seed_range_matches_arc_seeded_range` 使用 Java probe golden 锁定 Arc seeded range；
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖 `melting` name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime；
  - `standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles` 覆盖颜色、mix、count、length、radius 与 primitive 展开。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core mathf_random_seed_range --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
- 仍未完成：
  - `sapped/electrified/overdriven/overclocked` 等 square/poly 类效果需要新增 polygon/square primitive；
  - `missileTrailSmoke*` / `artilleryTrailSmoke` 仍需 multi-pass、局部 lifetime 与 per-particle light/alpha 表达；
  - 真实 renderer backend 仍需接入这些 primitive 数据。

### 12.210 Shockwave stroked circle Fx batch

- 2026-05-28：继续对照 `Fx.java`，迁移一组只依赖 `Lines.circle` 的 shockwave 圆环效果；这些效果可直接映射到当前 `StandardEffectDrawKind::StrokedCircle`，不需要新增 line/poly/square primitive。
- 本轮迁移：
  - `shockwave=143`
  - `shockwaveSmaller=144`
  - `bigShockwave=145`
  - `spawnShockwave=146`
  - `podLandShockwave=147`
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:1625` 附近：
    - `shockwave = new Effect(10f, 80f, ...)`，`Color.white -> Color.lightGray`，stroke `fout*2+0.2`，半径 `fin*28`；
    - `shockwaveSmaller = new Effect(9f, 80f, ...)`，半径 `fin*22`；
    - `bigShockwave = new Effect(10f, 80f, ...)`，stroke `fout*3`，半径 `fin*50`；
    - `spawnShockwave = new Effect(20f, 400f, ...)`，stroke `fout*3+0.5`，半径 `fin*(rotation+50)`；
    - `podLandShockwave = new Effect(12f, 80f, ...)`，`Pal.accent`，stroke `fout*2+0.2`，半径 `fin*26`。
  - `Pal.accent` 来自 `Pal.java`：`Color.valueOf("ffd37f")`，Rust RGBA 为 `0xffd37fff`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增上述 5 个 `FX_*` 常量；
    - 接入 `standard_effect_id(...)` 与 `standard_effect(...)`，保留 Java lifetime/clip；
    - 新增颜色符号 `Pal.accent`；
    - `standard_effect_draw_plan(...)` 新增 shared shockwave 分支，统一输出 `StrokedCircle`。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖 5 个 name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime/clip；
  - `standard_effect_draw_plan_covers_smoke_trails_and_ripple` 覆盖 radius、stroke、颜色插值、`Pal.accent` resolved color。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
- 仍未完成：
  - `dropItem` 需要 sprite/rect item data 渲染；
  - `sapped/electrified/overdriven/overclocked` 需要 square primitive；
  - `explosion/dynamicExplosion/reactorExplosion/impactReactorExplosion` 需要 scaled、多 pass、line/light 组合后再迁移。

### 12.211 Launch/heal/overdrive stroked circle Fx batch

- 2026-05-28：继续迁移后段只依赖 `Lines.circle` 的标准 Fx，并顺手修正 `Fx.ripple` 的声明顺序 ID：本地脚本按 `Fx.java` 中 `new Effect` 声明顺序（含 `none=0`）核对，`ripple=244`，此前 Rust 常量 `243` 偏小 1。
- 本轮迁移/修正：
  - `ripple=244`（修正常量）
  - `launchAccelerator=246`
  - `launch=247`
  - `healWaveMend=249`
  - `overdriveWave=250`
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:2720` 附近：
    - `ripple = new Effect(30, ...)`，实际声明顺序 ID `244`；
    - `launchAccelerator = new Effect(22, ...)`，`Pal.accent`，stroke `fout*2`，半径 `4 + finpow*160`；
    - `launch = new Effect(28, ...)`，`Pal.command`，stroke `fout*2`，半径 `4 + finpow*120`；
    - `healWaveMend = new Effect(40, ...)`，`color(e.color)`，stroke `fout*2`，半径 `finpow*rotation`；
    - `overdriveWave = new Effect(50, ...)`，`color(e.color)`，stroke `fout`，半径 `finpow*rotation`。
  - `Pal.command` 来自 `Pal.java`：`Color.valueOf("eab678")`，Rust RGBA 为 `0xeab678ff`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 修正 `FX_RIPPLE_ID = 244`；
    - 新增 4 个 `FX_*` 常量；
    - 接入 `standard_effect_id(...)` 与 `standard_effect(...)`；
    - 新增颜色符号 `Pal.command`；
    - `standard_effect_draw_plan(...)` 新增 launch/heal/overdrive shared `StrokedCircle` 分支；
    - `healWaveMend/overdriveWave` 使用 `input_color=Some(color)` 对齐 Java `color(e.color)`。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖新增 name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime；
  - `standard_effect_draw_plan_covers_smoke_trails_and_ripple` 覆盖 radius、stroke、`Pal.command` 和 input color。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
- 仍未完成：
  - `bubble=245` 需要 seeded stroked-circle particles（随机位置圆环）；
  - `launchPod=248` 需要 scaled circle + 随机 lineAngle；
  - `healBlock/rotateBlock/lightBlock/overdriveBlockFull` 等需要 square/rect/icon/block data 表达。

### 12.212 Heal/shield stroked circle Fx batch

- 2026-05-28：回到前段 healing/shield 一组纯 `Lines.circle` 效果，迁移无需 triangle/light/line/square 的圆环部分；这些效果可直接映射为 `StrokedCircle`。
- 本轮迁移：
  - `healWaveDynamic=70`
  - `healWave=71`
  - `heal=72`
  - `dynamicWave=73`
  - `shieldWave=74`
  - `shieldApply=75`
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:805` 附近：
    - `healWaveDynamic = new Effect(22, ...)`，`Pal.heal`，stroke `fout*2`，半径 `4 + finpow*rotation`；
    - `healWave = new Effect(22, ...)`，`Pal.heal`，半径 `4 + finpow*60`；
    - `heal = new Effect(11, ...)`，`Pal.heal`，半径 `2 + finpow*7`；
    - `dynamicWave = new Effect(22, ...)`，`color(e.color, 0.7f)`，半径 `4 + finpow*rotation`；
    - `shieldWave = new Effect(22, ...)`，`color(e.color, 0.7f)`，半径 `4 + finpow*60`；
    - `shieldApply = new Effect(11, ...)`，`color(e.color, 0.7f)`，半径 `2 + finpow*7`。
  - `Pal.heal` 来自 `Pal.java`：`Color.valueOf("98ffa9")`，Rust RGBA 为 `0x98ffa9ff`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 6 个 `FX_*` 常量；
    - 接入 `standard_effect_id(...)` 与 `standard_effect(...)`；
    - 新增颜色符号 `Pal.heal`；
    - `standard_effect_draw_plan(...)` 新增 heal/shield shared `StrokedCircle` 分支；
    - `dynamicWave/shieldWave/shieldApply` 使用 `input_color=Some(color)` 且 `alpha=0.7` 对齐 Java `color(e.color, 0.7f)`。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖 6 个 name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime；
  - `standard_effect_draw_plan_covers_smoke_trails_and_ripple` 覆盖 radius、stroke、`Pal.heal` 与输入色 alpha。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
- 仍未完成：
  - `dynamicSpikes/greenBomb/greenLaserCharge` 等同区域效果包含 triangle/light 或多粒子 light，需后续扩展 primitive；
  - `disperseTrail/hitBullet*` 包含 lineAngle/scaled/light，不能用单圆环近似；
  - 这些 primitive 数据后续仍需接入真实 renderer backend。

### 12.213 Square primitive + status/overdrive square Fx batch

- 2026-05-28：为 `Fill.square(...)` 类标准 Fx 增加最小 square primitive 表达，并迁移此前因缺少 square 而跳过的 status/overdrive 方块效果。
- 本轮迁移：
  - `sapped=136`
  - `electrified=137`
  - `overdriven=140`
  - `overclocked=141`
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:1572` 附近：
    - `sapped = new Effect(40f, ...)`，`Pal.sap`，`randLenVectors(id, 2, 1 + fin*2)`，`Fill.square(..., fslope*1.1, 45)`；
    - `electrified = new Effect(40f, ...)`，`Pal.heal`，同 sapped 参数；
    - `overdriven = new Effect(20f, ...)`，输入色，`randLenVectors(id, 2, 1 + fin*2)`，`Fill.square(..., fout*2.3 + 0.5)`；
    - `overclocked = new Effect(50f, ...)`，输入色，中心 `Fill.square(..., fslope*2, 45)`。
  - `Pal.sap` 来自 `Pal.java`：`Color.valueOf("665c9f")`，Rust RGBA 为 `0x665c9fff`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - `StandardEffectDrawKind` 新增 `FilledSquare` 与 `SeededSquareParticles`；
    - 新增 `StandardEffectSquareRenderPrimitive` 与 `square_render_primitives_from_seed()`；
    - 新增 4 个 `FX_*` 常量并接入 metadata/name lookup；
    - 新增颜色符号 `Pal.sap`；
    - `sapped/electrified/overdriven` 输出 seeded square particles；
    - `overclocked` 输出中心 filled square。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖 4 个 name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime；
  - `standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles` 覆盖 square kind、颜色、输入色、粒子数量、半径公式与 square primitive 展开。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles --lib`
- 仍未完成：
  - square primitive 目前是计划/测试层表达，真实 renderer backend 仍需按 `StandardEffectSquareRenderPrimitive` 接入；
  - `healBlock` 等 `Lines.square` 需要 stroked square 支持，不等同于本轮 `Fill.square`；
  - `bubble` 仍需 seeded stroked-circle particles。

### 12.214 StrokedSquare + block square Fx batch

- 2026-05-28：在上一轮 `FilledSquare` 基础上继续扩展 `StrokedSquare`，迁移后段 block square 类标准 Fx 中无需 rect/icon/poly/line 的部分。
- 本轮迁移：
  - `healBlock=251`
  - `rotateBlock=253`
  - `lightBlock=254`
  - `overdriveBlockFull=255`
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:2775` 附近：
    - `healBlock = new Effect(20, ...)`，`Pal.heal`，stroke `2*fout+0.5`，`Lines.square(..., 1 + (fin*rotation*tilesize/2 - 1))`；
    - `rotateBlock = new Effect(30, ...)`，`Pal.accent`，alpha `fout`，`Fill.square(..., rotation*tilesize/2)`；
    - `lightBlock = new Effect(60, ...)`，输入色，alpha `fout`，`Fill.square(..., rotation*tilesize/2)`；
    - `overdriveBlockFull = new Effect(60, ...)`，输入色，alpha `fslope*0.4`，`Fill.square(..., rotation*tilesize)`。
  - `tilesize` 对应 Rust `vars::TILE_SIZE = 8`。
- Rust 新增/变化：
  - `core/src/mindustry/entities/effect.rs`
    - `StandardEffectDrawKind` 新增 `StrokedSquare`；
    - `StandardEffectSquareRenderPrimitive` 新增 `stroke` 字段；
    - `square_render_primitives_from_seed()` 支持 `StrokedSquare`；
    - 新增 4 个 `FX_*` 常量并接入 name/metadata；
    - `standard_effect_draw_plan(...)` 新增 block square 分支。
- 新增/更新验证：
  - `standard_effect_ids_include_puddle_ripple_dependencies` 覆盖 4 个 name/id；
  - `standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers` 覆盖 lifetime；
  - `standard_effect_draw_plan_covers_smoke_trails_and_ripple` 覆盖 `StrokedSquare` stroke、半径、alpha、输入色与 `TILE_SIZE` 换算。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
- 仍未完成：
  - `healBlockFull` 需要 block icon/rect/mixcol 数据；
  - `shieldBreak` 及后续 shield/unit 类需要 poly/arc/line primitive；
  - square primitive 仍需 renderer backend 消费。
