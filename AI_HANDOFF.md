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

1. `5d70acc 接入客户端服务启动闭环`
2. `ea5b4bd 接入服务端网络事件轮询`
3. `ef67476 接入服务端真实网络启动器`
4. `355376b 保持服务端网络循环运行`
5. `643bef9 接入桌面客户端真实网络层`
6. `571d58d 支持桌面客户端参数连接`
7. `2d19084 保持桌面客户端网络循环运行`
8. `9c23edd 支持服务端端口参数`

最后确认时：

- 当前分支：`main`
- 最新提交：`9c23edd 支持服务端端口参数`
- `git status --short` 未显示已有未提交代码改动（创建本文档除外）。

---

## 5. 最近一次完成的具体实现

文件：

- `server/src/lib.rs`

完成内容：

1. `ServerLauncher::new()` 会读取端口参数并写入 `AppContext.port`。
2. 支持以下参数形式：
   - `--port 1234`
   - `--port=1234`
   - `-p 1234`
3. 新增/补齐：
   - `parse_port_arg(args: &[String]) -> Option<u16>`
   - 测试：`server_launcher_reads_port_arg_before_opening_network`

已验证：

```powershell
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' fmt
& 'C:/Users/yuyu/.cargo/bin/cargo.exe' test -p mindustry-server -- --test-threads=1
```

验证结果：

- `mindustry-server` 8 个测试全部通过。

已提交并推送：

```text
9c23edd 支持服务端端口参数
```

---

## 6. 当前迁移进度的真实判断

不要对用户虚报“快完成”。

当前 Rust 项目已经有一些网络启动、事件轮询、客户端参数连接、服务端端口参数等基础闭环，但距离“完整可游玩 Rust 版 Mindustry”仍很远。

粗略完成度建议口径：

- 若以“完整可游玩、联机互通、内容/世界/实体/UI/渲染完整”为 100%，当前约为 **5%～8%**。
- 若只以“项目结构与早期网络启动骨架”为目标，当前进度会更高，但这不是最终目标。

主要已具备：

1. Rust workspace 已存在并可运行部分 crate 测试。
2. 服务端有真实网络启动器与网络循环保持。
3. 桌面客户端有真实网络层与参数连接入口。
4. 服务端支持端口参数读取。
5. 部分 `NetClient` / `NetServer` / packet / service 生命周期已有骨架。

主要缺口：

1. 服务端尚未完整下发 world/map/save 数据。
2. 客户端尚未完整接收 world stream 并进入可渲染世界。
3. 内容系统、世界系统、Universe、GameState 生命周期未完整装配。
4. 实体 update/collision、单位、方块、队伍、规则、存档、地图加载仍大量缺失。
5. 桌面客户端仍缺少真正窗口、渲染、输入、UI 与游戏主循环体验。
6. 与 Java 原版服务端/客户端的协议级互通只完成了很早期的基础工作。

---

## 7. 下一步优先任务

下一步推荐继续推进：

### 7.1 服务端最小 world/map 数据下发闭环

目标：

1. 对照 Java 参考实现，定位客户端连接后服务端发送 world/map/save/world stream 的流程。
2. 在 Rust 侧实现最小可测试闭环：
   - 服务端在连接建立或握手完成后准备 world data。
   - 服务端发送 `StreamBegin` / `StreamChunk` 或当前 Rust 已定义的等价 packet。
   - 客户端能接收并记录 world data 状态。
   - 测试能验证发送顺序、payload 长度、客户端状态变化。
3. 先实现“测试可证明的数据流”，再逐步替换为真实 Java 兼容 payload。

优先查看 Rust 文件：

- `core/src/mindustry/core/net_server.rs`
- `core/src/mindustry/core/net_client.rs`
- `core/src/mindustry/core/world.rs`
- `core/src/mindustry/core/content_loader.rs`
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

4. 派发两个 `explorer` 做 world stream / world data 只读扫描。
5. 主线程同时查看 Rust 当前 `net_server` / `net_client` packet 定义。
6. 用测试驱动实现最小 world data 发送/接收闭环。
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
