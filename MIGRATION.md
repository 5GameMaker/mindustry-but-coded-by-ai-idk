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

仍需：

- 完整 Java 兼容 `NetworkIO.writeWorld/loadWorld`；
- markers/custom chunks 与完整 save dispatcher 的运行态接入；
- 将 `RawSaveEnvelope` region 层与 runtime world/entities 物化流程接成完整 save read/write dispatcher；
- `teamBlocks` 导出补 typed config 保真与 content header 临时映射写出；
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
- `BuildTurretBuild.write/read()` 已新增基于 `ContentLoader` 与 `TypeIO.writePlans/readPlans` 的 typed plans 读写路径；旧 raw 路径保留作兼容兜底。
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
- `force_projector_real_radius(...)`
- `force_projector_shield(...)`
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
- `write_force_projector_state(...)`
- `read_force_projector_state(...)`
- 已对照 `ForceProjector.updateTile()` 推进：
  - `broken / buildup / radscl / warmup / phaseHeat / hit` 的主状态机；
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
  - `EffectBlockRuntimeContext` 目前支持 `Projector / Radar / BaseShield / ShockwaveTower` 四类上下文；
  - `EffectBlockRuntimeResources` 将“已存储 state”之外的 FogControl、content、building/bullet/unit 候选等资源单独传入，方便后续 building store 只保存 `EffectBlockRuntimeState`；
  - `EffectBlockFrameInput` 开始承接 `GameState::advance_game_update_frame(...)` 输出的 `delta/update_id/tick` 以及 fog/tileSize 等帧级参数；
  - `effect_block_building_delta(...)` / `effect_block_building_edelta(...)` 对齐 Java `BuildingComp.delta() = Time.delta * timeScale` 与 `edelta() = efficiency * delta()` 的核心公式；
  - `effect_block_update_runtime(...)` 按传入上下文复用 `effect_projector_update_runtime(...)`、`effect_radar_update_runtime(...)`、`effect_base_shield_apply_runtime(...)` 与 `effect_shockwave_tower_apply_runtime(...)`；
  - `effect_block_update_runtime_state(...)` 已能从统一 `EffectBlockRuntimeState` 自动拆出具体 state 并调用对应 dispatcher，block/state/resource 不匹配时返回 `None`；
  - `effect_block_update_building_runtime(...)` 将 `BuildingComp.block.id -> EffectBlockData -> state store ensure -> runtime dispatch` 串成单栋建筑的一站式入口，为后续 `update_all_buildings(...)` 遍历打通最小调用链；
  - `effect_radar_update_building_frame(...)` 已能直接从 `BuildingComp.team/tile_pos/efficiency` 与帧输入组装 Radar runtime 资源，测试覆盖了 `GameState::advance_game_update_frame(...) -> RadarState` 的最小帧推进路径；
  - `effect_projector_update_building_frame(...)` 已能从 `BuildingComp` 与 `EffectBlockFrameInput` 组装 projector family runtime 输入，`RegenProjector` 测试覆盖了 `GameState::advance_game_update_frame(...) -> delta/edelta/update_id -> last_update_frame` 的状态门控链路；
  - `effect_projector_update_building_frame_with_timer(...)` 已开始把 `BuildingTimerState` 侧车接入 `MendProjector` 的 `timer(timerUse, useTime / timeScale)` 可选消耗门控；当前用 `MEND_PROJECTOR_TIMER_USE_SLOT = 0` 作为过渡槽位；
  - `effect_base_shield_update_building_frame(...)` 已能从 `BuildingComp`、bullet/unit 候选与帧 delta 组装 BaseShield runtime 输入，写回 `BulletComp::absorb()` 与 `BaseShieldState.smooth_radius`；
  - `effect_shockwave_tower_update_building_frame(...)` 已能从 `BuildingComp.potential_efficiency`、building delta/edelta、bullet 候选与 timer gate 组装 ShockwaveTower runtime 输入，写回 bullet damage/remove 与 `ShockwaveTowerState`；
  - `effect_shockwave_tower_update_building_frame_with_timer(...)` 已开始把 `BuildingTimerState` 侧车接入 ShockwaveTower 的 `timer(timerCheck, checkInterval)` 门控；当前用 `SHOCKWAVE_TOWER_TIMER_CHECK_SLOT = 0` 作为过渡槽位，后续应随完整 block timer slot 分配迁移替换；
  - `effect_build_turret_timer_target_ready(...)` 已开始把 `BuildingTimerState` 侧车接入 `BuildTurret` 的 `timer(timerTarget, targetInterval)` 队伍计划/跟随搜索门控；当前用 `BUILD_TURRET_TIMER_TARGET_SLOT = 0`、`BUILD_TURRET_TIMER_TARGET2_SLOT = 1` 作为过渡槽位；
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

补充已接入的 world/content 运行态桥：

- `World::wall_solid_with_content(...)` / `wall_solid_full_with_content(...)` / `passable_with_content(...)` 已通过 `ContentLoader` 查询真实 `BlockDef.base().solid/fills_tile`，不再只能用 “非 air 即 solid” 的临时 fallback；
- 这一步让 pathfinder、建筑放置、单位通行等后续 runtime 可以直接复用 content registry 中的 Java block metadata。

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
