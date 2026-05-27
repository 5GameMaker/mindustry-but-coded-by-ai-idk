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
- 仍未完成：当前 controller 写回只覆盖 Reconstructor 升级时的 command/defaultCommand 最小分支；尚未完整实现 Java `isCommandable()`/`commands.contains(command)` 过滤、UnitType.init 自动 commands 列表、完整 `UnitType.create(team)` 重建 health/mount/weapon 默认值，以及 UnitCreateEvent/effect/sound 联机广播。

### 12.109 UnitFactory 最小 updateTile runtime tick

- 2026-05-27：对照 Java `UnitFactoryBuild.updateTile()`，把 `UnitFactory` 从仅有状态读写/纯 helper 推进到 owned runtime tick，并接入 `advance_owned_runtime_blocks(...).unit.factory` 聚合报告。
- Java 依据：
  - `updateTile()` 会在不可配置 factory 上强制 `currentPlan = 0`，修正越界 plan，`efficiency > 0 && currentPlan != -1` 时推进 `progress/time/speedScl`；
  - 每 tick 先 `moveOutPayload()`，然后在 `currentPlan != -1 && payload == null` 时检查 plan unit banned，`progress >= plan.time` 后创建 unit payload、清零 `payVector`、`consume()` 并触发 `UnitCreateEvent`；
  - `shouldConsume()` 要求 `currentPlan != -1 && enabled && payload == null && team.activateUnitFactories()`，动态消耗按当前 plan requirements 和 `rules.unitCost(team)` 缩放。
- Rust 新增/变化：
  - `ensure_unit_state_for_building(...)` 现在会为 `BlockDef::UnitFactory` 懒创建 `GameRuntimeUnitBlockState::Factory { common, factory }`；
  - 新增 `GameRuntimeUnitFactoryFrameReport`，并把 `GameRuntimeOwnedUnitFrameReport` 拆为 `factory + reconstructor`；
  - 新增 `advance_owned_unit_factories(...)` / ticks：检查当前 plan requirements、调用 `unit_factory_update(...)` 推进进度，完成时用 `UnitComp::new(...).to_sync_wire()` 生成 Java-style `PayloadRef::Unit`，写入 factory command/defaultCommand controller，消耗 items；
  - `transfer_payload_output_to_front(...)` 现在也能把 `UnitFactory` 作为 payload source，允许 factory 输出到前方 payload conveyor/router/reconstructor 等真实目标。
- 新增 core 回归测试：
  - `game_runtime_unit_factory_produces_configured_unit_payload`（通过 `advance_owned_runtime_blocks` 验证聚合入口）
  - `game_runtime_unit_factory_outputs_payload_to_front_conveyor`
- 验证：
  - `cargo test -p mindustry-core unit_factory`
  - `cargo check --workspace`
- 仍未完成：当前 UnitFactory runtime 尚未完整实现 Java `config(UnitCommand)`/`configClear` 的命令配置入口、`canSetCommand()`/命令合法性过滤、`team.activateUnitFactories()` 的精确 team rule、创建 sound/effect/event、同 tick moveOut 后立即再生产的 Java 顺序，以及完整 UI/config/logic sense 行为。

### 12.110 UnitFactory 配置入口接入

- 2026-05-27：对照 Java `UnitFactory` 构造器中的 `config(Integer.class, ...)` 与 `config(UnitType.class, ...)`，在 `GameRuntime` 接入 owned building 的 unit factory 计划配置入口，避免 UnitFactory 只能靠测试手写 `current_plan` 才能运行。
- Java 依据：
  - `config(Integer)` 在 `!configurable` 时直接返回；plan 越界或负数会把 `currentPlan` 置为 `-1`，plan 变化时清零 `progress`；
  - `config(UnitType)` 会按 `plans.indexOf(p -> p.unit == val)` 映射 plan，未命中时等价清空当前 plan；
  - `config()` 返回当前 plan integer，联机/回滚侧需要保存为 Java-like tile config。
- Rust 新增/变化：
  - 新增 `GameRuntimeUnitFactoryConfigureResult`，区分 `Configured / Cleared / MissingBuilding / MissingRuntimeState / NotUnitFactory / NotConfigurable / UnknownUnit`；
  - 新增 `GameRuntime::configure_owned_unit_factory_plan(...)`，按 tile 找 building，懒创建 `GameRuntimeUnitBlockState::Factory`，调用 `unit_factory_configure_plan(...)`，并把 `BuildingComp.config` 写成 `TypeValue::Int(current_plan)` 或清空；
  - 新增 `GameRuntime::configure_owned_unit_factory_unit(...)`，按 `ContentId` 映射到 factory plan；未命中当前 factory plans 时清空 plan，且先检查 block 是否为可配置 UnitFactory，贴近 Java `if(!configurable) return` 的顺序；
  - 该入口直接修改真实 runtime sidecar 与 building config，后续可继续接入 `TileConfigCallPacket`/输入处理，而不是独立 helper。
- 新增 core 回归测试：
  - `game_runtime_configures_unit_factory_plan_and_clears_progress_like_java`
  - `game_runtime_configures_unit_factory_by_unit_id_like_java_unit_config`
  - `game_runtime_rejects_unit_factory_config_for_wrong_or_unconfigurable_blocks`
- 验证：
  - `cargo test -p mindustry-core unit_factory`
- 仍未完成：`UnitCommand` 配置/清空、`command` 是否属于目标 unit `commands` 的过滤、`UnitType.init()` 自动命令列表、UI 选择表、logic `senseObject(LAccess.config)`/网络 `TileConfigCallPacket` 分发仍需后续闭环。

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
- 验证：
  - `cargo test -p mindustry-core unit_cargo`
  - `cargo test -p mindustry-core unit_tether_block_spawned --lib`（本轮通过 2/2）
  - `cargo test -p mindustry-desktop unit_tether_block_spawned --lib`（本轮通过 1/1）
  - `cargo test -p mindustry-desktop unit_cargo --lib`（本轮通过 1/1）
  - `cargo test -p mindustry-server unit_cargo --lib`（本轮通过 6/6）
  - `cargo test -p mindustry-tests real_server_desktop_unit_cargo_loader_tether_spawn_syncs_to_client_runtime -- --nocapture`（本轮通过）
  - `cargo fmt --check`
  - `cargo check --workspace`（本轮通过，仅保留既有 unused warning）
- 仍未完成：server-side `BuildingTetherComp` 还未作为正式组件并入 `UnitComp`/Groups.unit 生命周期；当前 cargo AI 仍是 server launcher 驱动的最小 authoritative 闭环，尚未完整迁移 Java `CargoAI.retarget()/timer/dropSpacing/noDestTimer`、真实平滑移动与客户端单位实体 snapshot 可视化；loader 资源 consumer 的全局 shouldConsume/rollback 权限细节、unload config 的 UI 选择表/rollback 权限细节、Java 客户端/服务端更完整联机兼容仍待补。
