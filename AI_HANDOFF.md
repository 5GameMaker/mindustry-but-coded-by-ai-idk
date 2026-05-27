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
