# AI 交接文档：Mindustry Java → Rust 迁移

## 1. 最终目标（不得偏移）

把 Java 参考仓库：

- `D:/MDT/mindustry-upstream-v157.4`

逐文件、逐文件夹对照迁移/重写为 Rust 项目：

- `D:/MDT/rust-mindustry`

最终交付目标是一个尽可能接近原版 Mindustry v157.4 行为的 Rust 版 MDT/Mindustry：

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

最近已推送到 `main` 的提交包括：

1. `b51d8af 保留联机地图尾部原始数据`
2. `fc7798f 服务端发送运行时队伍计划`
3. `6f94257 接入联机队伍建造计划`
4. `1226e49 兼容载荷炮塔旧式弹药`
5. `a115351 锁定逻辑处理器旧版读取`
6. `bd17931 补齐载荷弹药炮塔状态读取`
7. `9cceec3 接入单位载荷精确读取`
8. `5d50757 支持无状态构造载荷读取`
9. `1ac3c6e 区分嵌套载荷读取模式`
10. `a309790 统计方块状态剩余字节`

本轮开始前最后确认时：

- 当前分支：`main`
- 最新提交：`b51d8af 保留联机地图尾部原始数据`
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
- 已验证：`cargo test -p mindustry-core linked_storage --lib` 通过 4/4；`cargo test -p mindustry-core directional_unloader --lib` 通过 5/5；`cargo test -p mindustry-core item_unloader --lib` 通过 3/3；`cargo test -p mindustry-core game_runtime_payload_unloader --lib` 通过 13/13；`cargo check -p mindustry-core` 通过（仅既有 unused warning）；`rustfmt --check core/src/mindustry/core/game_runtime.rs` 与 `git diff --check` 通过。
- 仍未完成：Java 共享 `ItemModule` 引用的完整等价、campaign `handleCoreItem(...)` 真实副作用、多核心间共享同一 items 模块、core placement/upgrade 全流程、完整 UI/renderer 行为。
