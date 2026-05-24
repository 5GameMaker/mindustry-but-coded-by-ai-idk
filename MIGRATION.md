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
- 已对照 `Door.getPlanRegion()`、`Door.draw()`、`AutoDoor.draw()` 锁定：
  - `config == Boolean.TRUE` 或 `open == true` 时选择 openRegion；
  - 否则选择默认 region。

仍需：

- `Door.updateChained()` 的真实 proximity flood-fill adapter；
- 接入真实 Units/tree/pathfinder/Call runtime。

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

- 已对照 `RegenProjector.drawPlace()/drawSelect()/drawLight()/setStats()` 锁定：
  - place/select 均使用 `range * tilesize` 的 dash square；
  - place 坐标来自 `tile * tilesize + offset`，select 坐标来自建筑当前 `x/y`；
  - selected alpha 由 `absin(4f, 1f)` 派生；
  - `DrawDefault` 下 `drawer.drawLight(this)` 无额外光效，当前 light plan 显式记录为无额外绘制；
  - repair time stat 按 Java `(int)(1f / (healPercent / 100f) / 60f)` 截断；
  - booster 使用 `optionalMultiplier`，range boost 为 `0f`。

仍需：

- `updateTargets()` 的真实 indexer 范围扫描接入；
- `lastUpdateFrame` 与真实 world.build(pos).heal/recentlyHealed 应用；
- drawPlace/drawSelect 的目标列表高亮接入真实 indexer / targets。

### 7.7 BaseShield

已推进：

- `BaseShieldState`
- `base_shield_update(...)`
- `base_shield_should_interact(...)`
- `base_shield_unit_overlap(...)`
- `base_shield_unit_action(...)`
- `write_base_shield_state(...)`
- `read_base_shield_state(...)`
- `BaseShieldDrawCommand`
- `BaseShieldDrawPlan`
- `BaseShieldRangePlan`
- `BaseShieldInteractionPlan`
- `BaseShieldTintPlan`
- `base_shield_clip_radius(...)`
- `base_shield_radius(...)`
- `base_shield_in_fog_to(...)`
- `base_shield_should_absorb_bullet(...)`
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
  - unit overlap 大于 0 时被 repel，大于 `hitSize * 1.5` 时 kill。
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

仍需：

- 接入真实 Groups.bullet / Units.nearbyEnemies 运行态；
- shieldColor/teamColor 到真实渲染颜色的 adapter；
- drawPlace/drawSelect dash circle helper 接入真实 renderer。

### 7.7a ShockMine

已推进：

- `ShockMineStatsPlan`
- `ShockMineDrawPlan`
- `ShockMineTriggerPlan`
- `shock_mine_should_trigger(...)`
- `shock_mine_stats_plan(...)`
- `shock_mine_stats_text(...)`
- `shock_mine_draw_plan(...)`
- `shock_mine_lightning_angles(...)`
- `shock_mine_bullet_angles(...)`
- `shock_mine_trigger_plan(...)`
- 已对照 `ShockMine.unitOn()/triggered()/draw()/setStats()` 锁定：
  - 触发条件为 `enabled && unit.team != team && timer(timerDamage, cooldown)`；
  - 触发后自身承受 `tileDamage`；
  - lightning 创建次数为 `tendrils`，每条角度来自 `Mathf.random(360f)`，当前由上层注入随机角度数组以保持纯函数可测；
  - bullet 非空时创建 `shots` 枚，角度为 `(360f / shots) * i + Mathf.random(inaccuracy)`，当前由上层注入 inaccuracy offsets；
  - team top 绘制使用 `teamAlpha`；
  - stats damage 文案保留 tendrils 与 damage 的 2 位格式需求。
  - stats 文案按上游 `Core.bundle.format("bullet.lightning", tendrils, Strings.autoFixed(damage, 2)).replace("[stat]", "[white]")` 的英文 bundle 形态收口。

仍需：

- 接入真实 `Lightning.create(...)`、`BulletType.create(...)` 与 `Building.damage(...)`；
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

仍需：

- 接入真实 `fogControl.forceUpdate(team, this)`；
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

仍需：

- 接入真实 `Groups.bullet.intersect(...)`、hit/wave effect、sound、shake 与 Trigger 事件；
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
- `ForceProjectorState`
- `force_projector_real_radius(...)`
- `force_projector_shield(...)`
- `force_projector_update(...)`
- `force_projector_sense(...)`
- `force_projector_set_shield(...)`
- `force_projector_picked_up(...)`
- `force_projector_bar_fraction(...)`
- `force_projector_absorb_bullet(...)`
- `force_projector_absorb_explosion(...)`
- `ForceProjectorBulletAbsorb`
- `ForceProjectorDrawCommand`
- `ForceProjectorDrawPlan`
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
- 已对照 `ForceProjector.setBars()` / `sense(...)` 锁定：
  - shield bar fraction = `1 - buildup / (shieldHealth + phaseShieldBoost * phaseHeat)`；
  - `LAccess.heat` 返回 `buildup`；
  - `LAccess.shield` 返回剩余护盾；
  - `setProp(LAccess.shield)` 反向设置 `buildup`。
- 已对照 `ForceProjector.pickedUp()` 锁定：
  - pickup 后只清零 `radscl` 与 `warmup`；
  - `broken / buildup / phaseHeat` 保持原状态，序列化仍按 Java 5 字段读写；
  - `hit` 为 transient，read 后恢复为 `0`。
- 已对照 `ForceProjector.draw()` / `drawShield()` 锁定：
  - buildup > 0 时先绘制 topRegion additive；
  - `broken` 或 `realRadius <= 0.001` 时不绘制盾体但仍 final reset；
  - animateShields 时走 shield layer + fill poly；
  - 非 animateShields 时 stroke 1.5、alpha `0.09 + clamp(0.08 * hit)`、fill poly + outline poly；
  - `shieldRotation`、`sides`、`hit` layer offset 已进入 draw plan。
- `MendProjectorState`
- `mend_projector_update(...)`
- `write_mend_projector_state(...)`
- `read_mend_projector_state(...)`
- `OverdriveProjectorState`
- `overdrive_projector_update(...)`
- `write_overdrive_projector_state(...)`
- `read_overdrive_projector_state(...)`
- `overdrive-dome` 变体当前按同类 overdrive 投射器状态机推进。

仍需：

- `ForceProjector.deflectBullets()` 接入真实 `Groups.bullet.intersect(...)` 与正多边形检测 adapter；
- `ForceProjector.draw()` 接入真实 renderer/Draw dispatcher；
- `ForceProjector.setBars()/sense/setProp` 接入真实 building runtime；
- `ForceProjector.onRemoved/inFogTo/shouldAmbientSound/overwrote` 生命周期辅助；
- `DirectionalForceProjector` 接入真实 Groups.bullet.intersect、absorb effect、shield break effect 与 renderer；
- `MendProjector` 真实 range indexer / world heal / drawPlace / drawSelect 接入；
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
- `OverdriveProjector`：纯语义层已补齐 stats、boost bar、place/select/light/draw plan 和 Java 线框公式；下一步接入范围扫描、加速状态应用、效果/音效与真实 block runtime；
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
