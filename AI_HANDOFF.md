# AI 交接文档：Mindustry Java → Rust 迁移

**固定 Rust 工作路径（上下文压缩后不要重新找）：`D:\MDT\rust-mindustry`；命令中统一写作 `D:/MDT/rust-mindustry`。**

```text
CONTEXT_BOOTSTRAP_RUST_WORKDIR=D:/MDT/rust-mindustry
CONTEXT_BOOTSTRAP_JAVA_REFERENCE=D:/MDT/mindustry-upstream-v157.4
CONTEXT_BOOTSTRAP_FORBIDDEN_OLD_RUST_DIR=D:/MDT/mindustry-rust
CONTEXT_BOOTSTRAP_GIT_BRANCH=main
```

> **压缩上下文后先读这一行：当前唯一 Rust 工作路径是 `D:\MDT\rust-mindustry`（等价命令路径 `D:/MDT/rust-mindustry`）。不要重新搜索、不要改用 `D:\MDT\mindustry-rust`，后者是废案。**

## 0. 固定路径速记（上下文压缩后优先看）

- Rust 工作仓库：`D:\MDT\rust-mindustry`（命令中可写作 `D:/MDT/rust-mindustry`）
- Java 参考仓库：`D:\MDT\mindustry-upstream-v157.4`（命令中可写作 `D:/MDT/mindustry-upstream-v157.4`）
- 废案目录，禁止参考/写入：`D:\MDT\mindustry-rust`
- Git 远端：`https://github.com/Anon-deisu/mindustry-rust`
- 只推送分支：`main`
- Cargo 完整路径：`C:/Users/yuyu/.cargo/bin/cargo.exe`

## 1. 最终目标（不得偏移）

把 Java 参考仓库：

- `D:/MDT/mindustry-upstream-v157.4`

逐文件、逐文件夹对照迁移/重写为 Rust 项目：

- `D:/MDT/rust-mindustry`

最终交付目标是一个尽可能接近原版 Mindustry v158.1 行为的 Rust 版 MDT/Mindustry：

1. 尽量保持原 Java 项目的模块结构、命名语义、运行生命周期与数据流。
2. 尽量实现 Rust 客户端与原版 Java 服务端/客户端在联机层面的互通。
3. 长期目标是可启动、可连接、可进入世界、可游玩的 Rust 版客户端/服务端。
4. 不要把任务降级成“只写框架”“只做协议 demo”或“只做局部示例”。

明确不要使用废案目录：

- `D:/MDT/mindustry-rust`

当前有效工作仓库是：

- `D:/MDT/rust-mindustry`

远端同步目标：

- `https://github.com/Anon-deisu/mindustry-rust`
- 只推送 `main` 分支
- 不要推送 `master`

---

## 2. 用户工作风格要求

后续 AI 必须保持以下风格：

1. 始终使用中文回复。
2. 用户希望持续推进，不要频繁停下来等待确认。
3. 可以适当同步“正在做什么 / 下一步做什么”，但不要因为同步而停止工作。
4. 遇到明确可执行的迁移、修复、测试、补齐任务时，直接执行。
5. 任务描述不清且会显著影响实现路径时才停下来问关键问题。
6. 修复或新增闭环后要自己运行验证测试。
7. 用户明确要求使用子代理辅助探索/写入；中大型任务要主动使用子代理。
8. 子代理使用规则：
   - `explorer`：只读探索、定位 Java/Rust 对应实现、梳理调用链和缺口。
   - `worker`：边界清晰的局部实现、测试补齐、文档更新。
   - `ultra_worker`：复杂、高风险、核心迁移或疑难调试。
9. 不要让 `worker` 做开放式探索；先探索再实现时，应先派 `explorer`。
10. 每完成一个明确迁移闭环或文件级重构，使用中文提交标题提交并推送到 `origin main`。

---

## 3. 当前环境与常用命令

当前环境是 Windows，项目位于 `D:/MDT`。

Cargo 不在默认 PATH，优先使用完整路径：

```powershell
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' fmt
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-core -- --test-threads=1
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-server -- --test-threads=1
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-desktop -- --test-threads=1
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test --workspace -- --skip mindustry::net::arc_net_provider::tests::* --test-threads=1
```

Git 常用命令：

```powershell
git -C 'D:/MDT/rust-mindustry' status --short
git -C 'D:/MDT/rust-mindustry' branch --show-current
git -C 'D:/MDT/rust-mindustry' log -8 --oneline
git -C 'D:/MDT/rust-mindustry' add <changed-files>
git -C 'D:/MDT/rust-mindustry' commit -m '<中文提交标题>'
git -C 'D:/MDT/rust-mindustry' push origin main
```

注意：

- 分支必须是 `main`。
- 提交标题必须用中文。
- 不要把 `D:/MDT/mindustry-rust` 当作工作仓库。
- 如果读取文件、`git log` 或命令输出出现中文乱码，优先按 UTF-8 重新读取/输出，再判断内容是否真的损坏。
- PowerShell 可优先尝试：

  ```powershell
  Get-Content -LiteralPath '<file>' -Encoding UTF8
  git -C 'D:/MDT/rust-mindustry' -c i18n.logOutputEncoding=utf-8 log --oneline
  ```

- 若 PowerShell 受限导致无法设置 `[Console]::OutputEncoding`，不要卡住；改用 `Get-Content -Encoding UTF8`、Git 的 `-c i18n.logOutputEncoding=utf-8`，或用可用 shell 再试。

---

## 4. 最近已完成并推送的提交

压缩上下文后不要依赖记忆里的旧提交号，优先在固定 Rust 工作路径执行：

```powershell
git -C 'D:/MDT/rust-mindustry' log -10 --oneline
git -C 'D:/MDT/rust-mindustry' status --short
```

本轮开始前最近已推送到 `main` 的提交包括：

1. `b5c318a 接入天气实体快照运行态`
2. `9c52591 接入水洼实体快照运行态`
3. `669806c 固定文档中的Rust工作路径`
4. `58259bb 补充火焰实体快照联机验证`
5. `b9c9231 接入火焰实体快照运行态`
6. `696e903 按实体类型分类拆包`
7. `5e48e9b 迁移实体类型编号基线`
8. `8808169 补充玩家实体快照联机验证`
9. `043424b 支持混合实体快照拆包`
10. `685308e 接入玩家实体快照运行态`

本轮开始前最后确认时：

- 当前分支：`main`
- 最新提交：`b5c318a 接入天气实体快照运行态`
- `git status --short` 未显示已有未提交代码改动。

---

## 5. 最近一次完成的具体实现

### 2026-05-26 续作：服务端 world stream 导出 owned building entity chunk

文件：

- `server/src/lib.rs`
- `MIGRATION.md`
- `AI_HANDOFF.md`

完成内容：

1. `GameRuntime::export_network_map_snapshot(&ContentLoader)` 已成为核心 runtime 的统一 map 导出入口，服务端不再保留独立 `runtime_world_*` helper 副本。
2. `ServerLauncher::network_world_data_template(...)` 组装 `map_snapshot` 时直接调用 `self.runtime.export_network_map_snapshot(&self.content_loader)`，使导出侧能通过 content registry 判定 block kind。
3. core 内部的 `export_network_map_snapshot_from_parts(...)` 现在会为 owned building footprint 生成 Java chunk-map entity records：
   - center tile：`has_entity=true`、`is_center=true`、`building=Some(bytes)`；
   - non-center footprint tile：`has_entity=true`、`is_center=false`、不写 building chunk；
   - 普通 block run 不再跨过 entity tile，保持 `write_chunk_map(...)` 的 run-cover 约束。
4. `network_map_building_payload(...)` 当前写入：
   - 前置 block/build `revision` byte：由 block kind / sidecar state 计算，当前 PowerGenerator 系列为 `1`，其余已覆盖 tail 多数为 `0`；
   - 后接 `BuildingComp::write_base(..., false)`；
   - 已接入 `GameRuntimeDefenseWallState` 的 block-specific tail：`Door` 写 1 byte open、`AutoDoor/blast-door` 写父/子双 bool、`ShieldWall` 写 shield f32；
   - 已接入 `GameRuntimePowerBlockState` 的 power/light tail：PowerGenerator 系列按 Java `version()==1` 写 `productionEfficiency/generateTime`，`Nuclear/Impact/Variable/Heater` 继续追加各自字段，`LightBlock` 按 revision 0 写 color；
   - 已接入 `EffectBlockRuntimeState` 的 effect tail：`MendProjector/OverdriveProjector/ForceProjector/Radar/BuildTurret` 按 revision 0 写 Java 子字段，`BaseShield` 按 Java `version()==1` 写 `smoothRadius/broken`；
   - 已接入 `GameRuntimeProductionBlockState` 的 production tail：`Drill/BeamDrill/BurstDrill` 均按 Java `version()==1` 写 `progress/warmup` 或 `time/warmup`；
   - 已接入 `GameRuntimeCraftingBlockState` 的 crafting/heat tail：`Generic/Attribute/HeatCrafter` 按 revision 0 写 `progress/warmup`，`Separator` 按 Java `version()==1` 写 `progress/warmup/seed`，`HeatProducer` 先写 generic 前缀再写 `heat`；
   - 已接入 `GameRuntimeLiquidBlockState` 的 liquid bridge tail：仅 `LiquidBridge/bridge-conduit` 按 Java `version()==1` 写 `link/warmup/incoming/moved`，不对 `DirectionLiquidBridge` 写 Java 不存在的 tail；
   - 已接入 `GameRuntimeStorageBlockState` 的 core tail：`CoreBlock` 按 Java `version()==1` 写 `commandPos`，不写 `storage_capacity/no_effect/iframes/thruster_time` 等运行时字段；
   - defense turrets、payload block、物流/logic 等其余运行态子状态仍需继续补导出。
5. 新增/保留测试：
   - `game_runtime_exports_network_map_snapshot_with_owned_building_chunks`：验证 core runtime 可导出 multi-tile building footprint entity records，并可被 `GameRuntime::load_network_map_with_buildings(...)` 读回；
   - `game_runtime_exports_defense_wall_state_tail_in_network_map_snapshot`：验证 `door`、`blast-door`、`shielded-wall` 的导出 payload tail 能被 loader 读回到 `defense_wall_runtime_states`；
   - `game_runtime_exports_power_and_light_state_tail_in_network_map_snapshot`：验证 `thermal-generator`、`thorium-reactor`、`impact-reactor`、`flux-reactor`、`neoplasia-reactor`、`illuminator` 的导出 payload tail 能被 loader 零 trailing bytes 读回到 `power_runtime_states`；
   - `game_runtime_exports_effect_block_state_tail_in_network_map_snapshot`：验证 `mend-projector`、`overdrive-projector`、`force-projector`、`radar`、`shield-projector`、`build-tower` 的导出 payload tail 能被 loader 零 trailing bytes 读回到 `effect_runtime_store`；
   - `game_runtime_exports_production_state_tail_in_network_map_snapshot`：验证 `mechanical-drill`、`plasma-bore`、`impact-drill` 的导出 payload tail 能被 loader 零 trailing bytes 读回到 `production_runtime_states`；
   - `game_runtime_exports_crafting_state_tail_in_network_map_snapshot`：验证 `graphite-press`、`cultivator`、`separator`、`oxidation-chamber` 的导出 payload tail 能被 loader 零 trailing bytes 读回到 `crafting_runtime_states`；
   - `game_runtime_exports_liquid_bridge_state_tail_in_network_map_snapshot`：验证 `bridge-conduit` 的导出 payload tail 能被 loader 零 trailing bytes 读回到 `liquid_runtime_states`；
   - `game_runtime_exports_core_storage_state_tail_in_network_map_snapshot`：验证 `core-shard` 的导出 payload tail 能被 loader 零 trailing bytes 读回到 `storage_runtime_states`；
   - `server_world_data_exports_owned_building_chunks_for_runtime_loader`：验证服务端 world stream 使用同一核心导出入口。
6. 服务端集成测试覆盖：
   - 构造服务端 runtime owned `router`；
   - 触发连接后 world stream 下发；
   - 解码 `NetworkWorldData.map_snapshot`；
   - 断言 center record 带 building chunk；
   - 再用 `GameRuntime::load_network_map_with_buildings(...)` 反向读回 team / rotation / health / tile_pos。
7. 已验证：
   - `cargo test -p mindustry-core game_runtime_exports_network_map_snapshot_with_owned_building_chunks -- --test-threads=1`
   - `cargo test -p mindustry-core game_runtime_exports_defense_wall_state_tail_in_network_map_snapshot -- --test-threads=1`
   - `cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader -- --test-threads=1`
   - `cargo test -p mindustry-server server_update_flushes_pending_world_data -- --test-threads=1`
   - `cargo check -p mindustry-server`

### 2026-05-26 续作：PayloadAmmoTurret 旧式 PayloadSeq fallback

文件：

- `core/src/mindustry/world/blocks/defense/turrets/mod.rs`
- `core/src/mindustry/core/game_runtime.rs`
- `MIGRATION.md`

完成内容：

1. `payload_ammo_turret_read_payloads(...)` 已支持 Java legacy 正数长度 block-only `PayloadSeq`：
   - `count: short >= 0`
   - `blockId: short`
   - `amount: int`
   - 以 `PayloadKey(ContentType::Block, blockId)` 过滤合法 ammo 后写入 `PayloadSeq`
2. 保留新格式 `contentType + id + amount` 读取路径。
3. 补 `game_runtime_reads_payload_ammo_turret_legacy_block_only_payloads`，确认 legacy payload ammo 通过 runtime reader 后会过滤非法 block 并更新 `totalAmmo`。
4. 已验证：
   - `cargo test -p mindustry-core item_liquid_and_power_turret_helpers_follow_upstream_ammo_rules`
   - `cargo test -p mindustry-core turret`
   - `cargo check -p mindustry-core`

### 2026-05-26 续作：LogicProcessor revision 0 runtime 回归

文件：

- `core/src/mindustry/core/game_runtime.rs`
- `MIGRATION.md`

完成内容：

1. 补 `game_runtime_loads_processor_revision_zero_legacy_code_and_links`，构造 Java 旧 revision 0 processor payload：
   - `code`：Java UTF `"end"`；
   - `links total`：`short`；
   - `link positions`：`int[]`；
   - `varcount = 0`；
   - `memory = 0`；
   - 不写 revision 1+ compressed、revision 2+ ipt、revision 3+ tag/iconTag、revision 4+ waits/accumulator。
2. 断言 `GameRuntime::load_network_map_with_buildings(...)` 能恢复 `LogicProcessorState.legacy_code` 与 `legacy_link_positions`。
3. 已验证：
   - `cargo test -p mindustry-core game_runtime_loads_processor_revision_zero_legacy_code_and_links`
   - `cargo test -p mindustry-core logic_processor`
   - `cargo check -p mindustry-core`

### 2026-05-26 续作：BuildTurret raw plans fallback

文件：

- `core/src/mindustry/world/blocks/defense/mod.rs`
- `core/src/mindustry/core/game_runtime.rs`
- `MIGRATION.md`

完成内容：

1. `build_turret_read_child_with_loader(...)` 改为先读完 rotation 后的 plan bytes，再尝试 `TypeIO.read_build_plans(...)`：
   - typed 解码成功且无尾字节：恢复 `BuildTurretState.plans`；
   - typed 解码为 `None` 且无尾字节：恢复为空 plans；
   - 解码失败或有尾字节：保留 `BuildTurretState.raw_plans`，避免旧图/内容映射不完整时整栋 building parse error。
2. `GameRuntime` 增加 `game_runtime_preserves_build_turret_unparseable_raw_plans`，确认 map loader 仍能将 `BuildTurret` 写入 `EffectBlockRuntimeStateStore`。
3. 已验证：
   - `cargo test -p mindustry-core build_turret`
   - `cargo check -p mindustry-core`

### 2026-05-26 续作：UnitAssembler 旧式 PayloadSeq fallback

文件：

- `core/src/mindustry/world/blocks/units/mod.rs`
- `core/src/mindustry/type/payload_seq.rs`
- `core/src/mindustry/core/game_runtime.rs`
- `MIGRATION.md`

完成内容：

1. `UnitAssembler` 使用的 `read_payload_seq(...)` 已支持 Java `PayloadSeq.read()` 的旧式 block-only 正数长度格式：
   - `count: short >= 0`
   - 循环读取 `blockId: short`
   - 循环读取 `amount: int`
   - 以 `PayloadKey(ContentType::Block, blockId)` 写入 `PayloadSeq`
2. 通用 `PayloadSeq::read_java_new(...)` 同步支持同一 legacy 格式，不再对旧格式直接报错。
3. 补测试：
   - `payload_seq_reads_java_legacy_block_only_format`
   - `game_runtime_loads_unit_assembler_state_from_legacy_block_only_payload_seq`
4. 已验证：
   - `cargo test -p mindustry-core payload_seq`
   - `cargo test -p mindustry-core unit_assembler`
   - `cargo check -p mindustry-core`

### 2026-05-26 续作：PayloadMassDriver revision 0 runtime 回归

文件：

- `core/src/mindustry/core/game_runtime.rs`
- `MIGRATION.md`

完成内容：

1. 补 `game_runtime_loads_payload_mass_driver_revision_zero_without_tail_fields`，构造 Java 旧 revision 0 building payload：
   - `PayloadBlockBuild` common 前缀；
   - `link:int`；
   - `turretRotation:float`；
   - `state:byte`；
   - 不写 revision 1 才有的 `reloadCounter/charge/loaded/charging`。
2. 断言 `GameRuntime::load_network_map_with_buildings(...)` 能把旧 payload 加载成 `GameRuntimePayloadBlockState::MassDriver`，并保留 revision 1 尾字段默认值。
3. 已验证：
   - `cargo test -p mindustry-core game_runtime_loads_payload_mass_driver`
   - `cargo check -p mindustry-core`

### 2026-05-26 续作：TypeIO 对象读取限制对齐

文件：

- `core/src/mindustry/io/type_io.rs`
- `MIGRATION.md`

完成内容：

1. `TypeIO.read_object(...)` 非 safe 数组上限收紧为 Java `readObject(... safe=false ...)` 的 200 项。
2. `TypeIO.read_object_safe(...)` 字符串上限收紧为 Java 的 1200 chars。
3. 补 `object_reader_limits_match_java_safe_and_non_safe_modes`。
4. 已验证：
   - `cargo test -p mindustry-core mindustry::io::type_io::tests`
   - `cargo test -p mindustry-core logic_processor`
   - `cargo check -p mindustry-core`

### 2026-05-26 续作：LogicProcessor Java wire parity 收紧

文件：

- `core/src/mindustry/world/blocks/logic/mod.rs`
- `core/src/mindustry/io/type_io.rs`
- `core/src/mindustry/core/game_runtime.rs`
- `MIGRATION.md`

完成内容：

1. `write_logic_processor_state(...)` 新增 `max_instructions_per_tick` 参数，`privileged && revision >= 2` 写出 `ipt` 时按 Java `Mathf.clamp(ipt, 1, maxInstructionsPerTick)` 收紧。
2. `LogicProcessor` 写出 memory 字段改为固定 `0`，匹配 Java `LogicBuild.write()` 的 `//no memory -> write.i(0)`；读入仍保留 legacy memory slot skip，兼容旧图/旧存档。
3. 补充 revision 2/3 原始字节 sentinel 边界测试：
   - revision 2 privileged 只消费 `ipt`，并验证 clamp 后 sentinel 未被吞；
   - revision 3 unprivileged 直接从 `tag/iconTag` 开始，不误读不存在的 `ipt`；
   - writer memory 固定 0 与 writer ipt clamp 均有单测锁定。
4. 新增 `read_object_boxed(...)`，用于 Java `TypeIO.readObjectBoxed(read, true)` 对齐：
   - 非 safe 字符串读取不再套用 safe 字符上限；
   - 非 safe object/seq/array 上限按 Java `200` 项处理；
   - building/unit boxed 引用暂以 `TypeValue::Building(i32)` / `TypeValue::Unit(i32)` 保留稳定 wire id，等待真实 world/loadBlock 阶段解析。
5. `LogicProcessor` 变量读取已改用 `read_object_boxed(...)`，补了长字符串变量与 boxed reader 边界测试。
6. `TypeIO.read_object(...)` 的非 safe 数组上限也已收紧为 Java `200` 项；`read_object_safe(...)` 的字符串上限收紧为 Java 的 `1200` chars。
7. 已验证：
   - `cargo test -p mindustry-core logic_processor`
   - `cargo test -p mindustry-core boxed_object_reader_matches_java_processor_var_limits`
   - `cargo test -p mindustry-core mindustry::io::type_io::tests`
   - `cargo test -p mindustry-core game_runtime_loads_processor_state_from_network_map_building_payload`
   - `cargo check -p mindustry-core`

后续注意：

- `LogicProcessor` 变量读取已使用 boxed reader；后续差距是把 `TypeValue::Building/Unit` 这类 boxed wire id 在真实 `loadBlock` / world runtime 阶段延迟 unbox 成 live reference，而不是只保留 sidecar 数据。
- `GameRuntime` 当前已能读入 processor sidecar，但还缺少统一写回/保存出口，以及将变量、links、wait timers 恢复到真实 `LExecutor` 的 runtime 接入。

### 较早完成：世界流前置信息

文件：

- `core/src/mindustry/net/network_io.rs`
- `core/src/mindustry/core/net_client.rs`
- `core/src/mindustry/net/mod.rs`
- `core/src/mindustry/io/versions/mod.rs`
- `core/src/mindustry/core/game_state.rs`

完成内容：

1. `NetworkIO.writeWorld/loadWorld` 对应的 world stream 前半段已经进一步迁移：
   - 能 zlib inflate/deflate world stream；
   - 能读写 Java `DataOutput.writeUTF` 的 modified UTF-8；
   - 能解析 `rules_json`、`map_locales_json`、`map_tags`、`wave`、`wavetime`、`tick`、随机种子和 `player_id`；
   - 能解析生成类 `mindustry.gen.Player.write(...)` 的 revision 0/1/2 玩家 wire body；
   - 能继续解析 `SaveIO` 尾部前缀：content header、content patches、map、team blocks；
   - marker/custom chunks 已能按 Java `markers -> custom chunks` 顺序拆分；成功时会填充 `markers_snapshot`、`marker_summary` 与 `custom_chunks_snapshot`，同时保存 `marker_custom_tail`；失败时写回优先保留 opaque raw tail，避免额外补空 custom chunk 或丢失未知尾部。
2. `NetClient` 收到 world stream 后：
   - 解析成功才自动发送 `ConnectConfirmCallPacket`；
   - 解析失败不确认、不结束加载态，并记录错误；
   - 记录 `last_loaded_world_data` 供后续 world 生命周期接入。
3. `SaveVersion` map wire 记录已经能展开成 Rust 轻量 `Tiles`，用于下一步真正应用地图。
4. `GameState::apply_network_world_data(...)` 已接入 world stream 前置信息：
   - 更新 wave / wavetime / tick；
   - 用 map tags + map snapshot 更新 `MapDescriptor`；
   - 解析并写入 `MapLocales`；
   - 将 content patches 记录到 `DataPatcherState`；
   - 将 `NetworkWorldData.team_blocks_snapshot` 通过 `content_header_snapshot` 的 Java content id/name 映射物化到 runtime `Teams` build plans，避免 `SaveVersion.readTeamBlocks(...)` 结果只缓存不生效。
5. `mindustry_server::ServerLauncher::flush_pending_world_data(...)` 已从 `write_minimal_world_data(...)` 升级为 runtime world-data 组装：
   - `network_world_data_template(...)` 会写入 base content header、空 content patches、当前 world 的轻量 map snapshot、runtime `Teams.plans` 导出的 `team_blocks_snapshot`、markers/custom chunks；
   - 当前 world map snapshot 由 `GameRuntime::export_network_map_snapshot(&ContentLoader)` 统一导出，并已开始写 owned building entity chunk：center tile 写 `revision byte + BuildingComp::write_base(...)`，multi-tile footprint 的非中心 tile 写 `has_entity=true/is_center=false`，防御墙类 tail 已覆盖 `Door/AutoDoor/ShieldWall`，power/light tail 已覆盖 PowerGenerator 系列与 LightBlock，effect tail 已覆盖 `Mend/Overdrive/Force/Radar/BaseShield/BuildTurret`，production tail 已覆盖 `Drill/BeamDrill/BurstDrill`，crafting tail 已覆盖 `Generic/Separator/HeatProducer`，liquid tail 已覆盖 Java `LiquidBridge`，storage tail 已覆盖 `CoreBlock.commandPos`；
   - 每个连接发送前补 `player_id` 与 `NetworkPlayerData::bootstrap()`，再通过 `write_world_data(...)` 形成 Java-like compressed world stream；
   - 其余 block-specific building tail serialization 仍需继续对照完整 `SaveVersion.writeMap(...)` 补炮塔/物流/payload/logic 等 state writers。

已验证：

```powershell
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' fmt
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-core mindustry::net::network_io -- --test-threads=1
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-core mindustry::core::net_client -- --test-threads=1
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-core mindustry::io::versions -- --test-threads=1
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-core mindustry::core::game_state -- --test-threads=1
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-core apply_network_world_data
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-core mindustry::net::network_io -- --test-threads=1
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-server server_update_flushes_pending_world_data
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test --workspace -- --skip mindustry::net::arc_net_provider::tests::* --test-threads=1
```

验证结果：

- workspace 测试通过；当前 `mindustry-core` 约 1472 个测试通过。
- 仅有既存 warning（例如 `ItemUnlockExt` 未使用等），未发现本轮新增失败。

已提交并推送：

```text
c76842c 解析世界流玩家与存档头
eaaec1e 展开存档地图到轻量瓦片
e154545 接入世界流前置信息到游戏状态
```

---

## 6. 当前迁移进度的真实判断

不要对用户虚报“快完成”。

当前 Rust 项目已经有一些网络启动、事件轮询、客户端参数连接、服务端端口参数等基础闭环，但距离“完整可游玩 Rust 版 Mindustry”仍很远。

粗略完成度建议口径：

- 若以“完整可游玩、联机互通、内容/世界/实体/UI/渲染完整”为 100%，当前约为 **6%～9%**。
- 若只以“项目结构与早期网络启动骨架”为目标，当前进度会更高，但这不是最终目标。

主要已具备：

1. Rust workspace 已存在并可运行部分 crate 测试。
2. 服务端有真实网络启动器与网络循环保持。
3. 桌面客户端有真实网络层与参数连接入口。
4. 服务端支持端口参数读取。
5. 部分 `NetClient` / `NetServer` / packet / service 生命周期已有骨架。
6. 服务端能自动下发最小 world stream，Rust 客户端能解析并确认。
7. Rust 客户端已经能解析 world stream 前置信息、生成类玩家 body，以及 SaveIO 尾部的 content/map/team-blocks 前缀。
8. `GameState` 可以接收 world stream 前置信息并更新轻量地图/波次/locales/patcher 状态。

主要缺口：

1. 服务端尚未完整生成 Java 兼容 `NetworkIO.writeWorld` payload（玩家、content、map、markers、custom chunks 仍需真实 runtime 数据）。
2. 客户端尚未把 parsed world stream 完整应用到 `World` / `Groups` / player / entity 生命周期。
3. marker/custom chunks 的 UBJSON/JsonIO 字节解析尚未迁移。
4. 实体 update/collision、单位、方块、队伍、规则、存档、地图加载仍大量缺失。
5. 桌面客户端仍缺少真正窗口、渲染、输入、UI 与游戏主循环体验。
6. 与 Java 原版服务端/客户端的协议级互通已推进到 world stream 前缀解析阶段，但仍不是可游玩的互通客户端。

---

## 7. 下一步优先任务

下一步推荐继续推进：

### 7.1 补完整 Java `NetworkIO.loadWorld` 后半段

目标：

1. 对照 Java `NetworkIO.loadWorld(InputStream)` 和 `SaveVersion`：
   - `readContentHeader`
   - `readContentPatches`
   - `readMap`
   - `readTeamBlocks`
   - `readMarkers`
   - `readCustomChunks`
2. 下一步最有价值的是补 `MapMarkers` 的 `JsonIO.writeBytes/readBytes` UBJSON 兼容读取，让 markers 与 custom chunks 能从 world stream tail 精确拆分。
3. 把 `NetworkWorldData.map_snapshot.to_tiles()` 的结果真正接入 `World::begin_map_load` / `resize` / `end_map_load` 生命周期。
4. 继续迁移 Player/Groups 应用逻辑：`Groups.clear()`、`player.reset()`、`player.read(...)`、`player.id = id`、`player.add()`。

优先查看 Rust 文件：

- `core/src/mindustry/core/net_server.rs`
- `core/src/mindustry/core/net_client.rs`
- `core/src/mindustry/core/world.rs`
- `core/src/mindustry/core/game_state.rs`
- `core/src/mindustry/core/content_loader.rs`
- `core/src/mindustry/io/save.rs`
- `core/src/mindustry/io/versions/mod.rs`
- `core/src/mindustry/net/*`
- `server/src/lib.rs`
- `desktop/src/*`

优先查看 Java 参考文件/关键词：

- `D:/MDT/mindustry-upstream-v157.4`
- `NetServer`
- `NetClient`
- `SendWorldData`
- `WorldStream`
- `ConnectPacket`
- `StreamBegin`
- `StreamChunk`
- `SaveIO`
- `Maps`

### 7.2 建议派发的子代理任务

建议先并行派两个 `explorer`：

#### Explorer A：Rust 当前 world/network 能力扫描

任务描述：

```text
只读探索任务：工作仓库为 D:/MDT/rust-mindustry，参考仓库为 D:/MDT/mindustry-upstream-v157.4。
请扫描 Rust 当前实现里与服务端向客户端发送 world data / map / save / connect 后初始化相关的路径，
重点文件 core/src/mindustry/core/net_server.rs、core/src/mindustry/core/net_client.rs、core/src/mindustry/core/world.rs、
content_loader.rs、net 包与 packet 定义。不要修改文件。
输出：1) 已有能力；2) 缺失点；3) 最小可落地闭环建议；4) 涉及文件和函数名；5) 建议测试点。
```

#### Explorer B：Java 原版 world stream 调用链扫描

任务描述：

```text
只读探索任务：参考仓库 D:/MDT/mindustry-upstream-v157.4。
请定位 Java 原版服务端在客户端连接后发送 world/map/save/world stream 的流程，
重点搜索 SendWorldData、WorldStream、ConnectPacket、StreamBegin、StreamChunk、NetServer、NetClient、SaveIO、Maps 等相关类/方法。
不要修改文件。
输出：1) Java 调用链；2) 关键 packet/字段；3) 最小互通所需 payload/顺序；4) 对 Rust 迁移的实现建议。
```

然后主线程根据结果决定是否派 `worker` 做局部实现。

如果实现点很复杂、涉及协议兼容、存档序列化或跨多个 crate，应使用 `ultra_worker`。

---

## 8. 验证策略

每个小闭环完成后，至少执行：

```powershell
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' fmt
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-core -- --test-threads=1
```

如果修改服务端入口：

```powershell
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-server -- --test-threads=1
```

如果修改桌面客户端入口：

```powershell
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-desktop -- --test-threads=1
```

在提交前尽量执行 workspace 测试：

```powershell
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test --workspace -- --skip mindustry::net::arc_net_provider::tests::* --test-threads=1
```

如果某些测试因已知网络/环境原因失败，要在最终回复中明确说明：

1. 执行了什么命令；
2. 哪个测试失败；
3. 失败原因判断；
4. 是否与本次修改相关。

---

## 9. 提交与推送规则

每完成一个明确文件级重构、迁移闭环或可验证行为：

```powershell
git -C 'D:/MDT/rust-mindustry' status --short
git -C 'D:/MDT/rust-mindustry' add <changed-files>
git -C 'D:/MDT/rust-mindustry' commit -m '<中文提交标题>'
git -C 'D:/MDT/rust-mindustry' push origin main
```

提交标题示例：

- `接入服务端世界数据下发`
- `补齐客户端世界流接收状态`
- `迁移世界加载最小闭环`
- `补齐连接后地图同步测试`

禁止：

- 推送到 `master`
- 使用英文提交标题
- 把未验证的大批改动一次性提交
- 修改废案目录 `D:/MDT/mindustry-rust`

---

## 10. 对后续 AI 的执行建议

接手后建议按以下顺序继续：

1. 读取本文件，确认最终目标和工作风格。
2. 执行：

   ```powershell
   git -C 'D:/MDT/rust-mindustry' status --short
   git -C 'D:/MDT/rust-mindustry' branch --show-current
   git -C 'D:/MDT/rust-mindustry' log -5 --oneline
   ```

3. 如果本文档未提交，先不要急于推送；可以在完成下一次实际迁移闭环时一起提交，或单独提交：

   ```powershell
   git -C 'D:/MDT/rust-mindustry' add 'AI_HANDOFF.md'
   git -C 'D:/MDT/rust-mindustry' commit -m '补充迁移交接文档'
   git -C 'D:/MDT/rust-mindustry' push origin main
   ```

4. 派发 `explorer` 对 Java `JsonIO.writeBytes/readBytes`、`UBJsonWriter/UBJsonReader` 和 `MapMarkers.read/write` 做只读扫描。
5. 主线程同时查看 Rust 当前 `io/save.rs`、`io/versions/mod.rs`、`game/map_markers.rs`、`core/world.rs`。
6. 用测试驱动实现 marker UBJSON 跳读/解析，先能精确定位 custom chunks 边界，再逐步还原 `MapMarkers`。
7. `fmt` + 相关 crate 测试。
8. 中文提交并推送到 `main`。
9. 继续下一个 Java 文件/模块迁移。

---

## 11. 重要提醒

1. 这是长期迁移任务，不可能一次回复内彻底完成。
2. 不要承诺“已经能完整游玩”，除非真实运行验证过。
3. 用户希望最终得到能游玩的 Rust 版 MDT；所有中间工作都要服务这个目标。
4. 迁移时要优先对照本地 Java 参考仓库，而不是凭记忆重写。
5. 如果遇到网络、GitHub、Cargo 依赖等问题，要先给出可复现命令和失败信息，再说明替代方案。
6. 当前最有价值的下一步不是继续铺空模块，而是打通连接后 world/map/save 数据流，让客户端进入世界成为可能。

---

## 12. 最新闭环记录：普通 item MassDriver

- 参考：`D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/world/blocks/distribution/MassDriver.java`
- Rust 主改动：`D:/MDT/rust-mindustry/core/src/mindustry/core/game_runtime.rs`
- 已接入：`item_mass_driver_waiting_shooters` runtime-only sidecar，按 Java `waitingShooters`/`shooterValid()` 思路清理队列；`advance_owned_item_mass_drivers_ticks(...)` 现在需要目标处于 accepting、源/目标旋转角进入 2° 误差并且 reload 就绪才发射。
- 已接入：`configure_owned_item_mass_driver(...)`、`GameRuntimeItemMassDriverConfig`、`GameRuntimeItemMassDriverConfigureResult`，对齐 Java `config(Point2.class)` / `config(Integer.class)` 的 relative/packed link 与清配置路径；配置变化会清理旧 waiting shooter 残留。
- 已接入：`GameRuntimeItemMassDriverInFlight` / `item_mass_driver_in_flight` runtime-only sidecar，发射时只扣源端物品并按 `mass_driver_time_to_arrive(distance, bulletSpeed, bulletLifetime)` 入队，到达 tick 才按 Java `handlePayload(...)` 的 `itemCapacity * 2` 上限写入目标、初始化目标 reload、清理 waiting shooter。
- 已接入：`expire_ticks/target_lost`，目标 tile/block/team 在飞行中失效时不会把 items 写入新目标或已移除目标，而是保留到 `bulletLifetime` 结束后清理，贴近 Java `data.to.dead()` 继续飞行到 despawn 的分支。
- 已接入：`GameRuntimeItemMassDriverDespawnEvent` / `item_mass_driver_despawn_events`，target-lost shot 过期时会用 `MassDriverBolt::despawn_drop_plans(...)` 与 `dynamic_explosion_plan(...)` 生成 runtime 可观测掉落/爆炸计划，并在 frame report 中累计 `despawned_shots/dropped_items/explosion_events`。
- 已验证：`cargo test -p mindustry-core mass_driver --lib` 通过 15/15，`cargo test game_runtime_payload_unloader --lib` 通过 13/13，`cargo check -p mindustry-core` 通过（仅既有 unused warning）。
- 仍未完成：真实 `MassDriverBolt` bullet entity、偏航相交、Java 随机掉落、真实 `Fx.dropItem` / `Damage.dynamicExplosion(...)` world effect、effects/sound/shake。
- 注意：`MassDriverState { link, rotation, state }` 是 Java-compatible 存档尾字段；`reloadCounter`、`waitingShooters` 与 `item_mass_driver_in_flight` 都是 runtime-only sidecar，不要写入 building payload。

---

## 13. 最新闭环记录：StorageBlock linkedCore / itemTaken

- 参考：`D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/world/blocks/storage/StorageBlock.java`、`CoreBlock.java`、`Unloader.java`、`world/blocks/distribution/DirectionalUnloader.java`。
- Rust 主改动：`D:/MDT/rust-mindustry/core/src/mindustry/core/game_runtime.rs`。
- 已接入：runtime-only `storage_linked_cores`，通过当前 building proximity 重建 Java `StorageBuild.linkedCore` 的最小关系；只使用 `D:/MDT/mindustry-upstream-v157.4` 作为参考，禁止误用废案目录。
- 已接入：普通 `Unloader` 从 linked storage 卸货时读取/移除 linked core 的 item module，保持 `unloader.allowCoreUnload=true` 的 Java 默认语义；`DirectionalUnloader` 遇到 core 或 linked storage 时按 `allowCoreUnload=false` 默认拒绝。
- 已接入：`GameRuntimeItemTakenEvent`，DirectionalUnloader 成功搬运后记录 Java `back.itemTaken(item)` hook；linked storage 的事件目标转发到 core tile。
- 已接入：core 移除时的 linked storage 拆分；`remove_building_at_index(...)` 会按 Java `CoreBuild.onRemoved()` 的容量比例公式把 core items 拆回仍存在的 linked storage，并清掉失效 link。
- 已接入：linked storage 刷新时同步 core runtime `storage_capacity`，core/linked storage accept 与普通 unloader load factor 会使用该容量；core 满自身 `itemCapacity` 时仍可通过 linked storage 增加容量。
- 已接入：同队多个 core 的 canonical item owner 近似；非 owner core / linked storage 的现有 items 会在 refresh 时合并到该队第一个 core，后续向任意同队 core 投递都会写入 canonical core，避免各 core 像独立模块一样分裂库存。
- 已接入：core 建筑生命周期与 `state.teams` registry；`add_building(...)` 会按 `BlockFlag::Core` 注册 `CoreInfo`，`remove_building_at_index(...)` / `clear_buildings(...)` 会注销，网络地图重载清理也不会残留旧 core。
- 已接入：core 升级最小 runtime 闭环；`can_place_owned_core_on(...)` 与 `upgrade_owned_core(...)` 会检查更大 core、footprint/core-zone、资源条件，复制旧 core items 并扣除新 core requirements，然后原地替换、刷新 world refs/team registry/storage capacity。
- 已接入：core-zone 普通新建放置最小 runtime 闭环；`place_owned_core(...)` 会在 footprint floor 全部允许 core placement 且不包含 core 时直接创建 core，并接入 world refs/team registry/storage capacity。
- 已接入：core handleItem / linked storage removeStack / directional itemTaken 的 campaign core item delta 最小副作用；默认队伍写 `GameStats.core_item_count`，campaign sector 镜像执行 `SectorInfo.handle_core_item(+/-1)`。
- 已接入：`coreIncinerates` 与 `incinerateNonBuildable` 的最小焚毁分支；core 或 linked storage owner 满仓时在规则允许下接收但不增加库存，`incinerateNonBuildable && !item.buildable` 时只扣来源、不写入 core items，`CoreBuildState.no_effect` 普通入库为 true、焚毁为 false；非 buildable 焚毁计入 stats 但不触发 campaign `handle_core_item(+1)`，满仓焚毁仍触发 campaign delta。
- 已接入：`GameRuntimeNetworkContext` 对齐 Java `CoreBuild.handleItem()` 的 `net.server() || !net.active()` 条件；默认离线仍按权威端写 core items/campaign，active client 只增加 stats，不直接写 core items，也不重复写 campaign delta；该 context 已接入 `ServerLauncher::new(...)`、desktop world data 应用与 world data 清空路径。
- 已验证：`cargo test -p mindustry-core core_building_lifecycle --lib` 通过 1/1；`cargo test -p mindustry-core clear_buildings --lib` 通过 1/1；该闭环确保 `Teams.cores/closest_core` 不再与 runtime core 建筑脱节。
- 已验证：`cargo test -p mindustry-core core_upgrade --lib` 通过 1/1；`core-shard -> core-foundation` 保留 `old - requirements` items 并更新 registry/capacity。
- 已验证：`cargo test -p mindustry-core places_core --lib` 通过 2/2；`core-zone` footprint 上直接放置 `core-shard` 并注册队伍 core。
- 已验证：`cargo test -p mindustry-core core_handle_item --lib` 通过 2/2；`cargo test -p mindustry-core core_incinerates --lib` 通过 2/2；`cargo test -p mindustry-core campaign_core_delta --lib` 通过 1/1；`cargo test -p mindustry-core canonical_item_owner --lib` 通过 1/1；`cargo test -p mindustry-core linked_storage --lib` 通过 5/5；`cargo test -p mindustry-core directional_unloader --lib` 通过 5/5；`cargo test -p mindustry-core item_unloader --lib` 通过 3/3；`cargo test -p mindustry-core game_runtime_payload_unloader --lib` 通过 13/13；`cargo check -p mindustry-core` 通过（仅既有 unused warning）；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。
- 仍未完成：Java 共享 `ItemModule` 引用的真正内存级等价、真实 `StorageBlock.incinerateEffect(...)` 视觉效果、更多 owned runtime tick 纳入 server update 主循环、非 core-zone construction flow 与升级 FX/Event、真实 player spawn、完整 UI/renderer 行为。

---

## 14. 最新闭环记录：服务端 owned runtime 主循环聚合

- 参考目标：把已迁移的 owned runtime block tick 接入真实 `server::ServerLauncher::update(...)`，避免它们只停留在单元测试或独立 helper 中；同时不能用多个 public `advance_owned_*` 串行调用造成 `GameState::advance_game_update_frame(...)` 被重复推进。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `core/src/mindustry/core/mod.rs`
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 已接入：`GameRuntimeOwnedItemTransportFrameReport` / `GameRuntimeOwnedFrameReport`，统一汇总 item transport 与 effect runtime 的 frame 结果。
- 已接入：`advance_owned_item_transport_blocks(...)` 和私有 `advance_owned_item_transport_blocks_ticks(...)`。public 入口仍可单独推进 item transport；私有 ticks 入口供更高层 aggregate 在同一 frame 内复用。
- 已接入：`advance_owned_runtime_blocks(...)`，单次调用只推进一次 `state.advance_game_update_frame(delta_seconds)`，随后刷新 update permission / linked storage / building timing，再运行 item transport ticks 与 effect building batch。
- 已接入：`ServerLauncher::update(...)` 现在调用 `update_runtime_owned_blocks(1.0 / 60.0)`，并缓存：
  - `last_runtime_item_transport_report`
  - `last_runtime_effect_report`
- 已新增测试：`server_update_drives_owned_item_transport_from_launcher_runtime`。该测试构造服务端 runtime 内的 `router -> item-void`，调用 `launcher.update()` 后断言 router 的 copper 被搬走、report 中 `router_forwarded_items == 1`，且 `runtime.state.update_id == 1`，用于锁定 server update 接入和单 frame 推进。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_item_transport_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_effect_building_from_launcher_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_payload_unloader --lib`
  - `cargo test -p mindustry-core game_runtime_item_router --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 当前重要限制：payload source/constructor/conveyor/loader/void 等 payload runtime 还没有纳入 `advance_owned_runtime_blocks(...)` / `ServerLauncher::update(...)` 的统一 single-frame 聚合；后续继续接入时必须复用私有 ticks/内部 frame 输入，禁止简单串多个 public advance 入口导致 update_id/timing 翻倍。
- 子代理提示：本轮尝试拉起 `explorer` 做下一闭环只读扫描时遇到 agent thread limit；如果后续线程可用，优先派 `explorer` 扫描 `game_runtime.rs` 里剩余 `advance_owned_*` public 入口与 `server/src/lib.rs` 的主循环缺口。

---

## 15. 最新闭环记录：服务端 PayloadVoid 主循环接入

- 目标：payload 族不能只停留在 `advance_owned_payload_*` 单测入口里；本闭环先把最小终端 `PayloadVoid` 接入服务端 `ServerLauncher::update()` 的 single-frame aggregate。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `core/src/mindustry/core/mod.rs`
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 已接入：`GameRuntimeOwnedPayloadFrameReport`，当前包含 `void: GameRuntimePayloadVoidFrameReport`；`GameRuntimeOwnedFrameReport` 新增 `payload` 字段。
- 已接入：`advance_owned_payload_voids(...)` 拆出 `advance_owned_payload_voids_ticks(...)`。public wrapper 继续独立推进 frame/timing；`advance_owned_runtime_blocks(...)` 在同一帧已推进过 `advance_game_update_frame(...)` 与 building timing 后调用 ticks 入口，避免重复 tick。
- 已接入：`ServerLauncher` 新增 `last_runtime_payload_report`，`update()` 会和 item/effect 一样缓存 payload batch。
- 新增测试：`server_update_drives_owned_payload_void_from_launcher_runtime`。该测试在服务端 runtime 中放置 `payload-void`，手动塞入 `BuildPayload(router)`，调用 `launcher.update()` 后断言：
  - `last_runtime_payload_report.unwrap().void.incinerated_payloads == 1`
  - payload void sidecar 仍存在但 payload 被清空
  - `runtime.state.update_id == 1`
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
- 仍未完成：`advance_owned_payload_sources/conveyors/constructors/loaders/deconstructors/payload_mass_drivers` 还没有进入 `advance_owned_runtime_blocks(...)`。下一步建议按子代理 Russell 的只读结论，继续拆 ticks 并接入 payload source + conveyor + constructor，形成服务端 payload 生成/搬运/消纳闭环；不要把多个 public advance 直接串起来。

---

## 16. 最新闭环记录：服务端 PayloadSource 主循环接入

- 目标：继续把 payload 族从单测入口并入服务端 `advance_owned_runtime_blocks(...)`，本闭环接入 `PayloadSource`，使服务端 update 能生成 block/unit payload 的最小状态。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 已接入：`GameRuntimeOwnedPayloadFrameReport` 新增 `source: GameRuntimePayloadSourceFrameReport`；`advance_owned_runtime_blocks(...)` 会在同一 frame 内先调用 `advance_owned_payload_sources_ticks(...)`，再调用 `advance_owned_payload_voids_ticks(...)`。
- 已接入：`advance_owned_payload_sources(...)` 拆出 `advance_owned_payload_sources_ticks(...)`。public wrapper 仍可独立推进 frame 和 timing；server aggregate 复用 ticks，保持 `update_id` 单帧只加 1。
- 新增测试：`server_update_drives_owned_payload_source_from_launcher_runtime`。该测试在服务端 runtime 中放置 `payload-source`，配置生成 `router`，调用 `launcher.update()` 后断言：
  - `last_runtime_payload_report.unwrap().source.spawned_block_payloads == 1`
  - source sidecar 中出现 `PayloadRef::Block(router)`
  - `runtime.state.update_id == 1`
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
- 仍未完成：`PayloadConveyor/Router/Constructor/Loader/Deconstructor/PayloadMassDriver` 还未接入 server aggregate。最建议下一步接入 `PayloadConveyor`，随后新增 server 测试覆盖 source 生成/移动到 conveyor/void 的实际链路。

---

## 17. 最新闭环记录：服务端 PayloadConveyor/Router 主循环接入

- 目标：继续把 payload 搬运链路接入服务端主循环；本闭环接入 `PayloadConveyor` 与同一 public 入口覆盖的 `PayloadRouter` 最小 ticks。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 已接入：`GameRuntimeOwnedPayloadFrameReport` 新增 `conveyor: GameRuntimePayloadConveyorFrameReport`；`advance_owned_runtime_blocks(...)` 的 payload 顺序现在是 source → conveyor/router → void。
- 已接入：`advance_owned_payload_conveyors(...)` 拆出 `advance_owned_payload_conveyors_ticks(content, frame_delta, tick)`。public wrapper 保留独立 frame/timing；server aggregate 复用 ticks，避免重复推进 `advance_game_update_frame(...)`。
- 新增测试：`server_update_drives_owned_payload_conveyor_from_launcher_runtime`。该测试在服务端 runtime 中放置携带 `BuildPayload(router)` 的 `payload-conveyor` 指向 `payload-void`，把 `state.tick` 设到 conveyor step 边界前一 tick，调用 `launcher.update()` 后断言：
  - `last_runtime_payload_report.unwrap().conveyor.attempted_moves == 1`
  - `conveyor.transferred_payloads == 1`
  - conveyor item 清空，void 收到 `PayloadRef::Block(router)`
  - `runtime.state.update_id == 1`
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
- 仍未完成：`PayloadConstructor/Loader/Deconstructor/PayloadMassDriver` 还没有进入 server aggregate。下一步建议接入 `PayloadConstructor`，让 server 主循环能从材料生产 build payload，再进入已经接入的 conveyor/void 链路。

---

## 18. 最新闭环记录：服务端 PayloadConstructor 主循环接入

- 目标：把 `PayloadConstructor` 生产逻辑接入服务端主循环，让 server runtime 能从配方和材料生成 `BuildPayload`，而不是只能在 core 单测里运行。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 已接入：`GameRuntimeOwnedPayloadFrameReport` 新增 `constructor: GameRuntimePayloadConstructorFrameReport`；`advance_owned_runtime_blocks(...)` 的 payload 顺序现在为 constructor → source → conveyor/router → void。
- 已接入：`advance_owned_payload_constructors_with_recipe_build_time(...)` 拆出 `advance_owned_payload_constructors_ticks(...)`。public wrapper 仍负责独立 frame/timing；server aggregate 复用 ticks，并使用 content registry 的 `BlockDef::effective_build_time(content.items())`。
- 新增测试：`server_update_drives_owned_payload_constructor_from_launcher_runtime`。该测试在服务端 runtime 中放置带 recipe/router 材料的 `constructor`，调用 `launcher.update()` 后断言：
  - `last_runtime_payload_report.unwrap().constructor.produced_payloads == 1`
  - `constructor.moved_out_payloads == 1`
  - constructor sidecar 内有 `PayloadRef::Block(router)` 且 `producer.has_payload`
  - `runtime.state.update_id == 1`
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
- 仍未完成：`PayloadLoader/Unloader`、`PayloadDeconstructor`、`PayloadMassDriver` 尚未接入 server aggregate。下一步建议接入 `PayloadLoader/Unloader`，因为它和普通 item/liquid/power 搬运及已接入的 item transport 有直接交叉。

---

## 19. 最新闭环记录：服务端 PayloadDeconstructor 主循环接入

- 目标：把 `PayloadDeconstructor` 的 move-in / start-deconstruction / progress 逻辑接入服务端主循环，避免只能通过 core 单测推进。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 已接入：`GameRuntimeOwnedPayloadFrameReport` 新增 `deconstructor: GameRuntimePayloadDeconstructorFrameReport`；`advance_owned_runtime_blocks(...)` 的 payload 顺序目前是 constructor → source → conveyor/router → deconstructor → void。
- 已接入：`advance_owned_payload_deconstructors(...)` 拆出 `advance_owned_payload_deconstructors_ticks(...)`。public wrapper 仍负责独立 frame/timing；server aggregate 复用 ticks。
- 新增测试：`server_update_drives_owned_payload_deconstructor_from_launcher_runtime`。该测试在服务端 runtime 中放置 `small-deconstructor`，预装 `BuildPayload(router)`，调用 `launcher.update()` 后断言：
  - `last_runtime_payload_report.unwrap().deconstructor.moved_in_payloads == 1`
  - `deconstructor.started_deconstructions == 1`
  - common payload 清空，`deconstructing` 中保留 `PayloadRef::Block(router)`
  - `runtime.state.update_id == 1`
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
- 仍未完成：`PayloadLoader/Unloader`、`PayloadMassDriver` 尚未接入 server aggregate。下一步建议优先接入 `PayloadLoader/Unloader`，但要注意它当前内部会调用 item transport ticks；接入 aggregate 时必须避免 item transport 被推进两次。

---

## 20. 最新闭环记录：服务端 PayloadLoader/Unloader 主循环接入

- 目标：把 `PayloadLoader/PayloadUnloader` 的 move-in/move-out、payload 内 items/liquids/power 装卸以及 unloader dump 路径接入服务端主循环，同时避免它内部的 item transport helper 与全局 item transport aggregate 重复推进同一帧。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 已接入：`GameRuntimeOwnedPayloadFrameReport` 新增 `loader: GameRuntimePayloadLoaderFrameReport`；`advance_owned_runtime_blocks(...)` 的 payload 顺序目前是 constructor → source → conveyor/router → loader/unloader → deconstructor → void。
- 已接入：`advance_owned_payload_loaders(...)` 拆出 `advance_owned_payload_loaders_ticks(content, frame_delta, run_item_transport)`。public wrapper 传 `true` 保持旧语义；server aggregate 传 `false`，因为 aggregate 开头已经统一执行过 `advance_owned_item_transport_blocks_ticks(...)`。
- 新增测试：`server_update_drives_owned_payload_loader_from_launcher_runtime`。该测试在服务端 runtime 中放置 `payload-loader`，预装 `BuildPayload(container)` 与 5 个 copper，调用 `launcher.update()` 后断言：
  - `last_runtime_payload_report.unwrap().loader.loader_candidates == 1`
  - `loader.updated_loaders == 1`
  - `loader.moved_in_payloads == 1`
  - `loader.loaded_items == 5`
  - loader building 中 copper 清零
  - `runtime.state.update_id == 1`
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_loader_from_launcher_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_payload_loader --lib`
  - `cargo test -p mindustry-core payload_unloader --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_item_transport_from_launcher_runtime --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 仍未完成：`PayloadMassDriver` 尚未接入 server aggregate。下一步建议按 explorer Beauvoir 的结论拆 `advance_owned_payload_mass_drivers(...)` 的 tick-only helper，把 `GameRuntimePayloadMassDriverFrameReport` 加进 `GameRuntimeOwnedPayloadFrameReport`，并新增 server-level fired/received/update_id smoke test。

---

## 21. 最新闭环记录：服务端 PayloadMassDriver 主循环接入

- 目标：把 `PayloadMassDriver` 的双端 queue / charge / fire / receive 最小运行态接入服务端主循环，结束 payload family 在 server aggregate 中的最后一个主要缺口。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 已接入：`GameRuntimeOwnedPayloadFrameReport` 新增 `mass_driver: GameRuntimePayloadMassDriverFrameReport`；`advance_owned_runtime_blocks(...)` 的 payload 顺序目前是 constructor → source → conveyor/router → loader/unloader → mass-driver → deconstructor → void。
- 已接入：`advance_owned_payload_mass_drivers(...)` 拆出 `advance_owned_payload_mass_drivers_ticks(content, frame_delta)`。public wrapper 仍负责独立 frame/timing；server aggregate 复用 ticks，保证 `update_id` 每次 `launcher.update()` 只增加一次。
- 新增测试：`server_update_drives_owned_payload_mass_driver_from_launcher_runtime`。该测试在服务端 runtime 中构造 linked source/target mass driver，源端预装 `BuildPayload(router)` 且已 loaded/charged，目标端 accepting 并等待源端，调用 `launcher.update()` 后断言：
  - `last_runtime_payload_report.unwrap().mass_driver.mass_driver_candidates == 2`
  - `mass_driver.charged_shots == 1`
  - `mass_driver.fired_payloads == 1`
  - `mass_driver.received_payloads == 1`
  - 源端 payload 清空且回到 Idle
  - 目标端收到 payload，`last_other == source_tile`，`effect_delay_timer > 0`
  - `runtime.state.update_id == 1`
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_mass_driver_from_launcher_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_advances_owned_payload_mass_driver_queues_and_fires_payload --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_loader_from_launcher_runtime --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_deconstructor_from_launcher_runtime --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 当前状态：payload constructor/source/conveyor-router/loader-unloader/mass-driver/deconstructor/void 都已进入 `advance_owned_runtime_blocks(...)` 的 single-frame aggregate。后续优先做跨多帧整体 smoke，证明这些节点在同一个 server `update()` 链里组成真实可游玩 runtime，而不是只各自有独立单测。

---

## 22. 最新闭环记录：服务端 payload aggregate 跨多帧整体 smoke

- 目标：回应“不要让模块独立存在”的总要求，新增一个 server-level 多帧 smoke，证明已迁移 payload 节点能在同一个 `ServerLauncher::update()` / `advance_owned_runtime_blocks(...)` 主循环中串成真实链路。
- Rust 主改动：
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`server_update_drives_owned_payload_constructor_conveyor_void_chain`。
- 测试链路：
  - `constructor` 预装 router 材料与 recipe；
  - constructor 前方放置空 `payload-conveyor`；
  - conveyor 前方放置空 `payload-void`；
  - 连续调用 `launcher.update()`，每帧断言 `runtime.state.update_id == frame`；
  - 累计 report，要求最终出现：
    - `constructor.produced_payloads == 1`
    - `constructor.transferred_payloads == 1`
    - `conveyor.transferred_payloads == 1`
    - `void.incinerated_payloads == 1`
  - 最终 void sidecar payload 为空，说明 payload 已真正走完 constructor→conveyor→void runtime 链。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_constructor_conveyor_void_chain --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_mass_driver_from_launcher_runtime --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 后续建议：继续补 `loader → deconstructor`、`source/router` 与 linked `payload-mass-driver` 的 server-level 多帧 smoke；随后把 payload 状态同步到 network snapshot 的更细联机测试，避免服务端运行态与客户端可见状态脱节。

---

## 23. 最新闭环记录：服务端 PayloadLoader → PayloadDeconstructor 跨多帧 smoke

- 目标：继续验证 payload 子模块接入真实整体 runtime，而不是单独 helper；本轮覆盖 loader 输出到 deconstructor 的跨多帧链路。
- Rust 主改动：
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`server_update_drives_owned_payload_loader_deconstructor_chain`。
- 测试链路：
  - `payload-loader` 预装 `BuildPayload(router)`，`PayloadLoaderState.exporting = true`；
  - loader 前方放置空 `small-deconstructor`；
  - 连续调用 `launcher.update()`，每帧断言 `runtime.state.update_id == frame`；
  - 累计 report，要求最终出现：
    - `loader.transferred_payloads == 1`
    - `deconstructor.moved_in_payloads == 1`
    - `deconstructor.started_deconstructions == 1`
  - 最终 deconstructor common payload 清空，`deconstructing` 中保留 router payload。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_loader_deconstructor_chain --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_constructor_conveyor_void_chain --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 后续建议：补 `PayloadSource/Router` 多帧选路 smoke、linked `PayloadMassDriver` 的多帧自然 charge/fire smoke，并继续把这些 runtime 状态与 world-data/network snapshot 测试绑定。

---

## 24. 最新闭环记录：服务端 PayloadSource → PayloadRouter → PayloadVoid 跨多帧 smoke

- 目标：补齐 source/router 的 server-level 多帧链路验证，让 sandbox payload source、payload router sort 选择、payload void 消纳都在同一个 server aggregate 中被证明能串联运行。
- Rust 主改动：
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`server_update_drives_owned_payload_source_router_void_chain`。
- 测试链路：
  - `payload-source` 配置为持续生成 `router` block payload；
  - source 前方放置 `payload-router`，router 配置同 block sort key；
  - router 记录方向前方放置 `payload-void`；
  - 连续调用 `launcher.update()`，每帧断言 `runtime.state.update_id == frame`；
  - 累计 report，要求至少出现一次：
    - `source.spawned_block_payloads >= 1`
    - `source.transferred_payloads >= 1`
    - `conveyor.transferred_payloads >= 1`
    - `void.incinerated_payloads >= 1`
  - 由于 payload-source 会持续生产，该 smoke 不要求最终所有中间 slot 清空，只锁定至少一次完整 source→router→void 流转。
- 已验证：
  - `cargo test -p mindustry-server server_update_drives_owned_payload_source_router_void_chain --lib`
  - `cargo test -p mindustry-server server_update_drives_owned_payload_constructor_conveyor_void_chain --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- Rawls the 2nd 只读结论：payload 低层 codec 与 runtime export/load roundtrip 已较完整，最明显缺口是 server runtime payload sidecar 经 `network_world_data_template()` / world stream / `read_world_data()` / 新 `GameRuntime::load_network_map_with_buildings(...)` 的端到端 smoke。下一闭环建议优先补 server world-data payload state roundtrip。

---

## 25. 最新闭环记录：服务端 world-data payload sidecar 端到端回读

- 目标：把 payload runtime sidecar 从 server 内部状态推进到 network world stream，再由新 runtime 回读，证明 payload 状态能成为客户端可见 world-data，而不是只存在于服务端单测。
- Rust 主改动：
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`server_world_data_roundtrips_payload_loader_state_through_runtime_loader`。
- 测试链路：
  - 构造带 `payload-loader` building 的 `ServerLauncher`；
  - 写入 `GameRuntimePayloadBlockState::Loader`，其中 common payload 是 `BuildPayload(container)`，`PayloadLoaderState.exporting = true`；
  - 通过 `Connect` + `ConnectPacket` 触发 pending world data；
  - 调用 `launcher.update()`，用 `CaptureProvider` 捕获真实 `WORLD_STREAM`；
  - `decode_captured_world_data(...)` / `read_world_data(...)` 得到 `NetworkWorldData`；
  - 从 `map_snapshot` 新建 `GameRuntime::default()` 并调用 `load_network_map_with_buildings(...)`；
  - 断言 building 恢复、payload loader sidecar 恢复、common payload block id 为 container、loader exporting 为 true。
- 已验证：
  - `cargo test -p mindustry-server server_world_data_roundtrips_payload_loader_state_through_runtime_loader --lib`
  - `cargo test -p mindustry-server server_world_data_exports_owned_building_chunks_for_runtime_loader --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 后续建议：把同类 world-data roundtrip 扩展到 `PayloadMassDriver`（link/reload/charge/loaded/charging）、`PayloadRouter`（sorted/recDir/matches）、`PayloadDeconstructor`（progress/deconstructing），再接 desktop/client `apply_network_world_data` smoke。

---

## 26. 最新闭环记录：服务端 world-data 多类 Payload sidecar 回读

- 目标：把 server world-data payload sidecar roundtrip 从单个 `payload-loader` 扩展到多类 payload building，降低客户端 world stream 丢运行态字段的风险。
- Rust 主改动：
  - `server/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`server_world_data_roundtrips_payload_router_mass_driver_and_deconstructor_states`。
- 测试链路：
  - 同一个 `ServerLauncher` runtime 中放置：
    - `payload-router`，带 conveyor item、sorted block key、`recDir=2`；
    - `payload-mass-driver`，带 `state=Shooting`、`turretRotation=45`、`reloadCounter=0.25`、`charge=0.5`、`loaded/charging=true`；
    - `small-deconstructor`，带 `progress=0.5`、`accum=[1,2]`、`deconstructing=BuildPayload(router)`；
  - 通过 connect handshake 触发 `WORLD_STREAM`；
  - `decode_captured_world_data(...)` → `NetworkWorldData.map_snapshot`；
  - 新 `GameRuntime::load_network_map_with_buildings(...)` 回读；
  - 断言三类 payload sidecar 关键字段全部恢复。
- 已验证：
  - `cargo test -p mindustry-server server_world_data_roundtrips_payload_router_mass_driver_and_deconstructor_states --lib`
  - `cargo test -p mindustry-server server_world_data_roundtrips_payload_loader_state_through_runtime_loader --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `rustfmt --check core/src/mindustry/core/game_runtime.rs core/src/mindustry/core/mod.rs server/src/lib.rs`
  - `git diff --check`
- 后续建议：下一闭环优先接 desktop/client `apply_network_world_data` smoke，证明 desktop launcher 收到 server world-data 后也能把 payload sidecar  materialize 到 runtime/game state，而不只是在 server 测试里手动回读。

---

## 27. 最新闭环记录：Desktop/client world-data payload sidecar materialize

- 目标：把 payload world-data 验证从 server 手动回读推进到 desktop/client 应用路径，证明客户端 launcher 收到 `NetworkWorldData.map_snapshot` 后会 materialize payload sidecar。
- Rust 主改动：
  - `desktop/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`desktop_launcher_materializes_payload_state_from_network_world_data`。
- 测试链路：
  - 构造含 `payload-loader` building 与 `GameRuntimePayloadBlockState::Loader` 的临时 runtime；
  - 用 `export_network_map_snapshot(&ContentLoader)` 生成 `NetworkWorldData.map_snapshot`；
  - 写入 desktop `NetClientState.last_loaded_world_data`；
  - `launcher.update()` 触发 `sync_loaded_world_data()` / `sync_runtime_state_from_world_data()`；
  - 断言 desktop runtime 进入 `GameRuntimeNetworkContext::client()`，`last_runtime_map_load_report` 成功，payload loader sidecar、common payload 与 `exporting=true` 恢复。
- 已验证：
  - `cargo test -p mindustry-desktop desktop_launcher_materializes_payload_state_from_network_world_data --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_materializes_network_map_buildings_into_runtime --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `rustfmt --check desktop/src/lib.rs`
  - `git diff --check`
- 后续建议：补真实 server/client ArcNetProvider 联机 world stream smoke，让 `ServerLauncher` 发送出的 payload world-data 被 `DesktopLauncher` 的 `NetClient` 实际接收并 materialize；随后扩展到 Java 客户端兼容验证。

---

## 28. 最新闭环记录：真实 ServerLauncher → DesktopLauncher world-stream payload smoke

- 目标：去掉手写 `NetClientState.last_loaded_world_data` 的假注入，验证真实 server/client 本地联机链路能把 payload sidecar 从服务端 runtime 传到 desktop runtime。
- Rust 主改动：
  - `core/src/mindustry/core/net_client.rs`
  - `tests/Cargo.toml`
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`real_server_desktop_world_stream_materializes_payload_sidecar`。
- 测试链路：
  - `ServerLauncher::new(...)` 创建真实服务端，runtime 中放置 `payload-loader` building；
  - 写入 `GameRuntimePayloadBlockState::Loader`，其中 common payload 是 `BuildPayload(container)`，`PayloadLoaderState.exporting = true`；
  - `server.init()` 打开真实 `ArcNetProvider` 本地 TCP/UDP 端口；
  - `mindustry_desktop::run --connect 127.0.0.1:port` 启动真实 desktop/client；
  - 循环 pump `desktop.update()` / `server.update()`，让客户端发送 `ConnectPacket`，服务端接受并 `flush_pending_world_data()`，客户端接收 `WORLD_STREAM` 后自动 `ConnectConfirmCallPacket`；
  - 断言 desktop `NetClientState.last_loaded_world_data` 存在、`connect_confirm_sent=true`、服务端 `world_streams_sent=1`、desktop runtime 进入 `GameRuntimeNetworkContext::client()`，并恢复 payload loader sidecar 与 `BuildPayload(container)`。
- 重要修正：
  - `ClientConnectConfig::default()` 现在给出非空 Java-like `uuid/usid`，否则真实 wire path 会因为空 UUID 生成的 `ConnectPacket` 无法被服务端 reader/validation 接受；capture-provider 单测不会暴露这个问题。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_world_stream_materializes_payload_sidecar --lib`
  - `cargo test -p mindustry-desktop desktop_run_connect_arg_starts_real_client_handshake --lib`
  - `cargo test -p mindustry-core update_sends_configured_connect_packet_once_after_connect_event --lib`
  - `cargo check -p mindustry-tests`
- 后续建议：把真实联机 world-stream smoke 扩展到 `PayloadRouter/PayloadMassDriver/PayloadDeconstructor`，随后推进 state snapshot/实时增量同步与 Java 客户端/服务端互通 smoke。

---

## 29. 最新闭环记录：真实联机 world-stream 多类 Payload sidecar materialize

- 目标：把真实 server/client world stream smoke 从单个 `payload-loader` 扩展到多类 payload sidecar，避免只验证单点字段。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`real_server_desktop_world_stream_materializes_multiple_payload_sidecars`。
- 测试链路：
  - 在真实 `ServerLauncher` runtime 中同时放置：
    - `payload-router`，带 conveyor item、sorted block key、`recDir=2`、`matches=true`；
    - `payload-mass-driver`，带 `state=Shooting`、`turretRotation=45`、`reloadCounter=0.25`、`charge=0.5`、`loaded/charging=true`；
    - `small-deconstructor`，带 `progress=0.5`、`accum=[1,2]`、`deconstructing=BuildPayload(router)`；
  - `server.init()` 打开真实本地 TCP/UDP；
  - `mindustry_desktop::run --connect 127.0.0.1:port` 建立真实客户端；
  - 复用 `pump_real_server_desktop_until(...)` 循环推进 `desktop.update()` / `server.update()`；
  - 断言 desktop `NetClient` 已收到并确认 world stream，服务端已收到 confirm；
  - 断言 desktop runtime 进入 `GameRuntimeNetworkContext::client()`，且三类 payload sidecar 关键字段全部恢复。
- 结构调整：
  - `tests/src/lib.rs` 提取 `free_local_port()` 与 `pump_real_server_desktop_until(...)` 测试 helper；
  - 本地端口探测尝试次数调到 128，降低 Windows 环境端口占用导致的偶发失败。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_world_stream_materializes_multiple_payload_sidecars --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-tests`
  - `rustfmt --check tests/src/lib.rs`
  - `git diff --check`
- 后续建议：继续推进 state snapshot/实时增量同步、Java 客户端/服务端互通 smoke，以及 renderer/UI/输入控制闭环，确保最终是完整可游玩的 Rust MDT 而不是 isolated modules。

---

## 30. 最新闭环记录：真实联机 StateSnapshot 增量同步 smoke

- 目标：在真实 world stream join 之后，验证服务端运行态增量包 `StateSnapshotCallPacket` 能通过真实 `ArcNetProvider` 到达 desktop/client，并应用到 `game_state/runtime`。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`real_server_desktop_state_snapshot_updates_runtime_after_world_stream`。
- 测试链路：
  - 启动真实 `ServerLauncher` 与 `DesktopLauncher --connect 127.0.0.1:port`；
  - 复用 `pump_real_server_desktop_until(...)` 等待 world stream 完成、客户端 `connect_confirm_sent=true`、服务端收到 confirm；
  - 从 `NetServerState.last_connect_confirm_connection_id` 取真实 connection id；
  - 调用 `server.net_server.send_state_snapshot(connection_id, snapshot)` 走真实 UDP/unreliable 发送；
  - 循环 `desktop.update()` / `server.update()` 等待客户端收到；
  - 断言 `NetClientState.last_state_snapshot`、`last_state_snapshot_mirror`、`state_snapshot_packets_seen` 更新；
  - 断言 `DesktopLauncher::sync_state_snapshot()` 已把 `waveTime/wave/enemies/paused/gameOver/tps/rand/timeData` 应用到 `game_state` 与 `runtime.state`。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_state_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-tests`
  - `rustfmt --check tests/src/lib.rs`
  - `git diff --check`
- 后续建议：继续补 `EntitySnapshotCallPacket`、`BlockSnapshotCallPacket`、`HiddenSnapshotCallPacket` 的真实联机增量同步，再推进客户端输入/构建请求回传和 Java↔Rust 互通 smoke。

---

## 31. 最新闭环记录：真实联机 Entity/Hidden snapshot 增量同步 smoke

- 目标：继续扩展真实联机增量同步，验证 `NetServer::send_entity_sync_snapshot(...)` 发出的 state/entity/hidden snapshot 能经真实 `ArcNetProvider` 到达 desktop `NetClient`。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream`。
- 测试链路：
  - 真实 `ServerLauncher` / `DesktopLauncher --connect` 完成 world stream join；
  - 使用真实 `last_connect_confirm_connection_id`；
  - 调用 `send_entity_sync_snapshot(connection_id, state_snapshot, [entity1, entity2], Some(hidden))`；
  - 循环推进 `desktop.update()` / `server.update()`；
  - 断言服务端记录 state/entity/hidden 三类包的发送统计；
  - 断言客户端 `NetClientState` 记录 `last_state_snapshot`、`entity_snapshot_packets_seen=2`、`last_entity_snapshot=entity2`、`hidden_snapshot_packets_seen=1`、`last_hidden_snapshot=hidden`；
  - 断言 desktop state snapshot 的 wave/TPS 已同步到 `game_state/runtime.state`。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-tests`
  - `rustfmt --check tests/src/lib.rs`
  - `git diff --check`
- 后续建议：继续补 `BlockSnapshotCallPacket` 真实联机 smoke，并把 entity/block snapshot bytes 进一步 materialize 到可查询的 world/entity mirror，而不是只停在 `NetClientState` 记录层。

---

## 32. 最新闭环记录：真实联机 BlockSnapshot 增量同步 smoke

- 目标：补齐 world stream join 后的 `BlockSnapshotCallPacket` 真实联机接收路径，并让服务端有对称的发送/记录 API。
- Rust 主改动：
  - `core/src/mindustry/core/net_server.rs`
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增 API：`NetServer::send_block_snapshot(connection_id, BlockSnapshotCallPacket)`。
  - 走 `PacketKind::BlockSnapshotCallPacket`；
  - unreliable 发送；
  - 更新 `NetServerState.last_block_snapshot_connection_id`、`last_block_snapshot`、`last_block_snapshot_sent_at`、`block_snapshot_packets_sent`、`last_block_snapshot_error`；
  - 记录 connection sent metadata。
- 新增测试：`real_server_desktop_block_snapshot_updates_net_client_after_world_stream`。
- 测试链路：
  - 真实 server/desktop 完成 world stream join；
  - 取真实 `last_connect_confirm_connection_id`；
  - 服务端调用 `send_block_snapshot(...)`；
  - 循环 `desktop.update()` / `server.update()`；
  - 断言服务端发送记录；
  - 断言客户端 `NetClientState.block_snapshot_packets_seen=1`、`last_block_snapshot`、`last_block_snapshot_at`、`last_server_snapshot_at`。
- 已验证：
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-server -p mindustry-tests`
  - `rustfmt --check core/src/mindustry/core/net_server.rs tests/src/lib.rs`
  - `git diff --check`
- 后续建议：下一步把 block/entity snapshot bytes materialize 到客户端 world/entity mirror；然后推进客户端输入、构建请求、单位控制回传与 Java↔Rust 互通 smoke。

---

## 33. 最新闭环记录：客户端 snapshot bytes 轻量镜像

- 目标：把客户端收到的 block/entity/hidden snapshot 从“只保存原始 packet”推进到 Java-like header 可查询镜像，为后续真正写入 world/entity runtime 打底。
- Rust 主改动：
  - `core/src/mindustry/core/net_client.rs`
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增/扩展状态：
  - `ClientBlockSnapshotRecordMirror { tile_pos, block_id, sync_bytes }`
  - `ClientBlockSnapshotMirror { amount, data, records, parse_error }`
  - `ClientEntitySnapshotRecordMirror { entity_id, type_id, sync_bytes }`
  - `ClientEntitySnapshotMirror { amount, data, records, parse_error }`
  - `ClientHiddenSnapshotMirror { ids }`
  - `NetClientState.last_block_snapshot_mirror`
  - `NetClientState.entity_snapshot_mirrors`
  - `NetClientState.last_hidden_snapshot_mirror`
- 解析约束：
  - Java block snapshot 子记录 header 是 `int pos` + `short blockId`，后续为 `build.writeSync(...)`；
  - Java entity snapshot 子记录 header 是 `int entityId` + `byte typeId`，后续为 `entity.writeSync(...)`；
  - 因子记录没有独立长度，本闭环只完整支持单记录 opaque bytes 与多记录 header-only；多记录且含 opaque sync bytes 时写 `parse_error`，不要误判为已完成字段级同步。
- 已扩展验证：
  - `update_records_block_snapshot_metadata_for_later_world_application`
  - `update_records_server_snapshots_when_client_loaded`
  - `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream`
  - `real_server_desktop_block_snapshot_updates_net_client_after_world_stream`
- 已跑：
  - `rustfmt --check core/src/mindustry/core/net_client.rs tests/src/lib.rs`
  - `git diff --check`
  - `cargo test -p mindustry-core update_records_server_snapshots_when_client_loaded --lib`
  - `cargo test -p mindustry-core update_records_block_snapshot_metadata_for_later_world_application --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-tests`
- 下一步建议：
  1. 把 `ClientBlockSnapshotMirror` 按 `tile_pos/block_id` 应用到 client-side world/block mirror 或真实 runtime building。
  2. 把 `ClientEntitySnapshotMirror` 按 `entity_id/type_id` 应用到 entity mirror collection。
  3. 按 Java 参考逐类补 `readSync/writeSync` 字段解析，而不是停留在 opaque bytes。
  4. 继续推进客户端输入、构建请求、单位控制回传与 Java↔Rust 联机 smoke。

---

## 34. 最新闭环记录：客户端 snapshot mirror 接入 GameRuntime sidecar

- 目标：把 `NetClient` 已解析出的 block/entity/hidden snapshot mirror 接入真实 `DesktopLauncher -> GameRuntime` 客户端 runtime 链路，避免停留在网络记录层。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `desktop/src/lib.rs`
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增 runtime sidecar：
  - `GameRuntimeClientBlockSnapshotRecord { tile_pos, building_tile_pos, block_id, sync_bytes }`
  - `GameRuntimeClientEntitySnapshotRecord { entity_id, type_id, sync_bytes, hidden }`
  - `GameRuntimeClientSnapshotApplyReport`
  - `GameRuntime.client_block_snapshot_records`
  - `GameRuntime.client_entity_snapshot_records`
  - `GameRuntime.client_hidden_entity_ids`
- 新增 runtime API：
  - `apply_client_block_snapshot_record(...)`
  - `apply_client_entity_snapshot_record(...)`
  - `apply_client_hidden_snapshot_ids(...)`
  - `note_client_block_snapshot_parse_error()`
  - `note_client_entity_snapshot_parse_error()`
- Desktop 接入：
  - `DesktopLauncher::update()` 现在调用 `sync_snapshot_mirrors()`；
  - 使用 cursor 避免重复应用 `entity_snapshot_mirrors`；
  - world data 变化时把 cursor 重置到当前 net state，避免旧 snapshot 套到新地图；
  - world data 清空时清理 cursor/report；
  - `sync_runtime_state_from_game_state()` 在 clone `game_state` 后重新 `sync_world_footprint_refs(...)`，修复 connect confirm 进入 Playing 时丢失 runtime world `BuildingRef` 的问题。
- 已验证真实联机：
  - `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream` 现在断言 entity mirror 进入 `desktop.runtime.client_entity_snapshot_records`，hidden ids 进入 `client_hidden_entity_ids`；
  - `real_server_desktop_block_snapshot_updates_net_client_after_world_stream` 现在先让 server world stream 携带真实 router building，再发送 matching `BlockSnapshotCallPacket`，并断言 snapshot 落入 `desktop.runtime.client_block_snapshot_records`。
- 已跑：
  - `cargo test -p mindustry-core game_runtime_applies_client --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 下一步建议：
  1. 选择一类简单 building（例如 router/conveyor/storage 中已有 typed state 的部分）实现 `sync_bytes -> typed readSync` 回放。
  2. 设计真实 client entity pool，逐步替代 `client_entity_snapshot_records` 的 raw sidecar。
  3. 继续推进客户端输入/构建/单位控制回传，保证 runtime 是整体联机闭环。

---

## 35. 最新闭环记录：BlockSnapshot 基础 `Building.readSync` 回放

- 目标：把上一闭环的 block snapshot runtime sidecar 再推进一步，至少回放 Java `Building.writeSync()` 的基础 `writeBase/readBase` 段，真实更新客户端 runtime building，而不是只保存 opaque bytes。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `desktop/src/lib.rs`
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为变化：
  - `GameRuntime::apply_client_block_snapshot_record(...)` 在 tile/building 存在且 block id 匹配后，会 clone 当前 building，使用 `BuildingComp::read_base(...)` 消费 `sync_bytes` 前缀；
  - 成功后替换 runtime building，并 `sync_world_footprint_refs(index)` 保持 world `BuildingRef` 同步；
  - 失败时不写入半解析 building，只增加 `block_base_read_errors`；
  - 原始 `sync_bytes` 仍保存到 `client_block_snapshot_records`，便于后续继续解析 child tail。
- 报表新增：
  - `block_base_records_applied`
  - `block_base_read_errors`
  - `block_remaining_sync_bytes`
- 测试更新：
  - core 单测用 `BuildingComp::write_base(...)` 构造真实 sync bytes，并断言 health/rotation 更新；
  - desktop 单测断言 snapshot mirror 进入 runtime 后实际更新 building；
  - 真实联机 block snapshot smoke 现在发送 matching router building 的真实 base sync bytes，并断言 desktop runtime building health/rotation 更新。
- 已跑：
  - `cargo test -p mindustry-core game_runtime_applies_client --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars --lib`
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 下一步建议：
  1. 给无 child tail 的简单 block 锁定 `block_remaining_sync_bytes == 0`。
  2. 按 block family 接入 child `read(...)` tail，例如 storage/distribution，再到 turret/payload。
  3. 实现 turret override `readSync(...)` 的 Java 特例：同步时保留 rotation/reload。

---

## 36. 最新闭环记录：Conveyor BlockSnapshot child tail 回放

- 目标：在基础 `Building.readBase` 之后，首个 block-specific tail 选择 Java `ConveyorBuild.write/read(version=1)` 接入 typed runtime state。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `desktop/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增/扩展 API：
  - `GameRuntime::apply_client_block_snapshot_record_with_content(&ContentLoader, ...)`
  - 内部复用原基础回放；当 tail 非空且 block 属于 distribution family 时，用 `client_block_snapshot_revision(...)` 选择 Java revision，再复用 `read_distribution_runtime_state_from_building_payload(...)` 写入 `distribution_runtime_states`。
- 报表新增：
  - `block_child_records_applied`
  - `block_child_read_errors`
- Desktop 接入：
  - `DesktopLauncher::sync_snapshot_mirrors()` 改为传入 `content_loader`，因此真实联机客户端可以从 block id 映射到 block family，再消费 child tail。
- 验证：
  - 新增/扩展 `game_runtime_applies_client_conveyor_snapshot_child_tail_with_content`，用 `write_base + write_conveyor_state` 构造真实 sync bytes，断言 health 与 conveyor item state 同步到 runtime。
  - 继续保留真实 server→desktop block snapshot smoke，确保基础回放链路未断。
- 已跑：
  - `cargo test -p mindustry-core game_runtime_applies_client --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars --lib`
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 下一步建议：
  1. 给 `Router` / `Junction` / `ItemBridge` / `Sorter` / `Unloader` 等 distribution dispatcher 分支补字段级单测和真实联机 smoke。
  2. 继续把 storage/payload/turret family 接入 child-tail dispatcher。
  3. 继续保持未知 family tail 只保留 remaining bytes，不误解析。

---

## 37. 最新闭环记录：真实联机 Conveyor BlockSnapshot child tail smoke

- 目标：把 conveyor child-tail 回放从 core 单测提升到真实 `ServerLauncher -> ArcNetProvider -> DesktopLauncher` 联机链路。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 测试变化：
  - `real_server_desktop_block_snapshot_updates_net_client_after_world_stream` 从 router base-only snapshot 改为 conveyor snapshot；
  - server world stream 先携带真实 conveyor building；
  - snapshot bytes 由 `BuildingComp::write_base(...) + write_conveyor_state(...)` 构造；
  - desktop 端断言：
    - `NetClient.last_block_snapshot_mirror` header 与 sync bytes 正确；
    - `GameRuntime.client_block_snapshot_records` 保存 raw bytes；
    - `GameRuntime.buildings()` 中 building health/rotation 被更新；
    - `GameRuntime.distribution_runtime_states` 中 materialize 出 `ConveyorState`，包含 copper item。
- 已跑：
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-tests`
  - `git diff --check`
- 下一步建议：
  1. 继续给更多 distribution child tail 做真实联机 smoke。
  2. 把 `apply_client_block_snapshot_child_tail(...)` 从 conveyor 专用扩展到可复用 dispatcher。
  3. 后续接 Java `TurretBuild.readSync` 时注意保留 rotation/reload 的 override 语义。

---

## 38. 最新闭环记录：Core BlockSnapshot child tail 回放

- 目标：在 block snapshot child-tail dispatcher 中接入 storage/core family 的首个 Java tail：`CoreBuild.commandPos`。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为变化：
  - `client_block_snapshot_revision(...)` 现在为 `StorageBlockKind::Core` 返回 revision `1`；
  - `apply_client_block_snapshot_child_tail(...)` 先尝试 distribution dispatcher，若 unsupported 再尝试 storage dispatcher；
  - core tail 通过既有 `read_storage_runtime_state_from_building_payload(...)` / `read_core_state(...)` 写入 `storage_runtime_states`。
- 验证：
  - `game_runtime_applies_client_core_snapshot_child_tail_with_content` 用 `write_base + write_core_state(command_pos)` 构造真实 sync bytes；
  - 断言 `GameRuntimeStorageBlockState::Core.command_pos` 被恢复。
- 已跑：
  - `cargo test -p mindustry-core game_runtime_applies_client --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 下一步建议：
  1. 继续把 payload/turret/logic family 接入 child-tail dispatcher。
  2. 对 storage/core 做真实联机 smoke，验证 commandPos 通过 `BlockSnapshotCallPacket` 到 desktop runtime。
  3. 后续不要把 core linked storage/shared item module 简化为单独普通 storage。

---

## 39. 最新闭环记录：Payload BlockSnapshot child tail 回放

- 固定工作路径再次强调：Rust 仓库是 `D:\MDT\rust-mindustry`，Java 参考是 `D:\MDT\mindustry-upstream-v157.4`，不要去 `D:\MDT\mindustry-rust`。
- 目标：把 client `BlockSnapshotCallPacket` 中 Java `Building.writeSync()` 的 payload block child-tail 回放到 `GameRuntime.payload_runtime_states`，避免 payload 状态只停留在 raw bytes。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为变化：
  - `client_block_snapshot_revision(...)` 为 `PayloadRouter`、`PayloadMassDriver`、`PayloadLoader/PayloadUnloader`、`PayloadSource` 返回 Java sync revision `1`；
  - `PayloadConveyor`、`PayloadDeconstructor`、`PayloadConstructor`、`PayloadVoid` 继续使用 Java 默认 revision `0`；
  - `apply_client_block_snapshot_child_tail(...)` 在 distribution/storage 未消费 tail 后，调用 `read_payload_runtime_state_from_building_payload(..., GameRuntimePayloadReadMode::TopLevel)`；
  - 成功解析时写入 `payload_runtime_states`，并记录 `block_child_records_applied == 1` / `block_remaining_sync_bytes == 0`；unsupported 不误消费，parse error 只记 `block_child_read_errors`。
- 新增测试：
  - `game_runtime_applies_client_payload_conveyor_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_router_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_mass_driver_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_loader_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_source_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_deconstructor_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_constructor_snapshot_child_tail_with_content`
  - `game_runtime_applies_client_payload_void_snapshot_child_tail_with_content`
- 已跑：
  - `cargo test -p mindustry-core game_runtime_applies_client --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars --lib`
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `cargo check -p mindustry-core`
- 下一步建议：
  1. 扩展真实联机 `BlockSnapshotCallPacket` smoke 到 payload-router 或 payload-mass-driver，验证 server→desktop 真实链路 materialize 到 `payload_runtime_states`。
  2. 继续推进 turret `readSync` 特例与 entity snapshot typed materialize；不要把 payload state 留成孤立 helper。

---

## 40. 最新闭环记录：真实联机 PayloadRouter BlockSnapshot child tail smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把 payload BlockSnapshot 回放从 core 单测推进到真实 server→desktop 联机链路，证明 payload-router child tail 能通过 `BlockSnapshotCallPacket` 更新客户端 runtime。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`real_server_desktop_payload_block_snapshot_updates_runtime_after_world_stream`
  - 服务端 world stream 先 materialize `payload-router` building；
  - 服务端随后发送 `BlockSnapshotCallPacket`，payload 为 `BuildingComp::write_base(...) + write_payload_conveyor_extra(...) + write_payload_router_extra(...)`；
  - desktop 端等待 `NetClient.last_block_snapshot` 与 `runtime.payload_runtime_states[payload_router_tile]` 同时更新。
- 断言覆盖：
  - `NetClient.last_block_snapshot_mirror.records[0]` 的 `tile_pos/block_id/sync_bytes`；
  - `GameRuntime.client_block_snapshot_records` 保留 raw bytes；
  - payload-router building 的 health/rotation 被 `read_base` 更新；
  - `GameRuntimePayloadBlockState::Router` 恢复 `item_rotation/sorted/rec_dir/matches`；
  - `network_context == GameRuntimeNetworkContext::client()`。
- 已跑：
  - `cargo test -p mindustry-tests real_server_desktop_payload_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests real_server_desktop_block_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 下一步建议：
  1. 扩展真实 payload snapshot smoke 到 `PayloadMassDriver`，因为它有 revision 1 tail 与更多字段。
  2. 再扩展 `PayloadLoader/Source/Deconstructor/Constructor/Void`，逐步覆盖所有 payload family。
  3. 继续推进 turret `readSync` override 与 entity snapshot typed runtime，不要停留在 raw sidecar。

---

## 41. 最新闭环记录：真实联机 PayloadMassDriver BlockSnapshot child tail smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：继续扩展真实 payload BlockSnapshot 联机覆盖，从 `PayloadRouter` 推进到 Java `PayloadDriverBuild.version()==1` 的 `PayloadMassDriver`。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`real_server_desktop_payload_mass_driver_block_snapshot_updates_runtime_after_world_stream`
  - 服务端 world stream 先 materialize `payload-mass-driver` building；
  - 服务端随后发送 `BlockSnapshotCallPacket`，sync bytes 为 `BuildingComp::write_base(...) + write_payload_block_build_common(...) + write_payload_mass_driver_extra(...)`；
  - desktop 端等待 `NetClient.last_block_snapshot` 与 `runtime.payload_runtime_states[driver_tile]` 同时更新。
- 断言覆盖：
  - mirror header 的 `tile_pos/block_id/sync_bytes`；
  - runtime raw sidecar `client_block_snapshot_records`；
  - building 基础 health/rotation；
  - `GameRuntimePayloadBlockState::MassDriver { common, driver }` 完整恢复，其中 `driver` 覆盖 `link/turret_rotation/state/reload_counter/charge/loaded/charging`。
- 已跑：
  - `cargo test -p mindustry-tests real_server_desktop_payload_mass_driver_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 下一步建议：
  1. 继续真实联机 payload snapshot：`PayloadLoader` 或 `PayloadSource`，它们同样是 revision 1 且和服务端 payload aggregate 已有关联。
  2. 再补 `PayloadDeconstructor/Constructor/Void` 的真实 snapshot smoke。
  3. 开始规划 turret `readSync` override：Java turret sync 需要保留 rotation/reload，不能直接复用 save-map child tail 语义。

---

## 42. 最新闭环记录：真实联机 PayloadLoader BlockSnapshot child tail smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：继续扩展真实 payload BlockSnapshot 联机覆盖，验证 Java `PayloadLoaderBuild.version()==1` 的 `exporting` 字段和 `PayloadBlockBuild` common tail 能从 server snapshot 同步到 desktop runtime。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`real_server_desktop_payload_loader_block_snapshot_updates_runtime_after_world_stream`
  - 服务端 world stream 先 materialize `payload-loader` building；
  - 服务端随后发送 `BlockSnapshotCallPacket`，sync bytes 为 `BuildingComp::write_base(...) + write_payload_block_build_common(...) + write_payload_loader_extra(...)`；
  - desktop 端等待 `NetClient.last_block_snapshot` 与 `runtime.payload_runtime_states[loader_tile]` 同时更新。
- 断言覆盖：
  - mirror header 的 `tile_pos/block_id/sync_bytes`；
  - runtime raw sidecar `client_block_snapshot_records`；
  - building 基础 health/rotation；
  - `GameRuntimePayloadBlockState::Loader { common, loader }` 完整恢复，其中 `loader.exporting == true`。
- 已跑：
  - `cargo test -p mindustry-tests real_server_desktop_payload_loader_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 下一步建议：
  1. 继续真实联机 payload snapshot 到 `PayloadSource`，覆盖 sandbox payload source 的 revision 1 `unit/configBlock/commandPos`。
  2. 再补 `PayloadDeconstructor/Constructor/Void` 的真实 snapshot smoke。
  3. 转入 turret `readSync` override 与 entity snapshot typed runtime。

---

## 43. 最新闭环记录：真实联机 PayloadSource BlockSnapshot child tail smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：覆盖 sandbox/payload source 的真实 BlockSnapshot 同步，验证 Java `PayloadSourceBuild.version()==1` 的 `unit/configBlock/commandPos` 字段能进客户端 runtime。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`real_server_desktop_payload_source_block_snapshot_updates_runtime_after_world_stream`
  - 服务端 world stream 先 materialize `payload-source` building；
  - 服务端随后发送 `BlockSnapshotCallPacket`，sync bytes 为 `BuildingComp::write_base(...) + write_payload_block_build_common(...) + write_payload_source_extra(...)`；
  - desktop 端等待 `NetClient.last_block_snapshot` 与 `runtime.payload_runtime_states[source_tile]` 同时更新。
- 断言覆盖：
  - mirror header 的 `tile_pos/block_id/sync_bytes`；
  - runtime raw sidecar `client_block_snapshot_records`；
  - building 基础 health/rotation；
  - `GameRuntimePayloadBlockState::Source { common, source }` 完整恢复，其中 `source.unit/config_block/command_pos/has_payload` 均对齐。
- 已跑：
  - `cargo test -p mindustry-tests real_server_desktop_payload_source_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 下一步建议：
  1. 补真实联机 `PayloadDeconstructor/Constructor/Void` snapshot，覆盖 revision 0 terminal payload/ref 分支。
  2. 或开始 turret `readSync` override：Java turret snapshot 需要保留 rotation/reload，不可直接套 save-map tail。
  3. 继续推进 entity snapshot typed runtime，替代 raw entity sidecar。

---

## 44. 最新闭环记录：真实联机 PayloadDeconstructor BlockSnapshot child tail smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：覆盖 payload revision 0 terminal payload/ref 分支，验证 `PayloadDeconstructorBuild.write/read` 的 `progress/accum/deconstructing` 能通过真实 BlockSnapshot 同步到 desktop runtime。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`real_server_desktop_payload_deconstructor_block_snapshot_updates_runtime_after_world_stream`
  - 服务端 world stream 先 materialize `small-deconstructor` building；
  - 服务端随后发送 `BlockSnapshotCallPacket`，sync bytes 为 `BuildingComp::write_base(...) + write_payload_block_build_common(...) + write_deconstructor_extra(...) + write_payload_ref(...)`；
  - `deconstructing` 使用 `PayloadRef::Block(router)` 并携带 router build bytes，专门覆盖 top-level `read_payload_ref_to_end(...)`。
- 断言覆盖：
  - mirror header 的 `tile_pos/block_id/sync_bytes`；
  - runtime raw sidecar `client_block_snapshot_records`；
  - building 基础 health；
  - `GameRuntimePayloadBlockState::Deconstructor { common, deconstructor }` 完整恢复，其中 `progress/accum/deconstructing` 均对齐。
- 已跑：
  - `cargo test -p mindustry-tests real_server_desktop_payload_deconstructor_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 下一步建议：
  1. 补真实联机 `PayloadConstructor` snapshot，覆盖 revision 0 `BlockProducerBuild.progress + Constructor.recipe`。
  2. 补真实联机 `PayloadVoid` snapshot，覆盖 terminal common tail。
  3. 转入 turret `readSync` override 与 entity snapshot typed runtime。

---

## 45. 最新闭环记录：真实联机 PayloadConstructor BlockSnapshot child tail smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：覆盖 payload constructor revision 0 child tail，验证 Java `BlockProducerBuild.progress` 与 `ConstructorBuild.recipe` 能通过真实 BlockSnapshot 同步到 desktop runtime。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`real_server_desktop_payload_constructor_block_snapshot_updates_runtime_after_world_stream`
  - 服务端 world stream 先 materialize `constructor` building；
  - 服务端随后发送 `BlockSnapshotCallPacket`，sync bytes 为 `BuildingComp::write_base(...) + write_payload_block_build_common(...) + write_block_producer_progress(...) + write_constructor_recipe(...)`；
  - desktop 端等待 `NetClient.last_block_snapshot` 与 `runtime.payload_runtime_states[constructor_tile]` 同时更新。
- 断言覆盖：
  - mirror header 的 `tile_pos/block_id/sync_bytes`；
  - runtime raw sidecar `client_block_snapshot_records`；
  - building 基础 health/rotation；
  - `GameRuntimePayloadBlockState::Constructor { common, producer, recipe }` 完整恢复。
- 已跑：
  - `cargo test -p mindustry-tests real_server_desktop_payload_constructor_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 下一步建议：
  1. 补真实联机 `PayloadVoid` snapshot，完成当前 payload family 的 BlockSnapshot smoke 覆盖。
  2. 开始 turret `readSync` override：同步时应保留 Java turret rotation/reload 语义。
  3. 继续 entity snapshot typed runtime，替换 raw entity sidecar。

---

## 46. 最新闭环记录：真实联机 PayloadVoid BlockSnapshot child tail smoke + pump 竞态修复

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：补完当前 payload family 的真实 BlockSnapshot smoke 覆盖，并修复真实联机测试中客户端确认包与服务端处理之间的竞态。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：`real_server_desktop_payload_void_block_snapshot_updates_runtime_after_world_stream`
  - 服务端 world stream 先 materialize `payload-void` building；
  - 服务端随后发送 `BlockSnapshotCallPacket`，sync bytes 为 `BuildingComp::write_base(...) + write_payload_block_build_common(...)`；
  - desktop 端等待 `NetClient.last_block_snapshot` 与 `runtime.payload_runtime_states[void_tile]` 同时更新。
- 断言覆盖：
  - mirror header 的 `tile_pos/block_id/sync_bytes`；
  - runtime raw sidecar `client_block_snapshot_records`；
  - building 基础 health；
  - `GameRuntimePayloadBlockState::Void(common)` 恢复。
- 测试驱动修复：
  - `pump_real_server_desktop_until(...)` 现在除客户端 `connect_confirm_sent` 和 runtime materialized 外，还等待服务端 `last_connect_confirm_connection_id.is_some()`；
  - 这是为真实联机 smoke 消除并发测试中的 race，不改变生产网络协议。
- 已跑：
  - `cargo test -p mindustry-tests real_server_desktop_payload_void_block_snapshot_updates_runtime_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 下一步建议：
  1. 转入 turret `readSync` override：Java turret snapshot 需要保留 rotation/reload。
  2. 或继续 entity snapshot typed runtime，把 raw entity sidecar 写入真实 entity pool/mirror。
  3. payload UnitPayload 完整恢复仍未完成，后续需要接实体/单位内容 registry。

---

## 47. 最新闭环记录：Turret BlockSnapshot `readSync` rotation/reload 保留

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- Java 依据：
  - `BuildingComp.writeSync(...) -> writeAll(...)`
  - `BuildingComp.readSync(...) -> readAll(...)`
  - `TurretBuild.readSync(...)` 覆盖：先保存旧 `rotation/reloadCounter`，调用 `readAll(read, revision)`，再恢复旧值，防止客户端炮塔同步时 snapping。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为变化：
  - `client_block_snapshot_revision(...)` 新增 turret revision 映射：`ItemTurret=2`、`Continuous=3`、`PayloadAmmo/Liquid/Power/Laser=1`；
  - `apply_client_block_snapshot_child_tail(...)` 在 distribution/storage/payload 后继续尝试 turret reader；
  - 新增 `preserve_client_turret_sync_fields(...)`，当已有 turret runtime state 时，保留旧 `TurretState.rotation/reload_counter`，同时接受 snapshot 中的 ammo/其他 child state；
  - 新增 `game_runtime_turret_state(...)` / `game_runtime_turret_state_mut(...)` helper，只对 Generic/Item/PayloadAmmo/Continuous 这类含 `TurretState` 的变体生效。
- 新增测试：
  - `game_runtime_applies_client_item_turret_snapshot_preserving_rotation_reload_with_content`
  - 覆盖 `duo` item turret：snapshot 更新 ammo 与 building base，但 runtime turret 的旧 rotation/reload 被保留。
- 已跑：
  - `cargo test -p mindustry-core game_runtime_applies_client_item_turret_snapshot_preserving_rotation_reload_with_content --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
  - `git diff --check`
- 下一步建议：
  1. 补真实联机 item turret BlockSnapshot smoke，验证 server→desktop 链路也保留 rotation/reload 并更新 ammo。
  2. 给 `Continuous/PayloadAmmo/Generic` turret 补 core 单测。
  3. 继续 entity snapshot typed runtime。

---

## 48. 最新闭环记录：真实联机 ItemTurret BlockSnapshot 保留 rotation/reload

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把上一闭环的 `TurretBuild.readSync(...)` 保留语义接到真实 server→desktop BlockSnapshot 路径。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：
  - `real_server_desktop_item_turret_block_snapshot_preserves_rotation_reload_after_world_stream`
- 测试覆盖：
  - 服务端 world stream 先 materialize `duo` building，并下发旧 `GameRuntimeTurretBlockState::Item`；
  - desktop runtime 先确认旧 `rotation/reload_counter` 已存在；
  - 服务端发送真实 `BlockSnapshotCallPacket`，sync bytes 为 `BuildingComp::write_base(...) + turret_write_child(...) + item_turret_write_ammo(...)`；
  - desktop 端确认 `NetClient.last_block_snapshot_mirror`、raw sidecar、building base health/rotation、turret ammo/total_ammo 都更新；
  - 同时确认 `TurretState.rotation/reload_counter` 保留旧值，匹配 Java `TurretBuild.readSync(...)` 的客户端抗抖逻辑。
- 已跑：
  - `cargo test -p mindustry-tests real_server_desktop_item_turret_block_snapshot_preserves_rotation_reload_after_world_stream --lib`
  - `cargo test -p mindustry-tests --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 给 `Continuous/PayloadAmmo/Generic` turret 补 core/真实联机测试，继续扩大 `TurretBuild.readSync` 覆盖。
  2. 继续 entity snapshot typed runtime，把 raw entity sidecar 接入真实 entity pool/mirror。

---

## 49. 最新闭环记录：Generic/Continuous/PayloadAmmo Turret readSync 保留单测

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：继续扩大 Java `TurretBuild.readSync(...)` 的旧 `rotation/reloadCounter` 保留语义覆盖面，避免只验证 `ItemTurret`。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：
  - `game_runtime_applies_client_generic_turret_snapshot_preserving_rotation_reload_with_content`
  - `game_runtime_applies_client_continuous_turret_snapshot_preserving_rotation_reload_with_content`
  - `game_runtime_preserves_payload_ammo_turret_snapshot_rotation_reload_after_reading_payloads`
- 覆盖说明：
  - `arc`/Generic(PowerTurret)：真实 content + client BlockSnapshot child-tail dispatcher；
  - `lustre`/ContinuousTurret：真实 content + `continuous_turret_write_child(...)` tail；
  - `PayloadAmmoTurret`：当前基础 content 无正式 payload-ammo turret，因此使用自定义 block + payload reader + `preserve_client_turret_sync_fields(...)` 验证保留逻辑。
- 已跑：
  - `cargo test -p mindustry-core rotation_reload --lib`
  - `cargo test -p mindustry-core game_runtime_exports_turret_state_tail_in_network_map_snapshot --lib`
  - `cargo check -p mindustry-core`
- 下一步建议：
  1. 继续给 `ContinuousLiquidTurret/LiquidTurret/LaserTurret` 补同类 content-level 单测。
  2. 或转入 entity snapshot typed runtime，把 raw entity sidecar 接入真实 entity pool/mirror。

---

## 50. 最新闭环记录：ContinuousLiquid/Liquid/Laser Turret readSync 保留单测

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：收敛 49 中剩余的 content-level turret kind 覆盖缺口。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 新增测试：
  - `game_runtime_applies_client_continuous_liquid_turret_snapshot_preserving_rotation_reload_with_content`
  - `game_runtime_applies_client_liquid_and_laser_turret_snapshots_preserving_rotation_reload_with_content`
- 覆盖说明：
  - `sublimate`/ContinuousLiquidTurret：真实 content + client BlockSnapshot child-tail dispatcher；
  - `wave`/LiquidTurret 与 `meltdown`/LaserTurret：真实 content + Generic turret child-tail reader；
  - 断言 building base 更新，但 `TurretState.rotation/reload_counter` 保留旧值。
- 已跑：
  - `cargo test -p mindustry-core rotation_reload --lib`
  - `cargo check -p mindustry-core`
- 下一步建议：
  1. 如果继续 turret 路线：补 `Continuous` 或 `Generic` 的真实 server→desktop smoke。
  2. 如果转入更大互通缺口：推进 entity snapshot typed runtime。

---

## 51. 最新闭环记录：EntitySnapshot typed Unit runtime 初步接入

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把 entity snapshot 从 raw sidecar 推进一步，至少能把真实 server→desktop `EntitySnapshotCallPacket` 中的 unit sync bytes materialize 到 typed `UnitComp`。
- Java 依据：
  - `core/src/mindustry/core/NetClient.java::readSyncEntity(...)`
  - `core/src/mindustry/core/NetClient.java::entitySnapshot(...)`
  - `core/src/mindustry/core/NetClient.java::hiddenSnapshot(...)`
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `desktop/src/lib.rs`
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为变化：
  - `GameRuntime` 新增 `client_unit_snapshot_entities`；
  - 新增 `apply_client_entity_snapshot_record_with_content(...)`：先保留 raw `GameRuntimeClientEntitySnapshotRecord`，再尝试 `type_io::read_unit_sync(...)`；
  - 成功解析后创建/更新 `UnitComp`，调用 `read_sync`，新建时 `snap_sync + add`，随后 `after_sync`；
  - hidden snapshot 对 typed unit 调 `handle_sync_hidden`；
  - `DesktopLauncher::sync_snapshot_mirrors(...)` 现在传入 `ContentLoader`，真实链路可落到 typed unit runtime。
- 测试：
  - `game_runtime_applies_client_unit_entity_snapshot_to_typed_runtime_with_content`
  - `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream` 已扩展为发送 `dagger` 的 `UnitSyncWire` bytes，并断言 desktop typed unit runtime。
- 已跑：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_entity_snapshot_to_typed_runtime_with_content --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 建立/迁移 Java `EntityMapping` class-id 对照，避免 typed unit 解析完全依赖“能否按 UnitSyncWire 读通”。
  2. 继续做 `PlayerComp` typed snapshot。
  3. 支持 entity snapshot 多 record 变长拆包，而不是当前仅 amount=1 能携带 opaque sync bytes。

---

## 52. 最新闭环记录：EntitySnapshot 多 UnitSync record fallback 拆包

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：补上 Java `NetClient.entitySnapshot(...)` 里一个 packet 内连续读取多条 `readSyncEntity(...)` 的一部分能力。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
  - `desktop/src/lib.rs`
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为变化：
  - 新增 `GameRuntime::apply_client_entity_snapshot_packet_with_content(...)`；
  - 当 `ClientEntitySnapshotMirror` 因多 record 变长 `sync_bytes` 无法固定拆分而出现 `parse_error` 时，`DesktopLauncher::sync_snapshot_mirrors(...)` 会把原始 `amount/data` 交给 runtime fallback；
  - fallback 按 `id + type_id + UnitSyncWire` 连续读取，成功后每条都写 raw sidecar 并 materialize typed `UnitComp`。
- 测试：
  - `game_runtime_applies_multi_unit_entity_snapshot_packet_with_content`
  - `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream` 现在额外覆盖一个 `amount=2`、含两条 unit sync bytes 的真实 server→desktop entity snapshot。
- 已跑：
  - `cargo test -p mindustry-core game_runtime_applies_multi_unit_entity_snapshot_packet_with_content --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_client_snapshot_mirrors_to_runtime_sidecars --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 迁移 Java `EntityMapping` class-id registry，避免“能按 UnitSyncWire 读通”这种临时判断。
  2. 扩展 `PlayerComp` typed snapshot 的多 record 混合拆包；注意 entity snapshot 应使用 `NetworkPlayerSyncData`，不要误用 revisioned `NetworkPlayerData`。
  3. 支持混合实体类型的通用变长拆包。

---

## 53. 最新闭环记录：PlayerComp typed entity snapshot 接入

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：让 Java `NetClient.entitySnapshot(...) -> readSyncEntity(...)` 中本地玩家 `player.readSync(...)` 的一部分语义落到 Rust 真实 desktop/player runtime。
- Java 依据：
  - `core/src/mindustry/entities/comp/PlayerComp.java:39-50`：`@SyncLocal` / `@NoSync lastCommand` / 基础字段；
  - `core/src/mindustry/entities/comp/PlayerComp.java:184-213`：`afterSync()` unit 纠偏与控制回放；
  - `annotations/src/main/java/mindustry/annotations/entity/EntityIO.java:131-180`：`writeSync/readSync` 不写 revision，`@SyncLocal` 本地只消费不覆盖，读完调用 `afterSync()`；
  - `core/src/mindustry/core/NetClient.java:452-486`：`id == player.id()` 时复用本地 player。
- Rust 主改动：
  - `core/src/mindustry/net/network_io.rs`
  - `core/src/mindustry/entities/comp/player.rs`
  - `core/src/mindustry/core/game_runtime.rs`
  - `desktop/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为变化：
  - 新增 `NetworkPlayerSyncData`，专门表示 Java `Player.writeSync(...)` wire body；它不带 `revision`，也不带 `lastCommand`；
  - `PlayerComp::apply_network_player_sync_data(..., is_local=true)` 保留本地 `@SyncLocal` 状态，只更新 `admin/color/name/team/unit` 等非本地输入字段；
  - `DesktopLauncher::sync_snapshot_mirrors(...)` 对 `entity_id == player.id` 的 record 解析 player sync，更新 `launcher.player`，调用 `after_sync_unit_state(...)`，并写入 `GameRuntime.client_player_snapshot_entities` typed sidecar；
  - raw `client_entity_snapshot_records` 仍保留原始 `sync_bytes`。
- 已跑：
  - `cargo test -p mindustry-core network_player_sync_data_round_trips_java_write_sync_shape --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_applies_local_player_entity_snapshot_to_typed_player_runtime --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 迁移 Java `EntityMapping` class-id registry，建立 `type_id -> Player/Unit/其他 Syncc` dispatcher。
  2. 继续把当前 mixed fallback 从“PlayerComp 特判 + 其他 UnitSyncWire 尝试”升级为真正 class-id dispatcher。
  3. 给真实 server→desktop smoke 增加本地 player entity snapshot，验证 `NetworkPlayerSyncData` 走真实 packet 解码链。

---

## 54. 最新闭环记录：混合 PlayerComp + UnitComp 多 record fallback

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：继续收窄 Java `NetClient.entitySnapshot(...)` 与 Rust mirror 的差距，让同一个多 record packet 中的本地 PlayerComp 与 UnitComp 不再因为 opaque 变长 payload 无法固定拆分而整体丢失。
- Java 依据补充：`annotations/src/main/resources/classids.properties` 可直接验证 `mindustry.entities.comp.PlayerComp=12`；`NetServer` 写 `entity.classId()`，`NetClient` 用 `EntityMapping.map(typeID & 0xFF)` 建实体。
- Rust 主改动：
  - `desktop/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为变化：
  - `DesktopLauncher::sync_snapshot_mirrors(...)` 遇到 `ClientEntitySnapshotMirror.parse_error` 时，先调用 mixed fallback；
  - mixed fallback 逐条读 `entity_id + type_id`；
  - `entity_id == launcher.player.id` 时按 `NetworkPlayerSyncData` 消费 Player sync body，落到 raw sidecar + typed player runtime；
  - 其他 record 先按 `read_unit_sync(...)` 消费 Unit sync body，再复用 runtime 的 typed `UnitComp` materialization；
  - 成功应用时不再只记录 parse error。
- 新增测试：
  - `desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet`
- 已跑：
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-desktop --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 继续扩展真实 server→desktop snapshot smoke 到其他 `Syncc`。
  2. 迁移 Java generated `EntityMapping` class-id registry，取代当前“本地 player 特判 + unit parse-shape 猜测”。
  3. 继续补其他 `Syncc` typed snapshot（Bullet/Fire/Weather/Effect 等按 Java entity mapping 优先级推进）。

---

## 55. 最新闭环记录：真实联机 PlayerComp + UnitComp 混合 EntitySnapshot smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：验证 `NetworkPlayerSyncData` 与 mixed fallback 不只是手工构造 mirror，可通过真实 `ServerLauncher -> DesktopLauncher` packet 链路落地。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为/覆盖变化：
  - `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream` 的第三个 entity snapshot packet 改成 `amount=3`：
    - 本地 `connection_id` 的 `PlayerComp` sync body；
    - `1004` 的 dagger `UnitSyncWire`；
    - `1005` 的 flare `UnitSyncWire`；
  - 该 packet 在 `NetClient` mirror 层仍是 parse_error，但 desktop mixed fallback 会拆包并落到 runtime/player；
  - 测试断言 `desktop.player` 的 `name/admin/color/team/unit_ref`、`runtime.client_player_snapshot_entities`、两个 typed unit、raw sidecar 都正确。
- 已跑：
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 用 class-id registry 继续替换 `DesktopLauncher` mixed fallback 里的 unit parse-shape 猜测。
  2. 继续把 registry 接到通用 entity snapshot dispatcher。
  3. 按 class-id 优先级继续迁移 Bullet/Fire/Weather/Effect 等 `Syncc` snapshot。

---

## 56. 最新闭环记录：Entity class-id registry 基线迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把 Java `annotations/src/main/resources/classids.properties` 的稳定 `entity.classId()` 映射迁入 Rust，为后续通用 snapshot dispatcher 做基础。
- Rust 主改动：
  - `core/src/mindustry/entities/mod.rs`
  - `desktop/src/lib.rs`
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为变化：
  - 新增 `ENTITY_CLASS_IDS` 静态表，覆盖 49 条上游 class-id；
  - 新增 `PLAYER_CLASS_ID = 12`；
  - 新增 `entity_class_id(name)` / `entity_class_name(id)`；
  - `DesktopLauncher` mixed fallback 处理本地 player record 时，现在同时要求 `entity_id == player.id` 且 `type_id == PLAYER_CLASS_ID`；
  - 测试里不再硬编码 PlayerComp 的 `12`。
- 已跑：
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 将 `EntityClassKind::Other` 继续拆成 Bullet/Fire/Weather/Effect 等实际 dispatcher。
  2. 继续迁移 `Other` 类 `Syncc` 的 readSync/writeSync wire。
  3. 后续按 class-id 表继续迁移其他 `Syncc` 的 readSync/writeSync wire。

---

## 57. 最新闭环记录：Entity class-id kind 分类接入 mixed fallback

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：让 mixed entity snapshot fallback 从“非 player 就尝试 UnitSyncWire”推进为基于 class-id registry 的第一层分发。
- Rust 主改动：
  - `core/src/mindustry/entities/mod.rs`
  - `desktop/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为变化：
  - 新增 `EntityClassKind::{Player, Unit, Other}` 与 `entity_class_kind(id)`；
  - `DesktopLauncher` mixed fallback 对本地 player 同时校验 `entity_id == player.id` 和 `type_id == PLAYER_CLASS_ID`；
  - 非 player record 只有 `entity_class_kind(type_id) == Unit` 时才走 `read_unit_sync(...)`；
  - `Other` class-id 暂不猜测解析，直接作为 parse error 暴露，避免把未迁移实体误读成 Unit。
- 已跑：
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 继续从 `Other` 中挑低复杂 `Syncc`（如 `PuddleComp`/`WeatherStateComp`）迁移 readSync/writeSync wire。
  2. 给 `EntityClassKind` 增加更细粒度枚举，避免 `Other` 一桶装。
  3. 逐步让 `GameRuntime`/`DesktopLauncher` 共享同一个 entity snapshot dispatcher。

---

## 58. 最新闭环记录：FireComp EntitySnapshot typed runtime 接入

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把 class-id `10` 的 Java `FireComp.writeSync/readSync` 从 opaque entity bytes 推进到 Rust typed runtime。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.FireComp=10`；
  - `annotations/src/main/resources/revisions/FireComp/1.json`：sync 字段顺序 `lifetime, tile, time, x, y`；
  - `FireComp.afterSync()` 调 `Fires.register(self())`。
- Rust 主改动：
  - `core/src/mindustry/io/type_io.rs`
  - `core/src/mindustry/io/mod.rs`
  - `core/src/mindustry/entities/comp/fire.rs`
  - `core/src/mindustry/entities/mod.rs`
  - `core/src/mindustry/core/game_runtime.rs`
  - `desktop/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为变化：
  - 新增 `FireSyncWire` 与 `read_fire_sync/write_fire_sync`；
  - `FireComp::apply_sync_wire(...)` 按 Java sync 字段更新并执行 `after_sync()`；
  - `GameRuntime` 新增 `client_fire_snapshot_entities` typed sidecar；
  - mixed fallback 中 `EntityClassKind::Fire` 会解析 Fire wire，写 raw sidecar 并 materialize typed fire；
  - 正常 single-record mirror 路径也会尝试 Fire typed apply；
  - hidden snapshot 对 typed fire 计为 existing。
- 已跑：
  - `cargo test -p mindustry-core fire_sync --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_fire_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 把 Fire typed sidecar 与 `Fires` tile-indexed collection 的注册/查询统一起来。
  2. 继续给真实 server→desktop entity snapshot smoke 加其他 `Syncc` record。
  3. 继续迁移 `PuddleComp` 或 `WeatherStateComp` 的 sync wire。

---

## 59. 最新闭环记录：真实联机 Fire EntitySnapshot smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：验证 Fire typed snapshot 能走真实 `ServerLauncher -> DesktopLauncher` packet 链路。
- Rust 主改动：
  - `tests/src/lib.rs`
  - `MIGRATION.md`
  - `AI_HANDOFF.md`
- 行为/覆盖变化：
  - `real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream` 的 mixed packet 从 `amount=3` 扩为 `amount=4`；
  - 新增 `1006 + FIRE_CLASS_ID + FireSyncWire`；
  - 测试断言 `runtime.client_fire_snapshot_entities[1006]`、raw sidecar、fire `lifetime/time/x/y/tile/registered`。
- 已跑：
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. `PuddleComp` 已由第 60 节补上；继续迁移 `WeatherStateComp` 或 `EffectStateComp` 的 sync wire。
  2. 将 Fire typed sidecar 与 `Fires` 集合统一，避免长期双存储。
  3. 后续把真实 smoke 扩展到更多 entity class-id。

---

## 60. 最新闭环记录：PuddleComp EntitySnapshot typed runtime 与真实联机 smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把 class-id `13` 的 Java `PuddleComp.writeSync/readSync` 从 opaque entity bytes 推进到 Rust typed runtime，并纳入真实 `ServerLauncher -> DesktopLauncher` mixed entity snapshot smoke。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.PuddleComp=13`；
  - `annotations/src/main/resources/revisions/PuddleComp/1.json`：sync 字段顺序 `amount, liquid, tile, x, y`；
  - `PuddleComp.afterSync()` 仅在 `liquid != null` 时注册到 `Puddles`。
- Rust 主改动：
  - `type_io::PuddleSyncWire` 与 `read_puddle_sync/write_puddle_sync`；
  - `EntityClassKind::Puddle` 与 `PUDDLE_CLASS_ID`；
  - `PuddleComp::apply_sync_wire(...)`；
  - `GameRuntime.client_puddle_snapshot_entities` 与 `apply_client_puddle_sync_wire(...)`；
  - `DesktopLauncher` mixed fallback 现在支持 Player + Unit + Fire + Puddle 分类拆包；
  - 真实联机 smoke 的第三个 entity snapshot packet 现在为 `amount=5`，新增 `1007 + PUDDLE_CLASS_ID + PuddleSyncWire`。
- 已跑：
  - `cargo test -p mindustry-core puddle_sync --lib`
  - `cargo test -p mindustry-core puddle_component_applies_sync_wire_and_registers_when_liquid_present --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_puddle_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 把 Puddle typed sidecar 与 `Puddles` tile-indexed collection 统一，避免长期双存储。
  2. `WeatherStateComp` 已由第 61 节补上；继续迁移 `EffectStateComp` / `BulletComp` 等 entity sync wire。
  3. 继续收敛 `GameRuntime` 与 `DesktopLauncher` 的 entity snapshot dispatcher，减少重复分发逻辑。

---

## 61. 最新闭环记录：WeatherStateComp EntitySnapshot typed runtime 与真实联机 smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把 class-id `14` 的 Java `Weather.WeatherStateComp.writeSync/readSync` 从 opaque entity bytes 推进到 Rust typed runtime，并继续扩展真实 `ServerLauncher -> DesktopLauncher` mixed entity snapshot smoke。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.type.Weather.WeatherStateComp=14`；
  - `annotations/src/main/resources/revisions/WeatherStateComp/2.json`：sync 字段顺序 `effectTimer, intensity, life, opacity, weather, windVector, x, y`；
  - `TypeIO.writeWeather/readWeather` 是 nullable short content id，`TypeIO.writeVec2/readVec2` 是两个 `float`。
- Rust 主改动：
  - `type_io::WeatherStateSyncWire` 与 `read_weather_state_sync/write_weather_state_sync`；
  - `EntityClassKind::Weather` 与 `WEATHER_STATE_CLASS_ID`；
  - `WeatherState` 增加 `x/y` 并新增 `apply_sync_wire(...)`；
  - `ContentLoader::weather/weather_by_name/weathers`；
  - `GameRuntime.client_weather_snapshot_entities` 与 `apply_client_weather_state_sync_wire(...)`；
  - `DesktopLauncher` mixed fallback 现在支持 Player + Unit + Fire + Puddle + Weather 分类拆包；
  - 真实联机 smoke 的第三个 entity snapshot packet 现在为 `amount=6`，新增 `1008 + WEATHER_STATE_CLASS_ID + WeatherStateSyncWire`。
- 已跑：
  - `cargo test -p mindustry-core weather_state_sync --lib`
  - `cargo test -p mindustry-core weather_state_applies_sync_wire_and_restores_position_fields --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_weather_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_applies_weather_entity_snapshot_packet_with_content --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 把 Weather typed sidecar 与未来完整 `Groups.weather`/renderer/weather update runtime 统一。
  2. `EffectStateComp` 已由第 62 节补上；继续迁移 `BulletComp` / `DecalComp` 等 entity snapshot wire。
  3. 将 single-record 与 mixed fallback dispatcher 继续收敛，避免新增 entity 类型时双处修改。

---

## 62. 最新闭环记录：EffectStateComp EntitySnapshot typed runtime 与真实联机 smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把 class-id `9` 的 Java `EffectStateComp` 最新 revision wire 从 opaque entity bytes 推进到 Rust typed sidecar，并继续扩展真实 `ServerLauncher -> DesktopLauncher` mixed entity snapshot smoke。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.EffectStateComp=9`；
  - `annotations/src/main/resources/revisions/EffectStateComp/6.json`：字段顺序 `color, data, effect, lifetime, offsetPos, offsetRot, offsetX, offsetY, parent, rotWithParent, rotation, time, x, y`；
  - `TypeIO.writeColor/readColor` 写 `int rgba`；`TypeIO.writeObject/readObject` 写动态 object；`TypeIO.writeEffect/readEffect` 写 effect short id；`TypeIO.writePosEntity/readPosEntity` 写 parent entity id。
- Rust 主改动：
  - `type_io::EffectStateSyncWire` 与 `read_effect_state_sync/write_effect_state_sync`；
  - `EntityClassKind::Effect` 与 `EFFECT_STATE_CLASS_ID`；
  - `EffectStateComp` 扩展 `data/effect_id/offset*/parent_id/rot_with_parent` 并新增 `apply_sync_wire(...)`；
  - `GameRuntime.client_effect_snapshot_entities` 与 `apply_client_effect_state_sync_wire(...)`；
  - `DesktopLauncher` mixed fallback 现在支持 Player + Unit + Effect + Fire + Puddle + Weather 分类拆包；
  - 真实联机 smoke 的第三个 entity snapshot packet 现在为 `amount=7`，新增 `1009 + EFFECT_STATE_CLASS_ID + EffectStateSyncWire`。
- 已跑：
  - `cargo test -p mindustry-core effect_state_sync --lib`
  - `cargo test -p mindustry-core effect_state_applies_sync_wire_fields_and_preserves_effect_clip --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_effect_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_applies_effect_entity_snapshot_packet_with_content --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 将 Effect typed sidecar 与完整 `EffectRegistry`/renderer/effect lifecycle 统一，避免长期只停留在 snapshot mirror。
  2. `DecalComp` 已由第 63 节补上；继续迁移 `BulletComp` / `LaunchCoreComp` 等 entity snapshot wire。
  3. 后续补服务端从真实 entity groups 枚举 EffectState 的发包路径，而不只是 smoke 中人工构造 packet。

---

## 63. 最新闭环记录：DecalComp EntitySnapshot typed runtime 与真实联机 smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把 class-id `8` 的 Java `DecalComp` sync wire 从 opaque entity bytes 推进到 Rust typed sidecar，并继续扩展真实 `ServerLauncher -> DesktopLauncher` mixed entity snapshot smoke。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.DecalComp=8`；
  - `annotations/src/main/resources/revisions/DecalComp/0.json`：字段顺序 `color, lifetime, region, rotation, time, x, y`；
  - `TypeIO.writeColor/readColor` 写 `int rgba`；
  - `region` 是 `TextureRegion`，上游 annotation serializer 没有对应 TypeIO，生成的 Java sync code 会跳过该字段，不产生 wire bytes。
- Rust 主改动：
  - `type_io::DecalSyncWire` 与 `read_decal_sync/write_decal_sync`，实际 wire 顺序为 `color, lifetime, rotation, time, x, y`；
  - `EntityClassKind::Decal` 与 `DECAL_CLASS_ID`；
  - `DecalComp::apply_sync_wire(...)` 恢复颜色/生命周期/位置/旋转，保留既有 `DecalRegion`；
  - `GameRuntime.client_decal_snapshot_entities` 与 `apply_client_decal_sync_wire(...)`；
  - hidden snapshot 对 typed decal 计为 existing；
  - `DesktopLauncher` mixed fallback 现在支持 Player + Unit + Decal + Effect + Fire + Puddle + Weather 分类拆包；
  - 真实联机 smoke 的第三个 entity snapshot packet 现在为 `amount=8`，新增 `1010 + DECAL_CLASS_ID + DecalSyncWire`。
- 已跑：
  - `cargo test -p mindustry-core decal_sync --lib`
  - `cargo test -p mindustry-core decal_component --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_decal_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_applies_decal_entity_snapshot_packet_with_content --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. `BulletComp` 已由第 64 节补上；继续迁移 `LaunchCoreComp` / `WorldLabelComp` 等 entity snapshot wire。
  2. 将 Decal typed sidecar 接入真实 renderer/texture atlas region lifecycle；注意 Java sync 不传 `region`，不能凭 snapshot 恢复贴图。
  3. 收敛 `GameRuntime` 与 `DesktopLauncher` 的 entity snapshot dispatcher，减少每新增实体类型都双处修改。

---

## 64. 最新闭环记录：BulletComp EntitySnapshot typed runtime 与真实联机 smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把 class-id `7` 的 Java `BulletComp` revision 2 sync wire 从 opaque entity bytes 推进到 Rust typed sidecar，并继续扩展真实 `ServerLauncher -> DesktopLauncher` mixed entity snapshot smoke。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.BulletComp=7`；
  - `annotations/src/main/resources/revisions/BulletComp/2.json`：字段顺序 `collided, damage, data, fdata, lifetime, owner, rotation, team, time, type, vel, x, y`；
  - `TypeIO.writeIntSeq/readIntSeq`：`int length + i32[]`；
  - `TypeIO.writeObject/readObject`：动态 object；
  - `TypeIO.writeEntity/readEntity`：owner entity id；
  - `TypeIO.writeTeam/readTeam`：`u8` team id；
  - `TypeIO.writeBulletType/readBulletType`：`short` bullet content id；
  - `TypeIO.writeVec2/readVec2`：两个 `float`；
  - `Mover` 为 Java transient runtime 字段，不在 snapshot wire 中。
- Rust 主改动：
  - `type_io::BulletSyncWire` 与 `read_bullet_sync/write_bullet_sync`；
  - `EntityClassKind::Bullet` 与 `BULLET_CLASS_ID`；
  - `BulletComp::apply_sync_wire(...)` 恢复 revision 2 字段；
  - `GameRuntime.client_bullet_snapshot_entities` 与 `apply_client_bullet_sync_wire(...)`；
  - bullet content id 通过 `ContentLoader::get_by_id(ContentType::Bullet, ...)` 校验；
  - hidden snapshot 对 typed bullet 计为 existing；
  - `DesktopLauncher` mixed fallback 现在支持 Player + Unit + Bullet + Decal + Effect + Fire + Puddle + Weather 分类拆包；
  - 真实联机 smoke 的第三个 entity snapshot packet 现在为 `amount=9`，新增 `1011 + BULLET_CLASS_ID + BulletSyncWire`。
- 已跑：
  - `cargo test -p mindustry-core bullet_sync --lib`
  - `cargo test -p mindustry-core bullet_component_applies_revision_2_sync_wire_fields --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_bullet_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_applies_bullet_entity_snapshot_packet_with_content --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. 将 Bullet typed sidecar 接入完整 `Groups.bullet` lifecycle、碰撞、渲染与服务端真实实体枚举发包。
  2. `WorldLabelComp` 已由第 65 节补上；继续迁移剩余 entity snapshot：`LaunchCoreComp`、`LocationPingComp` 等。
  3. 收敛 entity snapshot dispatcher，避免 `GameRuntime` / `DesktopLauncher` 双份 match 长期分叉。

---

## 65. 最新闭环记录：WorldLabelComp EntitySnapshot typed runtime 与真实联机 smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把 class-id `35` 的 Java `WorldLabelComp` revision 1 sync wire 从 opaque entity bytes 推进到 Rust typed sidecar，并继续扩展真实 `ServerLauncher -> DesktopLauncher` mixed entity snapshot smoke。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.WorldLabelComp=35`；
  - `annotations/src/main/resources/revisions/WorldLabelComp/1.json`：字段顺序 `flags, fontSize, parent, text, x, y, z`；
  - `flags`：`byte`，`FLAG_ONLY_PARENT_VISIBLE = 32`；
  - `fontSize/x/y/z`：`float`；
  - `parent`：`TypeIO.writePosEntity/readPosEntity`，当前 wire 是 `i32 entity id`，空引用为 `-1`；
  - `text`：`TypeIO.writeString/readString`。
- Rust 主改动：
  - `type_io::WorldLabelSyncWire` 与 `read_world_label_sync/write_world_label_sync`；
  - `EntityClassKind::WorldLabel` 与 `WORLD_LABEL_CLASS_ID = 35`；
  - `WorldLabelComp` 新增 `parent_id: Option<i32>`，并通过 `apply_sync_wire(...)` 恢复 flags、字体、父实体、文本、位置和 z；
  - `GameRuntime.client_world_label_snapshot_entities` 与 `apply_client_world_label_sync_wire(...)`；
  - hidden snapshot 对 typed world-label 计为 existing；
  - `DesktopLauncher` mixed fallback 现在支持 Player + Unit + WorldLabel + Bullet + Decal + Effect + Fire + Puddle + Weather 分类拆包；
  - 真实联机 smoke 的第三个 entity snapshot packet 现在为 `amount=10`，新增 `1012 + WORLD_LABEL_CLASS_ID + WorldLabelSyncWire`。
- 已跑：
  - `cargo test -p mindustry-core world_label_sync --lib`
  - `cargo test -p mindustry-core world_label_applies_revision_1_sync_wire_fields --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_world_label_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_applies_world_label_entity_snapshot_packet_with_content --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_fallback_splits_mixed_player_and_unit_entity_snapshot_packet --lib`
  - `cargo test -p mindustry-tests real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream --lib`
  - `cargo check -p mindustry-core -p mindustry-desktop -p mindustry-tests`
- 下一步建议：
  1. `LaunchCoreComp` 已由第 66 节补上；继续迁移 `LocationPingComp` 等剩余 entity/revision 形状。
  2. 将 WorldLabel typed sidecar 接入真实 label draw/lifecycle 与父实体可见性过滤；当前只保证 Java wire parity 与客户端 snapshot runtime mirror。
  3. 收敛 `GameRuntime` / `DesktopLauncher` entity snapshot dispatcher，减少每新增实体类型都双处修改。

---

## 66. 最新闭环记录：LaunchCoreComp revision 0 runtime 接入

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把 class-id `11` 的 Java `LaunchCoreComp` revision 0 字段形状落入 Rust runtime，避免现有实现只停留在独立绘制/烟雾 helper。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.LaunchCoreComp=11`；
  - `annotations/src/main/resources/revisions/LaunchCoreComp/0.json`：字段顺序 `block, lifetime, time, x, y`；
  - `core/src/mindustry/entities/comp/LaunchCoreComp.java`：`@EntityDef(value = LaunchCorec.class, serialize = false)`，不是常规 `Syncc.writeSync/readSync` snapshot；
  - `block` 是 `mindustry.world.Block` 内容引用，wire 使用 `short block id`，空为 `-1`；`lifetime/time/x/y` 均为 `float`。
- Rust 主改动：
  - `type_io::LaunchCoreRevisionWire` 与 `read_launch_core_revision/write_launch_core_revision`；
  - `entities::LAUNCH_CORE_CLASS_ID = 11` 并在 class-id baseline 测试中锁定；
  - `LaunchCoreComp` 新增 `block_id`、`with_block_id(...)`、`apply_revision_wire(...)`、`to_revision_wire(...)`；
  - `LaunchCoreBlock::from_block_def(...)` 从 content registry 的 `BlockDef` 恢复 size 与临时 icon 占位尺寸；
  - `GameRuntime.launch_core_entities` 与 `apply_launch_core_revision_wire(...)`，通过 `ContentLoader.block(id)` 将 revision 0 payload 接入整体 runtime。
- 已跑：
  - `cargo test -p mindustry-core launch_core --lib`
  - `cargo test -p mindustry-core launch_core_revision --lib`
  - `cargo test -p mindustry-core game_runtime_applies_launch_core_revision_zero_to_runtime_entity --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo check -p mindustry-core`
- 下一步建议：
  1. `LocationPingComp` / Player ping 行为已由第 67 节补上；继续迁移下一个 class-id/revision 明确的实体形状，并优先接入 runtime 而不是只放 helper。
  2. 后续渲染/atlas 迁移后，用真实 `TextureRegion.width/height/scl()` 替换 `LaunchCoreBlock::from_block_def(...)` 的占位 icon 尺寸。
  3. 将 launch lifecycle 接入真实 effect/group/update 流程；当前已经有 runtime sidecar，但还不是完整 Java `Groups` 生命周期。

---

## 67. 最新闭环记录：LocationPingComp class-id 与 Player ping runtime 行为

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：核查 class-id `48` 的 `LocationPingComp`，并将实际位置 ping 行为接入 `PlayerComp` runtime。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`mindustry.entities.comp.LocationPingComp=48`；
  - 未发现 `core/src/.../LocationPingComp.java` 或 `annotations/src/main/resources/revisions/LocationPingComp`；
  - 实际逻辑在 `InputHandler.pingLocation(...)` 与 `PlayerComp.drawPing()`：同队可见时写 `pingX/pingY/pingTime=1f/pingText`，文本按 `Vars.maxPingTextLength` 截断，`pingDuration = 20f * 60f`。
- Rust 主改动：
  - `entities::LOCATION_PING_CLASS_ID = 48`；
  - `PlayerComp::normalized_ping_text(...)`、`apply_ping_location(...)`、`advance_ping(...)`、`ping_alpha(...)`、`ping_draw_plan(...)`；
  - `PlayerPingDrawPlan` 作为 renderer 前置计划；
  - `input_handler::ping_location(...)` 复用 `PlayerComp` 的 ping 文本和 runtime 写入逻辑。
- 已跑：
  - `cargo test -p mindustry-core ping_location --lib`
  - `cargo test -p mindustry-core player --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo check -p mindustry-core`
- 下一步建议：
  1. `PowerGraphComp` / `PowerGraphUpdaterComp` 已由第 68 节推进；继续迁移 power graph 建图/并图/拆图与 Building 生命周期钩子。
  2. 将 `PlayerPingDrawPlan` 接到真实 graphics/UI renderer；当前只完成 Java 行为计划与 runtime 状态。
  3. 保持每个闭环完成后验证、更新 `MIGRATION.md`/`AI_HANDOFF.md`、中文提交并推送 `origin main`。

---

## 68. 最新闭环记录：PowerGraph runtime 与 updater 实体闭环

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：将 `PowerGraphUpdaterComp` 从独立泛型转发壳推进到可驱动真实 Rust power graph runtime。
- Java 依据：
  - `annotations/src/main/resources/classids.properties`：`PowerGraphComp=41`、`PowerGraphUpdaterComp=42`；
  - `core/src/mindustry/entities/comp/PowerGraphUpdaterComp.java`：`serialize=false, genio=false`，`transient PowerGraph graph`，`update(){ graph.update(); }`；
  - `annotations/src/main/resources/revisions/PowerGraphUpdaterComp/0.json`：空字段；
  - `PowerGraphComp.java` 当前参考树未找到，实际电网行为在 `core/src/mindustry/world/blocks/power/PowerGraph.java`。
- Rust 主改动：
  - `entities::POWER_GRAPH_CLASS_ID = 41`、`POWER_GRAPH_UPDATER_CLASS_ID = 42`；
  - 新增 `PowerProducer`、`PowerConsumer`、`PowerGraphRuntime`；
  - `PowerGraphRuntime::update_with_delta(...)` 聚合 produced/needed、battery use/charge、coverage、consumer status、lastScaled/lastStored/lastCapacity/powerBalance；
  - `PowerGraphRuntime::transfer_power(...)` 对齐 Java `transferPower` 的 battery 与 `energyDelta` 语义；
  - `PowerGraphUpdaterComp<PowerGraphRuntime>` 实现 `PowerGraphUpdate`，updater `update()` 现在能驱动真实 runtime。
- 已跑：
  - `cargo test -p mindustry-core power_graph_runtime --lib`
  - `cargo test -p mindustry-core power_graph_updater --lib`
  - `cargo test -p mindustry-core power_graph_beam_and_long_node_helpers_follow_upstream --lib`
  - `cargo test -p mindustry-core entity_class_ids_match_upstream_classids_properties_baseline --lib`
  - `cargo check -p mindustry-core`
- 下一步建议：
  1. `PowerGraph.addGraph/add/reflow/removeList/clear` 的最小 membership 行为已由第 69 节补上；继续迁移 Java `PowerGraph.remove(Building)` 的分支拆图。
  2. 将 `BuildingComp.onProximityAdded/updatePowerGraph/powerGraphRemoved/afterPickedUp` 接入 `PowerGraphRuntime`。
  3. 后续再考虑 `PowerGraphComp` 若生成产物恢复，应补实体壳/映射而不是只保留 class-id。

---

## 69. 最新闭环记录：PowerGraph membership/reflow 生命周期

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：在 `PowerGraphRuntime.update_with_delta(...)` 后继续补 Java `PowerGraph.add/addGraph/reflow/removeList/clear` 的 membership 管理。
- Java 依据：
  - `PowerGraph.add(Building)` 按 `outputsPower/consumesPower/buffered` 分类到 `producers/consumers/batteries/all`；
  - `PowerGraph.clear()` 清空四个列表；
  - `PowerGraph.reflow(Building)` 从起点经 `getPowerConnections(...)` BFS，closed set 去重；
  - `PowerGraph.removeList(Building)` 是 Java 测试 helper。
- Rust 主改动：
  - `PowerGraphNode` 作为 building/block power 输入视图；
  - `PowerGraphRuntime.all` 与 producer/consumer/battery 对应 node id 列表；
  - `add_node(...)`、`remove_list(...)`、`clear(...)`、`add_graph(...)`、`reflow_from(...)`；
  - `reflow_from(...)` 由 caller 提供 connections callback，先把 BFS 核心闭环，后续再接真实 `BuildingComp.getPowerConnections(...)`。
- 已跑：
  - `cargo test -p mindustry-core power_graph_runtime --lib`
  - `cargo test -p mindustry-core power_graph_updater --lib`
  - `cargo check -p mindustry-core`
- 下一步建议：
  1. `PowerGraph.remove(Building)` 的分支拆图核心已由第 70 节补上；继续把真实 `BuildingComp` 连接/生命周期接进来。
  2. 把 `BuildingComp` 的 proximity / pickup / remove / load 生命周期接到 `PowerGraphRuntime::add_node/reflow_from/remove_list`。
  3. 保持 power graph runtime 直接服务 world/building，而不是回退成孤立 helper。

---

## 70. 最新闭环记录：PowerGraph remove 分支拆图核心

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：迁移 Java `PowerGraph.remove(Building)` 的核心分支拆图逻辑。
- Java 依据：
  - `PowerGraph.remove(Building tile)` 遍历被移除 tile 的连接；
  - 对每个仍属于旧 graph 的邻接 building 新建分支 graph；
  - BFS 跳过被移除 tile，避免重复分配；
  - 每个新 graph 结束后立即 `update()`，旧 graph 失效。
- Rust 主改动：
  - `PowerGraphRuntime::remove_with_connections(...)`；
  - 由 caller 提供 connections 与 node lookup；
  - runtime 负责旧 membership 过滤、分支 BFS、新 graph 创建、新 graph update、旧 graph clear。
- 已跑：
  - `cargo test -p mindustry-core power_graph_runtime --lib`
  - `cargo check -p mindustry-core`
- 下一步建议：
  1. 将 `BuildingComp` 的真实邻接查询接入 `remove_with_connections(...)`。
  2. `BuildingComp` power graph lifecycle 入口已由第 71 节补上；继续把这些入口串入 GameRuntime/world 主链路。

---

## 71. 最新闭环记录：BuildingComp power graph lifecycle 接入点

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 仍禁止使用。
- 目标：把 Java `BuildingComp.updatePowerGraph/powerGraphRemoved/afterPickedUp` 的关键入口落到 Rust `BuildingComp`，为 `PowerGraphRuntime` 接真实 building 做准备。
- Java 依据：
  - `updatePowerGraph()` 合并邻接 power graph；
  - `powerGraphRemoved()` 调 `power.graph.remove(self())` 并清理 links；
  - `afterPickedUp()` 换新 graph、清空 links，非 buffered consumer status 置 0。
- Rust 主改动：
  - `BuildingComp::power_graph_node(...)`；
  - `BuildingComp::power_graph_removed_links(...)`；
  - `BuildingComp::after_picked_up_power(...)`；
  - 测试锁定 node 转换、links 清理、pickup 后 status 规则。
- 已跑：
  - `cargo test -p mindustry-core building_component_exposes_power_graph_node_and_lifecycle_helpers --lib`
  - `cargo test -p mindustry-core building_component --lib`
  - `cargo check -p mindustry-core`
- 下一步建议：
  1. 在 `GameRuntime` 中维护真实 `PowerGraphRuntime` 集合/索引，把 building proximity 刷新、删除、pickup 与 graph lifecycle 串起来。
  2. 继续减少 power graph helper 孤岛，把 runtime 接到 world/building 主调用链。

---

## 72. 最新闭环记录：v158.1 LandingPad waiting queue 运行时接入

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 目标：同步 Java `LandingPad.java` 在 v158.1 的 waiting queue 剪枝顺序，并接入 core/server/desktop 主链路，而不是只做孤立 helper。
- Java 依据：
  - `waiting.each((item, pads) -> { pads.removeAll(l -> l.config != item); if(pads.size > 0){ ... } })`
  - 剪枝必须发生在 size 判断之前；剪枝后为空不能 sort/first/Call。
- Rust 主改动：
  - `GameRuntime::landing_pad_waiting: BTreeMap<i16, Vec<i32>>`；
  - `advance_owned_landing_pads_ticks(...)`：更新 import cooldown、剪枝 stale config、选中 landing pad、驱动 arrival/liquid removal/item import/dump；
  - `GameRuntimeOwnedFrameReport.campaign.landing_pad`；
  - server 对 `landed_tiles` 广播 `LandingPadLandedCallPacket`；
  - desktop 新增 `sync_world_update_events_to_runtime()`，从 `NetClientState.last_world_update_packet` 回放 `LandingPadLandedCallPacket` 到 `GameRuntime::apply_client_landing_pad_landed_packet(...)`。
- 已跑：
  - `cargo test -p mindustry-core landing_pad --lib`
  - `cargo test -p mindustry-server landing_pad --lib`
  - `cargo test -p mindustry-desktop landing_pad --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 继续处理 v158.1 剩余 `UI.java` / `HudFragment.java` UI 差异；没有 Rust UI 层时只记录缺口，不要误改 runtime。
  2. 继续 UnitAssembler 深水区：`AssemblerAI.targetPos/targetAngle/inPosition()`、UnitPayload 投递目标建筑、effect/sound/event、Java↔Rust smoke。

---

## 73. 最新只读记录：v158.1 UI / HudFragment 差异

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用。
- 结论：`UI.java` 与 `HudFragment.java` 的 v158.1 差异均属于 UI/HUD 表现层；Rust 当前没有完整 `HudFragment` 对应模块，不要为了这些差异修改 `GameRuntime`。
- Java 差异：
  - `UI.showFollowUpMenu(...)`：callback 后若 `!state.isGame()` 则隐藏当前 menu/dialog；
  - `HudFragment`：sidebar 背景/颜色、health/shield/payload/ammo bar、status effect icon found 判定、无限时长 tooltip。
- Rust 当前相关路径：
  - `core/src/mindustry/ui/dialogs/*`
  - `core/src/mindustry/ui/displayable.rs`
  - `core/src/mindustry/input/desktop_input.rs`
  - `core/src/mindustry/input/mobile_input.rs`
- 下一步建议：
  1. 后续补 dialog stack/follow-up menu 时迁移 `!state.isGame() -> hide()`；
  2. 后续补 HUD renderer 时再迁移 `HudFragment` 绘制与 tooltip 差异；
  3. 目前继续主线 runtime/network/content 迁移，不要把 UI 差异写入 gameplay 层。

---

## 74. 最新闭环记录：UnitAssembler / AssemblerAI drone 真实到位接入

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `UnitAssembler.java` 与 `AssemblerAI.java`，把 UnitAssembler 生产进度从“所有 drone 视为到位”的临时逻辑改为按真实 drone 位置/角度计算。
- Java 依据：
  - `ai.targetPos.trns(i * 90f + 45f, areaSize / 2f * Mathf.sqrt2 * tilesize).add(spawn);`
  - `ai.targetAngle = i * 90f + 45f + 180f;`
  - `AssemblerAI.inPosition()`：10f 距离阈值 + 15f 角度阈值；
  - `eff = inPositionCount / dronesCreated` 后才推进 assembler progress。
- Rust 主改动：
  - `core/src/mindustry/world/blocks/units/mod.rs`
    - `UnitAssemblerDroneTarget`
    - `unit_assembler_drone_target(...)`
    - `unit_assembler_drone_in_position(...)`
  - `core/src/mindustry/core/game_runtime.rs`
    - `advance_owned_unit_assemblers_ticks(...)` 不再使用 `simulated_drones = drones_created`；
    - 按 `read_unit_ids` 的顺序去重、slot index 与 `client_unit_snapshot_entities` 里的 `UnitComp` pose 计算 `drones_in_position`；
    - `GameRuntimeUnitAssemblerFrameReport` 新增 `drones_in_position`；
    - 新增测试锁定无真实到位 drone 时 progress 不再推进。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` 在 owned runtime 前调用 `tick_runtime_unit_assembler_ai()`；
    - server-side assembler drone 朝 slot target 移动、接近后转向 targetAngle；
    - 移动后的 drone 同步回 `runtime.client_unit_snapshot_entities`，使 runtime 使用真实 pose 计算倍率。
- 已跑：
  - `cargo test -p mindustry-core assembler_geometry_tiers_acceptance_and_progress_follow_upstream --lib`
  - `cargo test -p mindustry-core unit_assembler --lib`
  - `cargo test -p mindustry-core assembler_module --lib`
  - `cargo test -p mindustry-server assembler --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `cargo check -p mindustry-desktop`
  - `cargo fmt --check`
- 当前仍需继续：
  1. `tick_runtime_unit_assembler_ai()` 是最小可玩近似，后续需更贴近 Java `AIController.moveTo(targetPos, 1f, 3f)` 的加速度/避障/旋转细节；
  2. 继续迁移 `UnitAssembler.spawned()` 的输出 unit 依据 `unit.buildOn()` 投递到出生点建筑 payload 逻辑；注意 `commandPos` 只写入新 unit 的 command controller，不用于选择 payload 目标建筑；
  3. 继续把 `AssemblerAI` 从 helper 进一步实体化为正式 controller runtime state，避免长期依赖 snapshot sidecar。

---

## 75. 最新闭环记录：UnitAssembler.spawned 输出 unit 的 buildOn payload 投递

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `UnitAssembler.java` 的 `UnitAssemblerBuild.spawned()`，把组装完成的输出 unit 接入真实 payload 目标建筑，而不是无条件作为普通 server unit 落地。
- Java 依据：
  - `commandPos` 只在 `unit.isCommandable()` 时写入输出 unit 的 command controller；
  - payload 目标来自 `unit.buildOn()`，即输出 unit 当前出生点所在建筑；
  - 目标建筑同队且 `acceptPayload(targetBuild, payload)` 成功时调用 `handlePayload(targetBuild, payload)`；这里 source/target 都是 `targetBuild`；
  - payload 投递失败且非 client 时才 `unit.add()` / `Units.notifyUnitSpawn(unit)`。
- Rust 主改动：
  - `server/src/lib.rs`
    - `apply_runtime_unit_assembler_spawns(...)` 创建 output `UnitComp` 后先尝试 `try_deliver_runtime_spawned_unit_payload(...)`；
    - payload 成功则不插入 `server_units`，失败才保留旧的 `server_units.insert(...)`；
    - `try_deliver_runtime_spawned_unit_payload(...)` 复用 `unit_entered_payload(...)` 与 `runtime.attach_unit_payload_to_building(...)`，成功后广播 `UnitEnteredPayloadCallPacket`；
    - `server_unit_build_on_tile_pos(...)` 增加 `footprint_tiles(...)` fallback，以覆盖多格 payload building 的 footprint。
- 已跑：
  - `cargo test -p mindustry-server server_launcher_unit_assembler_spawn_delivers_payload_to_build_on_target --lib`
  - `cargo test -p mindustry-server assembler --lib`
  - `cargo check -p mindustry-server`
  - `cargo fmt --check`
  - `git diff --check`
- 下一步建议：
  1. 继续 `UnitAssembler.spawned()` 剩余副作用：`createSound`、`Fx.unitAssemble`、`Events.fire(new UnitCreateEvent(unit, this))`；fallback `Units.notifyUnitSpawn(unit)` 已由第 76 节接入；
  2. 在 Rust 现有 sound/effect/event/network 包中寻找可复用 call packet 或事件分发入口，不要把副作用停留成孤立 helper；
  3. `AssemblerAI` 仍需从 helper + snapshot sidecar 继续实体化为正式 controller runtime state；
  4. 每个闭环继续按：对照 Java → 接入 runtime/server/client/network → 测试 → 更新本文档与 `MIGRATION.md` → 中文 commit → push `origin main`。

---

## 76. 最新闭环记录：UnitAssembler.spawned fallback UnitSpawn 同步

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：迁移 `UnitAssemblerBuild.spawned()` 中 payload 投递失败后的 `Units.notifyUnitSpawn(unit)`，保证 Rust 客户端不必等下一次 snapshot 就能看到 assembler 输出 unit。
- Java 依据：
  - `Units.notifyUnitSpawn(unit)` 在 server 侧发送 `Call.unitSpawn(new UnitSyncContainer(unit))`；
  - `unitSpawn` 是 server→client、unreliable、low priority；
  - 该分支只在 `targetBuild == null`、不同队或 `acceptPayload/handlePayload` 失败时执行，payload 成功时不发送普通 unit spawn。
- Rust 主改动：
  - `server/src/lib.rs`
    - `apply_runtime_unit_assembler_spawns(...)` 在 payload 投递失败时广播 `UnitSpawnCallPacket`，随后将 unit 插入 `server_units`；
    - `server_unit_spawn_packet(...)` 用 `entity_class_id(unit.type_info.name())` 与 `UnitComp::to_sync_wire()` 生成 `type_io::UnitSyncContainer`；
  - `core/src/mindustry/core/game_runtime.rs`
    - `apply_client_unit_spawn_packet(...)` 验证 class id 是 Unit，严格解码 `UnitSyncWire`，并复用 `apply_client_unit_sync_wire(...)` materialize/update `client_unit_snapshot_entities`；
  - `core/src/mindustry/core/net_client.rs`
    - `UnitSpawnCallPacket` 进入独立 `unit_spawn_packets` 队列，不再覆盖 `last_unit_lifecycle_packet`，避免同帧 `AssemblerUnitSpawnedCallPacket` 被覆盖；
  - `desktop/src/lib.rs`
    - 新增 `last_applied_unit_spawn_packet_count` cursor；
    - `update()` 先同步 unit lifecycle，再回放 unit spawn packets，让 assembler state reset 与 output unit materialize 同时成立。
- 已跑：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_spawn_packet_sync_container --lib`
  - `cargo test -p mindustry-core update_records_unit_spawn_separately_from_lifecycle_tail --lib`
  - `cargo test -p mindustry-server server_update_broadcasts_assembler_unit_spawn_packet_when_assembler_completes --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_spawn_packet_without_losing_assembler_spawned --lib`
- 下一步建议：
  1. `createSound.at(...)` 与 `Fx.unitAssemble.at(...)` 的客户端本地 sidecar 已由第 77 节接入；下一步把这些 sidecar 接到 desktop 实际 audio/renderer backend；
  2. 继续 `Events.fire(new UnitCreateEvent(unit, this))`：先找 Rust campaign stats/event 侧可接入口，不要只写孤立计数 helper。

---

## 77. 最新闭环记录：UnitAssembler.spawned 客户端本地 sound/effect 副作用

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：迁移 `UnitAssemblerBuild.spawned()` 中 `createSound.at(...)` 与 `Fx.unitAssemble.at(...)`，但保持 Java remote call 语义，不让 Rust server 给 Java client 额外重复发 sound/effect packet。
- Java 依据：
  - `Call.assemblerUnitSpawned(tile)` 到客户端后执行 `build.spawned()`，sound/effect 是客户端本地副作用；
  - `createSound = Sounds.unitCreateBig`，v158.1 `AssetsProcess.processSounds(...)` 生成 id：`unitCreateBig == 191`；
  - `Fx.unitAssemble` 在 `Effect.all` 中 id 为 `35`，data 是 output `UnitType`。
- Rust 主改动：
  - `core/src/mindustry/audio/mod.rs`
    - `standard_sound_id("unitCreate") == 190`
    - `standard_sound_id("unitCreateBig") == 191`
  - `core/src/mindustry/entities/effect.rs`
    - `FX_UNIT_ASSEMBLE_ID = 35`
  - `core/src/mindustry/core/game_runtime.rs`
    - `GameRuntime.client_local_sound_at_events: Vec<SoundAtCallPacket>`
    - `GameRuntime.client_local_effect_events: Vec<EffectCallPacket2>`
    - `apply_client_assembler_unit_spawned_packet(...)` 在 reset assembler 前，根据 building rotation/assembler area size 计算 spawn 点并排队本地 sound/effect；
    - sound 使用 `create_sound_volume` 和确定性 pitch `1.0`；effect 使用 `rotdeg - 90`、white color、`TypeValue::Content(UnitType)` data。
  - `desktop/src/lib.rs`
    - 既有 `AssemblerUnitSpawnedCallPacket` 回放路径现在会让 runtime 记录上述本地 aftereffects。
- 已跑：
  - `cargo test -p mindustry-core standard_sound_ids_follow_upstream_assets_process_order --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_assembler_unit_spawned_packet_like_java --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_assembler_unit_spawned_packet_to_runtime --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_spawn_packet_without_losing_assembler_spawned --lib`
- 下一步建议：
  1. 把 `client_local_sound_at_events` / `client_local_effect_events` 接到 desktop 实际播放/渲染层，当前只是 runtime sidecar；
  2. 恢复 Java `1f + Mathf.range(0.06f)` 的客户端 pitch 随机；
  3. 继续迁移 `Events.fire(new UnitCreateEvent(unit, this))` 到 Rust campaign stats/event bus。

---

## 78. 最新闭环记录：UnitAssembler.spawned UnitCreateEvent / 创建统计

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：迁移 `UnitAssemblerBuild.spawned()` 末尾 `Events.fire(new UnitCreateEvent(unit, this))` 的最小真实语义，不新增独立孤岛模块。
- Java 依据：
  - `EventType.UnitCreateEvent`：`unit`、`spawner`、`spawnerUnit`；
  - `Logic`：default team unit 增加 `state.stats.unitsCreated`；campaign 且非 net client 时增加 planet `unitsProduced`；
  - `GameService`：campaign default team 更新 `unitTypesBuilt` / `buildT5`，player team 增加 `SStat.unitsBuilt`。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `GameRuntimeUnitCreateEvent`；
    - `GameRuntime` 新增 `campaign_stats: CampaignStats` 与 `unit_create_events`；
    - 新增 `note_unit_create_event(...)`：default team 增加 `state.stats.units_created`；campaign 且 offline/server 权威端增加 `campaign_stats.units_produced`；记录 sidecar；
    - `advance_owned_unit_assemblers_ticks(...)` 在 assembler 完成后调用；
    - `apply_client_assembler_unit_spawned_packet(...)` 在 client 回放时调用，但 client 不写 campaign stats；
    - world reset 清理 `unit_create_events`。
  - `core/src/mindustry/service/game_service.rs`
    - `GameServiceUnitCreateSnapshot` 增加 `player_team_unit`；
    - `GameServiceUnitCreatePlan` 增加 `stat_additions`；
    - `unit_create_plan(...)` 能区分 Java 的 default-team `unitTypesBuilt/buildT5` 与 player-team `SStat.unitsBuilt`。
  - `server/src/lib.rs`
    - 更新 assembler 完成测试，验证 server runtime 侧 `unit_create_events`、`state.stats.units_created` 与 `campaign_stats.units_produced`。
- 已跑验证：
  - `cargo test -p mindustry-core assembler_unit_spawned --lib`
  - `cargo test -p mindustry-core game_runtime_owned_runtime_blocks_includes_unit_assembler_tick_like_java --lib`
  - `cargo test -p mindustry-core unit_create_plan --lib`
  - `cargo test -p mindustry-server server_update_broadcasts_assembler_unit_spawn_packet_when_assembler_completes --lib`
- 当前仍需继续：
  1. 跑 `cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check` 后提交推送。
  2. 后续把 `unit_create_events` drain/bridge 到正式事件 bus / platform service runtime；目前只是 runtime sidecar + stats。
  3. 继续把同一个 `note_unit_create_event(...)` 接到 `UnitFactory`、`Reconstructor`、`PayloadSource`、`UnitSpawnAbility`，避免只有 assembler 计数。
  4. `spawner_unit_id` 后续给 `UnitSpawnAbility` 等 unit-spawner 路径补齐。

---

## 79. 最新闭环记录：UnitFactory / Reconstructor UnitCreateEvent

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（`v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：把 `UnitCreateEvent` 的统一统计入口从 UnitAssembler 扩展到 Java 也会发事件的 `UnitFactory` 和 `Reconstructor`。
- Java 依据：
  - `UnitFactoryBuild.updateTile()`：创建 `plan.unit`、写 command、生成 `UnitPayload`、`consume()` 后 `Events.fire(new UnitCreateEvent(payload.unit, this))`；
  - `ReconstructorBuild.updateTile()`：升级 payload unit、写 command、效果、`consume()` 后 `Events.fire(new UnitCreateEvent(payload.unit, this))`。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - `advance_owned_unit_factories_ticks(...)` 在 payload 成功生成并 consume 后调用 `note_unit_create_event(None, unit_name, team, Some(factory_tile), None)`；
    - `advance_owned_unit_reconstructors_ticks(...)` 在 payload patch 成功并 consume 后调用同一入口；
    - Reconstructor target upgrade tuple 增加升级后 unit name，用于事件统计；
    - 扩展 `game_runtime_unit_factory_outputs_payload_to_front_conveyor` 与 `game_runtime_unit_reconstructor_upgrades_payload_on_tick_like_java`，断言 sidecar、`units_created`、`campaign_stats.units_produced`。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_unit_factory_outputs_payload_to_front_conveyor --lib`
  - `cargo test -p mindustry-core game_runtime_unit_reconstructor_upgrades_payload_on_tick_like_java --lib`
- 当前仍需继续：
  1. 跑 `cargo check -p mindustry-core`、`cargo fmt --check`、`git diff --check` 后中文提交并推送。
  2. 下一闭环优先做 `PayloadSource` 的 unit payload 分支：只在配置为 unit 时发 UnitCreateEvent，配置为 block 时不能发。
  3. `UnitSpawnAbility` 最后接，那里需要 `spawner_unit_id` 非空，并且不属于 block owned tick 路径。

---

## 80. 最新闭环记录：PayloadSource unit payload UnitCreateEvent

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（`v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：迁移 `PayloadSourceBuild.updateTile()` 中 unit payload 创建时的 `UnitCreateEvent`，同时保证 block payload 分支不误计数。
- Java 依据：
  - `unit != null`：`new UnitPayload(unit.create(team))`，可选 commandPos，然后 `Events.fire(new UnitCreateEvent(p, this))`；
  - `configBlock != null`：只创建 `BuildPayload`，不发 UnitCreateEvent。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - `advance_owned_payload_sources_ticks(...)` 在 `PayloadSourceSpawn::Unit` 成功创建 `PayloadRef::Unit` 后记录 unit 创建事件，并在 borrow 结束后调用 `note_unit_create_event(...)`；
    - `PayloadSourceSpawn::Block` 不调用事件入口；
    - `game_runtime_payload_source_spawns_configured_block_payload` 断言 block 分支无事件/无 `units_created`；
    - `game_runtime_payload_source_spawns_common_unit_payload_with_command_pos` 断言 unit 分支记录 sidecar、`units_created`、campaign `units_produced`，且原有 commandPos payload 编码仍正确。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_payload_source_spawns_configured_block_payload --lib`
  - `cargo test -p mindustry-core game_runtime_payload_source_spawns_common_unit_payload_with_command_pos --lib`
- 当前仍需继续：
  1. 跑 `cargo check -p mindustry-core`、`cargo fmt --check`、`git diff --check` 后中文提交并推送。
  2. 下一步处理 `UnitSpawnAbility`，这是 `spawner_unit_id` 非空的 UnitCreateEvent，不要硬塞进 block runtime helper。
  3. 后续把 `unit_create_events` bridge 到正式 event bus / `DefaultGameService` / achievement backend。

---

## 81. 最新闭环记录：UnitSpawnAbility 单位产子接入真实 UnitComp/server 链路

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已 fetch 并确认 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：迁移 `UnitSpawnAbility.update(Unit unit)` 的单位产子主链路：父单位 ability timer → `Units.canCreate` → 子单位创建 → `UnitCreateEvent(u, null, parent)` → server `UnitSpawnCallPacket`。
- Java 依据：
  - `UnitSpawnAbility.java:45-60`：累积 `Time.delta * unitBuildSpeed`，到点且 `Units.canCreate` 后创建子单位、设置位置/旋转、触发 `UnitCreateEvent(u, null, unit)`，非 client 端 `u.add(); Units.notifyUnitSpawn(u)`，再清零 timer。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - `UnitSpawnAbility::from_descriptor(...)` 支持 `UnitSpawnAbility:unit:spawnTime:spawnX:spawnY[:parentize]` 与 `UnitSpawnAbility(unit, spawnTime, spawnX, spawnY)`；
    - 新增 descriptor 解析测试。
  - `core/src/mindustry/entities/comp/unit.rs`
    - `UnitComp::update_unit_spawn_abilities(...)` 从 `type_info.abilities` 找 `UnitSpawnAbility` descriptor；
    - 用 `AbilityWire.data` 存 timer，同步 Java cap 阻塞时 timer 保持 ready 的行为；
    - 返回真实 `UnitSpawnPlan`，不直接碰全局 runtime/network。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` 在 owned runtime frame 内调用 `tick_server_unit_spawn_abilities(1.0)`；
    - server 对 `server_units` 的父单位逐个 tick unit spawn ability；
    - 用 `units_can_create(...)` 按当前 `server_units` 统计同 team/type 数量，结合 rules cap / banned unit 判断；
    - 产子成功后创建 `UnitComp`、`unit.add()`、记录 `note_unit_create_event(Some(child_id), unit, team, None, Some(parent_id))`，并复用 `broadcast_server_unit_spawn(...)` 发送 `UnitSpawnCallPacket`。
- 已跑验证：
  - `cargo test -p mindustry-core unit_spawn --lib`
  - `cargo test -p mindustry-server unit_spawn_ability --lib`
- 当前仍需继续：
  1. 跑全量收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`，然后中文提交并推送 `origin main`。
  2. 后续将普通 `UnitType.abilities: Vec<String>` 升级为结构化 ability content / mod patcher 表达，避免 descriptor 长期作为正式模型。
  3. 继续接 Java client 本地 ability tick / draw construct preview / `spawnEffect.at(...)` 表现层，以及 `unit_create_events` 到正式 event bus / service backend 的 bridge。
  4. 下一个迁移候选可按探索结果继续：`EnergyFieldAbility`、`ShieldArcAbility`、`MoveEffectAbility`、`StatusFieldAbility`、`SuppressionFieldAbility`，优先选择能接入真实 runtime 的闭环。

---

## 82. 最新闭环记录：EnergyFieldAbility / aegires server unit runtime

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（`v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `EnergyFieldAbility.java` 与 `UnitTypes.aegires`，把 EnergyField 从纯算法接入真实 content、`UnitComp` ability slot 和 server-side unit↔unit heal/damage/status runtime。
- Java 依据：
  - `EnergyFieldAbility.update(...)`：timer 到 `reload` 后收集附近目标，按距离排序，最多 `maxTargets`；
  - 同队受损目标治疗，治疗量 `healPercent / 100 * maxHealth`，同类型乘 `sameTypeHealMult`；
  - 敌对目标造成 `damage * unitDamage * damageMultiplier` 并应用 `status/statusDuration`；
  - `aegires` 参数：`damage=40`、`reload=65`、`range=180`、`statusDuration=360`、`maxTargets=25`、`healPercent=1.5`、`sameTypeHealMult=0.5`。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - `EnergyFieldTarget` 增加 `air/targetable`，用于 runtime 调度层过滤；
    - `EnergyFieldHit` 增加 `status_duration`；
    - `EnergyFieldAbility::from_descriptor(...)` 支持 `EnergyFieldAbility:40:65:180:1.5:0.5:25`；
    - 纯逻辑测试增加 descriptor 与 `status_duration` 断言。
  - `core/src/mindustry/entities/comp/unit.rs`
    - `UnitComp::update_energy_field_abilities(...)` 使用 `AbilityWire.data` 存 timer，调用 `EnergyFieldAbility::update_targets(...)`，并按 pulse 回写 ammo/timer；
    - 新增 UnitComp runtime slot 测试。
  - `core/src/mindustry/content/unit_types.rs`
    - `aegires` 挂载 `EnergyFieldAbility:40:65:180:1.5:0.5:25`；
    - 内容测试断言能力 descriptor 存在。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` 在同一 playing frame 内调用 `tick_server_energy_field_abilities(1.0)`；
    - server 从 `server_units` 收集目标，应用到真实 `HealthComp` / `StatusComp`；
    - 新增 `server_update_ticks_aegires_energy_field_against_units`：验证 parent timer 清零、同队 aegires heal 90、敌对 flare damage 40 并获得 `electrified` 360 tick。
- 已跑验证：
  - `cargo test -p mindustry-core energy_field --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server energy_field --lib`
- 注意：
  - v158.1 `EnergyFieldAbility.java` 有 `useAmmo` 字段，但当前 `update(...)` 未实际扣 ammo；server runtime 本轮传 `unit_ammo_rule=false`，保持 Java v158.1 观测行为。纯算法仍保留 ammo gate 以兼容既有测试/后续版本差异。
- 当前仍需继续：
  1. 跑全量收尾验证并提交推送。
  2. 后续补 `hitBuildings`、building privileged / derelict coreCapture、`Damage.findAbsorber(...)`。
  3. 后续补 EnergyField draw/effect/sound 表现层，以及结构化 ability content / mod patcher。
  4. 下一个优先候选：`ShieldArcAbility` 或 `StatusFieldAbility`，继续选择能接真实 runtime 的闭环。

---

## 81. 最新闭环记录：UnitSpawnAbility 单位产子 runtime / UnitCreateEvent

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已 fetch 确认 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `UnitSpawnAbility.update(Unit unit)`，把既有 Rust `UnitSpawnAbility` 从纯 plan/单测接入真实 `UnitComp` ability slot、server update、`UnitCreateEvent(spawnerUnit)` 与 `UnitSpawnCallPacket`。
- Java 依据：
  - `timer += Time.delta * state.rules.unitBuildSpeed(unit.team)`；
  - 到达 `spawnTime` 且 `Units.canCreate(...)` 时，按父单位旋转偏移创建子单位；
  - `Events.fire(new UnitCreateEvent(u, null, unit))`；
  - 非 client 端 `u.add(); Units.notifyUnitSpawn(u)`，随后 `timer = 0f`。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - `UnitSpawnAbility::from_descriptor(...)` 支持 `UnitSpawnAbility:unit:spawnTime:spawnX:spawnY[:parentize]` 与 `UnitSpawnAbility(unit, spawnTime, spawnX, spawnY)`，适配当前 `UnitType.abilities: Vec<String>` 过渡模型；
    - 新增 descriptor parse 测试。
  - `core/src/mindustry/entities/comp/unit.rs`
    - 新增 `UnitComp::update_unit_spawn_abilities(...)`；
    - 从 `AbilityWire.data` 读取/写回 timer，按父单位 `x/y/rotation` 产出 `UnitSpawnPlan`；
    - cap 阻止时 timer 保持 ready，cap 放开后 `delta=0` 也能产子，匹配 Java。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` 在 playing frame 的 owned runtime update 后调用 `tick_server_unit_spawn_abilities(1.0)`；
    - server 侧按 `server_units` 当前同 team/type 数量 + `units_can_create(...)` 判断是否允许创建；
    - 创建子 `UnitComp` 后记录 `note_unit_create_event(Some(child_id), unit, team, None, Some(parent_id))`，随后 `unit.add()`、复用 `broadcast_server_unit_spawn(...)` 与现有 `UnitSpawnCallPacket`；
    - 新增 `server_update_ticks_unit_spawn_ability_and_broadcasts_spawned_unit`，验证实体落地、packet、`spawner_unit_id`、`units_created` 与 campaign `units_produced`。
- 已跑验证：
  - `cargo test -p mindustry-core unit_spawn --lib`
  - `cargo test -p mindustry-server unit_spawn_ability --lib`
- 下一步：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 更新提交并推送 `origin main`，中文提交标题建议：`接入单位产子能力运行时`。
  3. 后续候选：优先从 `EnergyFieldAbility` / `ShieldArcAbility` / `StatusFieldAbility` 继续做真实 entity ability runtime 闭环；不要把能力继续做成孤立纯 helper。
  4. 长期欠账：普通 `UnitType.abilities` 需要结构化 content/mod patcher，`spawnEffect/draw` 表现层和 `unit_create_events` → 正式 event bus/service bridge 仍未完成。

---

## 83. 最新闭环记录：StatusFieldAbility / oxynoe server unit runtime

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `StatusFieldAbility.java` 与 `UnitTypes.oxynoe`，把状态场从纯 pulse 接入 `UnitType` content、`UnitComp` ability slot、`StatusComp` 与 server same-team unit runtime。
- Java 依据：
  - `StatusFieldAbility.update(Unit unit)`：`timer += Time.delta`，当 `timer >= reload` 且满足 `!onShoot || unit.isShooting` 时，对 `Units.nearby(unit.team, unit.x, unit.y, range, ...)` 内同队单位执行 `other.apply(effect, duration)`；
  - active effect 坐标按 `effectX/effectY` 与 `unit.rotation` 计算，参数为 `effectSizeParam ? range : unit.rotation`，最后 `timer = 0f`；
  - `oxynoe` 参数：`StatusFieldAbility(StatusEffects.overclock, 60f * 6, 60f * 6f, 60f)`，即 `overclock / duration=360 / reload=360 / range=60`。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - `StatusFieldPulse` 增加 `target_ids`，让 server runtime 可把 pulse 直接落到真实实体；
    - `StatusFieldAbility::from_descriptor(...)` 支持 `StatusFieldAbility:overclock:360:360:60` 与括号形式；
    - 新增 oxynoe descriptor 解析测试。
  - `core/src/mindustry/content/unit_types.rs`
    - 为 `oxynoe` 挂载 `StatusFieldAbility:overclock:360:360:60`；
    - 内容覆盖测试断言该 descriptor 存在。
  - `core/src/mindustry/entities/comp/unit.rs`
    - 新增 `UnitComp::update_status_field_abilities(...)`；
    - 使用 `AbilityWire.data` 保存 timer，调用方闭包提供目标 id，保留 `on_shoot` 与 active effect 坐标/参数计算语义。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` 的 playing frame 内调用 `tick_server_status_field_abilities(1.0)`；
    - server 从 `server_units` 收集同队、存活、范围内目标（包含自身，匹配 Java `Units.nearby` 未排除 self 的路径），对每个 `pulse.target_ids` 执行 `target.status.apply(effect, duration)` 并刷新组件视图；
    - 新增 `server_update_ticks_oxynoe_status_field_for_nearby_allies`，验证父单位/近距离同队获得 `overclock`，远同队与敌队不受影响。
- 已跑验证（局部已通过，收尾前仍需重跑完整验证）：
  - `cargo test -p mindustry-core status_field --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server status_field --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo fmt`、上述局部测试、`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入状态场单位能力运行时`。
  3. 后续补 `applyEffect` / `activeEffect` 的真实 effect packet 或 desktop 表现层、client 本地 ability tick、结构化 ability spec / mod patcher。

---

## 84. 最新闭环记录：SuppressionFieldAbility / server building heal suppression runtime

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（`v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `SuppressionFieldAbility.java`、`Damage.applySuppression(...)` 和 `UnitTypes` 中 navanax / quell / disrupt，把治疗抑制场接入 `UnitComp` ability slot 与 server unit→building runtime。
- Java 依据：
  - `SuppressionFieldAbility.update(Unit unit)`：`active` 为真时累积 `timer += Time.delta`，达到 `maxDelay` 后按 `x/y` 相对 `unit.rotation - 90f` 旋转求中心，调用 `Damage.applySuppression(unit.team, center, range, reload, maxDelay, ...)`，再 `timer = 0f`；
  - `Damage.applySuppression(...)` 对范围内敌方建筑调用 `build.applyHealSuppression(reload + 1f, effectColor)`；
  - `navanax`：默认 `reload=90/maxDelay=90/range=200`，`y=-10`；
  - `quell`：`reload=480`，`maxDelay=90`，`range=200`，`y=1`；
  - `disrupt`：主场 `reload=900/range=320/y=10`，两侧还有 `active=false` 的纯视觉副本。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - 新增 `SuppressionFieldAbility::from_descriptor(...)`，支持 `SuppressionFieldAbility:reload:maxDelay:range:x:y:active:applyParticleChance` 与括号形式；
    - 新增 descriptor 解析测试。
  - `core/src/mindustry/content/unit_types.rs`
    - 为 `navanax`、`quell`、`disrupt` 挂载 Java 参数对应 descriptor；
    - `disrupt` 保留 1 个 active 主场 + 2 个 inactive 视觉副本 descriptor；
    - 内容覆盖测试断言这些 descriptor 存在。
  - `core/src/mindustry/entities/comp/unit.rs`
    - 新增 `UnitComp::update_suppression_field_abilities(...)`；
    - 使用 `AbilityWire.data` 保存 timer，按单位 transform 返回 `SuppressionFieldPulse`。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` 的 playing frame 内调用 `tick_server_suppression_field_abilities(1.0)`；
    - server 对 pulse 范围内敌方 `runtime.buildings` 调用 `apply_heal_suppression(now, reload + 1)`；
    - 新增 `server_update_ticks_quell_suppression_field_for_enemy_buildings`，验证近距离敌方建筑被抑制，同队与范围外建筑不受影响。
- 已跑局部验证：
  - `cargo test -p mindustry-core suppression_field --lib`
  - `cargo test -p mindustry-core unit_component_ticks_suppression_field --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server suppression_field --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入治疗抑制单位能力运行时`。
  3. 后续补 `Fx.regenSuppressSeek` 延迟粒子、`effectColor/suppress_color_rgba` 表现层、client draw orb/particles、结构化 ability spec / mod patcher。

---

## 85. 最新闭环记录：ShieldRegenFieldAbility / scepter-pulsar-bryde server shield runtime

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（`v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `ShieldRegenFieldAbility.java` 与 `UnitTypes` 中 scepter / pulsar / bryde，把护盾回复场接入 `UnitType` content、`UnitComp` ability slot、server same-team unit shield runtime。
- Java 依据：
  - `ShieldRegenFieldAbility.update(Unit unit)`：`timer += Time.delta`，达到 `reload` 后遍历 `Units.nearby(unit.team, unit.x, unit.y, range, ...)`；
  - 对 `other.shield < max` 的同队单位执行 `other.shield = min(other.shield + amount, max)` 与 `other.shieldAlpha = 1f`；
  - 任一目标实际获得护盾时播放 `applyEffect` / `activeEffect` / `sound`，最后 `timer = 0f`；
  - Java 参数：`scepter(25,250,60,60)`、`pulsar(20,40,300,60)`、`bryde(20,40,240,60)`。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - `ShieldRegenFieldPulse` 增加 `target_ids`；
    - 新增 `ShieldRegenFieldAbility::from_descriptor(...)`，支持 `ShieldRegenFieldAbility:amount:max:reload:range[:parentizeEffects]` 与括号形式；
    - 新增 descriptor 解析测试。
  - `core/src/mindustry/content/unit_types.rs`
    - 将 scepter / pulsar / bryde 的裸 `ShieldRegenFieldAbility` 替换为 Java 参数化 descriptor；
    - 内容覆盖测试断言三个 descriptor 存在。
  - `core/src/mindustry/entities/comp/unit.rs`
    - 新增 `UnitComp::update_shield_regen_field_abilities(...)`；
    - 使用 `AbilityWire.data` 保存 timer，调用方闭包按 ability range 提供同队目标与当前 shield。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` 的 playing frame 内调用 `tick_server_shield_regen_field_abilities(1.0)`；
    - server 对同队、存活、范围内 `server_units`（包含自身）写回 `ShieldComp.shield`，实际增加时设置 `shield_alpha = 1.0` 并刷新组件视图；
    - 新增 `server_update_ticks_scepter_shield_regen_field_for_nearby_allies`，验证 parent 自身与近同队获得/封顶护盾，远同队和敌队不受影响。
- 已跑局部验证：
  - `cargo test -p mindustry-core shield_regen_field --lib`
  - `cargo test -p mindustry-core unit_component_ticks_shield_regen_field --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server shield_regen_field --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入护盾回复单位能力运行时`。
  3. 后续补 `applyEffect` / `activeEffect` / `sound`、`UnitType.shieldColor`、client local ability tick 与结构化 ability spec / mod patcher。

---

## 86. 最新闭环记录：RepairFieldAbility / nova-poly-oct server health runtime

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（`v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `RepairFieldAbility.java` 与 `UnitTypes` 中 nova / poly / oct，把单位治疗场接入 `UnitType` content、`UnitComp` ability slot、server same-team unit health runtime。
- Java 依据：
  - `RepairFieldAbility.update(Unit unit)`：`timer += Time.delta`，达到 `reload` 后遍历 `Units.nearby(unit.team, unit.x, unit.y, range, ...)`；
  - 若目标 `other.damaged()`，播放 `healEffect` 并设置 `wasHealed = true`；
  - 对范围内同队目标执行 `other.heal((amount + healPercent / 100f * other.maxHealth()) * healMult)`，同类型目标使用 `sameTypeHealMult`；
  - 任一目标受治疗时播放 `activeEffect` / `sound`，最后 `timer = 0f`；
  - Java 参数：`nova(10,240,60)`、`poly(5,480,50)`、`oct(130,120,140)`。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - `RepairFieldPulse` 增加 `target_ids`；
    - 新增 `RepairFieldAbility::from_descriptor(...)`，支持 `RepairFieldAbility:amount:reload:range[:healPercent[:sameTypeHealMult[:parentizeEffects]]]` 与括号形式；
    - 新增 descriptor 解析测试。
  - `core/src/mindustry/content/unit_types.rs`
    - 将 nova 的裸 `RepairFieldAbility` 替换为 `RepairFieldAbility:10:240:60`；
    - 为 poly / oct 补上 `RepairFieldAbility:5:480:50` 与 `RepairFieldAbility:130:120:140`；
    - 内容覆盖测试断言三者存在。
  - `core/src/mindustry/entities/comp/unit.rs`
    - 新增 `UnitComp::update_repair_field_abilities(...)`；
    - 使用 `AbilityWire.data` 保存 timer，调用方闭包按 ability range 提供同队目标、damaged/maxHealth/sameType。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` 的 playing frame 内调用 `tick_server_repair_field_abilities(1.0)`；
    - server 对同队、存活、范围内 `server_units`（包含自身）执行 `heal_mark(...)` 与 `HealthComp::heal(...)`，然后刷新组件视图；
    - 新增 `server_update_ticks_nova_repair_field_for_nearby_allies`，验证 parent 自身与近同队获得治疗，远同队和敌队不受影响。
- 已跑局部验证：
  - `cargo test -p mindustry-core repair_field --lib`
  - `cargo test -p mindustry-core unit_component_ticks_repair_field --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server repair_field --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入单位治疗场能力运行时`。
  3. 后续补 `healEffect` / `activeEffect` / `sound` 表现层、client local ability tick、结构化 ability spec / mod patcher。

---

## 87. 最新闭环记录：ForceFieldAbility / quasar-oct server shield created+regen runtime

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（`v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `ForceFieldAbility.java` 与 `UnitTypes` 中 quasar / oct，把 ForceField 的 created shield 与 per-tick shield regen 接入 `UnitType` content、`UnitComp` hook、server unit shield runtime。
- Java 依据：
  - `created(Unit unit)`：`unit.shield = max`；
  - `update(Unit unit)`：护盾刚破时扣除 `cooldown * regen`，随后按 `Time.delta * regen` 回复；`alpha` 衰减，`radiusScale` 随 active shield 向 1 插值；
  - 护盾为正时扫描敌方 absorbable bullets，命中正多边形内则 `b.absorb()` 并按 `b.type().shieldDamage(b)` 扣盾；
  - Java 参数：`quasar(60,0.4,500,360)`、`oct(140,4,7000,480,8,0)`。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - 新增 `ForceFieldAbility::from_descriptor(...)`，支持 `ForceFieldAbility:radius:regen:max:cooldown[:sides[:rotation]]` 与括号形式；
    - 新增 descriptor 解析测试；既有 `absorb_bullet(...)` 纯算法测试仍保留。
  - `core/src/mindustry/content/unit_types.rs`
    - 将 quasar 的裸 `ForceFieldAbility` 替换为 `ForceFieldAbility:60:0.4:500:360`；
    - 为 oct 补上 `ForceFieldAbility:140:4:7000:480:8:0`，并保留 `RepairFieldAbility:130:120:140`；
    - 内容覆盖测试断言 quasar/oct descriptor 存在。
  - `core/src/mindustry/entities/comp/unit.rs`
    - `UnitComp::new(...)` 调用 `apply_created_force_field_abilities()`，初始化 ForceField unit 的 `ShieldComp.shield = max`；
    - 新增 `UnitComp::update_force_field_abilities(...)`，使用 `AbilityWire.data` 保存 `radius_scale`（负值作为已初始化但破盾/无半径 sentinel），并把 `ForceFieldUpdate.shield` 写回 `ShieldComp.shield`。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` 的 playing frame 内调用 `tick_server_force_field_abilities(1.0)`；
    - 新增 `server_update_ticks_quasar_force_field_regen`，验证 quasar 创建即有 500 shield，server tick 后按 0.4/tick 回复并写入 ability runtime slot。
- 已跑局部验证：
  - `cargo test -p mindustry-core force_field --lib`
  - `cargo test -p mindustry-core unit_component_ticks_force_field --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server force_field --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入力场护盾单位能力运行时`。
  3. 后续必须补真实 bullet runtime 的 absorb/扣盾闭环、shield break/absorb effects 与 sound、结构化 ability runtime state（替代 `AbilityWire.data` sentinel）。

---

## 88. 最新闭环记录：ShieldArcAbility / tecta server ability-state runtime

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（`v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `ShieldArcAbility.java` 与 `UnitTypes.tecta`，把 tecta 弧形护盾的 created data 与 per-tick regen/active 状态接入 `UnitType` content、`UnitComp` hook、server ability-state runtime。
- Java 依据：
  - `created(Unit unit)`：`data = max`；
  - `update(Unit unit)`：`data < max` 时按 `Time.delta * regen` 回复；`active = data > 0 && (unit.isShooting || !whenShooting)`；active 时按 `x/y` 相对 `unit.rotation - 90f` 计算弧盾中心；
  - active 时扫描敌方 bullets / units，处理 absorb/deflect、missile unit 安全死亡与普通 unit push；
  - `tecta` 参数：`radius=45`、`angle=82`、`regen=0.75`、`cooldown=480`、`max=2500`、`y=-20`、`width=8`、`whenShooting=false`、`chanceDeflect=1`。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - 新增 `ShieldArcAbility::from_descriptor(...)`；
    - 新增 tecta descriptor 解析测试；既有 `apply_bullet_hit(...)` 纯算法测试仍保留。
  - `core/src/mindustry/content/unit_types.rs`
    - 为 `tecta` 挂载 `ShieldArcAbility:45:0.75:2500:480:82:0:0:-20:false:8:1`；
    - 内容覆盖测试断言 descriptor 存在。
  - `core/src/mindustry/entities/comp/unit.rs`
    - `UnitComp::new(...)` 调用 `apply_created_shield_arc_abilities()`，初始化对应 `AbilityWire.data = max`；
    - 新增 `UnitComp::update_shield_arc_abilities(...)`，使用 `AbilityWire.data` 保存弧盾 data，tick 后写回。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` 的 playing frame 内调用 `tick_server_shield_arc_abilities(1.0)`；
    - 新增 `server_update_ticks_tecta_shield_arc_regen`，验证 tecta 创建时 data=2500，server tick 后按 0.75/tick 回复。
- 已跑局部验证：
  - `cargo test -p mindustry-core shield_arc --lib`
  - `cargo test -p mindustry-core unit_component_ticks_shield_arc --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server shield_arc --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入弧形护盾单位能力运行时`。
  3. 后续必须补真实 bullet absorb/deflect、missile unit kill、enemy unit push、region/effects/sounds、结构化 ability runtime state。

---

## 89. 最新闭环记录：SpawnDeathAbility / latum dead-unit spawn runtime

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（`v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `SpawnDeathAbility.java` 与 `UnitTypes.latum`，把 latum 死亡生成 renale 接入 `UnitType` content、`UnitComp` death plan、server dead-unit removal/spawn runtime。
- Java 依据：
  - `SpawnDeathAbility.death(Unit unit)`：非 client 端计算 `spawned = amount + Mathf.random(randAmount)`；
  - 每个生成单位在 `spread` 范围内随机偏移，调用 `this.unit.spawn(unit.team, unit.x + offset.x, unit.y + offset.y)`；
  - `faceOutwards` 为真时新单位朝向偏移角；
  - `latum` 参数：`SpawnDeathAbility(renale, 5, 11f)`。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - `SpawnDeathAbility` 增加 `unit: String`；
    - 新增 `SpawnDeathAbility::from_descriptor(...)`，支持 `SpawnDeathAbility:unit:amount:spread[:randAmount[:faceOutwards]]`；
    - 新增 latum descriptor 解析测试。
  - `core/src/mindustry/content/unit_types.rs`
    - 为 `latum` 挂载 `SpawnDeathAbility:renale:5:11`；
    - 内容覆盖测试断言 descriptor 存在。
  - `core/src/mindustry/entities/comp/unit.rs`
    - 新增 `UnitComp::spawn_death_ability_plans(...)`；
    - 当前用确定性等角度 spread 生成计划，避免 server 测试/回放不可复现；后续应替换为可复现 RNG。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` 的 playing frame 内调用 `apply_server_unit_death_abilities()`；
    - server 移除 dead `server_units`，必要时广播 `UnitDespawnCallPacket`；
    - 对 SpawnDeath plans 创建子 `UnitComp`，调用 `unit.add()`、`broadcast_server_unit_spawn(...)`，并记录 `note_unit_create_event(Some(child_id), unit_name, team, None, Some(parent_id))`；
    - 新增 `server_update_spawns_renales_when_latum_dies`，验证 dead latum 被移除，生成 5 个同队 renale，记录 5 个 unit create events 与 stats。
- 已跑局部验证：
  - `cargo test -p mindustry-core spawn_death --lib`
  - `cargo test -p mindustry-core unit_component_plans_spawn_death --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server latum --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入死亡产子单位能力运行时`。
  3. 后续补 Java 等价随机 spread/randAmount 的可复现 RNG、死亡事件总线、death effect/wreck 与更完整 unit removal lifecycle。

---

## 90. 最新闭环记录：MoveEffectAbility / elude movement trail ability slot

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `MoveEffectAbility.java` 与 `UnitTypes.elude`，把 elude 的移动特效 ability 接入 `UnitType` content 与 `UnitComp` ability slot；本轮只生成 runtime sidecar plan，后续再接 desktop/client effect queue。
- Java 依据：
  - `MoveEffectAbility.update(Unit unit)` 在 `Vars.headless` 时直接返回；
  - 累加 `counter += Time.delta`，速度达到 `minVelocity`、interval/chance 满足且不在玩家雾中时，按 `unit.rotation - 90f` 计算 `x/y` 偏移；
  - 触发后 `counter %= interval`，按 `amount` 次调用 `effect.at(...)`；
  - `elude` 使用 `new MoveEffectAbility(0f, -7f, Pal.sapBulletBack, Fx.missileTrailShort, 4f){{ teamColor = true; }}`。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - `MoveEffectPlan` 增加 `effect`、`team_color`、`parentize_effects`；
    - `MoveEffectAbility` 增加 `effect` 字段；
    - 新增 `MoveEffectAbility::from_descriptor(...)`，支持 `MoveEffectAbility:x:y:interval:effect[:teamColor[:minVelocity[:amount]]]`；
    - 新增 elude descriptor 解析测试。
  - `core/src/mindustry/content/unit_types.rs`
    - 为 `elude` 挂载 `MoveEffectAbility:0:-7:4:missileTrailShort:true`；
    - 内容覆盖测试断言 descriptor 存在。
  - `core/src/mindustry/entities/comp/unit.rs`
    - 新增 `UnitComp::update_move_effect_abilities(delta, in_fog)`；
    - 使用 `AbilityWire.data` 保存 Java `counter`，按单位位置/旋转/速度生成 `MoveEffectPlan`。
- 已跑局部验证：
  - `cargo test -p mindustry-core move_effect --lib`
  - `cargo test -p mindustry-core unit_component_ticks_move_effect --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入移动特效单位能力槽`。
  3. 后续接入 `MoveEffectPlan` → desktop/client local effect event queue；补可复现 RNG、chance/random offset、fog/player team 可见性、`Fx.missileTrailShort` 真实表现层、结构化 ability spec/runtime state。

---

## 91. 最新闭环记录：RegenAbility / neoplasm server health runtime

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `RegenAbility.java` 与 `NeoplasmUnitType.java`，把 neoplasm preset 的自愈能力接入 `UnitType` content、`UnitComp` ability slot 与 server unit health runtime。
- Java 依据：
  - `RegenAbility.update(Unit unit)` 每帧调用 `unit.heal((unit.maxHealth * percentAmount / 100f + amount) * Time.delta)`；
  - `NeoplasmUnitType` 设置 `percentAmount = 1f / (70f * 60f) * 100f`，约 70 秒回满。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - 新增 `RegenAbility::from_descriptor(...)`，支持 `RegenAbility:percent[:amount]` 与 legacy 裸 `RegenAbility`；
    - 新增 neoplasm percent descriptor 解析测试。
  - `core/src/mindustry/type/unit/neoplasm_unit_type.rs`
    - 将 neoplasm 默认能力改为 `RegenAbility:0.023809524:0`；
    - 保留 `LiquidExplodeAbility:neoplasm` 与 `LiquidRegenAbility:neoplasm:neoplasmHeal`，后续继续接 world/puddle。
  - `core/src/mindustry/content/unit_types.rs`
    - 内容覆盖测试断言 renale 继承参数化 RegenAbility descriptor。
  - `core/src/mindustry/entities/comp/unit.rs`
    - 新增 `UnitComp::update_regen_abilities(delta)`，对活 unit 按 descriptor heal，并设置 `was_healed`。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` playing frame 内调用 `tick_server_regen_abilities(1.0)`；
    - 新增 `server_update_ticks_renale_neoplasm_regen`，验证受伤 renale 一帧后按 Java 70 秒回满公式回血。
- 已跑局部验证：
  - `cargo test -p mindustry-core regen --lib`
  - `cargo test -p mindustry-core unit_component_ticks_regen --lib`
  - `cargo test -p mindustry-core neoplasm_unit_type_constructor --lib`
  - `cargo test -p mindustry-core unit_kind_defaults_cover_java_constructor_and_init_side_effects --lib`
  - `cargo test -p mindustry-server renale_neoplasm_regen --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入新生物单位自愈运行时`。
  3. 后续优先接 `LiquidExplodeAbility` 死亡洒落 neoplasm puddle，再接 `LiquidRegenAbility` slurp puddle 回血与 `Fx.neoplasmHeal` 表现层。

---

## 92. 最新闭环记录：MoveEffectAbility / elude client local effect queue

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：在前一轮 `MoveEffectPlan` 基础上，把 elude 移动尾迹接入客户端 runtime/local effect queue，不再只停在 sidecar plan。
- Java 依据：
  - `MoveEffectAbility.update(Unit unit)` 非 headless 时本地播放 `effect.at(...)`；
  - `UnitTypes.elude` 使用 `Fx.missileTrailShort` 与 `teamColor=true`；
  - v158.1 `Fx.java` 静态创建顺序对应 `Fx.missileTrailShort` id 111。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_MISSILE_TRAIL_ID=110`、`FX_MISSILE_TRAIL_SHORT_ID=111`；
    - 新增 `standard_effect_id(...)`，当前覆盖 `missileTrail` / `missileTrailShort`。
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `GameRuntime::tick_client_move_effect_abilities(delta, in_fog)`；
    - 遍历 `client_unit_snapshot_entities`，把 `UnitComp::update_move_effect_abilities(...)` 生成的 plan 转成 `EffectCallPacket2`，写入 `client_local_effect_events`；
    - 新增 `game_runtime_ticks_client_move_effect_ability_into_local_effect_event`。
  - `desktop/src/lib.rs`
    - `DesktopLauncher::update()` 末尾调用 `runtime.tick_client_move_effect_abilities(1.0, false)`；
    - 新增 `desktop_launcher_ticks_elude_move_effect_to_local_effect_queue`。
- 已跑局部验证：
  - `cargo test -p mindustry-core client_move_effect --lib`
  - `cargo test -p mindustry-desktop elude_move_effect --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入移动特效客户端队列`。
  3. 后续补可复现 RNG、chance/random offset、fog/player team 可见性、`parentizeEffects` parent 语义与完整 `Fx` registry。

---

## 93. 最新闭环记录：LiquidExplodeAbility / neoplasm death puddle runtime

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `LiquidExplodeAbility.java` 与 `NeoplasmUnitType.java`，把 neoplasm 单位死亡洒落 `neoplasm` 液体接入 server death lifecycle 与 `Puddles` runtime。
- Java 依据：
  - `LiquidExplodeAbility.death(Unit unit)` 按 `unit.tileX()/tileY()` 与 `hitSize/tilesize` 半径遍历 tile；
  - 命中区域内调用 `Puddles.deposit(tile, liquid, amount * scaling)`；
  - neoplasm preset 设置 `liquid = Liquids.neoplasm`。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - `LiquidExplodeAbility` 默认 liquid 改为 Java 默认 `water`；
    - 新增 `LiquidExplodeDepositPlan`；
    - 新增 `LiquidExplodeAbility::from_descriptor(...)` 与 `deposit_plans(...)`；
    - 新增 descriptor/计划测试。
  - `core/src/mindustry/entities/comp/unit.rs`
    - 新增 `UnitComp::liquid_explode_ability_deposit_plans()`，死亡时从 unit ability descriptor 产出洒液计划。
  - `core/src/mindustry/entities/puddles.rs`
    - 给 `Puddles` 增加 `width()/height()`，便于 server runtime 根据 world 尺寸初始化 puddle grid。
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `server_puddles: Puddles`。
  - `server/src/lib.rs`
    - `apply_server_unit_death_abilities()` 在移除 dead unit 后调用 `apply_server_liquid_explode_deposits(...)`；
    - 新增 `server_update_deposits_neoplasm_when_renale_dies`。
- 已跑局部验证：
  - `cargo test -p mindustry-core liquid_explode --lib`
  - `cargo test -p mindustry-core unit_component_plans_liquid_explode --lib`
  - `cargo test -p mindustry-server neoplasm_when_renale_dies --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入新生物死亡洒液运行时`。
  3. 后续补 Java `Simplex.noise2d` 边缘噪声、server puddle entity snapshot 广播到 desktop、真实 floor/env/space/boil 随机上下文，再接 `LiquidRegenAbility` slurp puddle 回血。

---

## 94. 最新闭环记录：LiquidRegenAbility / neoplasm slurp puddle regen runtime

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 `LiquidRegenAbility.java` 与 `NeoplasmUnitType.java`，把 neoplasm 单位从同液体 puddle 吸液回血接入 server unit + puddle runtime。
- Java 依据：
  - damaged 且非 flying 时扫描 `hitSize / tilesize * 0.6` 半径内 tile；
  - 同液体 puddle 每帧扣 `min(puddle.amount, slurpSpeed * Time.delta)`；
  - 单位回血 `fractionTaken * regenPerSlurp`。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - `LiquidRegenAbility` 增加 `slurp_effect`；
    - 新增 `from_descriptor(...)`、`slurp_radius(...)`、`slurp_tiles(...)`；
    - 新增 neoplasm descriptor/半径测试。
  - `core/src/mindustry/entities/comp/unit.rs`
    - 新增 `UnitComp::liquid_regen_abilities()`。
  - `core/src/mindustry/entities/puddles.rs`
    - 新增 `Puddles::slurp_matching_liquid(...)` 与测试。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` playing frame 内在 passive Regen 后调用 `tick_server_liquid_regen_abilities(1.0)`；
    - 新增 `server_update_slurps_neoplasm_puddle_to_regen_renale`，验证 renale 从 neoplasm puddle 扣 5 液体并回血 30，同时保留 passive regen。
- 已跑局部验证：
  - `cargo test -p mindustry-core liquid_regen --lib`
  - `cargo test -p mindustry-core slurp_matching --lib`
  - `cargo test -p mindustry-core unit_component_reads_liquid_regen --lib`
  - `cargo test -p mindustry-server slurps_neoplasm --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入新生物吸液回血运行时`。
  3. 后续补 `Fx.neoplasmHeal` effect queue、chance/random offset、完整 flying/elevation 判断、server puddle snapshot 广播到 desktop。

---

## 95. 最新闭环记录：server puddle entity snapshot sync

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：把 `GameRuntime.server_puddles` 中的服务端 puddle 写入 `EntitySnapshotCallPacket`，让桌面客户端能通过已有 typed runtime materialize puddle entity。
- 已确认上游/基线：
  - `git -C "D:/MDT/mindustry-upstream-v157.4" fetch --tags origin`
  - `git -C "D:/MDT/mindustry-upstream-v157.4" describe --tags --always --dirty` => `v158.1`
  - `git -C "D:/MDT/mindustry-upstream-v157.4" rev-parse HEAD` => `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`
- Rust 主改动：
  - `core/src/mindustry/entities/puddles.rs`
    - 新增 `Puddles::entries()`，只读暴露内部 `(tile, PuddleEntry)` iterator。
  - `server/src/lib.rs`
    - import `PUDDLE_CLASS_ID`；
    - `server_unit_entity_snapshot_packet()` 继续写 `runtime.server_puddles`：
      - 跳过 removed / amount<=0 / content 中无法反查 liquid id 的 puddle；
      - `entity_id = entry.puddle.id`；
      - `type_id = PUDDLE_CLASS_ID`；
      - `PuddleSyncWire { amount, liquid_id: Some(...), tile_pos: point2_pack(tile.x,tile.y), x, y }`；
    - 新增 `server_entity_snapshot_packet_includes_runtime_puddles_for_client_sync`，验证 server packet 经 `GameRuntime::apply_client_entity_snapshot_packet_with_content(...)` 后进入 `client_puddle_snapshot_entities`。
- 已跑局部验证：
  - `cargo test -p mindustry-server server_entity_snapshot_packet_includes_runtime_puddles_for_client_sync --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`同步服务端液体坑实体快照`。
  3. 后续优先补：
     - puddle removal/evaporation 的 hidden/delete 同步，避免客户端残影；
     - `LiquidRegenAbility` 的 `Fx.neoplasmHeal` effect queue；
     - `LiquidExplodeAbility` 的 Java `Simplex.noise2d` 边缘噪声与真实 floor/env/space/boil 随机上下文；
     - 把当前混合 cargo unit + puddle 的 `server_unit_entity_snapshot_packet` 逐步泛化命名和拆分。

---

## 96. 最新闭环记录：LiquidRegenAbility / neoplasmHeal effect packet and client queue

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：把 `LiquidRegenAbility` 的 `slurpEffect=Fx.neoplasmHeal` 接到服务端 effect packet 与桌面客户端 local effect queue，避免吸液回血只有数值变化没有表现层事件。
- Java 依据：
  - `LiquidRegenAbility.update(Unit unit)` 在 healed 后按 `slurpEffectChance` 调用 `slurpEffect.at(..., unit)`；
  - `Fx.java:1445`：`neoplasmHeal = new Effect(120f, ...)`；
  - 统计 v158.1 `Fx.java` 中 `neoplasmHeal` 前有 122 个 effect 声明，因此 `neoplasmHeal` effect id 为 `122`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_NEOPLASM_HEAL_ID=122`；
    - `standard_effect_id("neoplasmHeal") -> Some(122)`。
  - `core/src/mindustry/entities/mod.rs`
    - re-export `FX_NEOPLASM_HEAL_ID`。
  - `server/src/lib.rs`
    - `tick_server_liquid_regen_abilities(...)` 改为 `io::Result<usize>`；
    - slurp/heal 成功后按 `slurp_effect != "none" && slurp_effect_chance > 0.0` 发送 `EffectCallPacket2`；
    - 新增 `broadcast_server_effect_with_data(...)`，写 `EffectCallPacket2 { effect_id, x, y, rotation, color=-1, data=TypeValue::Unit(unit_id) }`；
    - 更新 `server_update_slurps_neoplasm_puddle_to_regen_renale`，捕获网络包并断言 `neoplasmHeal` effect packet。
  - `desktop/src/lib.rs`
    - 新增 effect packet apply cursors；
    - `DesktopLauncher::update()` 调用 `sync_effect_packets_to_runtime()`；
    - `sync_effect_packets_to_runtime()` 将 `EffectCallPacket` / `EffectCallPacket2` / `EffectReliableCallPacket` 转入 `runtime.client_local_effect_events`；
    - 新增 `desktop_launcher_syncs_effect_call_packet2_to_local_effect_queue`。
- 已跑局部验证：
  - `cargo test -p mindustry-server slurps_neoplasm --lib`
  - `cargo test -p mindustry-desktop syncs_effect_call_packet2 --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入新生物吸液回血特效同步`。
  3. 后续补：
     - `Mathf.chanceDelta(slurpEffectChance)` 的可复现 RNG/delta 概率；
     - Java 随机 offset；
     - effect parent/follow/rotWithParent 在 renderer/EffectState 中的完整语义；
     - net client effect queue，避免同一 update 间隔多条同类 effect 只保留最后一条。

---

## 97. 最新闭环记录：LiquidExplodeAbility / Arc Simplex edge noise

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 Java `LiquidExplodeAbility.death`，把死亡洒液 tile 入选条件从确定性圆形改成 Java/Arc `Simplex.noise2d` 噪声削边。
- Java 依据：
  - `LiquidExplodeAbility.java`：
    - `rad = max((int)(unit.hitSize / tilesize * radScale), 1)`；
    - `realNoise = unit.hitSize / noiseMag`；
    - `x*x + y*y <= rad*rad - Simplex.noise2d(0, 2, 0.5f, 1f/noiseScl, x+tx, y+ty) * realNoise * realNoise`。
  - Arc `Simplex.noise2d`：
    - 2 octave；
    - persistence `0.5`；
    - `(raw2d + 1) / 2` 后按 amplitude 归一化；
    - `raw2d` 为标准 2D simplex，最后乘 `70`。
- Rust 主改动：
  - `core/src/mindustry/entities/abilities.rs`
    - `LiquidExplodeAbility::deposit_plans(...)` 引入 Java 同款噪声阈值；
    - 新增私有 helper：`simplex_noise2d`、`simplex_raw2d`、`simplex_corner2d`、`simplex_perm`、`simplex_fastfloor`；
    - `planned_noise_radius` 对 `noise_mag==0` 返回 `0.0`，避免除零；
    - 新增 `liquid_explode_deposit_plans_apply_java_simplex_edge_noise`，锁定 Java/Arc 样例：
      - `simplex_noise2d(0,2,0.5,1/5,1,8) ≈ 0.865014`；
      - `deposit_plans(0,35,10,5)` 生成 8 个格子，且不包含 `(1,8)`。
- 已跑局部验证：
  - `cargo test -p mindustry-core liquid_explode --lib`
  - `cargo test -p mindustry-server neoplasm_when_renale_dies --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入死亡洒液边缘噪声`。
  3. 后续补：
     - Arc Simplex 3D/4D/tiled 分支的系统化迁移（当前只迁移 2D 本能力所需分支）；
     - floor/env/space/boil 的真实 map/env/random 上下文；
     - puddle evaporation/removal 的客户端删除同步。

---

## 98. 最新闭环记录：Puddles lifecycle removal and hidden snapshot sync

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：让 `GameRuntime.server_puddles` 的空/removed puddle 在服务端生命周期中被移除，并通过 `HiddenSnapshotCallPacket` 同步到客户端，避免被吸干后残留。
- Rust 主改动：
  - `core/src/mindustry/entities/puddles.rs`
    - 新增 `Puddles::update_all(delta, headless) -> Vec<i32>`；
    - 调用 `PuddleComp::update(PuddleUpdateContext { nearby_spread_targets:0, registry_matches_self:true, headless, fire_chance_passed:false, ... })`；
    - 删除 removed / amount<=0 / liquid none 的 puddle，并返回 entity ids。
  - `core/src/mindustry/core/game_runtime.rs`
    - `apply_client_hidden_snapshot_ids(...)` 对 puddle 从 `contains_key` 改为 `remove`，hidden 后实际清掉 `client_puddle_snapshot_entities`。
  - `server/src/lib.rs`
    - `ServerLauncher::update()` 在 `tick_server_liquid_regen_abilities(1.0)` 后调用 `tick_server_puddles(1.0)`；
    - 新增 `broadcast_server_hidden_snapshot(...)`，对 removed puddle ids 发送 `HiddenSnapshotCallPacket`；
    - 新增 `server_update_hides_puddle_entity_when_liquid_regen_drains_it_empty`。
- 已跑局部验证：
  - `cargo test -p mindustry-core update_all_removes_empty_puddles --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_puddle --lib`
  - `cargo test -p mindustry-server server_update_slurps_neoplasm --lib`
  - `cargo test -p mindustry-server server_update_hides_puddle --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`同步液体坑移除快照`。
  3. 后续补：
     - puddle spread / affect units / fire / particle / ripple 事件；
     - hidden snapshot 对其他 typed entities 的最终 remove 语义（本轮只处理 puddle）；
     - 客户端渲染层真正消费 puddle add/remove 的表现。

---

## 99. 最新闭环记录：Puddles D4 spread runtime and snapshot

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：把 Java `PuddleComp.update()` 中 amount 过高时向 `Geometry.d4` 四邻扩散的行为接入 Rust `Puddles::update_all` 和 server snapshot。
- Rust 主改动：
  - `core/src/mindustry/entities/puddles.rs`
    - `update_all(...)` 现在按 in-bounds D4 neighbors 计算 `nearby_spread_targets`；
    - `PuddleComp::update(...)` 返回 `deposited_per_target` 后，延迟调用 `Puddles::deposit(... initial:false ...)` 写入邻居 puddle；
    - 新增 `d4_spread_targets(...)`；
    - 新增 `update_all_spreads_overfilled_puddles_to_d4_neighbors`。
  - `server/src/lib.rs`
    - 既有 `tick_server_puddles(1.0)` 自动驱动 spread；
    - 新增 `server_update_spreads_overfilled_puddle_and_snapshots_neighbors`，验证 server update 后生成 5 个 puddle 且 snapshot amount 为 5。
- 已跑局部验证：
  - `cargo test -p mindustry-core update_all_spreads --lib`
  - `cargo test -p mindustry-server server_update_spreads_overfilled --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入液体坑四向扩散`。
  3. 后续补：
     - spread passability 接真实 world block/floor，而不是仅 `in_bounds`；
     - puddle ripple/particle/fire/building puddleOn 事件；
     - `liquid.update(self())` hook。

---

## 100. 最新闭环记录：Puddles spread passability from server world/content

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：把 99 中的 puddle D4 spread passability 从 `in_bounds` 近似接到真实 server world/content solidity。
- Rust 主改动：
  - `core/src/mindustry/entities/puddles.rs`
    - 新增 `update_all_with_passability(delta, headless, passable)`；
    - `update_all(...)` 保持纯 core 默认行为，内部委托给 passability 版本；
    - `d4_spread_targets(...)` 同时检查 in-bounds 与 passability callback；
    - 新增 `update_all_spread_respects_passability_callback`。
  - `server/src/lib.rs`
    - `tick_server_puddles(...)` 传入 server `world` + `content_loader`；
    - spread 目标要求 tile 存在，且 `liquid.move_through_blocks || !world.wall_solid_with_content(x, y, content)`；
    - 更新 `server_update_spreads_overfilled_puddle_and_snapshots_neighbors`，用 `copper-wall` 阻挡 water east neighbor，snapshot amount 断言为 4。
- 已跑局部验证：
  - `cargo test -p mindustry-core update_all_spread --lib`
  - `cargo test -p mindustry-server server_update_spreads_overfilled --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入液体坑扩散通行判断`。
  3. 后续补：
     - 如果要严格 Java parity，把 passability 从 solidity 改为 block id `air` 或 `moveThroughBlocks`；
     - server world floor/liquid context 注入 spread/deposit；
     - puddle ripple/particle/fire/building puddleOn/liquid.update hook。

---

## 101. 最新闭环记录：Puddle update event report → server Fires runtime/snapshot

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：对照 Java `PuddleComp.update()` effects-only 分支，先闭合高温 puddle + building 的 `Fires.create(tile)` 链路，避免 `PuddleUpdatePlan::create_fire` 长期只停在 plan。
- Java 依据：
  - `amount >= maxLiquid / 2f && updateTime <= 0f`；
  - `liquid.temperature > 0.7f && tile.build != null && Mathf.chance(0.5)` 时 `Fires.create(tile)`；
  - 同一分支还会 `Units.nearby(...)` 与 `tile.build.puddleOn(self())`，本轮只把这些作为 report event 暴露，暂未消费。
- Rust 主改动：
  - `core/src/mindustry/entities/puddles.rs`
    - `PuddleLiquidInfo` 新增 `particle_effect` 字段；
    - 新增 `PuddleUpdateEvent` / `PuddleUpdateReport`；
    - 新增 `update_all_with_passability_report(...)`，在原 spread/remove 基础上额外注入 `build_present` 和 `fire_chance` callback，输出 `affect_units/create_fire/puddle_on_building/particle_effect` events；
    - 旧 `update_all(...)` / `update_all_with_passability(...)` 仍返回 removed ids，保持兼容。
  - `core/src/mindustry/entities/fires.rs`
    - 新增 `width()` / `height()` / `entries()`，让 server 能维护尺寸并枚举快照。
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `server_fires: Fires` runtime sidecar。
  - `server/src/lib.rs`
    - `tick_server_puddles(...)` 现在维护 `runtime.server_fires` 尺寸；
    - 从 server world 刷新 puddle tile build presence；
    - 用稳定 hash 暂时代替 Java `Mathf.chance(0.5)`，触发时调用 `Fires::create(...)`；
    - `server_unit_entity_snapshot_packet()` 把 `server_fires.entries()` 写入 `FIRE_CLASS_ID + FireSyncWire`，entity id 使用 `SERVER_FIRE_ENTITY_ID_BASE + point2_pack(x,y)` 的稳定 tile-derived id。
- 已跑局部验证：
  - `cargo test -p mindustry-core update_all_report_exposes_hot_puddle_fire_and_building_events --lib`
  - `cargo test -p mindustry-core create_adds_fire_and_refreshes_existing_lifetime --lib`
  - `cargo test -p mindustry-server server_entity_snapshot_packet_includes_runtime_fires_for_client_sync --lib`
  - `cargo test -p mindustry-server server_update_creates_fire_when_hot_puddle_touches_building --lib`
- 当前仍需继续：
  1. 跑完整收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入液体坑起火运行时`。
  3. 后续补：
     - `Units.nearby`：grounded/非 hovering 单位应用 liquid status 120 tick，移动时发送 `Fx.ripple`；
     - `standard_effect_id("ripple")`，已确认 v158.1 `Fx.ripple` id 为 `243`；
     - `tile.build.puddleOn(self())` 的真实 building consumer；
     - `CellLiquid.update(Puddle)`，尤其 neoplasm 从周边 building/puddle 吸收 spreadTarget、伤害 building、触发 neoplasmReact；
     - 起火概率从稳定 hash 替换为 Java 等价 RNG/delta。

---

## 102. 最新闭环记录：Fx.ripple standard effect id

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：为下一步接 `PuddleComp.update()` 的单位踩液体 `Fx.ripple` 分支补齐标准 effect id。
- Java 依据：
  - `Effect` id 按 `all.size` 顺序分配；
  - `Fx.ripple` 在 v158.1 `Fx.java` 中是第 244 个声明，0-based id 为 `243`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_RIPPLE_ID = 243`；
    - `standard_effect_id("ripple") -> Some(FX_RIPPLE_ID)`；
    - 顺手补 `standard_effect_id("unitAssemble") -> Some(FX_UNIT_ASSEMBLE_ID)`；
    - 新增 `standard_effect_ids_include_puddle_ripple_dependencies`。
  - `core/src/mindustry/entities/mod.rs`
    - re-export `FX_RIPPLE_ID`。
- 已跑局部验证：
  - `cargo test -p mindustry-core standard_effect_ids_include_puddle_ripple_dependencies --lib`
- 当前仍需继续：
  1. 跑收尾验证：`cargo check -p mindustry-core`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入液体坑波纹效果编号`。
  3. 后续补：
     - `PuddleUpdateEvent.affect_units` 的 server consumer；
     - 单位矩形查询、grounded/hovering 过滤、liquid status apply 120 tick；
     - 移动单位 `Fx.ripple` effect packet/local queue；
     - `tile.build.puddleOn` 与 `CellLiquid.update`。

---

## 103. 最新闭环记录：Puddle Units.nearby status/ripple server consumer

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：消费 101 中暴露的 `PuddleUpdateEvent.affect_units`，把 Java `Units.nearby` 的 liquid status 与移动 ripple 副作用接入 server runtime。
- Java 依据：
  - `rect.setSize(clamp(amount / (maxLiquid / 1.5f)) * 10f).setCenter(x, y)`；
  - `unit.isGrounded() && !unit.type.hovering`；
  - `unit.apply(liquid.effect, 60 * 2)`；
  - 移动单位 `Fx.ripple.at(unit.x, unit.y, unit.type.rippleScale, liquid.color)`。
- Rust 主改动：
  - `server/src/lib.rs`
    - `tick_server_puddles(...)` 现在遍历 `report.events`，对 `affect_units` 分支筛选 `server_units`；
    - 对命中单位应用 `ContentLoader::status_effect_by_name(liquid.effect)`，持续 `120.0` tick；
    - 移动单位收集 ripple 并通过新增 `broadcast_server_effect_colored(...)` 发送 `EffectCallPacket`；
    - 新增 `server_update_applies_puddle_liquid_status_and_ripple_to_ground_units`，验证 water puddle 给 dagger 施加 `wet` 并广播 `ripple` effect。
- 已跑局部/收尾验证：
  - `cargo test -p mindustry-server server_update_applies_puddle_liquid_status_and_ripple_to_ground_units --lib`
  - `cargo test -p mindustry-server server_update_creates_fire_when_hot_puddle_touches_building --lib`
  - `cargo check -p mindustry-server`
  - `cargo fmt --check`
  - `git diff --check`
- 当前仍需继续：
  1. 中文提交并推送 `origin main`，建议标题：`接入液体坑单位影响运行时`。
  2. 后续补：
     - `tile.build.puddleOn(self())` 的 Rust building consumer；
     - `CellLiquid.update(Puddle)` / neoplasm 周边液体吸收、建筑伤害、neoplasmReact；
     - Java `Units.nearby` 空间索引与 Groups 语义的更严格替代；
     - desktop renderer/audio 对 ripple effect sidecar 的真实表现层。

---

## 104. 最新闭环记录：CellLiquid.update 邻接建筑吸液转换

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：确认 `Building.puddleOn(Puddle)` 在 Java v158.1 是空钩子后，继续推进更有实际行为的 `CellLiquid.update(Puddle)`：neoplasm 从邻接 building 的 water 模块吸收并转换为 neoplasm puddle。
- Java 依据：
  - `CellLiquid` 默认 `maxSpread=0.75`、`spreadConversion=1.2`、`spreadDamage=0.11`、`removeScaling=0.25`；
  - `Liquids.neoplasm.spreadTarget = Liquids.water`；
  - 周边 `Geometry.d4c` building 若有 target liquid，则 remove `amount*removeScaling` 并 deposit `amount*spreadConversion` 的 cell liquid。
- Rust 主改动：
  - `core/src/mindustry/type/liquid.rs`
    - 新增 CellLiquid 字段：`cell_spread_target`、`cell_max_spread`、`cell_spread_conversion`、`cell_spread_damage`、`cell_remove_scaling`；
  - `core/src/mindustry/content/liquids.rs`
    - neoplasm 设置 `cell_spread_target=water`；
    - 补 `can_stay_on=[water, oil, cryofluid, arkycite]`；
  - `core/src/mindustry/entities/puddles.rs`
    - `PuddleLiquidInfo` 保留 CellLiquid 字段；
    - `PuddleUpdateEvent::from_plan` 现在也为 `liquid_update` 输出 event；
  - `server/src/lib.rs`
    - `tick_server_puddles(...)` 对带 `reaction_target` 的 `liquid_update` event 扫描邻接 building；
    - 从真实 `BuildingComp.liquids` 移除 water；
    - 按 `spreadConversion` 把 neoplasm 沉积到目标 tile 的 `server_puddles`；
    - 新增 `server_puddle_cell_liquid_update_absorbs_spread_target_from_neighbor_building`。
- 已跑验证：
  - `cargo test -p mindustry-server server_puddle_cell_liquid_update_absorbs_spread_target_from_neighbor_building --lib`
  - `cargo test -p mindustry-core liquid_defaults_match_java_constructor_shape --lib`
  - `cargo test -p mindustry-core liquid_core_properties_match_upstream_subset --lib`
  - `cargo test -p mindustry-core update_all_report_exposes_hot_puddle_fire_and_building_events --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `cargo fmt --check`
  - `git diff --check`
- 当前仍需继续：
  1. 中文提交并推送 `origin main`，建议标题：`接入新生物液体邻接吸收`。
  2. 后续补：
     - `CellLiquid.update` 的 nearby puddle 吸收/替换分支；
     - current-building water damage/spread 的严格测试；
     - `Events.fire(Trigger.neoplasmReact)` 等价事件；
     - 统一迁移 `Geometry.d4c` 常量，替代当前显式方向数组。

---

## 105. 最新闭环记录：CellLiquid.update 邻接 puddle 吸收/替换

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：补齐 Java `CellLiquid.update(Puddle)` 中 `spread to nearby puddles` 分支，让 neoplasm 能按 `spreadTarget=water` 吞噬四邻 water puddle，并在低残量时移除旧 puddle、替换成 neoplasm puddle。
- Java 依据：
  - `Geometry.d4` 四邻扫描；
  - `amount = min(other.amount, max(maxSpread * Time.delta * scaling, other.amount * 0.25f * scaling))`；
  - `other.amount -= amount; puddle.amount += amount`；
  - `other.amount <= maxLiquid / 3f` 时 `other.remove()`，再 `Puddles.deposit(tile, puddle.tile, this, max(amount, maxLiquid / 3f))`。
- Rust 主改动：
  - `core/src/mindustry/entities/puddles.rs`
    - 新增 `PuddleCellAbsorbReport`；
    - 新增 `Puddles::absorb_neighbor_target_puddles(...)`，在 `Puddles` 内部完成四邻 target puddle 吸收、source amount 增量、低残量 remove + replacement deposit，并返回 removed ids；
    - 新增 core 测试覆盖“吸收并替换”和“只处理 D4 target puddle，不处理对角/非目标液体”。
  - `server/src/lib.rs`
    - `tick_server_puddles(...)` 对 `liquid_update + reaction_target` event 调用该 helper；
    - `CellLiquid.update` 入口遵守 Java `Vars.state.rules.fire` 门控；
    - 将 helper 返回的 removed ids 合并进现有 hidden snapshot 广播；
    - 修正 current-building damage/spread 分支的早退：当前 tile 无 building 时不再跳过 nearby puddle 吸收。
- 已跑验证：
  - `cargo test -p mindustry-core cell_liquid_absorbs --lib`
  - `cargo test -p mindustry-server server_puddle_cell_liquid_update_absorbs_neighbor_target_puddle_and_hides_removed_id --lib`
- 当前仍需继续：
  1. 跑收尾验证：`server_puddle_cell_liquid_update_absorbs_spread_target_from_neighbor_building` 回归、`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo check -p mindustry-desktop`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`接入新生物液体坑邻接替换`。
  3. 后续补：
     - `Events.fire(Trigger.neoplasmReact)` 等价事件；
     - current-building water damage/spread 的严格 Java parity 测试；
     - building 吸收 deposit 与 nearby puddle 吸收之间的 Java 顺序精确化；
     - 统一迁移 `Geometry.d4/d4c` 常量。

---

## 106. 最新闭环记录：CellLiquid.update current-building damage/spread 回归

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：给已经接入的 `CellLiquid.update` current-building damage/spread 分支补 server 回归，防止该分支只停留在未锁定行为的入口。
- Java 依据：
  - 当前 puddle 所在 tile 的 building 含 `spreadTarget` 且 `spreadDamage > 0` 时触发；
  - `amountSpread = min(available * spreadConversion, maxSpread * Time.delta) / 2f`；
  - `Puddles.deposit(puddle.tile, other, puddle.liquid, amountSpread)` 通过 same-liquid deposit 回到 source puddle 的 accepting；
  - building 受到 `spreadDamage * Time.delta * scaling` 伤害；
  - 该分支不会移除 current building 中的 target liquid。
- Rust 主改动：
  - `server/src/lib.rs`
    - 新增 `server_puddle_cell_liquid_update_damages_target_liquid_building_and_reaccepts_spread`；
    - 测试构造 neoplasm puddle + 同 tile 带 water 的 `liquid-router`，断言 building health 下降、water 因 `Geometry.d4c` center 吸收而下降、source puddle accepting 接收转换沉积/`amountSpread`。
- 已跑验证：
  - `cargo test -p mindustry-server server_puddle_cell_liquid_update_damages_target_liquid_building_and_reaccepts_spread --lib`
- 当前仍需继续：
  1. 跑收尾验证：相关 CellLiquid 三条 server 测试、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`锁定新生物液体坑建筑伤害`。
  3. 后续补：
     - `Events.fire(Trigger.neoplasmReact)` 等价事件；
     - building 吸收 deposit 与 nearby puddle 吸收之间的 Java 顺序精确化；
     - 统一迁移 `Geometry.d4/d4c` 常量。

---

## 107. 最新闭环记录：CellLiquid.update neoplasmReact trigger runtime 记录

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：补齐 Java `CellLiquid.update()` 中 `reacted && this == Liquids.neoplasm -> Events.fire(Trigger.neoplasmReact)` 的 Rust 事件源记录。
- Java 依据：
  - `EventType.Trigger.neoplasmReact`；
  - `CellLiquid.update(Puddle)` 只要任一 neoplasm spread/reaction 分支发生 reacted 就 fire；
  - `GameService.trigger(Trigger.neoplasmReact, neoplasmWater)` 将其映射到事件型成就 `neoplasmWater`。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `GameRuntimeTriggerEvent { trigger, campaign }`；
    - 新增 `GameRuntime::note_trigger_event(Trigger)`；
    - `trigger_events` 加入 `GameRuntime` 并在 runtime sidecar 清理时清空。
  - `server/src/lib.rs`
    - `tick_server_puddles(...)` 对 CellLiquid 三类 reacted 分支（邻接 building 吸收、current-building damage/spread、nearby puddle 吸收/替换）汇总 `reacted`；
    - 当 liquid 是 `neoplasm` 时记录 `Trigger::NeoplasmReact`；
    - 扩展邻接 puddle 替换测试，在 campaign sector 下断言 runtime trigger event 被记录。
- 已跑验证：
  - `cargo test -p mindustry-server server_puddle_cell_liquid_update_absorbs_neighbor_target_puddle_and_hides_removed_id --lib`
  - `cargo test -p mindustry-core trigger_plan_maps_java_game_service_triggers --lib`
- 当前仍需继续：
  1. 跑收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`记录新生物反应触发事件`。
  3. 后续补：
     - 把 `GameRuntime.trigger_events` 自动应用到 `DefaultGameService` / 平台 achievement service；
     - building 吸收 deposit 与 nearby puddle 吸收之间的 Java 顺序精确化；
     - 统一迁移 `Geometry.d4/d4c` 常量。

---

## 108. 最新闭环记录：Arc Geometry.d4/d4c 常量对齐

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（已确认 `v158.1` / `05b2ecd4eb578ac38cace8118dbecc1bd548ff4a`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8。
- 本轮目标：修正并集中 Rust 侧 CellLiquid/Puddles 使用的 Java `Geometry.d4/d4c` 方向集合，尤其是 `d4c` 不是“八邻域”，而是“四邻 + center”。
- 本地验证：
  - 使用 Gradle 缓存 `arc-core-4d9760e264.jar` 直接运行小 Java 程序打印：
    - `Geometry.d4 = (1,0),(0,1),(-1,0),(0,-1)`；
    - `Geometry.d4c = (1,0),(0,1),(-1,0),(0,-1),(0,0)`。
- Rust 主改动：
  - `core/src/mindustry/world/build.rs`
    - 新增 `ORTHOGONAL_WITH_CENTER_NEIGHBORS`；
    - 新增 `orthogonal_neighbor_constants_match_arc_geometry_d4_and_d4c`。
  - `core/src/mindustry/entities/puddles.rs`
    - D4 spread 与 nearby target puddle 吸收改用 `ORTHOGONAL_NEIGHBORS`。
  - `server/src/lib.rs`
    - CellLiquid 周边 building 吸收改用 `ORTHOGONAL_WITH_CENTER_NEIGHBORS`；
    - current-building 测试更新为断言 water 会因 d4c center 吸收而下降，source accepting 至少包含 center 转换沉积。
- 当前仍需继续：
  1. 跑验证：`cargo test -p mindustry-core orthogonal_neighbor_constants_match_arc_geometry_d4_and_d4c --lib`、`cargo test -p mindustry-server server_puddle_cell_liquid_update --lib`、`cargo fmt --check`、`git diff --check`。
  2. 中文提交并推送 `origin main`，建议标题：`对齐液体坑邻接方向常量`。
  3. 后续补：
     - 其他散落 AI/pathfinder/block runtime 的私有 D4 常量可逐步统一；
     - building 吸收 deposit 与 nearby puddle 吸收之间的 Java 顺序精确化；
     - 把 `GameRuntime.trigger_events` 自动应用到 `DefaultGameService` / 平台 achievement service。

---

## 109. 最新闭环记录：cargo tether snapshot 保留 + trigger_events 接入 GameService

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：
  1. 修复刚暴露的真实联机 smoke：`UnitTetherBlockSpawnedCallPacket` 物化 cargo unit 后，后续 `EntitySnapshot` 不应把本地 CargoAI/tether sidecar 覆盖成 ground；
  2. 把此前记录到 `GameRuntime.trigger_events` 的 trigger 最小接入 `DefaultGameService` 成就/统计链路，不引入全局 event bus。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `GameRuntime::drain_trigger_events()`；
    - `apply_client_unit_sync_wire(...)` 对本地 cargo tether sidecar 做保留：已有 `UnitControllerState::Cargo` 且 incoming `ControllerWire::Ground` 时保留 `cargo_ai/building_tether/controller`；
    - 新增 `game_runtime_preserves_client_cargo_tether_when_unit_snapshot_arrives`；
    - 新增 `game_runtime_drain_trigger_events_returns_and_clears_local_queue`。
  - `core/src/mindustry/service/game_service.rs`
    - `DefaultGameService` 新增内存 backing store：`stats`、`achievements`、`stats_store_count`；
    - 覆盖实现 `StatService` / `AchievementService`，不再是全 no-op；
    - 更新默认服务测试并新增 `trigger_plan_apply_to_writes_trigger_stats_and_achievements_into_service_runtime`。
  - `core/src/mindustry/client_launcher.rs`
    - `ClientLauncher` 增加 `AchievementState` cache，供 service plan apply 去重/缓存。
  - `desktop/src/lib.rs`
    - `DesktopLauncher::update()` 在 `sync_world_update_events_to_runtime()` 后调用 `sync_runtime_trigger_events_to_service()`；
    - 该 bridge drain runtime trigger，转成 `GameServiceTriggerSnapshot`，复用 `trigger_plan -> apply_to` 写入 `client.service`；
    - 新增 `desktop_launcher_drains_runtime_trigger_events_into_game_service`。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_preserves_client_cargo_tether_when_unit_snapshot_arrives --lib`
  - `cargo test -p mindustry-tests real_server_desktop_unit_cargo_loader_tether_spawn_syncs_to_client_runtime --lib`
  - `cargo test -p mindustry-core game_runtime_drain_trigger_events_returns_and_clears_local_queue --lib`
  - `cargo test -p mindustry-core trigger_plan_apply_to_writes_trigger_stats_and_achievements_into_service_runtime --lib`
  - `cargo test -p mindustry-core default_game_service_platform_methods_persist_runtime_stats_and_achievements --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_drains_runtime_trigger_events_into_game_service --lib`
  - `cargo check --workspace`
  - `cargo test --workspace`
  - `cargo fmt --check`
  - `git diff --check`
- 当前仍需继续：
  1. 中文提交并推送 `origin main`，建议标题：`接入触发事件成就服务`；
  2. 后续补：server runtime trigger 通过网络/完整 event bus 传到远端客户端平台服务；`DefaultGameService` 的持久化仍是内存态，后续要接平台/磁盘存储。

---

## 110. 最新闭环记录：CellLiquid.update 即时 deposit 顺序

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：收紧 Java `CellLiquid.update(Puddle)` 中 building 吸收/damage 分支与 nearby puddle absorption 的执行顺序。
- Java 依据：
  - `CellLiquid.update()` 中 `Geometry.d4c` building 吸收立即 `Puddles.deposit(tile, this, amount * spreadConversion)`；
  - current-building damage/spread 分支立即对 `Geometry.d4` 执行 `Puddles.deposit(puddle.tile, other, puddle.liquid, amountSpread)`；
  - 然后才进入 `spread to nearby puddles`；
  - 最后如果 `reacted && this == Liquids.neoplasm` 才 `Events.fire(Trigger.neoplasmReact)`。
- Rust 主改动：
  - `server/src/lib.rs`
    - `tick_server_puddles(...)` 移除延迟 `cell_deposits` 队列；
    - building d4c 吸收后立即写入 `runtime.server_puddles.deposit(...)`；
    - current-building amountSpread 对 d4 方向立即 deposit，再 damage building；
    - nearby puddle absorb 仍在这些即时 deposit 之后执行。
  - 新增测试：
    - `server_puddle_cell_liquid_building_deposit_precedes_neighbor_absorb_like_java`
    - 构造“邻接 tile 同时有 water building + water puddle”的场景，断言 replacement neoplasm puddle 没有被延迟 building deposit 写成 same-liquid accepting。
- 已跑验证：
  - `cargo test -p mindustry-server server_puddle_cell_liquid_building_deposit_precedes_neighbor_absorb_like_java --lib`
  - `cargo test -p mindustry-server server_puddle_cell_liquid_update --lib`
  - `cargo check --workspace`
  - `cargo fmt --check`
  - `git diff --check`
- 当前仍需继续：
  1. 可补一次 `cargo test -p mindustry-server --lib` 或全 workspace test 后提交；
  2. 中文提交并推送 `origin main`，建议标题：`收紧新生物液体更新顺序`；
  3. 后续欠账：`Puddles::update_all_with_passability_report(...)` 普通 overfilled spread 仍是整轮后统一落库，若继续 Java tick 顺序路线，可补 `puddle_update_spread_deposits_are_visible_to_later_puddles_same_tick`。

---

## 111. 最新闭环记录：PuddleComp.update 普通外溢即时 deposit 顺序

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补齐上一节留下的普通 `PuddleComp.update()` overfilled spread 顺序欠账，让同 tick 中更早 puddle 的外溢 deposit 对后续既有 puddle 可见。
- Java 依据：
  - `PuddleComp.update()` 在 `amount >= maxLiquid / 1.5f` 时遍历 `Geometry.d4`；
  - 每个可通过邻格立即执行 `Puddles.deposit(other, tile, liquid, deposited, false)`；
  - `EntityGroup.update()` 以 entity group 顺序执行，后续既有 puddle 会在同 tick 消费 earlier deposit 写入的 `accepting`。
- Rust 主改动：
  - `core/src/mindustry/entities/puddles.rs`
    - `update_all_with_passability_report(...)` 先按 `(puddle.id, tile)` 排序 tick 开始时已有 puddle，避免 `HashMap` 顺序污染 Java-like entity update；
    - 每个 puddle 的 spread deposit 在该 puddle update 后立即写入，不再等整轮结束；
    - 新增 `puddle_update_spread_deposits_are_visible_to_later_puddles_same_tick`。
- 已跑验证：
  - `cargo test -p mindustry-core puddle_update_spread_deposits_are_visible_to_later_puddles_same_tick --lib`
  - `cargo test -p mindustry-core update_all_spread --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo test -p mindustry-server server_puddle_cell_liquid_building_deposit_precedes_neighbor_absorb_like_java --lib`
  - `cargo test -p mindustry-server puddle --lib`
- 当前仍需继续：
  1. 跑收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo fmt --check`、`git diff --check`；
  2. 中文提交并推送 `origin main`，建议标题：`收紧液体坑外溢更新顺序`；
  3. 后续欠账：
     - Java `EntityGroup.update()` 动态 `array.size` 让新创建 puddle 同 tick 更新的欠账已在下一闭环继续处理；
     - `puddle_on_building` 与 `particle_effect` 已有 event，但 server consumer 仍未完整接入；
     - `CellLiquid.update` 仍是 report 后统一处理，不是每个 puddle inline 执行，后续要继续收紧 phase boundary。

---

## 112. 最新闭环记录：Puddles 同 tick 追加新建外溢 puddle

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：对照 Java `EntityGroup.update()` 动态 `array.size`，让 `Puddles.deposit(...)` 在外溢 spread 中新建的 puddle 能追加到同一 tick 的 update queue。
- Java 依据：
  - `EntityGroup.update()` 不是固定快照，而是 `index < array.size`；
  - `Puddles.deposit(...)` 创建新 puddle 后 `puddle.add()` 追加到 group；
  - 因此新建外溢 puddle 会在同一 tick 继续执行一次自己的 `PuddleComp.update()`。
- Rust 主改动：
  - `core/src/mindustry/entities/puddles.rs`
    - `update_all_with_passability_report(...)` 改为 index-based queue；
    - tick 初始已有 puddle 按 `(id, tile)` 排序；
    - immediate spread deposit 若创建新 puddle，则把新 `(id, tile)` append 到当前 queue；
    - 用 `processed_ids` 避免同一 puddle id 重复更新；
    - `update_all_spreads_overfilled_puddles_to_d4_neighbors` 中新建邻居 amount 从 `0.3` 收紧到 `0.2`，锁定同 tick 追加后的一次蒸发。
- 已跑验证：
  - `cargo test -p mindustry-core update_all_spread --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo test -p mindustry-server puddle --lib`
- 当前仍需继续：
  1. 跑收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo fmt --check`、`git diff --check`；
  2. 中文提交并推送 `origin main`，建议标题：`追加新建液体坑同帧更新`；
  3. 后续欠账：
     - Java remove/swap 与 Rust `HashMap` 延迟 remove 的复杂顺序已在下一闭环先处理基础即时清理；
     - `CellLiquid.update` 仍未 inline 到单 puddle update 末尾；
     - `puddle_on_building`/`particle_effect` event consumer 仍需继续确认和接入。

---

## 113. 最新闭环记录：Puddles remove 对后续同帧 deposit 立即可见

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：对照 Java `PuddleComp.remove()` / `Puddles.remove(tile)`，让低量 puddle 被移除后立即清空 tile registry，避免后续同 tick deposit 写入一个即将删除的旧 puddle。
- Java 依据：
  - `PuddleComp.update()` 中 `amount <= 0f` 立即 `remove()`；
  - `PuddleComp.remove()` 实际执行 `Puddles.remove(tile)`；
  - `Puddles.remove(tile)` 立即 `world.tiles.setPuddle(tile.array(), null)`；
  - 后续 `Puddles.deposit(...)` 命中同 tile 时应看到无 puddle 并创建新实体。
- Rust 主改动：
  - `core/src/mindustry/entities/puddles.rs`
    - `update_all_with_passability_report(...)` 移除整轮末尾 `remove_keys` 延迟清理；
    - 单个 puddle update 判定 removed 后立即 `self.puddles.remove(&key)` 并继续后续 queue；
    - 新增 `update_all_removes_empty_puddle_before_later_same_tick_deposit`，覆盖“低量 puddle 先删、后续 source 同帧 spread 到该 tile 创建 replacement”的场景。
- 已跑验证：
  - `cargo test -p mindustry-core update_all_removes_empty_puddle_before_later_same_tick_deposit --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo test -p mindustry-server puddle --lib`
- 当前仍需继续：
  1. 跑收尾验证：`cargo check -p mindustry-core`、`cargo check -p mindustry-server`、`cargo fmt --check`、`git diff --check`；
  2. 中文提交并推送 `origin main`，建议标题：`即时清理移除液体坑`；
  3. 后续欠账：
     - `CellLiquid.update` report 后统一处理的时序欠账已在下一闭环继续处理；
     - Java `EntityGroup.remove` swap/index 行为与 Rust tile-keyed map 仍有复杂边界差异。

---

## 114. 最新闭环记录：CellLiquid.update 按 puddle 顺序 inline 到 server tick

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：把 server 侧 `CellLiquid.update(Puddle)` 从“整轮 puddle update 后批处理”提前到每个 puddle update 后的 callback，更接近 Java `PuddleComp.update()` 末尾立即 `liquid.update(self())`。
- Java 依据：
  - `PuddleComp.update()` 执行完基础蒸发、外溢、effects-only、`updateTime -= Time.delta` 后，马上调用 `liquid.update(self())`；
  - 较早 neoplasm puddle 的 `CellLiquid.update` 可以影响同 tick 后续 water puddle 的 base update。
- Rust 主改动：
  - `core/src/mindustry/entities/puddles.rs`
    - 新增 `update_all_with_passability_report_and_event_handler(...)`；
    - 旧 `update_all_with_passability_report(...)` 保持兼容，默认传 no-op callback；
    - per-puddle callback 在 current puddle update + immediate spread deposits 后调用；
    - callback 返回 touched tile keys，外层会追加新建/替换且未处理的 puddle；
    - 增加 current-id mismatch 检查，避免旧 queued key 在 tile replacement 后错误更新新 puddle。
  - `core/src/mindustry/entities/mod.rs`
    - re-export `PuddleUpdateEvent`，供 server callback 使用。
  - `server/src/lib.rs`
    - `tick_server_puddles(...)` 临时取出 `runtime.server_puddles`，用 world/content snapshot 做 passability；
    - per-puddle callback 调用 `process_server_puddle_liquid_update(...)`；
    - 移除旧的整轮后 `liquid_update` loop，避免双跑；
    - 新增 `server_puddle_cell_liquid_update_runs_before_later_puddle_base_update`。
- 新回归场景：
  - neoplasm 先创建、water 后创建；
  - water 初始 amount 在旧 batch 模型下会先外溢；
  - inline 后 neoplasm 先吸收 water，使 water 掉到 spread 阈值以下；
  - 断言 `(3,2)` 没有多余 water spread puddle。
- 已跑验证：
  - `cargo test -p mindustry-server server_puddle_cell_liquid_update_runs_before_later_puddle_base_update --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo test -p mindustry-server puddle --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `cargo fmt --check`
  - `git diff --check`
- 当前仍需继续：
  1. 中文提交并推送 `origin main`，建议标题：`内联新生物液体更新时机`；
  2. 后续欠账：
     - `affect_units/create_fire` 已在下一闭环继续搬到 per-puddle callback；`puddle_on_building/particle_effect` 仍未完全接入；
     - touched tile keys 当前覆盖 center+D4，后续更多 liquid side-effect 若触达更远范围要扩展。

---

## 115. 最新闭环记录：Puddle effects-only 单位状态与起火 inline 顺序

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：继续把 Java `PuddleComp.update()` effects-only 分支迁入 per-puddle callback，避免 server 侧 `affect_units/create_fire` 仍在整轮后批处理。
- Java 依据：
  - effects-only 分支内先 `Units.nearby(...)`；
  - 然后热液体 + building 概率 `Fires.create(tile)`；
  - 然后 `tile.build.puddleOn(self())`；
  - 最后才会走到 `liquid.update(self())`。
- Rust 主改动：
  - `server/src/lib.rs`
    - 新增 `process_server_puddle_affect_units(...)`，从旧 batch loop 提取单位 status/ripple 逻辑；
    - 新增 `process_server_puddle_create_fire(...)`，从旧 batch create_fire loop 提取 fire 创建逻辑；
    - `tick_server_puddles(...)` 的 per-puddle callback 顺序变为 `affect_units -> create_fire -> CellLiquid/liquid_update`；
    - 删除旧的整轮后 `affect_units` 和 `create_fire` batch loops，避免重复应用。
- 已跑验证：
  - `cargo test -p mindustry-server puddle --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `cargo fmt --check`
  - `git diff --check`
- 当前仍需继续：
  1. 中文提交并推送 `origin main`，建议标题：`内联液体坑效果分支`；
  2. 后续欠账：
     - `puddle_on_building` Java vanilla base 是 no-op 且目前未发现 core override，但 Rust 仍只有 event；
     - `particle_effect` 在 server headless 下不会触发，客户端/非 headless 渲染侧仍未接入；
     - ripple effect 当前仍是 callback 收集、tick 后统一广播，后续如追求 packet 顺序可继续内联发送。

---

## 116. 最新闭环记录：Puddle building hook no-op 与 particle payload

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：闭合 `PuddleComp.update()` 剩余 effects-only 分支中 `tile.build.puddleOn(self())` 的 vanilla no-op 消费边界，并让 `particle_effect` 事件携带 Java `Mathf.range(size)` 所需的 dispatch payload。
- Java 依据：
  - `PuddleComp.update()` 顺序：`Units.nearby(...)` → `Fires.create(tile)` → `tile.build.puddleOn(self())` → particle branch/updateTime decrement → `liquid.update(self())`；
  - `BuildingComp.puddleOn(Puddle)` 在 vanilla core 是空方法，当前参考仓库未发现 override；
  - particle branch 非 headless 时用 `size = Mathf.clamp(amount / (maxLiquid / 1.5f)) * 4f`，再对 x/y 分别 `Mathf.range(size)`。
- Rust 主改动：
  - `core/src/mindustry/entities/puddles.rs`
    - 新增 `PuddleParticleEffectEvent { effect, x, y, range }`；
    - `PuddleUpdateEvent::particle_effect` 从 `Option<String>` 改为 `Option<PuddleParticleEffectEvent>`；
    - `PuddleUpdateEvent::from_plan(...)` 按 Java size 公式生成 range，headless/none effect 仍不产出 payload。
  - `core/src/mindustry/entities/mod.rs`
    - re-export `PuddleParticleEffectEvent`。
  - `server/src/lib.rs`
    - 新增 `process_server_puddle_on_building(...)`；
    - `tick_server_puddles(...)` per-puddle callback 顺序变为 `affect_units -> create_fire -> puddle_on_building(no-op) -> CellLiquid/liquid_update`；
    - 当前 no-op consumer 不产生额外 side effect，符合 Java vanilla base。
- 新增测试：
  - `update_all_report_carries_particle_effect_spawn_range_for_non_headless_clients`
  - `server_puddle_on_building_vanilla_hook_is_consumed_as_noop`
- 已跑验证：
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo test -p mindustry-server puddle --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `cargo fmt --check`
  - `git diff --check`
- 当前仍需继续：
  1. 中文提交并推送 `origin main`，建议标题：`闭合液体坑建筑钩子`；
  2. 后续欠账：
     - desktop/client 非 headless renderer/effect runtime 仍未真正消费 `PuddleParticleEffectEvent`；
     - ripple effect 仍是 callback 收集、tick 后统一广播，后续如追求 packet 顺序可继续内联发送。

---

## 117. 最新闭环记录：Puddle particle payload 进入客户端本地 effect 队列

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：继续推进 `PuddleUpdateEvent::particle_effect`，把 116 中新增的 `PuddleParticleEffectEvent` 从 payload 接入 `GameRuntime` 现有 `client_local_effect_events`。
- Java 依据：
  - `PuddleComp.update()` 非 headless 分支调用 `liquid.particleEffect.at(x + Mathf.range(size), y + Mathf.range(size))`；
  - 该效果是客户端本地视觉 effect，不应走 server/headless。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `queue_client_puddle_particle_effects(...)`；
    - 输入为 `PuddleUpdateEvent` 列表和随机 offset provider；
    - 对 offset 按 `PuddleParticleEffectEvent::range` clamp，保持 Java `Mathf.range(size)` 范围；
    - 成功解析 `standard_effect_id(...)` 时写入 `client_local_effect_events`，默认 `rotation=0`、`color=-1`、`data=Null`。
- 新增测试：
  - `game_runtime_queues_puddle_particle_payloads_into_client_local_effects`
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core game_runtime_queues_puddle_particle_payloads_into_client_local_effects --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 当前仍需继续：
  1. 中文提交并推送 `origin main`，建议标题：`接入液体坑粒子队列`；
  2. 后续欠账：
     - `standard_effect_id(...)` 仍只覆盖少量已迁移内置 Fx，需要继续迁移完整 Fx registry；
     - desktop/client 仍缺真正 drain/render `client_local_effect_events` 的 renderer pass；
     - 非 headless client puddle tick 主循环仍需接入 `queue_client_puddle_particle_effects(...)`。

---

## 118. 最新闭环记录：Desktop 本地 effect 渲染 drain 边界

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：在 117 已把 puddle particle payload 接入 `client_local_effect_events` 后，为 desktop 侧提供一个明确可测的 renderer 消费边界。
- Rust 主改动：
  - `desktop/src/lib.rs`
    - 新增 `DesktopLauncher::drain_local_effect_events_for_render(...)`；
    - 通过 `std::mem::take` drain `runtime.client_local_effect_events`；
    - 暂不自动放进 `update()`，避免破坏既有同步测试和调用方可观察队列状态。
- 新增测试：
  - `desktop_launcher_drains_local_effect_events_for_render`
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-desktop desktop_launcher_drains_local_effect_events_for_render --lib`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. 中文提交并推送 `origin main`，建议标题：`接入桌面本地特效输出`；
  2. 后续欠账：
     - 真正图形 renderer 仍未迁移；
     - `EffectRegistry`、完整 Fx id/name 映射、`EffectStateComp::draw_with(...)` 到实际绘制命令仍需后续接入；
     - 非 headless client puddle tick 主循环仍需自动调用 `queue_client_puddle_particle_effects(...)`。

---

## 119. 最新闭环记录：Client puddle snapshot 自动触发 particle tick

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：把 puddle particle 从“手动 queue payload”推进为 client snapshot puddle 的自动 tick 行为，并接入 desktop `update()`。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `client_puddle_snapshot_liquids: BTreeMap<i32, PuddleLiquidInfo>`；
    - `apply_client_puddle_sync_wire(...)` 保存 liquid metadata sidecar；
    - hidden snapshot/runtime clear 同步清理 sidecar；
    - 新增 `tick_client_puddle_snapshot_particle_effects(...)`，按 `effect_time + delta >= particle_spacing` 触发本地 effect。
  - `desktop/src/lib.rs`
    - `DesktopLauncher::update()` 自动调用 `tick_client_puddle_snapshot_particle_effects(1.0, |_| (0.0, 0.0))`；
    - 暂用中心点 offset，后续需要替换为 Java `Mathf.range(size)` 等价 RNG。
- 新增/更新测试：
  - `game_runtime_ticks_client_puddle_snapshot_particle_effects`
  - `game_runtime_applies_client_puddle_entity_snapshot_to_typed_runtime` 增加 liquid sidecar 和 hidden cleanup 断言
  - `desktop_launcher_ticks_puddle_particle_snapshots_to_local_effect_queue`
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core game_runtime_ticks_client_puddle_snapshot_particle_effects --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_puddle_entity_snapshot_to_typed_runtime --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_ticks_puddle_particle_snapshots_to_local_effect_queue --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. 中文提交并推送 `origin main`，建议标题：`接入客户端液体坑粒子更新`；
  2. 后续欠账：
     - desktop puddle particle offset 仍是 `(0,0)`，需要接入 Java 等价 RNG；
     - 完整 Fx registry 和真实 renderer 仍未完成；
     - Puddle/CellLiquid 之外仍有大量 gameplay/block/client UI 文件待逐文件迁移。

---

## 120. 最新闭环记录：Desktop puddle particle range RNG

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：把 119 中 desktop puddle particle 的 `(0,0)` 临时 offset 替换为 `[-range, range]` 随机偏移。
- Java 依据：
  - `PuddleComp.update()` 的 particle 分支使用 `liquid.particleEffect.at(x + Mathf.range(size), y + Mathf.range(size))`；
  - X/Y 各采样一次 range offset。
- Rust 主改动：
  - `desktop/src/lib.rs`
    - `DesktopLauncher` 新增 `puddle_particle_rand_state`；
    - 新增 `mix_puddle_particle_seed(...)`、`next_puddle_particle_unit(...)`、`next_puddle_particle_range(...)`；
    - `sync_runtime_state_from_world_data(...)` 使用 world `rand_seed0/rand_seed1` 重置 puddle particle RNG；
    - `update()` 调用 `tick_client_puddle_snapshot_particle_effects(...)` 时生成 X/Y range offset。
- 更新测试：
  - `desktop_launcher_ticks_puddle_particle_snapshots_to_local_effect_queue`
    - 断言坐标在 Java range 范围内；
    - 断言不再退化为 puddle center。
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-desktop desktop_launcher_ticks_puddle_particle_snapshots_to_local_effect_queue --lib`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. 中文提交并推送 `origin main`，建议标题：`随机化液体坑粒子偏移`；
  2. 后续欠账：
     - RNG 目前不是 Arc `Rand` 位级同构；
     - 完整 Fx registry 和真实 renderer 仍未完成；
     - 继续向 Puddles/CellLiquid 外的大量 Java 文件迁移。

---

## 121. 最新闭环记录：扩展 standard Fx id 映射

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补齐当前已迁移 runtime/content 直接用到的一批高频 Fx name -> id 映射，降低本地 effect 队列/Java effect packet 因 `standard_effect_id(...)` 返回 `None` 而丢效果的概率。
- Java 依据：
  - `core/src/mindustry/content/Fx.java`
  - 本轮使用源码声明顺序/既有常量对照补充：
    - `smoke=28`
    - `hitLiquid=85`
    - `fire=119`
    - `fireSmoke=121`
    - `steam=123`
    - `vapor=128`
    - `fireballsmoke=130`
    - `smokeCloud=222`
  - `none` 仍保持 `None`，不入队无效 effect。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增上述 `FX_*_ID` 常量；
    - 扩展 `standard_effect_id(...)`；
    - 扩展 `standard_effect_ids_include_puddle_ripple_dependencies` 测试。
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include_puddle_ripple_dependencies --lib`
  - `cargo test -p mindustry-core puddle --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 当前仍需继续：
  1. 中文提交并推送 `origin main`，建议标题：`扩展标准特效映射`；
  2. 后续欠账：
     - 这仍不是完整 Fx registry；
     - `Fx.ripple` 当前沿用既有常量 `243`，后续完整 Fx id 审计时要统一确认；
     - 真实 renderer 与完整 effect registry 仍未完成。

---

## 122. 最新闭环记录：标准 Fx 本地渲染 primitive 链路

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：把 `EffectCallPacket2` / 本地 `EffectStateComp` 从“只保留 effect state”推进到桌面端每帧可消费的标准 Fx draw/circle/light primitive 缓存。
- 已推送提交链路：
  - `5b98a71 完善标准特效帧缓存清理测试`
  - `58d8453 扩展标准特效粒子绘制计划`
  - `bcc0832 展开标准特效粒子圆图元`
  - `5d7960a 复刻标准特效随机粒子向量`
  - `79a4583 统一标准特效圆形渲染图元`
  - `fe74e73 缓存桌面标准特效圆图元`
  - `b581d06 缓存标准特效光照图元`
  - `f02df0a 解析标准特效颜色符号`
- Java/Arc 依据：
  - `Fx.java` 中 `smoke/missileTrail/missileTrailShort/ripple/fire/fireSmoke/steam/vapor/fireballsmoke/smokeCloud` 的 renderer 公式；
  - Arc `Rand` / `Angles.randLenVectors(...)` / `Mathf.sin/cos` 通过本地 Gradle cache 的 `arc-core-4d9760e264.jar` + `javap` 对照；
  - `Pal.java` 中 `darkishGray/lightFlame/darkFlame`，Arc `Color.gray/lightGray/darkGray`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - `StandardEffectDrawKind` 增加 `SeededCircleParticles`；
    - 新增 `StandardEffectParticleSpec`、`StandardEffectParticleVector`、`StandardEffectCirclePrimitive`；
    - 复刻 Arc `Rand` / `Angles.randLenVectors` 当前所需 overload；
    - 新增 `StandardEffectCircleRenderPrimitive` 与 `StandardEffectLightRenderPrimitive`；
    - 新增 `standard_effect_color_symbol(...)` 与 `StandardEffectDrawPlan::resolved_draw_color()`；
    - `standard_effect_draw_plan(...)` 覆盖当前高频 Fx：`smoke/missileTrail/missileTrailShort/ripple/fire/fireSmoke/steam/vapor/fireballsmoke/smokeCloud`。
  - `desktop/src/lib.rs`
    - `DesktopLauncher` 现在每帧维护：
      - `standard_local_effect_draw_plans`
      - `standard_local_effect_circle_primitives`
      - `standard_local_effect_light_primitives`
    - `update()` 会从本地 effect state 自动生成上述缓存；
    - 世界卸载 / snapshot cursor 清理同步清空这些缓存。
  - `core/src/mindustry/entities/mod.rs`
    - 导出新增 standard effect primitive/color 类型与 helper。
- 最近已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo test -p mindustry-core standard_effect_particle --lib`
  - `cargo test -p mindustry-core standard_effect_plan_resolves --lib`
  - `cargo test -p mindustry-desktop standard_effect_draw --lib`
  - `cargo test -p mindustry-desktop ticks_elude --lib`
  - `cargo test -p mindustry-desktop fire_light --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. 真实桌面窗口/2D/GPU backend 仍未接入；当前是内存帧缓存和 primitive 数据；
  2. `standard_effect_color_symbol(...)` 只覆盖当前已迁移 Fx 所需颜色，不是完整 `Pal.java`/Arc `Color` registry；
  3. 完整 `Fx.java` renderer 仍待继续逐项迁移；
  4. 如果要引入 `winit/wgpu/pixels/sdl2` 等新外部后端依赖，需要按当前开发规则先确认；未确认前优先继续做无依赖 primitive/runtime 接入。

---

## 123. 最新闭环记录：render frame 边界、更多简单 Fx、单位镜像清空

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 已推送提交：
  - `e01904c 暴露桌面特效渲染帧数据`
  - `32ff250 扩展简单标准特效绘制计划`
  - `3846f94 补齐单位镜像清空回归`
- Rust 主改动：
  - `desktop/src/lib.rs`
    - 新增 `DesktopStandardEffectRenderFrame`；
    - 新增 `DesktopLauncher::standard_effect_render_frame()`，统一暴露 draw/circle/light 三组标准 effect 帧缓存；
    - 单位 item mirror 回归：`item=None` 会把 typed `UnitComp.items.stack` 清成空；
    - 单位 payload mirror 回归：`payload_count=0` 会把 typed `UnitComp.payload.payloads` 清空。
  - `core/src/mindustry/entities/effect.rs`
    - 扩展 7 个不需要新 kind 的简单 Fx：
      - `fallSmoke=29`
      - `rocketSmoke=31`
      - `rocketSmokeLarge=32`
      - `magmasmoke=33`
      - `burning=117`
      - `fireHit=120`
      - `blastsmoke=226`
    - 接入 `standard_effect_id(...)`、`standard_effect(...)`、`standard_effect_draw_plan(...)`；
    - 复用 `FilledCircle` / `SeededCircleParticles` 和现有 primitive 链路。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo test -p mindustry-desktop fire_light --lib`
  - `cargo test -p mindustry-desktop unit_item_mirror --lib`
  - `cargo test -p mindustry-desktop unit_payload_mirror --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. `Fx.java` 仍远未完整，下一批可继续做 `smokePuff/shootSmallSmoke/steamCoolSmoke/artilleryTrailSmoke` 等需要小幅新增 kind/overload 的 renderer；
  2. 真实 desktop 2D/GPU backend 尚未接入；当前只到 frame data/primitive；
  3. payload mirror 仍只是 kind/count 近似，不携带真实 payload 内容；
  4. `Fx.ripple` id 仍沿用既有 `243`，完整 content id 审计时需要统一确认。

---

## 124. 最新闭环记录：Fx.smokePuff 双圆粒子迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `Fx.smokePuff`，并让现有标准 effect primitive 链路能表达 Java 中“每个随机向量绘制主/副两枚圆”的 renderer 形态。
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:1814` 附近；
  - `new Effect(30, ...)`；
  - `color(e.color)`；
  - `randLenVectors(e.id, 6, 4f + 30f * e.finpow(), ...)`；
  - 主圆：`Fill.circle(e.x + x, e.y + y, e.fout() * 3f)`；
  - 副圆：`Fill.circle(e.x + x / 2f, e.y + y / 2f, e.fout())`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SMOKE_PUFF_ID = 154`；
    - 接入 `standard_effect_id("smokePuff")`；
    - 接入 `standard_effect(FX_SMOKE_PUFF_ID)`，lifetime `30.0`；
    - `StandardEffectParticleSpec` 新增 secondary circle 参数；
    - `expand_seeded_particle_circles(...)` 支持每个 vector 生成主圆 + 可选副圆；
    - `standard_effect_draw_plan(...)` 新增 `smokePuff`：
      - `count=6`
      - `length=4.0 + 30.0 * finpow`
      - 主圆半径 `3.0 * fout`
      - 副圆 offset scale `0.5`
      - 副圆半径 `1.0 * fout`
      - 颜色使用 packet/local effect 输入色 `e.color`。
- 已跑验证：
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo test -p mindustry-core standard_effect_particle --lib`
  - `cargo test -p mindustry-core standard_effect_plan_resolves --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 注意事项：
  - 这次没有改动真实 renderer backend；仍是无依赖 primitive/data 边界；
  - `Fx.ripple` id 仍沿用既有 `243`，完整 content id 审计时需要统一；
  - 子代理只读审计建议下一批优先迁移：`shootSmallSmoke`、`smokeAoeCloud`、`missileTrailSmokeSmall`、`missileTrailSmoke`、`neoplasmSplat`。

---

## 125. 最新闭环记录：Fx.shootSmallSmoke 方向扇区粒子迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `Fx.shootSmallSmoke`，同时把 `Angles.randLenVectors(seed, amount, length, angle, range, Floatc2)` 的方向扇区重载接入 Rust 标准 effect 粒子链路。
- Java/Arc 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:1850` 附近；
  - id 按 `new Effect` 声明顺序为 `159`；
  - lifetime `20f`；
  - 颜色 `Pal.lighterOrange -> Color.lightGray -> Color.gray`，插值参数 `e.fin()`；
  - 粒子 `randLenVectors(e.id, 5, e.finpow() * 6f, e.rotation, 20f, ...)`；
  - 圆半径 `e.fout() * 1.5f`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SHOOT_SMALL_SMOKE_ID = 159`；
    - 接入 `standard_effect_id("shootSmallSmoke")` 与 `standard_effect(...)`；
    - `StandardEffectParticleSpec` 新增 `angle/angle_range`；
    - `ArcRand` 新增 `range(...)`；
    - `seeded_vectors()` 支持 `angle + rand.range(angle_range)`；
    - `StandardEffectDrawPlan` 新增 `color_mid`；
    - `resolved_draw_color()` 支持三段颜色插值；
    - `standard_effect_color_symbol(...)` 新增 `Pal.lighterOrange`；
    - `standard_effect_draw_plan(...)` 新增 `shootSmallSmoke`。
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
- 下一步建议：
  1. 继续圆粒子/光照系列：`smokeAoeCloud`、`missileTrailSmokeSmall`、`missileTrailSmoke`；
  2. 后续如果迁移 `steamCoolSmoke`，需要补 `Interp.pow2Out/pow3Out` 与 direction/progress 组合；
  3. 三角形类 `shootSmall/shootBig` 需要先设计 triangle primitive。

---

## 126. 最新闭环记录：Fx.smokeAoeCloud 高数量烟云迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `Fx.smokeAoeCloud`，补齐高 count 圆粒子烟云和非默认 clip 的标准 effect metadata。
- Java 依据：
  - `D:/MDT/mindustry-upstream-v157.4/core/src/mindustry/content/Fx.java:604` 附近；
  - id 为 `55`；
  - lifetime `60f * 3f = 180f`；
  - clip `250f`；
  - 颜色 `e.color`，alpha `0.65f`；
  - `randLenVectors(e.id, 80, 90f, ...)`；
  - 半径 `6f * clamp(fin / 0.1f) * clamp(fout / 0.1f)`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SMOKE_AOE_CLOUD_ID = 55`；
    - 接入 `standard_effect_id("smokeAoeCloud")`；
    - 接入 `standard_effect(...)`，lifetime `180.0`、clip `250.0`；
    - `standard_effect_draw_plan(...)` 新增 `smokeAoeCloud`，复用 `SeededCircleParticles`，`count=80`、`length=90.0`、`alpha=0.65`、半径使用 plan 阶段固定 `radius_base`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `git diff --check`
- 下一步建议：
  1. 继续 `missileTrailSmokeSmall` / `missileTrailSmoke`，需要对 `rand.setSeed(b.id*2+i)` 多 pass 粒子和 `Drawf.light` 做可复用建模；
  2. 或先做 `steamCoolSmoke`，需要补 Interp pow2Out/pow3Out 与方向扇区公式。

---

## 127. 最新闭环记录：Arc Scaled.finpow 纠偏

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：纠正 Rust effect 基础时间曲线，保证 `e.finpow()` 与 Java/Arc 一致。
- Java/Arc 依据：
  - `EffectContainer` 只实现 `fin()`；
  - `arc.math.Scaled.finpow()` 字节码为 `Interp.pow3Out.apply(fin())`；
  - 等价公式：`1.0 - (1.0 - fin)^3`，不是 `fin^2`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `effect_finpow_from_fin(fin)`；
    - `standard_effect_draw_plan(...)` 的局部 `finpow` 改用 pow3Out；
    - `EffectContainer::finpow()` 改用同一 helper；
    - 更新 `vapor/smokePuff/shootSmallSmoke` 相关测试期望；
    - `shootSmallSmoke` 的 Java probe vector 期望已按 length `5.25` 重算。
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
- 下一步建议：
  1. 迁移 `steamCoolSmoke` 时可以直接复用已纠正的 `finpow`，还需补 `Interp.pow2Out` 和 `fout(Interp.pow3Out)`；
  2. 后续审计其它 `Scaled` 默认方法，避免类似 `finpow/foutpow` 曲线偏差。

---

## 128. 最新闭环记录：Fx.steamCoolSmoke 方向冷却烟迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `Fx.steamCoolSmoke`，补齐该效果所需的 `pow2Out` 颜色插值、`fout(pow3Out)` alpha 和方向扇区粒子。
- Java 依据：
  - `Fx.java:1804` 附近；
  - id `153`；
  - lifetime `35f`；
  - `Pal.water -> Color.lightGray`，mix 为 `e.fin(Interp.pow2Out)`；
  - alpha 为 `e.fout(Interp.pow3Out)`；
  - `randLenVectors(e.id, 4, e.finpow() * 7f, e.rotation, 30f, ...)`；
  - 半径 `max(fout, min(1, fin * 8)) * 2.8`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_STEAM_COOL_SMOKE_ID = 153`；
    - 接入 `standard_effect_id("steamCoolSmoke")` 与 `standard_effect(...)`；
    - 新增 `interp_pow2_out(...)` / `interp_pow3_out(...)` helper；
    - `standard_effect_color_symbol(...)` 新增 `Pal.water`；
    - `standard_effect_draw_plan(...)` 新增 `steamCoolSmoke`。
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
- 下一步建议：
  1. 继续 `missileTrailSmokeSmall/missileTrailSmoke`，需要多 pass 粒子和 per-particle light；
  2. 或先补 triangle primitive，再做 `shootSmall/shootBig` 系列。

---

## 129. 最新闭环记录：Fx.shootBigSmoke 系列方向烟雾迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `shootBigSmoke`、`shootBigSmoke2`、`shootSmokeDisperse` 三个与 `shootSmallSmoke` 同构的方向烟雾 Fx。
- Java 依据：
  - `Fx.java:1967-1989`；
  - ids：
    - `shootBigSmoke=166`
    - `shootBigSmoke2=167`
    - `shootSmokeDisperse=168`
  - lifetimes：
    - `17f`
    - `18f`
    - `25f`
  - 均使用 `randLenVectors(e.id, count, e.finpow() * scale, e.rotation, range, ...)`；
  - 均画 `Fill.circle`，无 light、无 triangle/poly。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增三个 `FX_*` 常量；
    - 接入 `standard_effect_id(...)` 与 `standard_effect(...)`；
    - 新增 `Pal.lightOrange` 颜色符号；
    - `standard_effect_draw_plan(...)` 新增共享分支，按 effect id 参数化：
      - color_from/color_mid；
      - particle count；
      - `finpow` length scale；
      - angle range；
      - radius base / fout scale。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 注意：
  - `artilleryTrailSmoke` 已核对但未迁移：它需要 per-particle lifetime/random alpha/conditional skip；
  - `shootSmokeSquare*` 需要 polygon/square primitive；
  - 下一步可以继续 `missileTrailSmokeSmall/missileTrailSmoke`，但需要设计多 pass + per-particle light。

---

## 130. 最新闭环记录：Fx.hitLiquid draw plan 迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补齐 `hitLiquid` 的标准方向粒子 draw plan。该 Fx 的 id/lifetime 之前已有，现在补实际 primitive 展开。
- Java 依据：
  - `Fx.java:963` 附近；
  - lifetime `16`；
  - `color(e.color)`；
  - `randLenVectors(e.id, 5, 1f + e.fin() * 15f, e.rotation, 60f, ...)`；
  - 半径 `e.fout() * 2f`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - `standard_effect_draw_plan(...)` 新增 `FX_HIT_LIQUID_ID` 分支；
    - 复用 `SeededCircleParticles`、input color、方向扇区和 `radius_fout_scale=2.0`；
    - lookup 测试明确断言 lifetime `16.0`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 注意：
  - `missileTrailSmoke*` 已核对但仍未迁移，因为需要多 pass、scaled lifetime、pow10/pow5、per-particle light；
  - 不要用单一 `SeededCircleParticles` 近似它们，后续应先设计专用 multi-pass spec。

---

## 131. 最新闭环记录：Fx.corrosionVapor / Fx.vaporSmall 迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补齐 `vapor` 邻近的 `corrosionVapor` 与 `vaporSmall` 标准 effect metadata/draw plan。
- Java 依据：
  - `Fx.java:1498-1524`；
  - ids：
    - `corrosionVapor=127`
    - `vaporSmall=129`
  - lifetimes 均为 `50f`；
  - `corrosionVapor`：`alpha=pow2Out(fslope)*0.5`、`count=2`、`length=8+finpow*3`、半径 `3`；
  - `vaporSmall`：`alpha=fout`、`count=4`、`length=2+finpow*5`、半径 `1+fin*4`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_CORROSION_VAPOR_ID` / `FX_VAPOR_SMALL_ID`；
    - 接入 `standard_effect_id(...)` / `standard_effect(...)`；
    - `standard_effect_draw_plan(...)` 新增两个 `SeededCircleParticles` 分支；
    - 测试覆盖 id、lifetime、alpha、count、length、radius 字段。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 继续挑选无需新 primitive 的 `Fill.circle` 类 Fx；
  2. 或设计 `missileTrailSmoke*` 的 multi-pass trail spec，避免近似。

---

## 132. 最新闭环记录：Fx.blockExplosionSmoke 双圆烟迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `blockExplosionSmoke`，复用此前为 `smokePuff` 增加的每 vector 双圆展开能力。
- Java 依据：
  - `Fx.java:1795-1803`；
  - id `152`；
  - lifetime `30`；
  - `Color.gray`；
  - `randLenVectors(e.id, 6, 4 + 30 * finpow, ...)`；
  - 主圆半径 `fout * 3`，副圆 offset `0.5`、半径 `fout`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_BLOCK_EXPLOSION_SMOKE_ID = 152`；
    - 接入 name/id/lifetime；
    - `standard_effect_draw_plan(...)` 新增 `blockExplosionSmoke` 分支；
    - 测试覆盖 6 vectors 展开为 12 circle primitives。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 继续找纯 `Fill.circle` 且无需 line/poly/light 的 Fx；
  2. 对复杂 explosion/trail 类，先设计 line/per-particle light/multi-pass spec。

---

## 133. 最新闭环记录：Debris/unit dust 圆粒子 Fx 批量迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移无需新增 primitive 的 debris/unit dust 圆粒子效果：
  - `breakProp=37`
  - `unitDrop=38`
  - `unitLand=39`
  - `unitDust=40`
  - `unitLandSmall=41`
  - `crawlDust=43`
- Java 依据：
  - `Fx.java:378-427`；
  - 这批都 `.layer(Layer.debris)`；
  - 均可表达为 `SeededCircleParticles`；
  - `unitDrop` 需要 `Pal.lightishGray = 0xa2a2a2ff`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 6 个 `FX_*` 常量；
    - 接入 `standard_effect_id(...)` / `standard_effect(...)`；
    - 设置 `Layer::DEBRIS`；
    - 新增 `Pal.lightishGray`；
    - `standard_effect_draw_plan(...)` 新增共享分支，参数化 color/color_mul/count/length/radius/angle。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 继续挑简单 `Fill.circle` Fx；
  2. 对 `unitPickup/landShock` 等 line/poly 类先补 primitive；
  3. 对 `missileTrailSmoke*` 先补 multi-pass/per-particle-light spec。

---

## 134. 最新闭环记录：fire/liquid/status 简单圆形 Fx 批量迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移无需新增 primitive 的 fire/liquid/status 简单圆形效果：
  - `ballfire=131`
  - `freezing=132`
  - `wet=134`
  - `muddy=135`
  - `sporeSlowed=138`
  - `oily=139`
- Java 依据：
  - `Fx.java:1533-1599`；
  - `Liquids.java` / `Pal.java` 中对应颜色；
  - 这批只需要 `SeededCircleParticles` 或 `FilledCircle`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 6 个 `FX_*` 常量；
    - 接入 `standard_effect_id(...)` / `standard_effect(...)`；
    - 新增颜色符号：
      - `Liquids.water.color`
      - `Liquids.cryofluid.color`
      - `Liquids.oil.color`
      - `Pal.muddy`
      - `Pal.spore`
    - `standard_effect_draw_plan(...)` 覆盖粒子/中心圆参数。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 注意：
  - `melting` 暂未迁移，需要先复刻 `Mathf.randomSeedRange(...)` 颜色扰动；
  - `sapped/electrified/overdriven/overclocked` 是 square/poly 类，需新增 primitive；
  - `sporeSlowed` Fx 本体已迁移，但 Java `StatusEffects.sporeSlowed` wiring 后续要单独核对。

---

## 135. 最新闭环记录：Fx.melting 熔融圆形粒子迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `Fx.melting=133`，并补齐 Arc `Mathf.randomSeedRange(...)` 的 Rust 对齐 helper，避免颜色扰动近似实现。
- Java 依据：
  - `Fx.java:1550-1556`；
  - lifetime `40f`；
  - `Liquids.slag.color -> Color.white`；
  - `colorMix = fout / 5 + Mathf.randomSeedRange(e.id, 0.12f)`；
  - `randLenVectors(e.id, 2, 1 + fin * 3, ...)`；
  - 半径 `0.2 + fout * 1.2`。
- Arc 语义核对：
  - `javap` 证实 `Mathf.randomSeedRange(seed, range)` 先 `seed * 99999L` 再 `Rand.setSeed(...)`；
  - 返回 `(nextFloat() - 0.5f) * range * 2f`；
  - `jshell` golden：`randomSeedRange(133, 0.12) = -0.085423604`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_MELTING_ID = 133`；
    - 接入 `standard_effect_id(...)` / `standard_effect(...)`；
    - 新增 `mathf_random_seed_range(seed, range)`；
    - 新增 `Liquids.slag.color = 0xffa166ff`；
    - `standard_effect_draw_plan(...)` 新增 `FX_MELTING_ID` 分支；
    - 测试覆盖 name/id、lifetime、Arc seeded range golden、draw plan 和 primitive 展开。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core mathf_random_seed_range --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan --lib`
- 下一步建议：
  1. 继续挑无需新增 primitive 的 `Fill.circle` 类 Fx；
  2. 对 `sapped/electrified/overdriven/overclocked` 新增 square/poly primitive 后迁移；
  3. 对 `missileTrailSmoke*` / `artilleryTrailSmoke` 先设计 multi-pass、局部 lifetime、per-particle light/alpha spec。

---

## 136. 最新闭环记录：Shockwave 圆环 Fx 批量迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移只使用 `Lines.circle` 的 shockwave 圆环效果，直接复用 Rust 已有 `StrokedCircle` primitive。
- 本轮迁移：
  - `shockwave=143`
  - `shockwaveSmaller=144`
  - `bigShockwave=145`
  - `spawnShockwave=146`
  - `podLandShockwave=147`
- Java 依据：
  - `Fx.java:1625-1647`；
  - `shockwave/shockwaveSmaller/bigShockwave/spawnShockwave` 均为 `Color.white -> Color.lightGray` 圆环；
  - `spawnShockwave` 的半径依赖 `rotation + 50`；
  - `podLandShockwave` 使用 `Pal.accent = 0xffd37fff`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 5 个 `FX_*` 常量；
    - 接入 `standard_effect_id(...)` / `standard_effect(...)`；
    - 新增 `Pal.accent` 颜色符号；
    - `standard_effect_draw_plan(...)` 新增 shared `StrokedCircle` 分支；
    - 测试覆盖 id、lifetime、clip、radius、stroke、颜色插值和 `Pal.accent`。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
- 下一步建议：
  1. 可继续迁移 `launchAccelerator=246`、`launch=247`、`healWaveMend=249`、`overdriveWave=250` 这类后段纯 `Lines.circle` Fx；
  2. 若要处理当前相邻的 `sapped/electrified/overdriven/overclocked`，需先新增 square primitive；
  3. `bubble=245` 是随机位置圆环，建议等 seeded stroked-circle particles 能力补齐后再做。

---

## 137. 最新闭环记录：Launch/heal/overdrive 圆环 Fx 迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：继续迁移后段纯 `Lines.circle` Fx，并修正 `ripple` 的 Java 声明顺序 ID。
- 本轮迁移/修正：
  - `ripple=244`（此前 Rust 常量是 `243`，本轮按 `Fx.java` `new Effect` 顺序修正）
  - `launchAccelerator=246`
  - `launch=247`
  - `healWaveMend=249`
  - `overdriveWave=250`
- Java 依据：
  - `Fx.java:2720-2772`；
  - `launchAccelerator`：`Pal.accent`，stroke `fout*2`，半径 `4 + finpow*160`；
  - `launch`：`Pal.command`，stroke `fout*2`，半径 `4 + finpow*120`；
  - `healWaveMend`：输入颜色，stroke `fout*2`，半径 `finpow*rotation`；
  - `overdriveWave`：输入颜色，stroke `fout`，半径 `finpow*rotation`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 修正 `FX_RIPPLE_ID`；
    - 新增 4 个 `FX_*` 常量；
    - 新增 `Pal.command = 0xeab678ff`；
    - 接入 metadata/name lookup/draw plan；
    - `healWaveMend/overdriveWave` 走 `input_color`。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
- 下一步建议：
  1. 若继续做 `bubble=245`，先把 `SeededCircleParticles` 扩展为支持 stroked circle 粒子；
  2. `launchPod=248` 需要 scaled 子时间片与 lineAngle，先不要近似硬塞；
  3. 可以转去补 square primitive，以迁移 `sapped/electrified/overdriven/overclocked/healBlock` 等。

---

## 138. 最新闭环记录：Heal/shield 圆环 Fx 迁移

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移前段 healing/shield 中只使用 `Lines.circle` 的效果，继续复用 `StrokedCircle`。
- 本轮迁移：
  - `healWaveDynamic=70`
  - `healWave=71`
  - `heal=72`
  - `dynamicWave=73`
  - `shieldWave=74`
  - `shieldApply=75`
- Java 依据：
  - `Fx.java:805-829`；
  - `heal*` 使用 `Pal.heal = 0x98ffa9ff`；
  - `dynamicWave/shieldWave/shieldApply` 使用输入色，alpha `0.7`；
  - 半径分别是 `4 + finpow*rotation`、`4 + finpow*60`、`2 + finpow*7`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 6 个 `FX_*` 常量；
    - 新增 `Pal.heal` 颜色符号；
    - 接入 metadata/name lookup/draw plan；
    - `dynamic/shield` 输入色分支设置 `alpha=0.7`。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
- 下一步建议：
  1. 若继续 Fx，可优先设计 square primitive，解锁 `sapped/electrified/overdriven/overclocked/healBlock`；
  2. 若做 `bubble`，先扩展 seeded stroked circle particles；
  3. 避免把 `dynamicSpikes/greenBomb/hitBullet*` 这类含 triangle/light/line 的效果近似硬塞到单圆环。

---

## 139. 最新闭环记录：Square primitive 与 status/overdrive 方块 Fx

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补最小 `Fill.square` 表达，迁移之前被 square primitive 阻塞的 4 个 Fx。
- 本轮迁移：
  - `sapped=136`
  - `electrified=137`
  - `overdriven=140`
  - `overclocked=141`
- Java 依据：
  - `Fx.java:1572-1614`；
  - `sapped/electrified`：2 个随机 45° 方块，半径 `fslope*1.1`；
  - `overdriven`：2 个随机无旋转方块，半径 `fout*2.3+0.5`；
  - `overclocked`：中心 45° 方块，半径 `fslope*2`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FilledSquare` / `SeededSquareParticles`；
    - 新增 `StandardEffectSquareRenderPrimitive` 与 `square_render_primitives_from_seed()`；
    - 新增 `FX_SAPPED_ID` / `FX_ELECTRIFIED_ID` / `FX_OVERDRIVEN_ID` / `FX_OVERCLOCKED_ID`；
    - 新增 `Pal.sap = 0x665c9fff`；
    - 接入标准 metadata 与 draw plan。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles --lib`
- 下一步建议：
  1. 若继续方形类，补 `StrokedSquare` 后做 `healBlock` 等 `Lines.square`；
  2. 若继续圆环粒子，补 seeded stroked-circle particles 后做 `bubble=245`；
  3. 真实 renderer backend 仍需消费 `StandardEffectSquareRenderPrimitive`。

---

## 140. 最新闭环记录：StrokedSquare 与 block square Fx

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补 `Lines.square` 需要的 `StrokedSquare`，并迁移 block 方块类简单 Fx。
- 本轮迁移：
  - `healBlock=251`
  - `rotateBlock=253`
  - `lightBlock=254`
  - `overdriveBlockFull=255`
- Java 依据：
  - `Fx.java:2775-2795`；
  - `healBlock`：`Pal.heal`，stroke `2*fout+0.5`，半径约 `fin*rotation*tilesize/2`；
  - `rotateBlock`：`Pal.accent`，alpha `fout`，半径 `rotation*tilesize/2`；
  - `lightBlock`：输入色，alpha `fout`；
  - `overdriveBlockFull`：输入色，alpha `fslope*0.4`，半径 `rotation*tilesize`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `StrokedSquare`；
    - `StandardEffectSquareRenderPrimitive` 增加 `stroke`；
    - `square_render_primitives_from_seed()` 支持 stroked square；
    - 新增 4 个 `FX_*` 常量；
    - 引入 `vars::TILE_SIZE` 做 Java `tilesize` 对齐。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
- 下一步建议：
  1. `healBlockFull` 需要 block icon/rect/mixcol，不能直接塞 square；
  2. `bubble=245` 可通过 seeded stroked-circle particles 解锁；
  3. `shieldBreak` 需要 poly/arc primitive。

---

## 141. 最新闭环记录：Seeded stroked-circle particles 与 Fx.bubble

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补随机位置圆环粒子表达，迁移 `bubble=245`。
- Java 依据：
  - `Fx.java:2727-2732`；
  - `color(Tmp.c1.set(e.color).shiftValue(0.1f))`；
  - `stroke(fout+0.2)`；
  - `randLenVectors(id, 2, rotation*0.9)`；
  - 圆环半径 `1+fin*3`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `SeededStrokedCircleParticles`；
    - `circle_render_primitives_from_seed()` 支持展开为 `StrokedCircle`；
    - 新增 `shift_color_value(...)` / `color_from_hsv(...)` 对齐 `Color.shiftValue`；
    - 新增 `FX_BUBBLE_ID = 245` 并接入 draw plan。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
- 下一步建议：
  1. `launchPod=248` 需要 scaled circle + lineAngle；
  2. `healBlockFull=252` 需要 rect/icon/mixcol；
  3. `shieldBreak` 需要 poly/arc primitive。

---

## 142. 最新闭环记录：Seeded line primitive 与 Fx.disperseTrail

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补 `lineAngle` 的最小 seeded line primitive，并迁移 `disperseTrail=76`。
- Java 依据：
  - `Fx.java:841-850`；
  - lifetime `13`；
  - 颜色 `Color.white -> e.color`，mix `fin`；
  - stroke `0.6 + fout*1.7`；
  - 随机顺序：`range(15)`、`random(fin*27)`、`random(2,7)`；
  - line length `fout*random(2,7)+1.5`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `SeededLineParticles`；
    - 新增 `StandardEffectLineRenderPrimitive`；
    - 新增 `line_render_primitives_from_seed()`；
    - `ArcRand` 新增 `random_between(min,max)`；
    - 新增并接入 `FX_DISPERSE_TRAIL_ID = 76`；
    - 测试使用 Java probe golden 锁定两个线段。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
- 下一步建议：
  1. 可继续向 `hitBulletSmall/hitBulletColor` 推进，但需要 scaled + line + light；
  2. `launchPod=248` 现在已有 line primitive，但还缺 scaled 子时间片组合表达；
  3. `shieldBreak` 仍需 poly/arc primitive。

---

## 143. 最新闭环记录：Radial line primitive、hitBulletBig/hitFlame 与 desktop line/square cache

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补 `randLenVectors(id, count, len, rotation, cone)` + `lineAngle` 的径向线段表达，并把已有 line/square primitive 接入 desktop frame cache，不再只停留在 core 测试 helper。
- 本轮迁移：
  - `hitBulletBig=82`
  - `hitFlameSmall=83`
  - `hitFlamePlasma=84`
- Java 依据：
  - `Fx.java:934-942`：`hitBulletBig`，lifetime `13`，颜色 `Color.white -> Pal.lightOrange`，stroke `0.5 + fout*1.5`，8 条 cone line，line length `fout*4+1.5`。
  - `Fx.java:944-952`：`hitFlameSmall`，lifetime `14`，颜色 `Pal.lightFlame -> Pal.darkFlame`，2 条 cone line，line length `fout*3+1`。
  - `Fx.java:954-962`：`hitFlamePlasma`，lifetime `14`，颜色 `Color.white -> Pal.heal`，其余 line 参数同 `hitFlameSmall`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `SeededRadialLineParticles`；
    - `line_render_primitives_from_seed()` 对该 kind 使用 seeded vector 展开，line angle 由 `atan2(y,x)` 对齐 Java `Mathf.angle(x,y)`；
    - 新增并接入 `FX_HIT_BULLET_BIG_ID`、`FX_HIT_FLAME_SMALL_ID`、`FX_HIT_FLAME_PLASMA_ID`；
    - 测试覆盖 radial line kind、颜色、stroke、粒子数、cone、长度与 primitive 展开。
  - `core/src/mindustry/entities/mod.rs`
    - re-export `StandardEffectSquareRenderPrimitive` / `StandardEffectLineRenderPrimitive`。
  - `desktop/src/lib.rs`
    - `DesktopLauncher` 增加 square/line primitive cache；
    - `DesktopStandardEffectRenderFrame` 增加 `square_primitives` / `line_primitives`；
    - `update()` 同时展开 circle/square/line/light；
    - 新增 square/line collect 函数；
    - snapshot cursor 清理同步清空 square/line。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_caches_square_and_line_primitives_for_render --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_caches_fire_light_primitives_for_render --lib`
- 下一步建议：
  1. 先补 `hitBulletSmall=77` / `hitBulletColor=78` / `hitFuse=81` 的 multi-pass 表达：需要 scaled circle + radial line + light，不能只迁移线段。
  2. 把 `desktop` frame 的 circle/square/line/light primitives 继续接到真实 renderer/backend；当前 `desktop/src/main.rs` 仍只是 launcher loop。
  3. `launchPod=248` 可在 multi-pass 表达完成后迁移，避免只做 line 部分。
  4. 每次上下文压缩后先检查：`git -C "D:/MDT/rust-mindustry" status --short`，再读后续最新闭环记录和 `MIGRATION.md` 最新章节。

---

## 144. 最新闭环记录：Hit radial-line Fx batch without scaled pass

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：继续吃掉无需 `scaled(...)` 的 hit lineAngle 类效果，避免在 multi-pass 表达未完成前迁移半截组合效果。
- 本轮迁移：
  - `hitLaserBlast=86`
  - `hitEmpSpark=87`
  - `hitLancer=88`
  - `hitLancerLow=89`
  - `hitBeam=90`
  - `hitMeltdown=92`
  - `hitMeltHeal=93`
- Java 依据：
  - `Fx.java:972-1048`；
  - 这组核心形态都是 `randLenVectors(...)` 后 `lineAngle(...)`；
  - `hitEmpSpark` 用 `rotation, 360f` cone；
  - `hitLaserBlast`/`hitBeam` 使用输入色；
  - `hitMeltdown` 需要 `Pal.meltdownHit`，`Pal.java:41` 值为 `ffb98b`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 7 个 `FX_*` 常量、name lookup 与 lifetime metadata；
    - `standard_effect_draw_plan(...)` 新增 hit radial-line batch 分支；
    - 继续复用 `SeededRadialLineParticles`；
    - 新增颜色符号 `Pal.meltdownHit = 0xffb98bff`；
    - 新增测试 `standard_effect_draw_plan_covers_hit_radial_line_batch`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_hit_radial_line_batch --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 后续最新状态见 145 节；`hitFlameBeam=91` 已迁移。
  2. `hitBulletSmall=77` / `hitBulletColor=78` / `hitSquaresColor=79` / `hitFuse=81` 需要先扩展 multi-pass 或附加 scaled circle/light 表达，不要只迁移主粒子。
  3. 继续推进 renderer/backend 消费 `DesktopStandardEffectRenderFrame.square_primitives` 与 `.line_primitives`。

---

## 145. 最新闭环记录：Fx.hitFlameBeam seeded circle batch

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `hitFlameBeam=91`，这是纯 `randLenVectors + Fill.circle`，不需要 multi-pass。
- Java 依据：
  - `Fx.java:1022-1028`
  - lifetime `19`
  - 输入色 `e.color`
  - 7 个粒子，长度 `finpow*11`
  - 半径 `fout*2 + 0.5`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_HIT_FLAME_BEAM_ID=91`；
    - 接入 `standard_effect_id` / `standard_effect` / `standard_effect_draw_plan`；
    - draw kind 使用 `SeededCircleParticles`；
    - 扩展 `standard_effect_draw_plan_covers_hit_radial_line_batch` 覆盖 circle primitive 展开。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_hit_radial_line_batch --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 下一组如果继续 Fx，可考虑 `instTrail`/`instShoot`/`instHit` 前先对照是否需要 polygon/tri/laser primitives。
  2. `hitBulletSmall`、`hitBulletColor`、`hitSquaresColor`、`hitFuse` 暂不要半迁移，先设计 multi-pass/附加 circle+light 表达。
  3. 也可以转向 desktop renderer，把 frame cache 里的 square/line primitive 真正画出来。

---

## 146. 最新闭环记录：Fx.hitLaser 与 Fx.despawn

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移当前 primitive 能完整表达的 `hitLaser=98` 与 `despawn=100`；跳过 `hitLaserColor=99`，因为它需要 input-color light，当前 light primitive 还不支持。
- Java 依据：
  - `Fx.java:1129-1135`：`hitLaser`，lifetime `8`，`Color.white -> Pal.heal`，圆环半径 `fin*5`，stroke `0.5+fout`，light `Pal.heal` 半径 `23` opacity `fout*0.7`。
  - `Fx.java:1145-1154`：`despawn`，lifetime `12`，`Pal.lighterOrange -> Color.gray`，7 条 cone radial line，长度 `fin*7`，line length `fout*2+1`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_HIT_LASER_ID=98` 与 `FX_DESPAWN_ID=100`；
    - 接入 id lookup、metadata 和 draw plan；
    - `hitLaser` 使用 `StrokedCircle` + `light_render_primitives()`；
    - `despawn` 使用 `SeededRadialLineParticles`；
    - 扩展 `standard_effect_draw_plan_covers_hit_radial_line_batch`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_hit_radial_line_batch --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 若要迁移 `hitLaserColor=99`，先扩展 light primitive 支持输入色，而不是丢掉 light。
  2. 后续最新状态见 147 节；`artilleryTrail`、`incendTrail`、`colorTrail`、`absorb` 已迁移；`airBubble` 需要 texture，爆炸系列需要 multi-pass。
  3. `hitBulletSmall`/`hitBulletColor`/`hitSquaresColor`/`hitFuse` 仍应等 multi-pass 表达。

---

## 147. 最新闭环记录：Simple trail + absorb Fx batch

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：按只读探索结果迁移现有 primitive 能完整覆盖的简单 trail/absorb 批次。
- 本轮迁移：
  - `artilleryTrail=108`
  - `incendTrail=109`
  - `colorTrail=113`
  - `absorb=114`
- Java 依据：
  - `Fx.java:1304-1307`：`artilleryTrail`，输入色 filled circle，半径 `rotation*fout`，layer `Layer.bullet - 0.01`。
  - `Fx.java:1309-1312`：`incendTrail`，`Pal.lightOrange` filled circle。
  - `Fx.java:1351-1354`：`colorTrail`，输入色 filled circle。
  - `Fx.java:1356-1360`：`absorb`，`Pal.accent` stroked circle，半径 `5*fout`，stroke `2*fout`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 4 个 `FX_*` 常量、id lookup、metadata；
    - 复用 `FilledCircle` / `StrokedCircle` draw plan；
    - 扩展 `standard_effect_draw_plan_covers_smoke_trails_and_ripple`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 暂避 `airBubble=101`（texture bubble）、`forceShrink=115`（polygon）、爆炸系列（multi-pass）。
  2. 后续最新状态见 148 节；input-color light 已补，可继续找单 pass circle/line/square 效果，或优先扩展 multi-pass / renderer backend。

---

## 148. 最新闭环记录：Input.color draw/light semantics 与 Fx.hitLaserColor

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补动态输入色 light 表达，迁移此前暂缓的 `hitLaserColor=99`。
- Java 依据：
  - `Fx.java:1137-1143`
  - lifetime `8`
  - `Color.white -> e.color`
  - stroked circle 半径 `fin*5`
  - light 半径 `23`，颜色 `e.color`，opacity `fout*0.7`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_HIT_LASER_COLOR_ID=99`；
    - `resolved_draw_color()` 支持 `color_to = "Input.color"`；
    - `light_render_primitives()` 支持 `light_color = "Input.color"` 并输出 `input_color` 的 rgba；
    - `hitLaser`/`hitLaserColor` 共用同形 stroked-circle 分支。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_hit_radial_line_batch --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 用 `Input.color` light 能力回头处理 `hitBulletColor=78`，但必须先补 multi-pass/附加 scaled circle 表达。
  2. 扫描其它 `Drawf.light(..., e.color, ...)` 候选，优先挑不需要 texture/polygon/multi-pass 的效果。
  3. 继续推进 desktop renderer/backend 真正消费 primitive frame。

---

## 149. 最新闭环记录：Fx.fluxVapor seeded vapor particles

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补齐 `steam=123` 与 `corrosionVapor=127` 之间缺失的 `fluxVapor=126`，避免 Fx id 序列继续出现可完整迁移但遗漏的简单 vapor 效果。
- Java 依据：
  - `Fx.java:1489` 附近
  - lifetime `140`
  - `color(e.color)`
  - `alpha(e.fout() * 0.7f)`
  - 2 个 seeded circle particles
  - length `3 + finpow * 10`
  - radius `0.6 + fin * 5`
  - layer `Layer.bullet - 1f`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_FLUX_VAPOR_ID=126`；
    - 接入 name lookup 与 metadata，metadata layer 为 `Layer::BULLET - 1.0`；
    - `standard_effect_draw_plan(...)` 新增 `SeededCircleParticles` 分支，使用输入色、`fout*0.7` alpha、`count=2`、`length=3+finpow*10`、半径 `0.6+fin*5`；
    - 扩展 ids/lifetime/draw-plan 测试。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 并行探索结果：
  - 子代理扫描 `Fx.java` id 115-140，确认大量 single-pass circle/square 效果已由现有 primitive 支撑；本轮实际补的是它漏掉但同样无阻塞的 `fluxVapor=126`。
- 下一步建议：
  1. 继续查漏 `Fx.java` 中可由现有 primitive 完整表达但未接入的简单 single-pass 效果；
  2. 暂缓 `ventSteam=124` / `drillSteam=125`，除非先补随机粒子数量/随机半径/`scaled` 生命周期语义；
  3. 继续推进 desktop renderer/backend 消费 `circle_primitives`/`line_primitives`/`square_primitives`/`light_primitives`。

---

## 150. 最新闭环记录：Fx.select selection ring

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补齐早期 `Fx.select=27`，减少 `smoke=28` 前的可完整迁移缺口。
- Java 依据：
  - `Fx.java:311-315`
  - lifetime `23`
  - `Pal.accent`
  - stroke `fout * 3`
  - circle radius `3 + fin * 14`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SELECT_ID=27`；
    - 接入 `standard_effect_id(...)` / `standard_effect(...)`；
    - `standard_effect_draw_plan(...)` 使用 `StrokedCircle`，输出 `Pal.accent`、`radius=3+fin*14`、`stroke=fout*3`；
    - 扩展 id、metadata、draw-plan 测试。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 继续补 `placeBlock=22` / `tapBlock=24` / `upgradeCoreBloom=21` 这种单个描边 square/circle；
  2. 对 `breakBlock=25` / `coreLaunchConstruct=23` 先确认随机 square/line 批次是否无需新语义；
  3. 不要忘记中期目标是把这些 effect primitives 接入真实 desktop renderer/backend。

---

## 151. 最新闭环记录：Early block feedback stroked shapes

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：接续 `select=27`，补齐可由现有 `StrokedSquare` / `StrokedCircle` 完整表达的早期方块反馈效果。
- 本轮迁移：
  - `upgradeCoreBloom=21`
  - `placeBlock=22`
  - `tapBlock=24`
- Java 依据：
  - `Fx.java:257-266`：`upgradeCoreBloom` / `placeBlock`
  - `Fx.java:279-283`：`tapBlock`
  - 都使用 `Pal.accent`、简单 stroke、单个 square/circle。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 3 个 `FX_*` id 常量；
    - 接入 `standard_effect_id(...)` / `standard_effect(...)`；
    - draw plan 分别生成 `StrokedSquare` 或 `StrokedCircle`；
    - 半径公式使用 `TILE_SIZE as f32` 对齐 Java `tilesize`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `breakBlock=25` / `coreLaunchConstruct=23` 需要 shape + seeded square/line 的组合表达，可能要先扩展 multi-pass 或确认当前 plan 不能只表达一个 kind；
  2. 可继续找单 kind 的早期 wave/shockwave 类效果补迁移；
  3. 中期应回到 renderer/backend，把这些 primitives 从测试数据真正绘制出来。

---

## 152. 最新闭环记录：Early point/command stroked circles

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补齐早期点命中/指令反馈中可由单个 `StrokedCircle` 完整表达的效果。
- 本轮迁移：
  - `pointHit=11`
  - `moveCommand=17`
  - `commandSend=19`
- Java 依据：
  - `Fx.java:161-165`：`pointHit`
  - `Fx.java:231-235`：`moveCommand`
  - `Fx.java:243-247`：`commandSend`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 3 个 `FX_*` id 常量；
    - 接入 `standard_effect_id(...)` / `standard_effect(...)`；
    - `pointHit` 使用 `Color.white -> Input.color` 插值；
    - `moveCommand` 使用 `Layer::OVERLAY_UI`；
    - 新增 draw-plan 测试 `standard_effect_draw_plan_covers_early_command_and_point_shapes`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_early_command_and_point_shapes --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `pointBeam=10` 需要 line segment + line light primitive，不要半迁移；
  2. `attackCommand=18` 需要 polygon primitive；
  3. 可继续补单 kind circle/square wave，或转去 renderer/backend 消费 primitives。

---

## 153. 最新闭环记录：Fx.coreBuildShockwave dynamic lifetime ring

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `coreBuildShockwave=14`，重点补 Java draw-time `e.lifetime = e.rotation` 的动态寿命规则。
- Java 依据：
  - `Fx.java:207-213`
  - static lifetime `120`、clip `500`
  - draw 内改 lifetime 为 `e.rotation`
  - color `Pal.command`
  - stroke `e.fout(Interp.pow5Out) * 4`
  - circle radius `e.fin() * e.rotation * 2`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_CORE_BUILD_SHOCKWAVE_ID=14`；
    - `standard_effect_render_lifetime(...)` 对该 id 返回 `rotation`；
    - 新增 `interp_pow5_out(...)`；
    - draw plan 使用 `StrokedCircle` 输出 `Pal.command`、动态半径和 pow5Out stroke；
    - 扩展 id/metadata/render-lifetime/draw-plan 测试。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_render_lifetime_applies_ripple_dynamic_rotation_rule --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_early_command_and_point_shapes --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- renderer/backend 探索结论：
  - 子代理只读确认：当前 standard effect primitives 只到 `DesktopLauncher` 缓存和 `DesktopStandardEffectRenderFrame`；
  - `desktop/src/main.rs` 仍未消费 frame；
  - 最小后续接入方案是在 desktop 层增加薄 renderer/backend 接口，并在 main loop 的 `launcher.update()` 后消费 `standard_effect_render_frame()`。
- 下一步建议：
  1. 若继续 Fx：优先单 kind circle/square wave；`pointShockwave=16` 需要 multi-pass，不要半迁移；
  2. 若转集成：在 `desktop/src/lib.rs`/`desktop/src/main.rs` 增加最小 renderer 消费口，并用现有 desktop primitive tests 扩展验证；
  3. 不要把 frame cache 误判为真实渲染完成。

---

## 154. 最新闭环记录：Desktop effect frame backend consumption seam

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：把 standard effect primitive frame 从“只在 `DesktopLauncher` 缓存/测试中存在”推进到 desktop 主循环中的 backend 消费 seam。
- 重要边界：
  - 本轮不是实际图形绘制；
  - `HeadlessDesktopEffectRenderer` 只是无窗口/headless backend，用于打通调用链和记录 stats；
  - 不要把它写成“真实 renderer 已完成”。
- Rust 主改动：
  - `desktop/src/lib.rs`
    - 新增 `DesktopEffectRenderStats`；
    - 新增 `DesktopEffectRenderer` trait；
    - 新增 `HeadlessDesktopEffectRenderer`；
    - 新增 `DesktopLauncher::render_standard_effect_frame_with(...)`；
    - 扩展 fire/light primitive 测试，验证 renderer 消费 frame 并记录 draw/circle/square/line/light 数量。
  - `desktop/src/main.rs`
    - 主循环中创建并调用 headless renderer，`launcher.update()` 后消费 `standard_effect_render_frame()`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-desktop desktop_launcher_caches_fire_light_primitives_for_render --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_caches_square_and_line_primitives_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 在此 trait 后接真实窗口/绘制 backend，而不是继续只加 headless 统计；
  2. renderer 需要处理 layer、alpha、color、stroke、filled/stroked circle/square/line/light；
  3. 继续迁移 Fx 时，避免把需要 multi-pass/texture/polygon 的效果硬塞进单 kind plan。

---

## 155. 最新闭环记录：Multi-pass standard effect plans + Fx.pointShockwave

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：开始解决此前反复阻塞的 standard Fx multi-pass 表达问题，不再只能返回单个 `StandardEffectDrawPlan`。
- 本轮迁移：
  - `pointShockwave=16`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_POINT_SHOCKWAVE_ID=16`；
    - 新增 `standard_effect_draw_plans(...)`；
    - 默认路径兼容原单 pass `standard_effect_draw_plan(...)`；
    - `pointShockwave` 返回两个 pass：`StrokedCircle` + `SeededRadialLineParticles`。
  - `core/src/mindustry/entities/mod.rs`
    - 导出 `standard_effect_draw_plans(...)`。
  - `desktop/src/lib.rs`
    - `collect_standard_local_effect_draw_plans_for_render()` 改为 flatten 多 pass；
    - 新增 `desktop_launcher_flattens_multi_pass_standard_effects_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_point_shockwave_multi_pass --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_multi_pass_standard_effects_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 用 multi-pass 接口回头迁移 `hitBulletColor=78`、`hitSquaresColor=79`、`hitFuse=81`，但仍要逐个对照 Java；
  2. `pointBeam=10` 仍需要 line-to-data-position 与 light line primitive，不属于当前接口已解决范围；
  3. renderer/backend 仍需从 headless seam 发展到真实绘制。

---

## 156. 最新闭环记录：Fx.hitBulletSmall / Fx.hitBulletColor multi-pass hit effects

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：用上轮新增的 multi-pass 接口回迁此前被 `scaled(...) + radial lines + light` 阻塞的 hit bullet 效果。
- 本轮迁移：
  - `hitBulletSmall=77`
  - `hitBulletColor=78`
- 关键对照：
  - `Effect.java:317` 确认 `scaled(7f, ...)` 在 `time <= 7` 时执行；
  - pass 1 对应 scaled circle；
  - pass 2 对应 radial line batch，并附带 light；
  - `hitBulletColor` 使用 `Input.color` draw/light。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 2 个 id 常量、lookup、metadata；
    - `standard_effect_draw_plans(...)` 为两者生成 multi-pass；
    - 新增 core 测试 `standard_effect_draw_plans_cover_hit_bullet_scaled_circle_lines_and_light`。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_hit_bullet_multi_pass_with_light_for_render`，验证 desktop flatten 后有 2 plans、1 circle、5 lines、1 light，并进入 headless backend stats。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_hit_bullet_scaled_circle_lines_and_light --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_hit_bullet_multi_pass_with_light_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 继续 `hitSquaresColor=79`，它与本轮结构相似，但第二 pass 是 `SeededSquareParticles`；
  2. 继续 `hitFuse=81`，结构相似但颜色 `Pal.surge`、scaled circle 半径 `7`、line count `6`；
  3. 之后考虑把这些 multi-pass 迁移集中抽 helper，避免重复逻辑膨胀。

---

## 157. 最新闭环记录：Fx.hitFuse multi-pass hit effect

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：继续处理 hit bullet 阻塞簇，迁移不需要新增 primitive 语义的 `hitFuse=81`。
- 本轮迁移：
  - `hitFuse=81`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_HIT_FUSE_ID=81`；
    - 接入 lookup/metadata；
    - `standard_effect_draw_plans(...)` 复用 multi-pass hit bullet 分支：
      - scaled circle 半径 `scaled_fin * 7`；
      - radial lines count `6`；
      - 颜色 `Color.white -> Pal.surge`；
      - 无 light。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_hit_bullet_scaled_circle_lines_and_light --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `hitSquaresColor=79` 需要先扩展 `SeededSquareParticles` 支持 per-particle radial rotation，对齐 Java `Fill.square(..., ang)`；
  2. 扩展时要验证 desktop `square_primitives` 中每个 square 的 rotation，而不是统一 rotation；
  3. 不要在缺少该语义时半迁移 `hitSquaresColor`。

---

## 158. 最新闭环记录：Fx.hitSquaresColor radial square particles

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补 per-particle radial square rotation 后完整迁移 `hitSquaresColor=79`。
- 本轮迁移：
  - `hitSquaresColor=79`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_HIT_SQUARES_COLOR_ID=79`；
    - 新增 `StandardEffectDrawKind::SeededRadialSquareParticles`；
    - `square_render_primitives_from_seed()` 中对该 kind 按 seeded vector 的 `atan2(y,x)` 设置每个 square rotation；
    - `standard_effect_draw_plans(...)` 为 `hitSquaresColor` 输出 scaled circle + radial square particles + Input.color light。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_hit_squares_multi_pass_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_hit_bullet_scaled_circle_lines_and_light --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_hit_squares_multi_pass_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 暂缓 `squareWaveEffect=80`，它需要随机 radius/stroke/rotation/light，不等同于本轮径向 square；
  2. 可继续 Fx 后续简单 single/multi-pass，或抽象 hit bullet/fuse/squares 共用 helper；
  3. renderer/backend 仍需从 headless seam 走向真实绘制。

---

## 159. 最新闭环记录：Fx.squareWaveEffect seeded rotated square

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `squareWaveEffect=80` 的 seeded 随机描边旋转方块，并让 desktop standard effect frame 能消费该 primitive。
- 本轮迁移：
  - `squareWaveEffect=80`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SQUARE_WAVE_EFFECT_ID=80`；
    - 接入 lookup/metadata，lifetime `14.0`、clip `40.0`；
    - 新增 `StandardEffectDrawKind::StrokedRotatedSquare`；
    - `standard_effect_draw_plan(...)` 按 Java `rand.setSeed(e.id)` 的调用顺序生成 color mix、stroke、rot/sign、radius、rotation；
    - `square_render_primitives_from_seed()` 输出单个旋转描边 square；
    - 当前临时使用 `particles.angle` 保存单 square rotation，后续可抽正式 `square_rotation` 字段。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_square_wave_effect_for_render`，验证 1 draw plan、1 square primitive、1 light primitive，并进入 `HeadlessDesktopEffectRenderer` stats。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_square_wave_effect --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_square_wave_effect_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 继续从 Java `Fx.java` 后续效果中挑不需要 texture/polygon/triangle/data-position 的低风险候选；
  2. 可以优先考虑已有 primitive 能覆盖的 circle/line/square/light 效果；
  3. 不要把 headless renderer seam 当真实渲染完成，后续仍要把 primitive 接入真实图形 backend。

---

## 160. 最新闭环记录：Drawf.tri triangle pair seam + shoot muzzle Fx

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补 `Drawf.tri` 的最小 triangle pair primitive seam，并迁移一组射击口效果，不把三角效果继续列为无法表达。
- 本轮迁移：
  - `shootSmall=155`
  - `shootSmallColor=156`
  - `shootHeal=157`
  - `shootHealYellow=158`
  - `shootBig=160`
  - `shootBig2=161`
  - `shootBigColor=162`
  - `shootTitan=165`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增上述 Fx id、lookup、metadata；
    - 新增 `StandardEffectDrawKind::TrianglePair` 与 `StandardEffectTriangleRenderPrimitive`；
    - `triangle_render_primitives_from_seed()` 将 front/back 两个 `Drawf.tri` 输出为 primitive；
    - `resolved_draw_color()` 支持 `Input.color -> static color` lerp；
    - 新增颜色符号 `Pal.lightTrail`。
  - `core/src/mindustry/entities/mod.rs`
    - 导出 triangle primitive。
  - `desktop/src/lib.rs`
    - standard effect frame/stats/launcher 缓存新增 triangle primitive；
    - 新增 desktop triangle flatten 测试。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_shoot_triangle_pairs --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_shoot_triangle_pairs_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 可以继续迁移 `shootScepterSecondary=163`，但它需要 multi-pass/multi-color triangle group，不等同于本轮简单 pair；
  2. `instBomb=101` / `instTrail=102` 也可借用 triangle primitive，但需要 circle/light 或 seed range；
  3. renderer/backend 仍需真实绘制 triangle primitive，当前只是 headless frame seam。

---

## 161. 最新闭环记录：Fx.instBomb / Fx.instTrail triangle fan and trail pairs

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：在 triangle pair 基础上补固定角度 triangle fan，并迁移 `instBomb=101`、`instTrail=102`。
- 本轮迁移：
  - `instBomb=101`
  - `instTrail=102`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_INST_BOMB_ID=101`、`FX_INST_TRAIL_ID=102`；
    - 接入 lookup/metadata；
    - 新增 `StandardEffectDrawKind::TriangleFan`；
    - `standard_effect_draw_plans(...)`：
      - `instBomb` 输出 circle + 4 个大 triangle fan + 4 个小 triangle fan + light；
      - `instTrail` 输出两组 `TrianglePair`，第一组携带 light，并复用 `mathf_random_seed_range(e.id, 15f)`；
    - 新增颜色符号 `Pal.bulletYellow` / `Pal.bulletYellowBack`。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_inst_bomb_and_trail_triangles_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_inst_bomb_and_trail_triangles --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_inst_bomb_and_trail_triangles_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 可继续 `instShoot=103`，需要 scaled circle + 4 个固定方向 triangle + light；
  2. `instHit=104` 更复杂，涉及随机多 triangle、scaled circle、seeded square；
  3. 真实 renderer/backend 仍需接入 triangle primitive 绘制。

---

## 162. 最新闭环记录：Fx.instShoot scaled circle and triangle fans

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：继续 inst 系列，迁移可由 scaled circle + triangle fan 表达的 `instShoot=103`。
- 本轮迁移：
  - `instShoot=103`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_INST_SHOOT_ID=103`；
    - 接入 lookup/metadata；
    - `standard_effect_draw_plans(...)` 输出 early scaled circle、side triangle fan、core triangle fan，并在 side fan 上携带 light。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_inst_shoot_scaled_circle_and_triangles_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_inst_shoot_scaled_circle_and_triangles --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_inst_shoot_scaled_circle_and_triangles_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `instHit=104` 是相邻目标，但复杂度明显更高：randomSeedRange 多 triangle、scaled circle、seeded square；
  2. 若要保守推进，可先迁移 `shootScepterSecondary=163` 的 multi triangle pass；
  3. triangle primitive 仍要接入真实 renderer/backend。

---

## 163. 最新闭环记录：Fx.shootScepterSecondary multi-pass triangles

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `shootScepterSecondary=163`，验证 triangle fan + pair 可表达带 layer 的 multi-pass triangle 效果。
- 本轮迁移：
  - `shootScepterSecondary=163`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SHOOT_SCEPTER_SECONDARY_ID=163`；
    - 接入 lookup/metadata，layer `Layer::EFFECT + 1.0`；
    - `standard_effect_draw_plans(...)` 输出 side `TriangleFan` + front/back `TrianglePair` 两个 pass。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_shoot_scepter_secondary_triangles_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_shoot_scepter_secondary_triangles --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_shoot_scepter_secondary_triangles_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `shootQuellPulse=164` / `instHit=104` 更复杂，涉及随机三角簇与多 pass；
  2. 可考虑先把 triangle primitive 接入真实 renderer/backend，避免 headless seam 积累过多；
  3. 若继续 Fx，优先挑已有 primitive 能完整表达的效果。

---

## 164. 最新闭环记录：Fx.instHit random triangles, scaled circle and seeded squares

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `instHit=104`，验证 randomSeedRange triangle pair + scaled circle + seeded square 的组合表达。
- 本轮迁移：
  - `instHit=104`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_INST_HIT_ID=104`；
    - 接入 lookup/metadata，clip `200.0`；
    - `standard_effect_draw_plans(...)` 输出 10 个 `TrianglePair`，并按 Java `Mathf.randomSeedRange(e.id + j, ...)` 对齐角度与 front length；
    - early `StrokedCircle` pass 对齐 `e.scaled(10f, ...)`；
    - `SeededSquareParticles` pass 对齐 `e.scaled(12f, ...)`。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_inst_hit_triangles_circle_and_squares_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_inst_hit_triangles_circle_and_squares --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_inst_hit_triangles_circle_and_squares_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `shootQuellPulse=164` 是相邻但复杂，涉及随机三角簇、alpha、coreRadius 和 scaled circle；
  2. 可以先做真实 renderer/backend 的 triangle/square/circle primitive 消费，减少 headless seam 技术债；
  3. 若继续 Fx，优先挑已有 primitive 能覆盖且不会引入 texture/polygon/data-position 的效果。

---

## 165. 最新闭环记录：Fx.shootQuellPulse circle layers and offset triangle clusters

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `shootQuellPulse=164`，验证多层 circle + seeded 偏移 triangle cluster 能进入 core plan 与 desktop headless primitive frame。
- 本轮迁移：
  - `shootQuellPulse=164`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SHOOT_QUELL_PULSE_ID=164`；
    - 接入 lookup/metadata，lifetime `40.0`；
    - 新增 `interp_smooth`、`interp_smooth2`、`interp_pow2_in_inverse`；
    - `ArcRand` 新增 bounded integer helper，用于对齐 Java `rand.random(8, 13)`；
    - `standard_effect_draw_plans(...)` 输出 early circle、8 层 fill circle、core circle、9 组环上偏移 `TriangleFan`、8~13 组外侧随机 `TriangleFan`。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_shoot_quell_pulse_circles_and_triangle_clusters_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_shoot_quell_pulse_circles_and_triangle_clusters --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_shoot_quell_pulse_circles_and_triangle_clusters_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `shootSmokeSquare=169`、`shootSmokeSquareSparse=170`、`shootSmokeSquareBig=171` 是相邻 Fx，但需要四边形/rotated square 随机粒子；已有 `SeededSquareParticles` 可部分复用，需核对随机旋转与角度范围；
  2. 也可转向真实 desktop renderer/backend，把当前 headless circle/square/line/triangle primitive 接入可见窗口；
  3. 无论继续 Fx 还是 renderer，都必须保持最终目标：整体可玩 Rust MDT，而不是独立 helper。

---

## 166. 最新闭环记录：Fx.shootSmokeSquare rotated square particles

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `shootSmokeSquare=169`、`shootSmokeSquareSparse=170`、`shootSmokeSquareBig=171`，补齐每粒子随机 rotation 的 square/poly smoke。
- 本轮迁移：
  - `shootSmokeSquare=169`
  - `shootSmokeSquareSparse=170`
  - `shootSmokeSquareBig=171`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增三个 Fx ID 并接入 lookup/metadata；
    - 新增 `StandardEffectDrawKind::SeededRotatedSquareParticles`；
    - `square_render_primitives_from_seed()` 现在能按 Java `rand.range(...) -> rand.random(length) -> rand.random(360f)` 的顺序生成 offset square 与逐粒子 rotation；
    - `standard_effect_draw_plan(...)` 输出 white→input color、count/angleRange/lengthScale/radiusScale 分别对齐 Java。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_shoot_smoke_square_particles_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_shoot_smoke_square_particles --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_shoot_smoke_square_particles_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 相邻 `shootSmokeTitan` / `shootSmokeSmite` 需要 per-particle scaled lifetime、局部 `b.fin()/b.fout()` 与更复杂颜色，不要半迁移；
  2. 如果继续 Fx，可优先寻找现有 primitives 已能完整表达的效果；
  3. 更长期应推进真实 renderer/backend，避免 headless primitive seam 继续堆积。

---

## 167. 最新闭环记录：Fx.shootSmokeTitan per-particle scaled circles

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `shootSmokeTitan=172`，对齐 Java 每粒子随机 offset、随机局部 lifetime、局部 color mix 与 circle radius。
- 本轮迁移：
  - `shootSmokeTitan=172`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SHOOT_SMOKE_TITAN_ID=172`；
    - 接入 lookup/metadata，lifetime `70.0`；
    - `standard_effect_draw_plans(...)` 逐粒子复现 `rand.range(30)`、`rand.random(finpow*40)`、`rand.random(0.3,1)` 顺序；
    - active 粒子输出 concrete `FilledCircle` plan，颜色为 input color → `Pal.lightishGray`，mix 为局部 `b.fin()`。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_shoot_smoke_titan_scaled_circles_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_shoot_smoke_titan_scaled_circles --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_shoot_smoke_titan_scaled_circles_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `shootSmokeSmite=173` 需要 direct lineAngle primitive 或新的 line plan，不应伪装成现有 radial line；
  2. `shootSmokeMissile=174/175` 需要 alpha、clip `300f`、35 粒子、per-particle jitter 与 scaled lifetime，可在 lineAngle 后继续；
  3. 继续提醒：这些仍是局部 Fx seam，最终仍需真实 renderer/backend 和整体游戏 runtime。

---

## 168. 最新闭环记录：Fx.shootSmokeSmite direct lineAngle particles

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `shootSmokeSmite=173`，补 direct `Lines.lineAngle` 的标准 effect plan 与 desktop flatten。
- 本轮迁移：
  - `shootSmokeSmite=173`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SHOOT_SMOKE_SMITE_ID=173`；
    - 接入 lookup/metadata，lifetime `70.0`；
    - 新增 `StandardEffectDrawKind::LineAngle`；
    - `line_render_primitives_from_seed()` 支持 direct line：`center` 是 start，`particles.angle` 是 angle，`radius` 是 length，`stroke` 是 line stroke；
    - `standard_effect_draw_plans(...)` 逐粒子复现 Java `range(30)`、`random(finpow*50)`、`random(0.3,1)` 顺序，并输出 active `LineAngle`。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_shoot_smoke_smite_scaled_lines_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_shoot_smoke_smite_scaled_lines --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_shoot_smoke_smite_scaled_lines_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `shootSmokeMissile=174` 与 `shootSmokeMissileColor=175` 是相邻目标；需要 alpha `0.5`、clip `300f`、35 个 scaled circles、`rotation + 180 + rand.range(21)` 和额外 `rand.range(3)` 抖动；
  2. 可复用 `shootSmokeTitan` 的 concrete circle plan 思路，但要补 alpha 与 jitter；
  3. 真实 renderer/backend 仍未接入，headless primitives 只是过渡 seam。

---

## 169. 最新闭环记录：Fx.shootSmokeMissile scaled smoke circles

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `shootSmokeMissile=174` 与 `shootSmokeMissileColor=175`，对齐 clip、alpha、35 粒子 jitter 与 scaled circle。
- 本轮迁移：
  - `shootSmokeMissile=174`
  - `shootSmokeMissileColor=175`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增两个 Fx ID 并接入 lookup/metadata；
    - `standard_effect(...)` lifetime `130.0`、clip `300.0`；
    - 新增 `Pal.redLight` 颜色符号；
    - `standard_effect_draw_plans(...)` 逐粒子复现 Java `range(21)`、`random(finpow*90)`、两次 `range(3)` jitter、`random(0.2,1)` local lifetime 顺序；
    - active 粒子输出 `FilledCircle`，半径 `b.fout()*9+0.3`，alpha `0.5`。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_shoot_smoke_missile_scaled_circles_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_shoot_smoke_missile_scaled_circles --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_shoot_smoke_missile_scaled_circles_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 可继续 `regenParticle` / `regenSuppressParticle` 附近简单 Fx；
  2. 也应规划真实 renderer/backend，当前 primitives 仍不可见；
  3. 不要把 Fx seam 误认为完整游戏迁移完成。

---

## 170. 最新闭环记录：Fx.regenParticle and regenSuppressParticle

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `regenParticle=176` 与 `regenSuppressParticle=177`，复用现有 square/line primitive seam。
- 本轮迁移：
  - `regenParticle=176`
  - `regenSuppressParticle=177`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增两个 Fx ID 并接入 lookup/metadata；
    - 新增 `Pal.regen` 颜色符号；
    - `regenParticle` 输出 `FilledSquare`，radius `fslope*1.5+0.14`，rotation `45`；
    - `regenSuppressParticle` 输出 `SeededRadialLineParticles`，count `4`，offset length `17*fin`，stroke `fout*1.4+0.5`，line length `fslope*3+0.5`，颜色 input→white。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_regen_particles_square_and_lines_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_regen_particles_square_and_lines --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_regen_particles_square_and_lines_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `regenSuppressSeek=178` 需要 data `Position`、随机 lifetime 和 Bezier 轨迹，应先补 effect data/position plan 再迁移；
  2. 或跳到后续不依赖 data 的 smoke/simple particle Fx；
  3. 真实 renderer/backend 仍未接入，当前仍只是 headless primitive seam。

---

## 171. 最新闭环记录：Fx.surgeCruciSmoke/neoplasiaSmoke/heatReactorSmoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1` / `05b2ecd`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：跳过需 data/Bezier seam 的 `regenSuppressSeek=178`，迁移后续 simple smoke circle 效果。
- 本轮迁移：
  - `surgeCruciSmoke=179`
  - `neoplasiaSmoke=180`
  - `heatReactorSmoke=181`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增三个 Fx ID 并接入 lookup/metadata；
    - 新增 `Pal.slagOrange` 与 `Pal.neoplasmMid`；
    - `standard_effect_draw_plans(...)` 逐粒子复现 Java `len -> rot -> random local lifetime` 顺序；
    - active 粒子输出 concrete `FilledCircle` plans，分别对齐 alpha/radius。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_reactor_and_neoplasia_smoke_circles_for_render`。
- 已跑验证：
  - `cargo fmt`
  - `cargo fmt --check`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_reactor_and_neoplasia_smoke_circles --lib`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_reactor_and_neoplasia_smoke_circles_for_render --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `circleColorSpark=182`、`colorSpark=183`、`colorSparkBig=184` 是相邻 line effects，已有 line primitive 可承载；
  2. `regenSuppressSeek=178` 仍需单独处理 data Position + Bezier；
  3. 当前仍是 headless primitive seam，真实 renderer/backend 未接入。

---

## 172. 最新闭环记录：Fx.circleColorSpark/colorSpark/colorSparkBig

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前已是 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先按 UTF-8 读取。
- 本轮目标：迁移相邻 line spark 效果 `circleColorSpark=182`、`colorSpark=183`、`colorSparkBig=184`，继续推进 `Fx.java` 到 Rust 标准 effect render seam。
- 本轮迁移：
  - `circleColorSpark=182`
  - `colorSpark=183`
  - `colorSparkBig=184`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增三者的 Fx ID 常量、name lookup 与 metadata lifetime；
    - `standard_effect_draw_plans(...)` 输出 concrete `LineAngle` plans；
    - `circleColorSpark` 按 Java `randLenVectors(seed, amount, length, randLength, ...)` 的 base-length + random-range 语义实现；
    - `colorSpark` / `colorSparkBig` 按 `rotation + rand.range(range)` 与 `rand.random(length)` 实现。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_color_spark_lines_for_render`，验证 22 条 line primitives 进入 headless frame。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `cargo test -p mindustry-core standard_effect_lookup --lib`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_color_spark_lines --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_color_spark_lines_for_render --lib`
- 下一步建议：
  1. `randLifeSpark=185` 需要 per-particle scaled lifetime，不要硬塞到现有 line spec；
  2. `shootPayloadDriver=186` 需要 line start jitter 和 per-line random length/stroke seam；
  3. `shootSmallFlame=187` 可优先迁移，能直接复用 `SeededCircleParticles`；
  4. 继续维护 `MIGRATION.md`，每个闭环验证、中文提交并推送 `origin main`。

---

## 173. 最新闭环记录：Fx.shootSmallFlame/shootPyraFlame/shootLiquid

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先按 UTF-8 读取。
- 本轮目标：跳过需要新增复杂 primitive/seam 的 `randLifeSpark=185`、`shootPayloadDriver=186`，迁移后续可直接接入 circle particle seam 的 flame/liquid shoot Fx。
- 本轮迁移：
  - `shootSmallFlame=187`
  - `shootPyraFlame=188`
  - `shootLiquid=189`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增三个 Fx ID、lookup、metadata；
    - 对齐 `clip=80`，`shootSmallFlame`/`shootPyraFlame` 保留 `.followParent(false)`；
    - 新增 `Pal.lightPyraFlame` / `Pal.darkPyraFlame` 颜色符号；
    - `standard_effect_draw_plan(...)` 复用 `SeededCircleParticles`，覆盖 count/angle_range/length/radius 公式。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_shoot_flame_circle_particles_for_render`，验证三项共 27 个 circle primitives。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_shoot_flame_circle_particles --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_shoot_flame_circle_particles_for_render --lib`
- 下一步建议：
  1. 可继续从 `casing1=190` 起对照，但 casing 系列需要 `Fill.rect`/rotated rectangle primitive；
  2. 如优先少造 primitive，可跳到后续仍为 circle/line 的 Fx；
  3. 若要补 `randLifeSpark`，先给 per-particle scaled lifetime 设计明确字段；
  4. 若要补 `shootPayloadDriver`，先给 line start jitter/per-line random length 增加字段或 concrete plan。

---

## 174. 最新闭环记录：Fx.randLifeSpark/shootPayloadDriver

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `randLifeSpark=185` 与 `shootPayloadDriver=186`，继续把 `Fx.java` line effects 接到 Rust 标准 effect render seam。
- 本轮迁移：
  - `randLifeSpark=185`
  - `shootPayloadDriver=186`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增两个 Fx ID、lookup、metadata；
    - `standard_effect_draw_plans(...)` 按 Java rand 调用顺序生成 concrete `LineAngle` plans；
    - `randLifeSpark` 保留 per-particle scaled lifetime；
    - `shootPayloadDriver` 保留 start jitter 与 per-line random length。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_rand_life_and_payload_driver_lines_for_render`，验证 35 条 line primitives。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_rand_life_and_payload_driver_lines --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_rand_life_and_payload_driver_lines_for_render --lib`
- 下一步建议：
  1. `casing1=190` 及后续 casing 系列需要 `Fill.rect` / rotated rectangle primitive；
  2. 若新增 rectangle seam，应同步补 desktop flatten/cache/render stats；
  3. 继续避免只做孤立 helper，新增 primitive 必须接入 `standard_effect_draw_plan(s)` 与 desktop frame。

---

## 175. 最新闭环记录：Fx.reactorsmoke/redgeneratespark/turbinegenerate

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移后续无需新增 primitive 的 generation circle effects。
- 本轮迁移：
  - `reactorsmoke=207`
  - `redgeneratespark=208`
  - `turbinegenerate=209`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增三项 Fx ID、lookup、metadata；
    - `redgeneratespark` / `turbinegenerate` 对齐 `Layer::BULLET - 1.0`；
    - 新增 `Pal.redSpark` / `Pal.vent`；
    - `reactorsmoke` 复用 `SeededCircleParticles`；
    - `redgeneratespark` / `turbinegenerate` 用 concrete `FilledCircle` plans 保留每粒子随机 radius。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_reactor_generation_particles_for_render`，验证 9 个 circle primitives。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_reactor_generation_particles --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_reactor_generation_particles_for_render --lib`
- 下一步建议：
  1. 若继续低成本迁移，可做 `sparkShoot=204`、`lightningShoot=205`、`thoriumShoot=206`（line particles）；
  2. `lancerLaserChargeBegin=202` 可用 two filled-circle concrete plans；
  3. `rail*` 可用 circle/triangle/light 多段 plan；
  4. `casing*` 需要 rect/sprite primitive，不要硬塞成 square。

---

## 176. 最新闭环记录：Fx.sparkShoot/lightningShoot/thoriumShoot

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `Fx.java` 中三项 shoot line particle 效果，继续把标准 Fx 接入 Rust `standard_effect_draw_plan(...)` 与 desktop headless primitive frame。
- 本轮迁移：
  - `sparkShoot=204`
  - `lightningShoot=205`
  - `thoriumShoot=206`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增三项 Fx ID、lookup 与 metadata；
    - 新增颜色符号 `Pal.lancerLaser=0xa9d8ffff`、`Pal.thoriumPink=0xf9a3c7ff`；
    - 三者复用 `StandardEffectDrawKind::SeededRadialLineParticles`；
    - `sparkShoot` 对齐 `Color.white -> Input.color`、stroke `fout*1.2+0.6`、7 条 `rotation±3`、offset `25*finpow`、line length `fslope*5+0.5`；
    - `lightningShoot` 对齐 `Color.white -> Pal.lancerLaser`、stroke `fout*1.2+0.5`、7 条 `rotation±50`、line length `fin*5+2`；
    - `thoriumShoot` 对齐 `Color.white -> Pal.thoriumPink`，其余参数同 `lightningShoot`。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_spark_lightning_thorium_shoot_lines_for_render`，验证三项共 21 条 line primitives 进入 headless render frame。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_spark_lightning_thorium_shoot_lines --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_spark_lightning_thorium_shoot_lines_for_render --lib`
- 下一步建议：
  1. 优先迁移 `railShoot=196`、`railTrail=197`、`railHit=198`、`lancerLaserShoot=199`、`lancerLaserCharge=201`、`lancerLaserChargeBegin=202`，多数可由现有 circle/line/triangle/light primitive 承载；
  2. `lancerLaserShootSmoke=200` 需要 data Float 通道；
  3. `lightningCharge=203` 需要 seeded triangle particles；
  4. `casing*` 需要 rect/sprite primitive，不能硬塞成 square；
  5. 当前仍是 headless primitive seam，真实 renderer/backend 与 gameplay runtime 接入仍待继续推进。

---

## 177. 最新闭环记录：Fx.rail*/lancerLaser*/lightningCharge

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：回补 `sparkShoot=204` 前面的 rail/lancer charge 段，让 `Fx.java` 的 `196..203` 标准 effect 进入 Rust effect primitive seam。
- 本轮迁移：
  - `railShoot=196`
  - `railTrail=197`
  - `railHit=198`
  - `lancerLaserShoot=199`
  - `lancerLaserShootSmoke=200`
  - `lancerLaserCharge=201`
  - `lancerLaserChargeBegin=202`
  - `lightningCharge=203`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 8 个 Fx ID、lookup、metadata；
    - 新增 `Pal.orangeSpark`；
    - 新增 `StandardEffectDrawKind::SeededRadialTriangleParticles`，用于 `lightningCharge` 的 seeded vector triangle；
    - 新增 `standard_effect_draw_plans_with_data_float(...)`，原 `standard_effect_draw_plans(...)` 兼容保留并传 `None`；
    - `lancerLaserShootSmoke` 读取可选 data float：无 data 长度 70，有 Float data 时用该长度；
    - `railShoot` = scaled stroked circle + triangle pair；`railTrail` = triangle pair + light；`railHit` = triangle fan；`lancerLaserShoot` = triangle pair；`lancerLaserCharge`/`Smoke` = radial lines；`lancerLaserChargeBegin` = two filled circles；`lightningCharge` = radial triangles。
  - `core/src/mindustry/entities/mod.rs`
    - 导出 `standard_effect_draw_plans_with_data_float(...)`。
  - `desktop/src/lib.rs`
    - 从 `EffectRenderInput.data` 提取 `TypeValue::Float`，传入标准 effect draw plan；
    - 新增 `desktop_launcher_flattens_rail_and_lancer_charge_primitives_for_render`，覆盖 circle/line/triangle/light flatten 与 data Float。
- 已跑验证：
  - `cargo fmt`
  - `cargo test -p mindustry-core standard_effect_draw_plans_cover_rail_and_lancer_charge_primitives --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_rail_and_lancer_charge_primitives_for_render --lib`
- 下一步建议：
  1. 继续做最终收尾前需要跑 `standard_effect_ids_include`、`standard_effect_lookup`、本轮 core/desktop 定向测试、`cargo check -p mindustry-core`、`cargo check -p mindustry-desktop`、`git diff --check`；
  2. 下一批 Fx 可转向 `casing1=190` 起，但需要先设计 rect/sprite primitive；如果要继续少造 primitive，可扫描 `Fx.java` 后续可由现有 circle/line/triangle 表达的项；
  3. 真实 renderer/backend 仍未接入；当前只是标准 effect headless primitive seam，不要宣称可玩。

---

## 178. 最新闭环记录：Fx.casing1/casing2/casing3/casing4/casing2Double/casing3Double

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：处理之前一直提示“需要 rect/sprite primitive”的 casing 系列，避免错误复用 square，以 Java `Fill.rect` / atlas `rect(Core.atlas.find("casing"), ...)` 语义新增 rect seam。
- 本轮迁移：
  - `casing1=190`
  - `casing2=191`
  - `casing3=192`
  - `casing4=193`
  - `casing2Double=194`
  - `casing3Double=195`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 6 个 casing Fx ID、lookup、metadata，layer 全部对齐 `Layer::BULLET`；
    - 新增 `StandardEffectDrawKind::FilledRect` / `TexturedRect`；
    - 新增 `StandardEffectRectRenderPrimitive` 与 `rect_render_primitives_from_seed(...)`；
    - `casing1` 输出 `FilledRect`，其余输出 `TexturedRect` 且 `region=Some("casing")`；
    - casing 分支对齐 Java `rot = abs(rotation)+90`、`i=-sign(rotation)` 或 `Mathf.signs`、`len/lr`、seed jitter、尺寸与颜色渐变；
    - 新增 `standard_effect_draw_plans_cover_casing_rects`。
  - `core/src/mindustry/entities/mod.rs`
    - 导出 `StandardEffectRectRenderPrimitive`。
  - `desktop/src/lib.rs`
    - `DesktopStandardEffectRenderFrame`、`DesktopEffectRenderStats`、`DesktopLauncher` 增加 rect primitive 缓存与统计；
    - update/render/clear 流程接入 rect；
    - 新增 `desktop_launcher_flattens_casing_rect_primitives_for_render`，验证 6 个 casing event 生成 8 个 rect primitives。
- 已跑验证：
  - `cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plans_cover_casing_rects --lib`
  - `cargo test -p mindustry-desktop desktop_launcher_flattens_casing_rect_primitives_for_render --lib`
- 注意：
  - 第一次 core 定向测试编译阶段遇到一次 `rustc-LLVM ERROR: out of memory`，随后用 `CARGO_BUILD_JOBS=1` 重跑通过；后续完整闭环仍需用低并发跑 check/test，避免 OOM 假失败。
- 下一步建议：
  1. 本轮提交前继续跑 `standard_effect_ids_include`、`standard_effect_lookup`、casing core/desktop 定向测试、`cargo check -p mindustry-core`、`cargo check -p mindustry-desktop`、`git diff --check`；
  2. 后续继续扫描 `Fx.java` 后续区间，优先迁移可由现有 circle/line/triangle/rect primitive 表达的效果；
  3. 当前 rect 仍是 headless primitive/cache seam；`TexturedRect.region="casing"` 只是 atlas 名称保留，真实 atlas/GPU backend 仍待接入。

---

## 179. 最新闭环记录：Fx.generatespark/fuelburn/incinerateSlag/coreBurn/plasticburn/conveyorPoof/pulverize*/producesmoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `Fx.java` 中 `210..220` 的生成、燃烧、粉碎粒子段，继续把标准 Fx 接入 Rust `standard_effect_draw_plan(...)` 与 desktop headless primitive frame。
- 本轮迁移：
  - `generatespark=210`
  - `fuelburn=211`
  - `incinerateSlag=212`
  - `coreBurn=213`
  - `plasticburn=214`
  - `conveyorPoof=215`
  - `pulverize=216`
  - `pulverizeRed=217`
  - `pulverizeSmall=218`
  - `pulverizeMedium=219`
  - `producesmoke=220`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `210..220` Fx ID、lookup 与 metadata；
    - 新增颜色符号 `Pal.stoneGray`、`Pal.redDust`、`Pal.plasticBurn`；
    - `generatespark` / `fuelburn` / `incinerateSlag` / `coreBurn` / `plasticburn` / `conveyorPoof` 复用 `SeededCircleParticles`；
    - `pulverize*` / `producesmoke` 复用 `SeededSquareParticles`，统一 `stroke=45` 表示 Java `Fill.square(..., 45)` 的旋转角；
    - 新增 `standard_effect_draw_plan_covers_generate_burn_and_pulverize_particles`。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_generate_burn_and_pulverize_particles_for_render`，验证 11 个 effect event 展开为 28 个 circle primitives 与 26 个 square primitives。
  - `MIGRATION.md`
    - 新增 `12.253` 节记录 Java 依据、Rust 接入点、验证命令与未完成项。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plan_covers_generate_burn_and_pulverize_particles --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_flattens_generate_burn_and_pulverize_particles_for_render --lib`
- 提交前仍需跑完整收尾验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt --check`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_lookup --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plan_covers_generate_burn_and_pulverize_particles --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_flattens_generate_burn_and_pulverize_particles_for_render --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 下一组优先做 `artilleryTrailSmoke=221`、`smeltsmoke=223`、`formsmoke=225`、`lava=227`、`mineWallSmall=233`、`payloadReceive=240`，这些可低风险复用现有 circle/square primitive；
  2. `dooropen/doorclose=228..231` 可复用 `StrokedSquare`，但要注意 Java 的 tile size 与 `rotation` 含义；
  3. `generate=232` 需要 `Lines.spikes` seam，`mineImpactWave` / `teleport*` 是复合 primitive，建议后置；
  4. 当前仍是 headless primitive seam，真实 renderer/backend 与整体可玩 runtime 仍需继续推进，不要宣称可玩。

---

## 180. 最新闭环记录：Fx.artilleryTrailSmoke/smeltsmoke/formsmoke/lava/door*/mine*/payloadReceive/teleport*

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：继续迁移 `Fx.java:2534-2717` 中 `producesmoke=220` 后面可由现有 primitive 承载的一批 smoke/door/mine/teleport 标准 effect。
- 本轮迁移：
  - `artilleryTrailSmoke=221`
  - `smeltsmoke=223`
  - `formsmoke=225`
  - `lava=227`
  - `dooropen=228`
  - `doorclose=229`
  - `dooropenlarge=230`
  - `doorcloselarge=231`
  - `mineWallSmall=233`
  - `mineSmall=234`
  - `mine=235`
  - `mineBig=236`
  - `mineHuge=237`
  - `mineImpact=238`
  - `mineImpactWave=239`
  - `payloadReceive=240`
  - `teleportActivate=241`
  - `teleport=242`
  - `teleportOut=243`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 19 个 Fx ID、lookup、metadata；
    - 新增颜色符号 `Color.orange`、`Color.yellow`、`Pal.plasticSmoke`、`Pal.coalBlack`；
    - `artilleryTrailSmoke` 用 concrete `FilledCircle` plans，保留 Java 每粒子随机局部 `fin/fout/alpha/radius`；
    - `smeltsmoke` / `formsmoke` / `mine*` / `payloadReceive` 复用 `SeededSquareParticles`；
    - `lava` / `mineWallSmall` 复用 `SeededCircleParticles`；
    - `door*` 复用 `StrokedSquare`；
    - `mineImpactWave` / `teleport*` 复用 `StrokedCircle` + `SeededRadialLineParticles` 多 plan seam；
    - 新增 `standard_effect_draw_plans_cover_smoke_door_mine_and_teleport_primitives`。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_smoke_door_mine_and_teleport_primitives_for_render`，覆盖 35 个 draw plans、22 个 circle primitives、63 个 square primitives、82 条 line primitives。
  - `MIGRATION.md`
    - 新增 `12.254` 节记录 Java 依据、Rust 接入点、验证命令与未完成项。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo fmt --check`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_lookup --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plans_cover_smoke_door_mine_and_teleport_primitives --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_flattens_smoke_door_mine_and_teleport_primitives_for_render --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 本轮提交后优先补 `coalSmeltsmoke=224` 的 fractional/progress + `finpowdown` 颜色 easing，和 `generate=232` 的 `Lines.spikes` primitive（当前已由第 181 节补齐）；
  2. 也可继续推进 `ripple=244` 后已有/相邻 Fx 的缺口扫描，但不要跳过 224/232 的文档记录；
  3. 当前仍是 headless primitive seam，真实 renderer/backend 与整体可玩 runtime 仍待继续推进，不要宣称可玩。

---

## 181. 最新闭环记录：Fx.coalSmeltsmoke/generate

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：补齐上一闭环暂留的 `coalSmeltsmoke=224` 与 `generate=232`，避免 `Fx.java:2534-2717` 中出现明显空洞。
- 本轮迁移：
  - `coalSmeltsmoke=224`
  - `generate=232`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_COAL_SMELT_SMOKE_ID` / `FX_GENERATE_ID`、lookup、metadata；
    - 新增 `effect_finpowdown_from_fin(...)`，对齐 Arc `Scaled.finpowdown()` / `Interp.pow3In`；
    - `coalSmeltsmoke` 使用 `SeededCircleParticles`，`progress=Some(0.2 + fin)`，半径 `0.35 + out*2`，颜色 `Color.darkGray -> Pal.coalBlack` 且 mix `fin^3`；
    - `generate` 使用 8 个 deterministic `LineAngle` plans 表达 `Lines.spikes(e.x, e.y, e.fin()*5f, 2, 8)`，颜色 `Color.orange -> Color.yellow`，stroke `1`。
  - `desktop/src/lib.rs`
    - 扩展 `desktop_launcher_flattens_smoke_door_mine_and_teleport_primitives_for_render`，加入 `coalSmeltsmoke` / `generate` 后，统计为 44 draw plans、26 circle primitives、63 square primitives、90 line primitives。
  - `MIGRATION.md`
    - 新增 `12.255` 节，并把 `12.254` 中“暂留 224/232”说明改为已由 `12.255` 补齐。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo fmt --check`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_lookup --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plans_cover_smoke_door_mine_and_teleport_primitives --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_flattens_smoke_door_mine_and_teleport_primitives_for_render --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 继续从 `ripple=244` / `bubble=245` 附近向后扫描；其中部分已有 Rust 支持，要优先补缺口而不是重复迁移；
  2. `generate` 当前是 `LineAngle` seam，不是独立 GPU `Lines.spikes` backend，后续真实 renderer 接入时需要合并到 renderer 的 spike/line 绘制层；
  3. 当前仍是 headless primitive seam，真实 renderer/backend 与整体可玩 runtime 仍待继续推进，不要宣称可玩。

---

## 182. 最新闭环记录：Fx.launchPod/coreLandDust/podLandDust

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：按子代理扫描结果，先迁移 246–265 段里无需新增复杂 primitive 的 `launchPod=248`、`coreLandDust=258`、`podLandDust=259`。
- 本轮迁移：
  - `launchPod=248`
  - `coreLandDust=258`
  - `podLandDust=259`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 3 个 Fx ID、lookup、metadata；
    - 新增颜色符号 `Pal.engine=0xffbb64ff`；
    - `launchPod` 使用 `StrokedCircle` + `SeededRadialLineParticles` 多 plan 表达 Java scaled circle + 24 条 radial line；
    - `coreLandDust` / `podLandDust` 用 concrete `FilledCircle` plans，保留 seeded random offset/radius、`e.fout(0.1/0.2)` 与 `Layer::GROUND_UNIT + 1.0`。
  - `desktop/src/lib.rs`
    - 新增/扩展 `desktop_launcher_flattens_launch_pod_circle_and_lines_for_render`，验证 4 个 draw plans、3 个 circle primitives、24 条 line primitives。
  - `MIGRATION.md`
    - 新增 `12.256` 节。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo fmt --check`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_lookup --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_flattens_launch_pod_circle_and_lines_for_render --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 继续 246–265 段剩余缺口：`healBlockFull=252`、`shieldBreak=256`、`arcShieldBreak=257`、`unitShieldBreak=260`、`chainLightning=261`、`chainEmp=262`、`legDestroy=263`、`debugLine=264`、`debugRect=265`；
  2. 最值得新增的通用 seam 是 `Polyline/Path` primitive，可覆盖 shield/chain/debug line 类效果；
  3. 纹理相关的 `healBlockFull` / `legDestroy` 需要 texture region / textured line seam，建议在 polyline 后处理；
  4. 当前仍是 headless primitive seam，真实 renderer/backend 与整体可玩 runtime 仍待继续推进，不要宣称可玩。

---

## 183. 最新闭环记录：Fx.shieldBreak

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `shieldBreak=256` 的默认 fallback 六边形 polygon，先补齐无 `ForceFieldAbility` typed data 时的 Java 行为。
- 本轮迁移：
  - `shieldBreak=256`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_SHIELD_BREAK_ID`、lookup、metadata；
    - 用 6 个 deterministic `LineAngle` plans 表达 Java `Lines.poly(e.x, e.y, 6, e.rotation + e.fin())`；
    - 颜色走 `Input.color`，stroke `3*fout`。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_shield_break_polygon_lines_for_render`，验证 6 个 draw plans / 6 条 line primitives。
  - `MIGRATION.md`
    - 新增 `12.257` 节。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo fmt --check`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_lookup --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_flattens_shield_break_polygon_lines_for_render --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `shieldBreak` 还缺 `ForceFieldAbility` data 分支，后续应新增 typed data resolver；
  2. 继续 246–265 段剩余缺口时，优先考虑 `Polyline/Path` primitive 覆盖 chain/debug line；
  3. 当前仍是 headless primitive seam，真实 renderer/backend 与整体可玩 runtime 仍待继续推进，不要宣称可玩。

---

## 184. 最新闭环记录：Fx.chainLightning/chainEmp

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `chainLightning=261` / `chainEmp=262`，同时把标准 effect data seam 从 Float-only 扩展到完整 `TypeValue`，为后续数据驱动 Fx 铺路。
- 本轮迁移：
  - `chainLightning=261`
  - `chainEmp=262`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_CHAIN_LIGHTNING_ID` / `FX_CHAIN_EMP_ID`、lookup、metadata；
    - 新增 `standard_effect_draw_plans_with_data_value(...)`，原 `standard_effect_draw_plans_with_data_float(...)` 保持兼容；
    - `TypeValue::Vec2` 暂作 Java `Position` 等价目标点；
    - chain 折线拆为多段 `LineAngle` plans，保留 Java `range=6`、seeded jitter、stroke `2.5*fout` / `4*fout`、颜色 `Color.white -> Input.color`。
  - `core/src/mindustry/entities/mod.rs`
    - 导出 `standard_effect_draw_plans_with_data_value(...)`。
  - `desktop/src/lib.rs`
    - 标准 effect plan 收集改为传入完整 `EffectRenderInput.data`；
    - 新增 `desktop_launcher_flattens_chain_effect_vec2_data_lines_for_render`，验证 2 个 event 展开 10 条 line primitives。
  - `MIGRATION.md`
    - 新增 `12.258` 节。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo fmt --check`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_lookup --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_flattens_chain_effect_vec2_data_lines_for_render --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 继续用 `TypeValue` seam 攻克 `debugLine=264`（`Vec2Array`）和 `debugRect=265`（当前可能需要新增 Rect TypeValue）；
  2. `unitShieldBreak=260` 需要从 Unit id resolve hitSize，适合下一步补 typed resolver；
  3. 当前仍是 headless primitive seam，真实 renderer/backend 与整体可玩 runtime 仍待继续推进，不要宣称可玩。

---

## 185. 最新闭环记录：Fx.debugLine

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `debugLine=264`，利用已接入的完整 `TypeValue` data seam 支持 Java `Vec2[]` 数据驱动折线。
- 本轮迁移：
  - `debugLine=264`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_DEBUG_LINE_ID=264`、lookup、metadata；
    - `TypeValue::Vec2Array` 对应 Java `Vec2[]`，不足 2 点或 data 类型不符时返回空；
    - 2 点/多点统一按相邻点展开成 `LineAngle` plans，stroke `2.0`，颜色走 `Input.color`；
    - core 测试覆盖 ID、metadata、Vec2Array 折线展开和错误 data 空输出。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_debug_line_vec2_array_for_render`，验证 desktop 本地 effect event 的 `TypeValue::Vec2Array` 能展开为 2 条 line primitives 并进入 headless renderer stats。
  - `MIGRATION.md`
    - 新增 `12.259` 节，注明当前仍是 headless primitive seam。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo fmt --check`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_lookup --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_flattens_debug_line_vec2_array_for_render --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 继续 `debugRect=265`：需要新增 `Rect` typed data（或先定义等价 `TypeValue::Rect` / `RectValue`），再用现有 rect primitive 表达 `Lines.rect(rect)`；
  2. `legDestroy=263` 需要 textured line/region seam，建议等 texture-region 抽象更明确后推进；
  3. `unitShieldBreak=260` / `arcShieldBreak=257` 需要 Unit/Ability typed resolver，不要只做孤立 helper；
  4. 当前总迁移仍约 9% 左右，距离完整可玩 Rust MDT/Mindustry 很远，不能宣称可玩。

---

## 186. 最新闭环记录：Fx.debugRect

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `debugRect=265`，补齐本地 `Rect` effect data seam，并避免伪造 Java 不支持的 TypeIO wire tag。
- 本轮迁移：
  - `debugRect=265`
- Rust 主改动：
  - `core/src/mindustry/io/type_io.rs`
    - 新增 `TypeValue::Rect(Rect)`，仅用于本地 debug effect 数据；
    - `write_object(TypeValue::Rect(_))` 返回 `rect object is local-only`，不向 Java 端写出未知 tag。
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_DEBUG_RECT_ID=265`、lookup、metadata；
    - `TypeValue::Rect` 被展开为 4 条 `LineAngle` plans，等价表达 Java `Lines.rect(rect)`，stroke `2.0`，颜色走 `Input.color`；
    - core 测试覆盖 ID、metadata、四边 line 展开和错误 data 空输出。
  - `core/src/mindustry/core/game_state.rs`、`core/src/mindustry/entities/comp/building.rs`
    - 补齐 `TypeValue::Rect` 的 config/stringification exhaustive match。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_debug_rect_data_for_render`，验证 desktop 本地 effect event 展开为 4 条 line primitives。
  - `MIGRATION.md`
    - 新增 `12.260` 节。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo fmt --check`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_lookup --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_flattens_debug_rect_data_for_render --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. 继续 `unitShieldBreak=260`：优先设计 effect render resolver，从 `TypeValue::Unit(id)` 解析到 client/runtime unit hit size，而不是硬塞 float；
  2. `legDestroy=263` 需要 `LegDestroyData` 进入 effect data 通道，并新增 textured line/region seam；
  3. `arcShieldBreak=257` 需要 `ShieldArcAbility` typed resolver/arc primitive；
  4. 本轮 `Rect` 是 local-only，后续若要跨 Java/Rust 网络传输必须先设计双方兼容的 wire convention。

---

## 187. 最新闭环记录：Fx.unitShieldBreak

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `unitShieldBreak=260`，让 Java `Unit` effect data 在 Rust 端通过 `TypeValue::Unit(id)` 接入客户端单位快照，解析 `hitSize()` 后生成护盾破裂 draw plans。
- 本轮迁移：
  - `unitShieldBreak=260`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_UNIT_SHIELD_BREAK_ID=260`、lookup、metadata，lifetime `35.0`；
    - 新增 `standard_effect_draw_plans_with_data_value_and_unit_hit_size(...)`，用于在 standard effect plan 中传入 runtime 已解析的单位 hit size；
    - `TypeValue::Unit(_) + hit_size` 时生成 Java 对应的前 16 帧 seeded radial line particles，以及全生命周期 `StrokedCircle`；缺少 unit data 或 hit size 时返回空，保持 Java guard 语义；
    - core 测试覆盖 ID、metadata、粒子数、圆半径、stroke、缺 hit size 空输出。
  - `core/src/mindustry/entities/mod.rs`
    - 导出新的 typed resolver 入口。
  - `desktop/src/lib.rs`
    - `collect_standard_local_effect_draw_plans_for_render(...)` 从 `runtime.client_unit_snapshot_entities` 建立 `unit_id -> hit_size` 映射；
    - 本地 standard effect event 若带 `TypeValue::Unit(id)`，会把对应 hit size 传入 core plan；
    - 新增 `desktop_launcher_resolves_unit_shield_break_hit_size_for_render`，验证 effect event、client runtime unit snapshot、circle/line primitives 与 headless renderer stats 的完整 seam。
  - `MIGRATION.md`
    - 新增 `12.261` 节，并更新 `12.260` 的剩余项。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo fmt --check`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_lookup --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_resolves_unit_shield_break_hit_size_for_render --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `healBlockFull=252` 可能是下一步最小闭环：需要确认 block/fullIcon seam，可先用现有 square/rect/circle primitive 表达底层效果，但不要脱离 block/content registry；
  2. `arcShieldBreak=257` 需要 `ShieldArcAbility` typed data/resolver，建议先让能力实例/参数进入 effect data 或 runtime sidecar；
  3. `legDestroy=263` 需要 `LegDestroyData` 与 textured line/region seam，复杂度高于普通 shape 特效；
  4. 当前总迁移仍约 9% 左右，远未可玩；后续继续把每个 helper/plan 接入真实 runtime、renderer 与联机路径。

---

## 188. 最新闭环记录：Fx.healBlockFull

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `healBlockFull=252`，让 Java `Block` effect data 在 Rust 端通过 `TypeValue::Content(ContentType::Block, id)` 接入 desktop content registry，解析 `block.size` 后生成 `block.fullIcon` 纹理矩形 primitive。
- 本轮迁移：
  - `healBlockFull=252`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_HEAL_BLOCK_FULL_ID=252`、lookup、metadata，lifetime `20.0`；
    - 新增 `standard_effect_draw_plans_with_data_value_and_resolved_context(...)`，把 unit hitSize 与 block fullIcon size 统一作为 runtime 解析上下文传入；
    - `TypeValue::Content(Block, id) + block_size` 时输出 `TexturedRect`，side 为 `block.size * TILE_SIZE`，颜色走输入 color，alpha 走 `fout`；缺少 block data 或 block size 时返回空，保持 Java guard 语义；
    - `StandardEffectRectRenderPrimitive.region` 改为 `Option<String>`，新增 `block-fullIcon:<id>` region 约定，同时保持 casing region 为 `casing`；
    - core 测试覆盖 ID、metadata、纹理矩形尺寸/颜色/region、缺 resolver 返回空，并回归 casing rect region。
  - `core/src/mindustry/entities/mod.rs`
    - 导出新的 resolved context 入口。
  - `desktop/src/lib.rs`
    - `collect_standard_local_effect_draw_plans_for_render(...)` 从 `content_loader.blocks()` 建立 `block_id -> block.size` 映射；
    - 本地 standard effect event 若带 `TypeValue::Content(Block, id)`，会把对应 block size 传入 core plan；
    - 新增 `desktop_launcher_resolves_heal_block_full_icon_for_render`，验证 effect event、content registry、textured rect primitive 与 headless renderer stats 的完整接线。
  - `MIGRATION.md`
    - 新增 `12.262` 节。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo fmt --check`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_lookup --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plans_cover_casing_rects --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_resolves_heal_block_full_icon_for_render --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `arcShieldBreak=257` 已在后续 `189` 节继续闭环；如回看本节点，关键遗留是专用 arc primitive/真实 renderer；
  2. 继续 `legDestroy=263`：需要 `LegDestroyData` effect data seam 与 textured line primitive；
  3. 当前 `block-fullIcon:<id>` 仍只是 renderer-facing region 约定，真实图形 renderer 后续必须把它解析到 content atlas 的 `Block.fullIcon`；
  4. 当前总迁移仍约 9% 左右，远未可玩，不能把 headless primitive seam 当成最终渲染完成。

---

## 189. 最新闭环记录：Fx.arcShieldBreak

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到乱码优先 UTF-8。
- 本轮目标：迁移 `arcShieldBreak=257`，让 Java `Unit` effect data 在 Rust 端通过 `TypeValue::Unit(id)` 接入客户端单位快照，并从 `unit.type_info.abilities` 解析第一个 `ShieldArcAbility` descriptor，生成弧盾破裂线段。
- 本轮迁移：
  - `arcShieldBreak=257`
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_ARC_SHIELD_BREAK_ID=257`、lookup、metadata，lifetime `40.0`；
    - 新增 `StandardEffectShieldArcBreak`，承载 unit position/rotation 与 shield arc ability 的 `x/y/radius/width/angle/angle_offset`；
    - `standard_effect_draw_plans_with_data_value_and_resolved_context(...)` 新增 `resolved_shield_arc_break`；
    - `TypeValue::Unit(_) + ShieldArcAbility` 时，用多段 `LineAngle` 近似 Java 的外/内两条 `Lines.arc(...)`，再补两条端点连接线；缺少 unit data 或 ability resolver 时返回空，保持 Java guard/first-match 语义；
    - core 测试覆盖 ID、metadata、line plan 数量、stroke、颜色、缺 resolver 返回空。
  - `core/src/mindustry/entities/mod.rs`
    - 导出 `StandardEffectShieldArcBreak`。
  - `desktop/src/lib.rs`
    - `collect_standard_local_effect_draw_plans_for_render(...)` 从 `runtime.client_unit_snapshot_entities` 建立 `unit_id -> StandardEffectShieldArcBreak` 映射；
    - 每个单位按 Java `Structs.find(...)` 等价语义取第一个可解析的 `ShieldArcAbility` descriptor；
    - 新增 `desktop_launcher_resolves_arc_shield_break_ability_for_render`，验证 effect event、client unit snapshot、ability descriptor、line primitives 与 headless renderer stats 的完整接线。
  - `MIGRATION.md`
    - 新增 `12.263` 节，并更新 `12.262` 剩余项。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo fmt --check`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_lookup --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_resolves_arc_shield_break_ability_for_render --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
  - `git diff --check`
- 下一步建议：
  1. `legDestroy=263` 的最小 `LegDestroyData` / textured line seam 已在后续 `190` 节补齐；下一步应继续 `LegsComp.destroy()` 触发链、`UnitType.legRegion/legBaseRegion` 与真实 atlas renderer；
  2. 或先把 `Lines.arc` 从多段 `LineAngle` 升级为专用 arc primitive，并接 desktop stats/renderer seam；
  3. 当前 total 仍约 9% 左右，远未可玩；继续避免让 helper/plan 停留为孤立模块。

---

## 190. 最新闭环记录：Fx.legDestroy

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：迁移 `legDestroy=263` 的本地 `LegDestroyData` effect data seam 与 textured line primitive，让腿部碎裂效果不再只是缺口记录，而能通过 desktop 本地 effect event 展开为带 region 的 line primitive。
- 本轮迁移：
  - `legDestroy=263`
- Rust 主改动：
  - `core/src/mindustry/entities/leg_destroy_data.rs`
    - `TextureRegionRef` 增加 `width/height` 与 `with_size(...)`，用于对齐 Java 的 `data.region.width / 8f` offset 与 `data.region.height * scl` stroke。
  - `core/src/mindustry/io/type_io.rs`
    - 新增 `TypeValue::LegDestroyData(LegDestroyData)`；
    - `write_object` 对该 variant 返回 `InvalidInput`，明确 local-only，不发明 Java 不存在的 `TypeIO` tag；
    - 新增 `leg_destroy_data_is_local_only_and_rejected_by_typeio` 防回归测试。
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_LEG_DESTROY_ID=263`、lookup、metadata，lifetime `90.0`、clip `100.0`、layer `Layer::GROUND_UNIT + 5.0`；
    - 新增 `StandardEffectDrawKind::TexturedLine`，`StandardEffectLineRenderPrimitive.region: Option<String>`；
    - `TypeValue::LegDestroyData` 时生成单个 textured line plan，按 Java 语义保留 seeded random lifetime/offset、`foutpowdown` alpha、region height stroke、`a -> b` 长度；
    - 当前为适配 `StandardEffectDrawPlan.color_from: Option<&'static str>`，使用 effect-region interning seam 缓存 region 名称，避免每个事件重复 `Box::leak`；后续应升级为正式 `String`/`Cow` region 字段或 renderer 端 region 解析。
  - `core/src/mindustry/core/game_state.rs`、`core/src/mindustry/entities/comp/building.rs`
    - 补齐 `TypeValue::LegDestroyData` 的 config kind/stringification exhaustive match。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_flattens_leg_destroy_textured_line_for_render`，验证 `EffectCallPacket2.data = TypeValue::LegDestroyData(...)` 进入 draw plan、line primitive 与 headless renderer stats。
  - `MIGRATION.md`
    - 新增 `12.264` 节，并更新 `12.263` 的遗留说明。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core leg_destroy --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_ids_include --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_lookup --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core standard_effect_draw_plan_covers_smoke_trails_and_ripple --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-desktop desktop_launcher_flattens_leg_destroy_textured_line_for_render --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
- 下一步建议：
  1. `UnitType.legRegion/legBaseRegion` 字段表达与 `LegsComp.destroy()` 到本地 effect 队列的最小链路已在后续 `191` 节补齐；下一步继续 UnitComp 持久腿状态、atlas fallback/尺寸 resolve 与真实 renderer；
  2. 把 `TexturedLine.region` 从 headless primitive seam 接到真实 renderer/backend，支持沿线段绘制 atlas region；
  3. 将 `StandardEffectDrawPlan` 的 region/颜色字段从临时 `&'static str` 约束升级为 `String`/`Cow` 或专用 region 字段，移除 interning seam；
  4. 当前总迁移约 9%～10%，仍远未可玩；继续保持所有 helper/plan 最终接入 runtime/content registry/world/entity/network/client-server 调用链。

---

## 191. 最新闭环记录：LegsComp.destroy → Fx.legDestroy 本地队列链路

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：在 `Fx.legDestroy` 已能渲染 textured line primitive 的基础上，继续把 Java `LegsComp.destroy()` 的腿部碎裂事件接到 Rust 的 `UnitType` region 字段和 `GameRuntime.client_local_effect_events` 本地 effect 队列。
- 本轮迁移：
  - `UnitType.legRegion / legBaseRegion` 字段表达；
  - `LegsComp.destroy()` 语义的 destroy plan；
  - `GameRuntime` 本地 `legDestroy` event queue 接入。
- Rust 主改动：
  - `core/src/mindustry/type/unit_type.rs`
    - 新增 `leg_region: TextureRegionRef`、`leg_base_region: TextureRegionRef`；
    - `UnitType::new(...)` 默认生成 `<name>-leg` 与 `<name>-leg-base` 名称；
    - 单测在 `unit_type_leg_mech_tank_segment_and_missile_defaults_match_java` 锁定默认值。
  - `core/src/mindustry/entities/comp/legs.rs`
    - `LegsType` 新增 `leg_extension`；
    - 新增 `LegsDestroyRegions`、`LegsDestroyEffectEvent`、`LegsDynamicExplosionEvent`、`LegsDestroyPlan`；
    - `LegsComp::destroy_plan(...)` 按 Java guard 跳过未加入/headless，并对每条腿生成两段 `LegDestroyData` 和三个 dynamic explosion plan；
    - 单测覆盖 base→joint、joint+extension→base、region、explosion radius 与 headless/is_added guard。
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `queue_client_legs_destroy_effects(...)`，将 `LegsComp::destroy_plan` 的 effect 事件写入现有 `client_local_effect_events`；
    - 队列事件为 `EffectCallPacket2` + `TypeValue::LegDestroyData`，随后可由现有 `drain_client_local_effect_events_to_states(...)` materialize；
    - 单测 `game_runtime_queues_legs_destroy_into_local_effect_events` 覆盖 component → runtime queue → effect state materialize。
  - `core/src/mindustry/entities/comp/mod.rs`、`core/src/mindustry/entities/mod.rs`
    - 导出新 destroy plan 类型。
  - `MIGRATION.md`
    - 新增 `12.265` 节，并更新 `12.264` 的遗留说明。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core legs --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core game_runtime_queues_legs_destroy_into_local_effect_events --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core unit_type_leg_mech_tank_segment_and_missile_defaults_match_java --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
- 下一步建议：
  1. `LegsComp` 已在后续 `192` 节接入 `UnitComp` 持久状态；下一步继续真实 unit update tick、死亡/移除生命周期和 atlas fallback/尺寸 resolve；
  2. 为 `TextureRegionRef` 接入真实 atlas/content registry 尺寸与 Java `legBaseRegion` fallback：`name + "-leg-base"` fallback 到 `name + "-leg"`；
  3. 将 `LegsDynamicExplosionEvent` 接到 damage/explosion runtime，补齐 Java `Damage.dynamicExplosion(...)` 的实际副作用；
  4. 把 `TexturedLine.region` 接入真实图形 renderer，移除 `StandardEffectDrawPlan` region interning 过渡 seam；
  5. 当前总迁移约 10% 左右，仍远未可玩，继续避免 helper/plan 停留为孤立模块。

---

## 192. 最新闭环记录：UnitComp 持久 LegsComp 接入

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：把 `LegsComp` 从测试手工构造/外部传参推进为 `UnitComp` 的真实持久子组件，并让 `GameRuntime` 可以按 unit id 触发腿部碎裂本地 effect 队列。
- 本轮迁移：
  - `UnitComp.legs: Option<LegsComp>`；
  - `UnitType -> LegsType` 映射；
  - `UnitComp.legs_destroy_plan(...)`；
  - `GameRuntime.queue_client_unit_legs_destroy_effects(...)`。
- Rust 主改动：
  - `core/src/mindustry/entities/comp/unit.rs`
    - 新增 `legs: Option<LegsComp>`；
    - `set_type(...)` 通过 `legs_type_from_unit_type(...)` 创建/复用/清空 legs，使用 `allow_leg_step && leg_count > 0` 判定当前 legged unit；
    - `refresh_component_views()` 同步 `x/y/rotation/status.speed_multiplier` 到 legs；
    - 新增 `legs_destroy_regions()` 与 `legs_destroy_plan(headless)`，从 `UnitType.leg_region/leg_base_region/death_explosion_effect` 生成 destroy plan；
    - `LegsType.speed` 修正为映射 `UnitType.speed`，用于 Java `legContinuousMove` 的总推进；`UnitType.leg_speed` 的步进插值语义后续仍需单独迁移。
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `queue_client_unit_legs_destroy_effects(unit_id, headless)`，从 `client_unit_snapshot_entities` 取 `UnitComp` 后调用其 `legs_destroy_plan(...)`；
    - 抽出私有 `queue_client_leg_destroy_plan(...)`，让 component 入口和 unit-id 入口复用同一 `EffectCallPacket2` 构造路径。
  - `MIGRATION.md`
    - 新增 `12.266` 节，并更新 `12.265` 的遗留说明。
- 已跑验证：
  - `CARGO_BUILD_JOBS=1 cargo fmt`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core unit_component_initializes_and_syncs_legs_from_unit_type --lib`
  - `CARGO_BUILD_JOBS=1 cargo test -p mindustry-core game_runtime_queues_legs_destroy_into_local_effect_events --lib`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-core`
  - `CARGO_BUILD_JOBS=1 cargo check -p mindustry-desktop`
- 下一步建议：
  1. 把 `LegsComp::update(...)` 接到真实 `UnitComp`/`GameRuntime` tick：需要 delta、deltaX/deltaY、deep feet/floor 输入；
  2. 在 unit 死亡/移除生命周期中自动调用 `queue_client_unit_legs_destroy_effects(...)`，而不是仅由测试或外部手动触发；
  3. 为 `TextureRegionRef` 接真实 atlas 尺寸与 Java `legBaseRegion` fallback；
  4. 将 `LegsDynamicExplosionEvent` 接入 damage/explosion runtime；
  5. 当前总迁移约 10% 左右，仍远未可玩，继续保证 helper/plan 最终接入真实 runtime/content/world/entity/network 链路。

---

## 193. 最新闭环记录：UnitDestroyCallPacket 接入腿部碎裂生命周期

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：把上一轮的 `GameRuntime.queue_client_unit_legs_destroy_effects(...)` 从手动 helper 推进到真实客户端单位销毁 packet 生命周期，避免腿部碎裂效果只停留在测试/外部调用层。
- Java 对照：
  - `core/src/mindustry/entities/Units.java`
    - `unitDestroy(int uid)`：`netClient.addRemovedEntity(uid)` 后查 `Groups.unit.getByID(uid)`，存在则 `unit.destroy()`；
    - `unitDespawn(Unit unit)`：`Fx.unitDespawn.at(...)` 后 `unit.remove()`，不等价于 destroy。
  - `core/src/mindustry/entities/comp/UnitComp.java`
    - `destroy()` 负责 explosion、death sound、ability death、`type.killed(...)` 并最终 `remove()`。
  - `core/src/mindustry/entities/comp/LegsComp.java`
    - `destroy()` 在 entity added 且非 headless 时触发每条腿 2 个 `Fx.legDestroy` 与 3 个 `Damage.dynamicExplosion(...)` 点。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `apply_client_unit_destroy_packet(&UnitDestroyCallPacket)`；
    - 在删除 `client_unit_snapshot_entities` 前调用 `queue_client_unit_legs_destroy_effects(uid, false)`，随后执行 `UnitComp::remove(true)`；
    - 新增 `game_runtime_applies_client_unit_destroy_packet_to_legged_unit_effects`，覆盖 destroy packet → 4 条 `legDestroy` 本地 effect、snapshot 移除、重复/负 id 不重复触发。
    - `game_runtime_applies_client_unit_despawn_packet_to_materialized_unit` 改为 legged unit 回归，锁定 despawn 不触发腿部 destroy 特效。
  - `desktop/src/lib.rs`
    - `sync_unit_lifecycle_to_runtime(...)` 新增 `PacketKind::UnitDestroyCallPacket` 分支；
    - 新增 `desktop_launcher_syncs_unit_destroy_packet_to_leg_destroy_effects`，覆盖 net 收包 → launcher update → runtime apply → local effect materialize → renderer-facing textured line primitive。
  - `core/src/mindustry/entities/comp/unit.rs`
    - `set_type(...)` 保留同 type/同 leg count 的现有 legs transient 状态，防止 snapshot/类型刷新重置 `stage` 等腿部运行态；换 type/count 时才 reset。
  - `MIGRATION.md`
    - 新增 `12.267`，并把 `12.266` 的“死亡/移除尚未接入”更新为更精确的剩余项。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_destroy_packet_to_legged_unit_effects`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_despawn_packet_to_materialized_unit`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_destroy_packet_to_leg_destroy_effects`
  - `cargo test -p mindustry-core legs_destroy`
  - `cargo test -p mindustry-core legs_comp`
  - `cargo test -p mindustry-core unit_component_`
  - `git diff --check`
- 当前仍需继续：
  1. `UnitDeathCallPacket` / `UnitSafeDeathCallPacket` / `UnitEnvDeathCallPacket` / `UnitCapDeathCallPacket` 仍需按 Java `killed()` / safe death / env death 语义分别接入；
  2. 当前 `UnitDestroyCallPacket` 只覆盖客户端视觉 effect 与 snapshot 移除，尚未迁移 `UnitComp.destroy()` 的完整 explosion、weapon shoot-on-death、ability death、death sound、wreck/scorch 与 event bus；
  3. `NetClient` 的 unit lifecycle 已在 `194` 节改为增量队列；后续仍需做 consumed cursor compact，避免长时间运行内存增长；
  4. `LegsDynamicExplosionEvent` 仍是 plan，未接真实 damage/explosion runtime；
  5. 当前总迁移仍约 10% 左右，远未可玩，后续必须继续把 helper/plan 收进真实 runtime/content/world/entity/network/client-server 调用链。

---

## 194. 最新闭环记录：unit lifecycle packet 增量队列

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：修掉 `NetClient` 只保留最后一个 unit lifecycle packet 的真实联机缺口，确保一次 `DesktopLauncher.update()` 前收到多个 `UnitDestroy/Despawn/Death/...` packet 时不会只应用尾包。
- Rust 主改动：
  - `core/src/mindustry/core/net_client.rs`
    - `NetClientState` 新增 `unit_lifecycle_packets: Vec<PacketKind>`；
    - `record_unit_lifecycle_packet(...)` 在维护 `last_unit_lifecycle_packet` / `unit_lifecycle_packets_seen` 的同时追加完整 packet 队列；
    - `Debug` 输出新增 `unit_lifecycle_packets_len`；
    - 新增 `update_records_multiple_unit_lifecycle_packets_without_overwriting_queue`，锁定队列顺序与 last-tail 兼容。
  - `desktop/src/lib.rs`
    - `sync_unit_lifecycle_to_runtime()` 改为读取 `unit_lifecycle_packets`，从 `last_applied_unit_lifecycle_packets_seen` cursor 开始逐个分发；
    - 保留现有 `UnitBlockSpawn / AssemblerUnitSpawned / AssemblerDroneSpawned / UnitDespawn / UnitDestroy` 分支；
    - 新增 `desktop_launcher_syncs_multiple_unit_lifecycle_packets_in_one_update`，验证两个 `UnitDestroyCallPacket` 在同一 update 前到达时，两个单位都被移除且产生 8 个腿部 textured-line primitive。
  - `MIGRATION.md`
    - 新增 `12.268`。
- 已跑验证：
  - `cargo test -p mindustry-core update_records_multiple_unit_lifecycle_packets_without_overwriting_queue`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_multiple_unit_lifecycle_packets_in_one_update`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_destroy_packet_to_leg_destroy_effects`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. `unit_lifecycle_packets` 暂时增长式保存，后续需要像真正消息队列一样在所有 cursor 消费后 compact，避免长时间运行内存增长；
  2. `UnitDeathCallPacket` 已在 `195` 节接入 typed runtime；`UnitSafeDeathCallPacket` / `UnitEnvDeathCallPacket` / `UnitCapDeathCallPacket` 仍未按 Java `Units.java` 分支接入；
  3. server 侧死亡路径仍要继续对照 Java `UnitComp.killed()/destroy()`，避免把 destroy 语义误发成 despawn；
  4. 当前总迁移仍约 10% 左右，远未可玩，继续保证所有 helper/plan 最终落到真实 runtime/content/world/entity/network/client-server 链路。

---

## 195. 最新闭环记录：UnitDeathCallPacket 接入 UnitComp.killed 最小语义

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：把已经进入增量队列的 `UnitDeathCallPacket` 从“记录但不应用”推进到 Rust typed runtime，按 Java `Units.unitDeath(uid) -> unit.killed()` 的最小语义处理。
- Java 对照：
  - `core/src/mindustry/entities/Units.java`
    - `unitDeath(int uid)`：加入 removed entity，若 unit 存在则 `unit.killed()`。
  - `core/src/mindustry/entities/comp/UnitComp.java`
    - `killed()`：`health <= 0`、`dead = true`；
    - 当 `!type.flying || !type.createWreck` 时立即 `destroy()`；
    - 否则播放 wreck sound，保留 dead flying unit。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `apply_client_unit_death_packet(&UnitDeathCallPacket)`；
    - 设置本地 unit `health.health = min(health, 0)` 与 `health.dead = true`；
    - ground / 非 wreck 分支复用 `apply_client_unit_destroy_packet(...)`，因此会触发 legged unit 的 `legDestroy` 并移除 snapshot；
    - flying + create_wreck 分支先只保留 dead/added 状态，不伪造尚未迁移的 wreck sound/update/renderer；
    - 新增 `game_runtime_applies_client_unit_death_packet_like_java_killed`。
  - `desktop/src/lib.rs`
    - lifecycle 增量分发新增 `PacketKind::UnitDeathCallPacket`；
    - `desktop_launcher_syncs_multiple_unit_lifecycle_packets_in_one_update` 改成一次 update 前混合 `UnitDeathCallPacket` 与 `UnitDestroyCallPacket`，验证两个 packet 都被应用。
  - `MIGRATION.md`
    - 新增 `12.269`，并更新 `12.268` 的剩余项。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_death_packet_like_java_killed`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_multiple_unit_lifecycle_packets_in_one_update`
  - `cargo test -p mindustry-core update_records_multiple_unit_lifecycle_packets_without_overwriting_queue`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. `UnitComp.destroy()` 的完整 explosion、death sound、weapon shoot-on-death、ability death、wreck/scorch 与 event bus 仍待迁移；
  2. flying wreck 分支目前只是 dead/added 状态，后续要接 wreck sound、坠毁 update、残骸/renderer；
  3. `UnitSafeDeathCallPacket` 已在 `196` 节接入最小 remove/effect 语义；`UnitCapDeathCallPacket`、`UnitEnvDeathCallPacket` 仍需按 Java `Units.java` 对应分支接入；
  4. server 侧死亡路径仍需对照 Java packet 选择，不能长期用 despawn 代替 death/destroy；
  5. 当前总迁移仍约 10% 左右，远未可玩，继续保证所有模块不是孤立 helper，而是逐步接到真实 runtime/content/world/entity/network/client-server 链路。

---

## 196. 最新闭环记录：UnitSafeDeathCallPacket 接入 remove + deathExplosionEffect 最小语义

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：把 `UnitSafeDeathCallPacket` 从 lifecycle 队列记录推进到客户端 runtime apply，对齐 Java `Units.unitSafeDeath(Unit unit)` 的最小 remove/effect 语义。
- Java 对照：
  - `core/src/mindustry/entities/Units.java::unitSafeDeath(Unit unit)`：
    - null 直接 return；
    - `unit.type.deathExplosionEffect.at(unit.x, unit.y, unit.hitSize / 8f)`；
    - `Effect.shake(...)`；
    - `unit.type.deathSound.at(...)`；
    - `unit.remove()`。
  - 该路径不是 `unit.destroy()`，因此不应触发 `LegsComp.destroy()` / `Fx.legDestroy`。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `apply_client_unit_safe_death_packet(&UnitSafeDeathCallPacket)`；
    - 从 `UnitRef::Unit { id }` 找到本地 unit 后，按 `death_explosion_effect` 尝试排入 `client_local_effect_events`，rotation = `hit_size / 8.0`；
    - 若 `death_sound` 在当前 `standard_sound_id` 窄表可解析，则排入 `client_local_sound_at_events`；
    - 从 `client_unit_snapshot_entities` 移除并调用 `UnitComp::remove(true)`；
    - 新增 `game_runtime_applies_client_unit_safe_death_packet_like_java_remove_with_effect`，验证 safe death 只产生 death effect，不触发 `legDestroy`。
  - `desktop/src/lib.rs`
    - lifecycle 分发新增 `PacketKind::UnitSafeDeathCallPacket`；
    - 新增 `desktop_launcher_syncs_unit_safe_death_packet_to_runtime_remove_effect`。
  - `MIGRATION.md`
    - 新增 `12.270`，并更新 `12.269` 的剩余项。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_safe_death_packet_like_java_remove_with_effect`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_safe_death_packet_to_runtime_remove_effect`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_multiple_unit_lifecycle_packets_in_one_update`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. `Fx.dynamicExplosion` 已在 `198` 节进入 `standard_effect_id`/metadata/动态 lifetime/最小 line draw seam；后续仍需补完整圆形粒子、光照和真实 renderer；
  2. `standard_sound_id` 已在 `199` 节覆盖 `unitExplode1/2/3` 与 `wreckFall/wreckFallBig`；更多声音和真实 backend 播放仍需继续；
  3. `Effect.shake(...)` 的 safe death 本地 camera shake 事件已在 `200` 节接入；后续仍需接 desktop camera/backend；
  4. `UnitCapDeathCallPacket` 与 `UnitEnvDeathCallPacket` 已在 `197` 节接入 mark-dead + local effect 最小语义；后续仍需 post-destroy 与真实 icon renderer；
  5. 当前总迁移仍约 10% 左右，远未可玩，继续把 helper/plan 下沉到真实 runtime/content/world/entity/network/client-server 链路。

---

## 197. 最新闭环记录：UnitCapDeathCallPacket / UnitEnvDeathCallPacket 接入 mark-dead + effect

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：把 `UnitCapDeathCallPacket` 与 `UnitEnvDeathCallPacket` 从 lifecycle 队列记录推进到客户端 runtime apply，对齐 Java `Units.unitCapDeath/unitEnvDeath` 的最小前置语义。
- Java 对照：
  - `Units.unitCapDeath(Unit unit)`：
    - `unit.dead = true`；
    - `Fx.unitCapKill.at(unit)`；
    - `Core.app.post(() -> Call.unitDestroy(unit.id))`。
  - `Units.unitEnvDeath(Unit unit)`：
    - `unit.dead = true`；
    - `Fx.unitEnvKill.at(unit)`；
    - `Core.app.post(() -> Call.unitDestroy(unit.id))`。
  - 两者本身不立即 remove，最终销毁由后续 `unitDestroy` 完成。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_UNIT_CAP_KILL_ID = 4`、`FX_UNIT_ENV_KILL_ID = 5`；
    - `standard_effect_id("unitCapKill"/"unitEnvKill")`；
    - `standard_effect(...)` metadata lifetime `80.0`；
    - 更新标准 effect id/metadata 测试。
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增私有 `apply_client_unit_mark_dead_effect_packet(...)`；
    - 新增 `apply_client_unit_cap_death_packet(...)` 与 `apply_client_unit_env_death_packet(...)`；
    - 对本地 unit 设置 `health.dead = true`、`health <= 0`，排入 local effect，但保留 unit snapshot/added 状态；
    - 新增 `game_runtime_applies_client_unit_cap_and_env_death_packets_like_java_mark_dead`。
  - `desktop/src/lib.rs`
    - lifecycle 分发新增 `PacketKind::UnitCapDeathCallPacket` / `PacketKind::UnitEnvDeathCallPacket`；
    - 新增 `desktop_launcher_syncs_unit_cap_and_env_death_packets_to_runtime_mark_dead`。
  - `MIGRATION.md`
    - 新增 `12.271`，并更新 `12.270` 的剩余项。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_cap_and_env_death_packets_like_java_mark_dead`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_cap_and_env_death_packets_to_runtime_mark_dead`
  - `cargo test -p mindustry-core standard_effect_ids_include`
  - `cargo test -p mindustry-core standard_effect_lookup`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. Java 的 `Core.app.post(() -> Call.unitDestroy(unit.id))` 后续 destroy 调用尚未在 Rust 客户端主动发出；需要结合真实网络方向决定由 server 后续 packet 负责还是客户端 call；
  2. `Fx.unitCapKill` / `Fx.unitEnvKill` 当前只有 id/metadata 与 effect state，真实 warning/cancel icon renderer 还没迁移；
  3. `Fx.dynamicExplosion` 已在 `198` 节补最小 seam；`unitExplode/wreckFall` id 已在 `199` 节补齐；safeDeath camera shake 已在 `200` 节补本地事件；event bus、weapon shoot-on-death、ability death 仍未完整；
  4. 当前总迁移仍约 10% 左右，远未可玩，继续把 helper/plan 下沉到真实 runtime/content/world/entity/network/client-server 链路。

---

## 198. 最新闭环记录：Fx.dynamicExplosion 最小 metadata / lifetime / line draw seam

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：补齐默认单位死亡爆炸效果 `dynamicExplosion` 的 effect 表缺口，让 `UnitType::new(...).death_explosion_effect = "dynamicExplosion"` 能被 safe death 与本地 renderer seam 解析。
- Java 对照：
  - `Fx.java:1676`：`dynamicExplosion = new Effect(30, 500f, b -> ...)`；
  - `intensity = b.rotation`；
  - renderer 内重写 `b.lifetime = 43f + intensity * 35f`；
  - 主线段分支使用 `(int)(9 * intensity)`、`40f * intensity`、线长 `1f + out * 4 * (3f + intensity)`。
- Rust 主改动：
  - `core/src/mindustry/entities/effect.rs`
    - 新增 `FX_DYNAMIC_EXPLOSION_ID = 149`；
    - 新增 `standard_effect_id("dynamicExplosion")`；
    - `standard_effect(...)` 返回 lifetime `30.0`、clip `500.0`；
    - `standard_effect_render_lifetime(...)` 对 dynamicExplosion 使用 `43.0 + rotation * 35.0`；
    - `standard_effect_draw_plan(...)` 新增 `SeededRadialLineParticles` 最小 seam，覆盖主 radial line 分支；
    - 新增 `standard_effect_draw_plan_covers_dynamic_explosion_lines`，并更新 id/metadata/lifetime 测试。
  - `core/src/mindustry/core/game_runtime.rs`
    - safe death 测试恢复使用默认 `dynamicExplosion`，不再临时改成 `despawn`。
  - `desktop/src/lib.rs`
    - safe death desktop 测试同样使用默认 `dynamicExplosion` materialize 本地 effect state。
  - `MIGRATION.md`
    - 新增 `12.272`，并更新 `12.269`、`12.270`、`12.271` 的剩余项。
- 已跑验证：
  - `cargo test -p mindustry-core standard_effect_ids_include`
  - `cargo test -p mindustry-core standard_effect_lookup`
  - `cargo test -p mindustry-core standard_effect_render_lifetime_applies`
  - `cargo test -p mindustry-core standard_effect_draw_plan_covers_dynamic_explosion_lines`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_safe_death_packet_like_java_remove_with_effect`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_safe_death_packet_to_runtime_remove_effect`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. Java `dynamicExplosion` 的灰色圆形粒子 multi-pass、`baseLifetime` 子阶段与 `Drawf.light(...)` 精确行为还没完整；
  2. 真实 renderer/backend 对该效果还只是 primitive seam；
  3. `unitExplode1/2/3` 与 `wreckFall/wreckFallBig` 已在 `199` 节补入 sound id；safeDeath camera shake 已在 `200` 节补本地事件；完整 `UnitComp.destroy()` side effects 仍需继续；
  4. 当前总迁移仍约 10% 左右，远未可玩，继续把 helper/plan 下沉到真实 runtime/content/world/entity/network/client-server 链路。

---

## 199. 最新闭环记录：unitExplode / wreckFall 死亡音效 id 映射

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：补齐 `UnitSafeDeathCallPacket` / 后续 wreck 分支会用到的基础死亡音效 id，让 `deathSound.at(...)` seam 可以实际写入本地 sound event。
- Java/资源对照：
  - 按 `core/assets/sounds` 递归文件名排序确认：
    - `unitCreate = 190`
    - `unitCreateBig = 191`
    - `unitExplode1 = 192`
    - `unitExplode2 = 193`
    - `unitExplode3 = 194`
    - `wreckFall = 203`
    - `wreckFallBig = 204`
- Rust 主改动：
  - `core/src/mindustry/audio/mod.rs`
    - `standard_sound_id(...)` 新增 `unitExplode1/2/3`、`wreckFall`、`wreckFallBig`；
    - 更新 `standard_sound_ids_follow_upstream_assets_process_order`。
  - `core/src/mindustry/core/game_runtime.rs`
    - safe death 回归设置 `death_sound = "unitExplode1"`、`death_sound_volume = 0.7`，验证 `client_local_sound_at_events` 的 id、位置和音量。
  - `desktop/src/lib.rs`
    - desktop safe death 回归验证 launcher update 后 runtime 保留本地 sound event。
  - `MIGRATION.md`
    - 新增 `12.273`，并更新 `12.270` / `12.272` 剩余项。
- 已跑验证：
  - `cargo test -p mindustry-core standard_sound_ids_follow_upstream_assets_process_order`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_safe_death_packet_like_java_remove_with_effect`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_safe_death_packet_to_runtime_remove_effect`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. `standard_sound_id` 仍是窄表，完整 Java sounds 表尚未迁移；
  2. desktop/backend 真实播放层仍需从 `client_local_sound_at_events` 下沉到 audio backend；camera shake backend 已在 `200` 节有本地事件 seam，仍需接实际 camera；
  3. flying wreck 分支尚未触发 `wreckFall*`；
  4. 当前总迁移仍约 10% 左右，远未可玩，继续把 helper/plan 接到真实 runtime/content/world/entity/network/client-server 链路。

---

## 200. 最新闭环记录：UnitSafeDeath Effect.shake 本地 camera shake 事件

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：补齐 `Units.unitSafeDeath(...)` 中 `Effect.shake(shake, shake, unit)` 的 runtime seam，让 safe death 不只产生 effect/sound/remove，还能记录 camera shake 副作用。
- Java 对照：
  - `float shake = unit.type.deathShake < 0 ? unit.hitSize / 3f : unit.type.deathShake;`
  - `Effect.shake(shake, shake, unit);`
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `GameRuntimeClientCameraShakeEvent { x, y, intensity, duration }`；
    - `GameRuntime` 新增 `client_local_camera_shake_events`，并在 reset/clear 中清理；
    - `apply_client_unit_safe_death_packet(...)` 按 Java 公式计算 shake，写入本地 camera shake event；
    - safe death core 测试验证 x/y、`hit_size / 3.0` 默认强度、duration。
  - `desktop/src/lib.rs`
    - safe death desktop 测试验证 launcher update 后 runtime 保留 camera shake event。
  - `MIGRATION.md`
    - 新增 `12.274`，并更新 `12.270` / `12.271` / `12.272` / `12.273` 剩余项。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_safe_death_packet_like_java_remove_with_effect`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_safe_death_packet_to_runtime_remove_effect`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. camera shake runtime 本地事件已在 `201` 节下沉到 desktop pending/state；真实 camera backend 随机 offset 仍未接；
  2. `UnitComp.destroy()`、flying wreck、其他 effect shake 调用仍未全面接入；
  3. 多个 shake event 的真实 camera 合并、随机方向 offset 与 settings 读取还需对照 Java/Arc 实现；
  4. 当前总迁移仍约 10% 左右，远未可玩，继续把 helper/plan 接到真实 runtime/content/world/entity/network/client-server 链路。

---

## 201. 最新闭环记录：UnitSafeDeath camera shake 下沉到 desktop render seam

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：把 `GameRuntime.client_local_camera_shake_events` 从 core runtime 队列继续接到 desktop launcher 层，避免 `Effect.shake(...)` 只停在孤立事件上。
- Java 对照：
  - `Effect.shake(intensity, duration, x, y)` 根据 camera 到事件点距离和 `shakeFalloff` 衰减强度；
  - `Renderer.shake(intensity, duration)` 对 `shakeIntensity` / `shakeTime` 取 max，设置 `shakeReduction = shakeIntensity / shakeTime`；
  - renderer update 用 `screenshake / 4f * 0.75f` 得到随机 camera offset 最大强度，并按 delta 衰减。
- Rust 主改动：
  - `desktop/src/lib.rs`
    - 新增 `DesktopCameraShakeState`、`DesktopCameraShakeFrame`；
    - `DesktopLauncher` 新增 `pending_camera_shake_events`、`camera_shake_state`、`last_camera_shake_frame`；
    - `update()` 中从 `runtime.client_local_camera_shake_events` 转移到 desktop pending queue，用 `shake_intensity(...)` 解析距离衰减，并 tick 出 `last_camera_shake_frame`；
    - 新增 `sync_local_camera_shake_events_for_render(...)`、`tick_camera_shake_for_render(...)`、`drain_camera_shake_events_for_render(...)`；
    - 更新 safe death desktop 测试，新增 `desktop_launcher_resolves_camera_shake_events_for_render_like_java_effect_shake`。
  - `MIGRATION.md`
    - 新增 `12.275`，并更新 `12.274` 剩余项。
- 已跑验证：
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_safe_death_packet_to_runtime_remove_effect`
  - `cargo test -p mindustry-desktop desktop_launcher_resolves_camera_shake_events_for_render_like_java_effect_shake`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_safe_death_packet_like_java_remove_with_effect`
  - `cargo check -p mindustry-desktop`
- 当前仍需继续：
  1. 真实 desktop camera/backend 还没有使用 `last_camera_shake_frame.max_offset` 生成随机 `camShakeOffset` 并应用到 camera；
  2. `screenshake` 暂时在 `update()` 以 Java 默认最大值 `4` 传入，后续要接 settings；
  3. `sync_local_camera_shake_events_for_render(...)` 暂用 `player.x/y` 作为 camera 参考，后续需要接真实 camera state；
  4. flying wreck death sound 已在 `202` 节接入；audio backend、flying wreck update/renderer、完整 `UnitComp.destroy()` side effects 仍是后续主线；
  5. 当前总迁移仍约 10% 左右，远未可玩。

---

## 202. 最新闭环记录：UnitDeath flying wreck 分支音效

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：补齐 Java `UnitComp.killed()` 的 flying + `createWreck` 分支，让 `UnitDeathCallPacket` 不只标 dead，还能产生 `wreckSound.at(this, 1f, wreckSoundVolume)` 对应事件。
- Java 对照：
  - 非 flying 或不 createWreck：`destroy()`
  - flying 且 createWreck：`type.wreckSound.at(this, 1f, type.wreckSoundVolume)`
  - `UnitType.init()` 默认 wreck sound：`hitSize >= 22f ? wreckFallBig : wreckFall`
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - `apply_client_unit_death_packet(...)` 在 flying + `create_wreck` 分支保留 unit，不走 destroy；
    - 用 `UnitType::pure_init_plan().wreck_sound` 解析默认 `wreckFall` / `wreckFallBig`；
    - 写入 `client_local_sound_at_events`，保留位置、`wreck_sound_volume`、pitch `1.0`；
    - 更新 `game_runtime_applies_client_unit_death_packet_like_java_killed`。
  - `desktop/src/lib.rs`
    - 新增 `desktop_launcher_syncs_flying_unit_death_to_wreck_sound_without_remove`，覆盖 NetClient lifecycle -> DesktopLauncher -> GameRuntime 的真实同步路径。
  - `MIGRATION.md`
    - 新增 `12.276`。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_death_packet_like_java_killed`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_flying_unit_death_to_wreck_sound_without_remove`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_multiple_unit_lifecycle_packets_in_one_update`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
- 当前仍需继续：
  1. flying wreck 残骸实体/坠落 update/renderer/撞击伤害仍未迁移；
  2. `client_local_sound_at_events` 已在 `203` 节下沉到 desktop pending audio seam；真实 desktop audio backend 播放仍未接；
  3. 完整 `UnitComp.destroy()` 的 `Damage.dynamicExplosion`、`Effect.shake`、scorch、weapon `shootOnDeath`、ability death、event bus 等仍未完成；
  4. 当前总迁移仍约 10% 左右，远未可玩。

---

## 203. 最新闭环记录：本地 sound-at 事件下沉到 desktop audio seam

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：把 core runtime 产生的本地 sound-at 事件继续下沉到 desktop launcher，给真实 audio backend 留出统一消费入口。
- Rust 主改动：
  - `desktop/src/lib.rs`
    - `DesktopLauncher` 新增 `pending_sound_at_events: Vec<SoundAtCallPacket>`；
    - `update()` 调用 `sync_local_sound_at_events_for_audio()`，把 `runtime.client_local_sound_at_events` 转移到 desktop pending queue；
    - 新增 `drain_sound_at_events_for_audio()`；
    - reset/clear 路径清空 pending sound queue；
    - 新增 `desktop_launcher_syncs_and_drains_local_sound_at_events_for_audio`；
    - 更新 assembler spawn、unit spawn + assembler、safe death、flying wreck death 测试，改为验证 runtime sound 队列已清空、desktop pending 队列持有待播放事件。
  - `MIGRATION.md`
    - 新增 `12.277`。
- 已跑验证：
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_and_drains_local_sound_at_events_for_audio`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_flying_unit_death_to_wreck_sound_without_remove`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_safe_death_packet_to_runtime_remove_effect`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_assembler_unit_spawned_packet_to_runtime`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_spawn_packet_without_losing_assembler_spawned`
  - `cargo check -p mindustry-desktop`
- 当前仍需继续：
  1. `pending_sound_at_events` 已在 `204` 节接入 desktop audio frame/headless renderer seam；真实平台 audio backend 还没有播放；
  2. sound asset/backend、距离衰减、音量设置、重复事件合并还没完整；
  3. camera shake 也仍只是 pending/state seam，没有真实 camera offset；
  4. flying wreck update/renderer、完整 `UnitComp.destroy()` 仍是后续主线；
  5. 当前总迁移仍约 10% 左右，远未可玩。

---

## 204. 最新闭环记录：desktop sound-at audio frame 与 headless renderer seam

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：在 `pending_sound_at_events` 后补 backend 消费入口，使本地 sound-at 事件能沿 desktop audio frame/renderer seam 继续下沉。
- Rust 主改动：
  - `desktop/src/lib.rs`
    - 新增 `DesktopSoundAtAudioFrame`；
    - 新增 `DesktopSoundAudioStats`；
    - 新增 `DesktopAudioRenderer` trait；
    - 新增 `HeadlessDesktopAudioRenderer`；
    - `DesktopLauncher` 新增 `sound_at_audio_frame()`、`play_sound_at_audio_frame_with(...)`、`drain_and_play_sound_at_audio_frame_with(...)`；
    - 新增测试 `desktop_launcher_plays_sound_at_audio_frame_with_headless_renderer`。
  - `MIGRATION.md`
    - 新增 `12.278`。
- 已跑验证：
  - `cargo test -p mindustry-desktop desktop_launcher_plays_sound_at_audio_frame_with_headless_renderer`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_and_drains_local_sound_at_events_for_audio`
  - `cargo check -p mindustry-desktop`
- 当前仍需继续：
  1. 真实平台 audio backend 仍需实现 `DesktopAudioRenderer` 并接 assets/device；
  2. sound 距离衰减、音量设置、重复事件合并、完整 sound id/asset 表仍未做；
  3. camera shake 已在 `205` 节接入 renderer/headless seam；真实 camera offset backend 仍未做；
  4. flying wreck update/renderer、完整 `UnitComp.destroy()` 仍是后续主线；
  5. 当前总迁移仍约 10%~11%，远未可玩。

---

## 205. 最新闭环记录：desktop camera shake renderer 与 headless seam

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：把 `DesktopCameraShakeFrame` 接到 renderer/backend seam，给真实 camera offset 应用留出明确入口。
- Rust 主改动：
  - `desktop/src/lib.rs`
    - 新增 `DesktopCameraShakeRenderStats`；
    - 新增 `DesktopCameraShakeRenderer` trait；
    - 新增 `HeadlessDesktopCameraShakeRenderer`；
    - `DesktopLauncher` 新增 `apply_camera_shake_frame_with(...)`；
    - 新增测试 `desktop_launcher_applies_camera_shake_frame_with_headless_renderer`。
  - `MIGRATION.md`
    - 新增 `12.279`。
- 已跑验证：
  - `cargo test -p mindustry-desktop desktop_launcher_applies_camera_shake_frame_with_headless_renderer`
  - `cargo test -p mindustry-desktop desktop_launcher_resolves_camera_shake_events_for_render_like_java_effect_shake`
  - `cargo check -p mindustry-desktop`
- 当前仍需继续：
  1. 真实 camera backend 仍需按 Java `Renderer` 随机方向 offset 应用/回退 camera position；
  2. `screenshake` setting、真实 camera 坐标、真实 delta 仍需接；
  3. audio backend 仍需真实播放；
  4. `UnitDestroyCallPacket` 已在 `206` 节接入 `UnitComp.destroy()` 的 dynamicExplosion/deathSound/deathShake 主副作用；scorch/weapon/ability/event bus/flying wreck update 仍是后续主线；
  5. 当前总迁移仍约 10%~11%，远未可玩。

---

## 206. 最新闭环记录：UnitDestroy 主死亡副作用 dynamicExplosion / deathSound / deathShake

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：继续补 Java `UnitComp.destroy()`，让 `UnitDestroyCallPacket` 不再只是 legDestroy + remove，而是接入动态爆炸、死亡音效、死亡镜头震动这些主副作用。
- Java 对照：
  - `Damage.dynamicExplosion(..., type.deathExplosionEffect, 0f)`；
  - `type.deathExplosionEffect.at(...)` for `spawnedByCore`；
  - `deathShake < 0 ? 3f + hitSize / 3f : deathShake`；
  - `Effect.shake(shake, shake, this)`；
  - `type.deathSound.at(this, 1f, type.deathSoundVolume)`；
  - `remove()`。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `queue_client_unit_destroy_side_effects(...)`；
    - `apply_client_unit_destroy_packet(...)` 移除 unit 前写入 `death_explosion_effect`、death shake event、death sound event，并保留 legDestroy；
    - 更新 `game_runtime_applies_client_unit_destroy_packet_to_legged_unit_effects`；
    - 更新 `game_runtime_applies_client_unit_death_packet_like_java_killed`。
  - `desktop/src/lib.rs`
    - 更新 `desktop_launcher_syncs_unit_destroy_packet_to_leg_destroy_effects`；
    - 更新 `desktop_launcher_syncs_multiple_unit_lifecycle_packets_in_one_update`；
    - 覆盖 NetClient lifecycle -> GameRuntime -> local effect/materialize + pending sound/camera seam。
  - `MIGRATION.md`
    - 新增 `12.280`。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_destroy_packet_to_legged_unit_effects`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_death_packet_like_java_killed`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_destroy_packet_to_leg_destroy_effects`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_multiple_unit_lifecycle_packets_in_one_update`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
- 当前仍需继续：
  1. `Damage.dynamicExplosion(...)` 的 lightning/fire/wave damage/shockwave 仍未完整；
  2. `createScorch -> Effect.scorch(...)` 已在 `207` 节接入本地 decal seam；wreck decal、weapon `shootOnDeath`、ability `death(...)`、`type.killed(...)`、event bus、suicide trigger 仍未接；
  3. 真实 audio/camera backend 仍需继续；
  4. flying wreck update/renderer/坠毁伤害仍需继续；
  5. 当前总迁移仍约 10%~11%，远未可玩。

---

## 207. 最新闭环记录：UnitDestroy createScorch 本地 decal seam

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：补 Java `UnitComp.destroy()` 中 `createScorch -> Effect.scorch(...)`，让单位销毁能产生本地 scorch decal。
- Java 对照：
  - `Effect.scorch(x, y, (int)(hitSize / 5))`
  - size clamp `0..9`
  - region `scorch-{size}-{random(2)}`
  - rotation `random(4) * 90`
  - lifetime `3600`
  - color `Pal.rubble`
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - `GameRuntime` 新增 `next_client_local_decal_id`；
    - reset/clear 路径重置本地 decal id；
    - `queue_client_unit_destroy_side_effects(...)` 在 `create_scorch` 时创建本地 `DecalComp`，插入 `client_decal_snapshot_entities` 负 id；
    - 使用 `DecalColor::from_rgba(0x1c1817ff)` 对齐 `Pal.rubble`；
    - 更新 core destroy 测试。
  - `desktop/src/lib.rs`
    - 更新 unit destroy 和多 lifecycle 测试，验证 scorch decal 经过 NetClient lifecycle 路径进入 runtime decal map。
  - `MIGRATION.md`
    - 新增 `12.281`。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_destroy_packet_to_legged_unit_effects`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_destroy_packet_to_leg_destroy_effects`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_multiple_unit_lifecycle_packets_in_one_update`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. scorch 的 random variant/rotation 目前是 deterministic seam，后续需接真实随机/renderer；
  2. wreckRegions decal 随机散布仍未做；
  3. `UnitDestroyEvent` 已在 `208` 节记录为 runtime sidecar；`Damage.dynamicExplosion` lightning/fire/wave damage、weapon `shootOnDeath`、ability death、完整 event bus 仍未完成；
  4. 真实 audio/camera backend 仍需继续；
  5. 当前总迁移仍约 10%~11%，远未可玩。

---

## 208. 最新闭环记录：UnitDestroyEvent runtime sidecar

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：对照 Java `Events.fire(new UnitDestroyEvent(self()))`，在 Rust runtime 中先记录 UnitDestroyEvent sidecar，后续再接真实 event bus / service trigger。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `GameRuntimeUnitDestroyEvent`；
    - `GameRuntime` 新增 `unit_destroy_events`；
    - reset/clear 路径清空该队列；
    - 新增 `drain_unit_destroy_events()`；
    - `queue_client_unit_destroy_side_effects(...)` 记录 unit id/name/team/x/y；
    - core destroy 测试覆盖事件字段与 drain。
  - `desktop/src/lib.rs`
    - unit destroy / 多 lifecycle 测试验证事件会经 NetClient lifecycle -> GameRuntime 保留。
  - `MIGRATION.md`
    - 新增 `12.282`。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_destroy_packet_to_legged_unit_effects`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_destroy_packet_to_leg_destroy_effects`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_multiple_unit_lifecycle_packets_in_one_update`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. `UnitDestroyEvent` 还只是 runtime sidecar，未接全局 event bus；
  2. weapon `shootOnDeath` 已在 `209` 节记录 runtime sidecar 并触发 override effect；suicide trigger、ability `death(...)`、`type.killed(...)` 未迁移；
  3. `Damage.dynamicExplosion` lightning/fire/wave damage 未完整；
  4. 真实 audio/camera backend 仍需继续；
  5. 当前总迁移仍约 10%~11%，远未可玩。

---

## 209. 最新闭环记录：UnitDestroy weapon shootOnDeath runtime sidecar

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：对照 Java `UnitComp.destroy()` 的 weapon `shootOnDeath` 分支，先记录死亡射击 sidecar，并在存在 override effect 时写入本地 effect seam。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `GameRuntimeUnitShootOnDeathEvent`；
    - `GameRuntime` 新增 `unit_shoot_on_death_events`；
    - reset/clear 路径清空该队列；
    - 新增 `drain_unit_shoot_on_death_events()`；
    - `queue_client_unit_destroy_side_effects(...)` 遍历 `unit.weapons.mounts`，对 `shoot_on_death` weapon 记录事件；
    - 若 `shoot_on_death_effect` 存在且 unit 无目标，排入对应标准 effect，并记录 `allow_shoot_effects=false`；
    - core destroy 测试覆盖 sidecar、override `smoke` effect、drain。
  - `desktop/src/lib.rs`
    - unit destroy desktop 测试增加 death weapon，验证 lifecycle 后事件存在且 local effect entity 数增加。
  - `MIGRATION.md`
    - 新增 `12.283`。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_destroy_packet_to_legged_unit_effects`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_destroy_packet_to_leg_destroy_effects`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. 还没有真正执行 `mount.weapon.update(...)` / bullet spawn；
  2. `bullet.killShooter && totalShots > 0` 条件尚未完整，因为该路径还未解析 BulletType；
  3. ability `death(...)`、`type.killed(...)`、suicide trigger、完整 event bus 仍未迁移；
  4. 真实 audio/camera backend 仍需继续；
  5. 当前总迁移仍约 10%~11%，远未可玩。

---

## 210. 最新闭环记录：UnitDestroy ability death runtime sidecar

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：对照 Java `UnitComp.destroy()` 的 `for(Ability a : abilities) a.death(self())`，先在 Rust runtime 中记录单位死亡能力 sidecar，后续再接到具体 ability death 实现与真实 event bus。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `GameRuntimeUnitAbilityDeathEvent`；
    - `GameRuntime` 新增 `unit_ability_death_events`；
    - reset/clear 路径清空该队列；
    - 新增 `drain_unit_ability_death_events()`；
    - `queue_client_unit_destroy_side_effects(...)` 遍历 `unit.type_info.abilities`，提取 `ability_kind` 并记录 descriptor/x/y；
    - core destroy 测试覆盖 `SpawnDeathAbility:flare,2,8` 的 sidecar 字段与 drain。
  - `desktop/src/lib.rs`
    - unit destroy desktop 测试增加 ability descriptor，验证 lifecycle 后 runtime 中保留 ability death sidecar。
  - `MIGRATION.md`
    - 新增 `12.284`。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_destroy_packet_to_legged_unit_effects`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_destroy_packet_to_leg_destroy_effects`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. 当前只是 sidecar，尚未真正执行具体 ability 的 `death(...)` 行为；
  2. 优先把已有 `SpawnDeathAbility`、`LiquidExplodeAbility`、`ForceFieldAbility` 等 death runtime 与 `UnitDestroyCallPacket` / event bus 打通；
  3. `type.killed(self())`、suicide trigger、wreckRegions decal、weapon bullet spawn 仍未迁移；
  4. `Damage.dynamicExplosion(...)` lightning/fire/wave damage、真实 audio/camera backend 仍需继续；
  5. 当前总迁移仍约 10%~11%，远未可玩。

---

## 211. 最新闭环记录：UnitDestroy UnitType.killed runtime sidecar

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 子代理只读结论：参考基线里的 `UnitType.killed(Unit unit)` 是默认空实现，未发现 `MissileUnitType` / `NeoplasmUnitType` / `ErekirUnitType` 等子类重写；死亡主行为仍集中在 `UnitComp.destroy()`、`Ability.death()` 与 weapon `shootOnDeath`。
- 本轮目标：对照 Java `UnitComp.destroy()` 的 `type.killed(self())` 调用点，先在 Rust runtime 中记录类型级 killed sidecar，不伪造当前 Java 默认空实现之外的行为。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 新增 `GameRuntimeUnitTypeKilledEvent`；
    - `GameRuntime` 新增 `unit_type_killed_events`；
    - reset/clear 路径清空该队列；
    - 新增 `drain_unit_type_killed_events()`；
    - `queue_client_unit_destroy_side_effects(...)` 在 ability death sidecar 后记录 `unit_id/unit_type_name/team/x/y`；
    - core destroy 测试覆盖字段和 drain。
  - `desktop/src/lib.rs`
    - unit destroy desktop 测试验证 lifecycle 后 runtime 中保留 `unit_type_killed_events`。
  - `MIGRATION.md`
    - 新增 `12.285`。
- 已跑验证：
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_destroy_packet_to_legged_unit_effects`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_unit_destroy_packet_to_leg_destroy_effects`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-desktop`
  - `git diff --check`
- 当前仍需继续：
  1. 具体 `Ability.death(...)` 执行仍未接入客户端/统一死亡调度；
  2. 优先打通 `SpawnDeathAbility` / `LiquidExplodeAbility` 已有 plan 到真实 runtime，而不是继续只记录 descriptor；
  3. suicide trigger、wreckRegions decal、weapon bullet spawn 仍未迁移；
  4. `Damage.dynamicExplosion(...)` lightning/fire/wave damage、真实 audio/camera backend 仍需继续；
  5. 当前总迁移仍约 10%~11%，远未可玩。

---

## 212. 最新闭环记录：Server death ability lifecycle sidecar 接入

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 子代理只读结论：`SpawnDeathAbility` 是当前最短真实闭环，已经存在 server 死亡处理、子单位生成、`UnitSpawnCallPacket` 广播链路；`LiquidExplodeAbility` 也已有 server puddle runtime，但后续还要补 server→client puddle snapshot smoke。不要在 client 侧执行 ability death。
- 本轮目标：把已记录的 ability/type killed sidecar 接入服务器真实 death ability 路径，而不是只在客户端 `UnitDestroyCallPacket` 回放时记录。
- Rust 主改动：
  - `core/src/mindustry/core/game_runtime.rs`
    - 抽出 `note_unit_ability_death_events(&UnitComp)`；
    - 抽出 `note_unit_type_killed_event(&UnitComp)`；
    - client `queue_client_unit_destroy_side_effects(...)` 复用统一记录入口。
  - `server/src/lib.rs`
    - `apply_server_unit_death_abilities(...)` 移除 dead parent 后，先记录 ability death 与 type killed sidecar，再执行已有 `liquid_explode_ability_deposit_plans()` / `spawn_death_ability_plans()`；
    - 扩展 `server_update_spawns_renales_when_latum_dies`，断言 latum 的多 ability death 都被记录，并且 `SpawnDeathAbility:renale:5:11` 确实生成 5 个 renale；
    - 扩展 `server_update_deposits_neoplasm_when_renale_dies`，断言 `LiquidExplodeAbility:neoplasm` sidecar 与真实 `server_puddles` 写入同时成立。
  - `MIGRATION.md`
    - 新增 `12.286`。
- 已跑验证：
  - `cargo test -p mindustry-server server_update_spawns_renales_when_latum_dies --lib`
  - `cargo test -p mindustry-server server_update_deposits_neoplasm_when_renale_dies --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_destroy_packet_to_legged_unit_effects`
  - `cargo test -p mindustry-desktop desktop_launcher_syncs_multiple_unit_lifecycle_packets_in_one_update --lib`
  - `cargo test -p mindustry-core game_runtime_applies_client_unit_death_packet_like_java_killed --lib`
  - `cargo check -p mindustry-core`
  - `cargo check -p mindustry-server`
  - `git diff --check`
- 当前仍需继续：
  1. `SpawnDeathAbility` 仍需更完整 Java random/rotation parity、unit cap/visibility/event bus 细节；
  2. `LiquidExplodeAbility` 需要补 server→desktop/client puddle snapshot smoke；
  3. suicide trigger、wreckRegions decal、weapon bullet spawn 仍未迁移；
  4. `Damage.dynamicExplosion(...)` lightning/fire/wave damage、真实 audio/camera backend 仍需继续；
  5. 当前总迁移仍约 10%~11%，远未可玩。

---

## 213. 最新闭环记录：LiquidExplodeAbility death puddle server→client snapshot smoke

- 固定工作路径：Rust 仓库 `D:\MDT\rust-mindustry`；Java 参考 `D:\MDT\mindustry-upstream-v157.4`（当前 `v158.1 / 05b2ecd4eb`）；废案 `D:\MDT\mindustry-rust` 禁止使用；遇到文字乱码优先 UTF-8 再尝试读取。
- 本轮目标：把 `LiquidExplodeAbility.death(Unit)` 从“server puddle 已生成”推进到“server death deposit 生成的 puddle 能进入 entity snapshot 并被 client runtime materialize”，继续避免模块孤立。
- Rust 主改动：
  - `server/src/lib.rs`
    - 扩展 `server_update_deposits_neoplasm_when_renale_dies`；
    - 测试现在启用 `CaptureProvider` + active `NetServer`；
    - renale 死亡后从发送队列取最新非可靠 `EntitySnapshotCallPacket`；
    - 用客户端 `GameRuntime::apply_client_entity_snapshot_packet_with_content(...)` 解码；
    - 断言客户端 typed puddle 的 amount、tile 与 liquid properties 正确。
  - `MIGRATION.md`
    - 新增 `12.287`。
- 已跑验证：
  - `cargo test -p mindustry-server server_update_deposits_neoplasm_when_renale_dies --lib`
  - `cargo test -p mindustry-server server_update_spawns_renales_when_latum_dies --lib`
  - `cargo check -p mindustry-server`
  - `git diff --check`
- 当前仍需继续：
  1. 把这条 smoke 升级到 `mindustry-tests` 的真实 server↔desktop loop；
  2. 覆盖 puddle 扩散/删除后的连续 snapshot 和 client mirror 清理；
  3. 继续 `UnitComp.destroy()`：suicide trigger、wreckRegions decal、weapon bullet spawn；
  4. `Damage.dynamicExplosion(...)` lightning/fire/wave damage、真实 audio/camera backend 仍需继续；
  5. 当前总迁移仍约 10%~11%，远未可玩。
