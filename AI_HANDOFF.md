# AI 交接文档：Mindustry Java → Rust 迁移

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
