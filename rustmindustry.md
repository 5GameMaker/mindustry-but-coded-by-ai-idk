# rust-mindustry 逐文件还原账本

更新日期：2026-06-10

## 约束

- 工作目录：`D:/MDT/rust-mindustry`
- 参考目录：`D:/MDT/mindustry-upstream-v157.4`
- 不读取工作目录既有文档；本文件作为新的迁移/验证账本维护。
- 文件读取按 UTF-8 处理。
- 命令通过 Git Bash 执行，当前显式使用 `C:/Program Files/Git/bin/bash.exe`。
- 不做防御性编程；按参考项目运行语义逐步还原。

## 当前验证基线

- `cargo metadata --no-deps --format-version 1`：通过。
- `cargo check --workspace --all-targets`：通过。
- `git diff --check`：通过。
- `cargo test -p mindustry-core about_dialog -- --nocapture`：通过，`4 passed; 0 failed`。
- `cargo test -p mindustry-core campaign_rules_dialog -- --nocapture`：通过，`7 passed; 0 failed`。
- `cargo test -p mindustry-core color_picker -- --nocapture`：通过，`4 passed; 0 failed`。
- `cargo test -p mindustry-core content_info_dialog -- --nocapture`：通过，`4 passed; 0 failed`。
- `cargo test -p mindustry-core database_dialog -- --nocapture`：通过，`4 passed; 0 failed`。
- `cargo test -p mindustry-core discord_dialog -- --nocapture`：通过，`2 passed; 0 failed`。
- `cargo test -p mindustry-core editor_maps_dialog -- --nocapture`：通过，`8 passed; 0 failed`。
- `cargo test -p mindustry-core file_chooser -- --nocapture`：通过，`8 passed; 0 failed`。
- `cargo test -p mindustry-core loadout_dialog -- --nocapture`：通过，`19 passed; 0 failed`。
- `cargo test -p mindustry-core map_list_dialog -- --nocapture`：通过，`9 passed; 0 failed`。
- `cargo test -p mindustry-core palette_dialog -- --nocapture`：通过，`2 passed; 0 failed`。
- `cargo test -p mindustry-core research_dialog -- --nocapture`：通过，`9 passed; 0 failed`。
- `cargo test -p mindustry-core schematics_dialog -- --nocapture`：通过，`7 passed; 0 failed`。
- `cargo test -p mindustry-core settings_menu_dialog -- --nocapture`：通过，`8 passed; 0 failed`。
- `cargo test -p mindustry-core mods_dialog -- --nocapture`：通过，`8 passed; 0 failed`。
- `cargo test -p mindustry-core custom_rules_dialog -- --nocapture`：通过，`10 passed; 0 failed`。
- `cargo test -p mindustry-core cubemap_mesh -- --nocapture`：通过，`4 passed; 0 failed`。
- `cargo test -p mindustry-core planet_renderer -- --nocapture`：通过，`3 passed; 0 failed`。
- `cargo test -p mindustry-core planet_dialog -- --nocapture`：通过，`10 passed; 0 failed`。
- `cargo test -p mindustry-core g3d -- --nocapture`：通过，`32 passed; 0 failed`。
- `cargo test -p mindustry-core debug_collision -- --nocapture`：通过，`3 passed; 0 failed`。
- `cargo test -p mindustry-core intel_gpu -- --nocapture`：通过，`4 passed; 0 failed`。
- `cargo test -p mindustry-core nv_gpu -- --nocapture`：通过，`3 passed; 0 failed`。
- `cargo test -p mindustry-core ui -- --nocapture`：通过，`873 passed; 0 failed; 2 ignored`。
- `cargo test -p mindustry-desktop desktop_launcher_campaign_route_builds_planet_renderer_params_from_dialog_state -- --nocapture`：通过，`1 passed; 0 failed`。
- `cargo test -p mindustry-desktop desktop_launcher_campaign_route_cursor_move_updates_hovered_sector_like_java -- --nocapture`：通过，`1 passed; 0 failed`。
- `cargo test -p mindustry-desktop desktop_launcher_campaign_route_planet_surface_hover_uses_ray_picking_like_java -- --nocapture`：通过，`1 passed; 0 failed`。
- `cargo test -p mindustry-desktop desktop_launcher_campaign_route_planet_surface_hover_label_projects_to_surface_like_java -- --nocapture`：通过，`1 passed; 0 failed`。
- `cargo test -p mindustry-desktop desktop_launcher_campaign_route_planet_surface_hover_ -- --nocapture`：通过，`2 passed; 0 failed`。
- `cargo test -p mindustry-desktop desktop_launcher_campaign_route_ -- --nocapture`：通过，`18 passed; 0 failed`。
- Workspace crate：
  - `mindustry-core`
  - `mindustry-server`
  - `mindustry-desktop`
  - `mindustry-android`
  - `mindustry-ios`
  - `mindustry-annotations`
  - `mindustry-tools`
  - `mindustry-tests`

## 参考项目 UI/游玩流程优先链

1. `Vars` / `Platform` / `ClientLauncher`
2. `UI` / `Styles` / `Fonts`
3. `MenuFragment` / `LoadingFragment` / `HudFragment`
4. `Renderer` / `MenuRenderer` / `LoadRenderer` / `MinimapRenderer`
5. 其余 dialogs / fragments / editor / graphics / g3d

## 当前代码映射观察

- `core/src/mindustry/ui` 已有共享 UI 基础：
  - `bar.rs`
  - `border_image.rs`
  - `displayable.rs`
  - `fonts.rs`
  - `items_display.rs`
  - `links.rs`
  - `menus.rs`
  - `minimap.rs`
  - `mobile_button.rs`
  - `styles.rs`
  - `warning_bar.rs`
  - `dialogs/base_dialog.rs`
  - `dialogs/full_text_dialog.rs`
  - `dialogs/keybind_dialog.rs`
  - `dialogs/language_dialog.rs`
  - `dialogs/map_locales_dialog.rs`
- 大量桌面端菜单、对话框、渲染流程集中在 `desktop/src/lib.rs`。
- `core/src/mindustry/graphics` 已覆盖多数基础渲染模块，`g3d/PlanetRenderer` 已提供后端无关 `PlanetScenePlan`。
- `desktop/src/lib.rs::push_campaign_route_page` 已将 campaign planet card 的视觉核接入 `PlanetRendererParams -> PlanetScenePlan`，并以 `planet-renderer-scene-plan` / `planet-renderer-scene-step` 自定义渲染命令保留 trace；`push_campaign_planet_scene_preview` 已按 `PlanetSceneStep` 生成可见 `DrawPixel` / `DrawCircle` / `DrawPolygon` / `DrawLine` / `DrawSprite` 投影 primitive；`CursorMoved` 已同步当前桌面可命中的 sector selector / sector list hover 到 `CampaignPlanetDialogState.hovered_sector_id`。
- `desktop/src/lib.rs` 已补 PlanetDialog 地表 hover 首段：`campaign_route_planet_scene_plan` 的 `cam_pos` 跟随 `selected_sector_id` 对应的 `PlanetGrid` tile，新增 `campaign_planet_surface_sector_id_at_surface_point` 用预览相机基向量把 surface point 还原为球面 ray pick，并接入 `campaign_hovered_sector_id_at_surface_point`；新增 `desktop_launcher_campaign_route_planet_surface_hover_uses_ray_picking_like_java` 覆盖中心 ray 命中 selected sector 且 `CursorMoved` 只更新 hover。
- `desktop/src/lib.rs` 已补 PlanetDialog hover label 投影：`campaign_hovered_sector_projected_label_like_java` 复用 `campaign_planet_surface_sector_preview_point` 将 `hoverLabel` 放到 hovered sector 的球面投影点，替代旧的 sector card 固定位置；`campaign_sector_hover_label_like_java` 的 selectable 名称优先走 runtime `Sector::name(...)`，覆盖 numbered sector 与 preset localized name；新增 `desktop_launcher_campaign_route_planet_surface_hover_label_projects_to_surface_like_java` 锁定投影位置。
- 后续继续补真实 OpenGL 3D backend 执行、完整 numbered sector 选择/面板、projection 图标/弧线与 launch cutscene。

## UI/图形缺口清单

### UI 已逐文件补齐到 Rust 源码的类

- 顶层 `core/src/mindustry/ui`：
  - `Bar`
  - `BorderImage`（已补 `core/src/mindustry/ui/border_image.rs`）
  - `CoreItemsDisplay`
  - `Displayable`
  - `Fonts`
  - `GridImage`
  - `IntFormat`
  - `ItemsDisplay`（已补 `core/src/mindustry/ui/items_display.rs`）
  - `Links`（已补 `core/src/mindustry/ui/links.rs`）
  - `Menus`（已补 `core/src/mindustry/ui/menus.rs`）
  - `Minimap`（已补 `core/src/mindustry/ui/minimap.rs`）
  - `MobileButton`
  - `MultiReqImage`
  - `ReqImage`
  - `Styles`
  - `WarningBar`
- `fragments` 已补：
  - `BlockConfigFragment`
  - `BlockInventoryFragment`
  - `ChatFragment`（已补 `core/src/mindustry/ui/fragments/chat_fragment.rs`）
  - `ConsoleFragment`（已补 `core/src/mindustry/ui/fragments/console_fragment.rs`）
  - `FadeInFragment`
  - `HintsFragment`（已补 `core/src/mindustry/ui/fragments/hints_fragment.rs`）
  - `HudFragment`（已补 `core/src/mindustry/ui/fragments/hud_fragment.rs`）
  - `LoadingFragment`（已补 `core/src/mindustry/ui/fragments/loading_fragment.rs`）
  - `MenuFragment`（已补 `core/src/mindustry/ui/fragments/menu_fragment.rs`）
  - `MinimapFragment`
  - `PlacementFragment`（已补 `core/src/mindustry/ui/fragments/placement_fragment.rs`）
  - `PlanConfigFragment`
  - `PlayerListFragment`
  - `layout/BranchTreeLayout`
  - `layout/RadialTreeLayout`
  - `layout/RowTreeLayout`
  - `layout/TreeLayout`
- `dialogs` 已补：
  - `AdminsDialog`
  - `AboutDialog`（已补 `core/src/mindustry/ui/dialogs/about_dialog.rs`）
  - `BansDialog`
  - `CampaignCompleteDialog`
  - `CampaignRulesDialog`（已补 `core/src/mindustry/ui/dialogs/campaign_rules_dialog.rs`）
  - `CanvasEditDialog`（已补 `core/src/mindustry/ui/dialogs/canvas_edit_dialog.rs`）
  - `ColorPicker`（已补 `core/src/mindustry/ui/dialogs/color_picker.rs`）
  - `ContentInfoDialog`（已补 `core/src/mindustry/ui/dialogs/content_info_dialog.rs`）
  - `CustomRulesDialog`（已补 `core/src/mindustry/ui/dialogs/custom_rules_dialog.rs`）
  - `CustomGameDialog`（已补 `core/src/mindustry/ui/dialogs/custom_game_dialog.rs`）
  - `DatabaseDialog`（已补 `core/src/mindustry/ui/dialogs/database_dialog.rs`）
  - `DiscordDialog`（已补 `core/src/mindustry/ui/dialogs/discord_dialog.rs`）
  - `EditorMapsDialog`（已补 `core/src/mindustry/ui/dialogs/editor_maps_dialog.rs`）
  - `EffectsDialog`（已补 `core/src/mindustry/ui/dialogs/effects_dialog.rs`）
  - `FileChooser`（已补 `core/src/mindustry/ui/dialogs/file_chooser.rs`）
  - `GameOverDialog`（已补 `core/src/mindustry/ui/dialogs/game_over_dialog.rs`）
  - `HostDialog`（已补 `core/src/mindustry/ui/dialogs/host_dialog.rs`）
  - `IconSelectDialog`
  - `JoinDialog`（已补 `core/src/mindustry/ui/dialogs/join_dialog.rs`）
  - `LaunchLoadoutDialog`（已补 `core/src/mindustry/ui/dialogs/launch_loadout_dialog.rs`）
  - `LoadDialog`（已补 `core/src/mindustry/ui/dialogs/load_dialog.rs`）
  - `LoadoutDialog`（已补 `core/src/mindustry/ui/dialogs/loadout_dialog.rs`）
  - `MapListDialog`（已补 `core/src/mindustry/ui/dialogs/map_list_dialog.rs`）
  - `MapPlayDialog`（已补 `core/src/mindustry/ui/dialogs/map_play_dialog.rs`）
  - `ModsDialog`（已补 `core/src/mindustry/ui/dialogs/mods_dialog.rs`）
  - `PaletteDialog`（已补 `core/src/mindustry/ui/dialogs/palette_dialog.rs`）
  - `PausedDialog`（已补 `core/src/mindustry/ui/dialogs/paused_dialog.rs`）
  - `PlanetDialog`（已补首批核心纯模型 `core/src/mindustry/ui/dialogs/planet_dialog.rs`）
  - `ResearchDialog`（已补 `core/src/mindustry/ui/dialogs/research_dialog.rs`）
  - `SaveDialog`（已补 `core/src/mindustry/ui/dialogs/save_dialog.rs`）
  - `SchematicsDialog`（已补 `core/src/mindustry/ui/dialogs/schematics_dialog.rs`）
  - `SectorSelectDialog`（已补 `core/src/mindustry/ui/dialogs/sector_select_dialog.rs`）
  - `SettingsMenuDialog`（已补 `core/src/mindustry/ui/dialogs/settings_menu_dialog.rs`）
  - `TraceDialog`

### UI 类名仍未在 Rust 源码中找到明确实现痕迹

- 暂无明确 dialogs 类名缺口；后续转入 graphics/g3d 与更细粒度运行时/UI 行为对齐。

### dialogs 还原优先级

- P0 主流程类名已补完；后续继续深化 `PlanetDialog` 3D 渲染/载荷发射细节，并进入 P1 高频支撑对话框。
- P1 高频支撑已补完。
- P2 工具/信息型已补完：`CustomRulesDialog`。

### Graphics 类名未在 Rust 源码中找到明确实现痕迹

- 暂无当前清单内明确 graphics/g3d 类名缺口；后续转入更细粒度行为/渲染后端接入复核。

### Graphics/g3d 本轮已补齐到 Rust 源码的类

- `CubemapMesh`：已补 `core/src/mindustry/graphics/cubemap_mesh.rs`，覆盖上游固定 skybox cube 顶点、linear filter、`u_cubemap` slot 0、`u_proj`/triangles render plan 与 dispose 语义。
- `DebugCollisionRenderer`：已补 `core/src/mindustry/graphics/debug_collision_renderer.rs`，覆盖 hitbox square、solid tile boundary line、avoidance square、ground unit tile rect、unit physics circle 与 reset 分支。
- `IntelGpuCheck`：已补 `core/src/mindustry/graphics/intel_gpu_check.rs`，覆盖 Windows Intel vendor marker 写入/删除、非 Windows no-op 与 last launch cache 语义。
- `NvGpuInfo`：已补 `core/src/mindustry/graphics/nv_gpu_info.rs`，覆盖 `GL_NVX_gpu_memory_info` extension 一次性检测、total/current memory pname 查询与 unsupported 返回 0。
- `g3d/HexMesher`：已补 `core/src/mindustry/graphics/g3d/hex_mesher.rs`，覆盖默认 height/color/emissive/skip 语义、固定颜色 mesher、Vec3/Color 与噪声入口。
- `g3d/PlanetGrid`：已补 `core/src/mindustry/graphics/g3d/planet_grid.rs`，覆盖上游 tile/corner/edge 数量公式、初始 12 五边形与 subdivision 连接结构。
- `g3d/MeshBuilder`：已补 `core/src/mindustry/graphics/g3d/mesh_builder.rs`，覆盖 icosphere 计数、planet grid line mesh、hex indexed/non-indexed fan、height cache、color/emissive、skip 与 normal pack 位布局。
- `g3d/HexMesh`：已补 `core/src/mindustry/graphics/g3d/hex_mesh.rs`，覆盖默认 planet shader 构造、custom mesher/shader 构造与 planet shader preRender plan。
- `g3d/HexSkyMesh`：已补 `core/src/mindustry/graphics/g3d/hex_sky_mesh.rs`，覆盖 cloud mesher height/color/skip、`relRot=globalTime*speed/40`、`uiAlpha≈1` skip、cloud alpha 与 transform/preRender plan。
- `g3d/MatMesh`：已补 `core/src/mindustry/graphics/g3d/mat_mesh.rs`，覆盖 `transform * local mat` 包裹渲染与 dispose 转发。
- `g3d/MultiMesh`：已补 `core/src/mindustry/graphics/g3d/multi_mesh.rs`，覆盖子 mesh 顺序 render/dispose fan-out。
- `g3d/NoiseMesh`：已补 `core/src/mindustry/graphics/g3d/noise_mesh.rs`，覆盖单色/双色噪声 mesh、`7+seed` height、`8+seed` color、`5f` 坐标偏移与 `intensity=0.2`。
- `g3d/PlanetRenderer`：已补最小场景壳 `core/src/mindustry/graphics/g3d/planet_renderer.rs`，覆盖上游 `fov=60`、`far=150`、`projector scaling=1/150`、skybox/bloom/depth/cull/planet/clouds/sectors/atmosphere/orbit/interface projection 的数据化阶段顺序；`desktop/src/lib.rs::push_campaign_route_page` 已开始消费该 plan，并用 `PlanetSceneStep` 驱动可见 preview primitives；`CursorMoved` 已能通过 `PlanetGrid` surface ray picking 更新 PlanetDialog hover，hover label 已随 hovered sector 投到 planet preview 表面；后续仍需接入完整扇区选择/launch cutscene 与 OpenGL 实绘。
- `g3d/SunMesh`：已补 `core/src/mindustry/graphics/g3d/sun_mesh.rs`，覆盖 zero height、simplex/pow/mag 离散 palette clamp 与 `Shaders.unlit`。

## 下一步

1. 继续补齐当前 UI 明确缺口：
   - dialogs 剩余 0 个明确类名文件；`desktop/src/lib.rs::push_campaign_route_page` 已接入 `g3d/PlanetRenderer` 场景壳；不要在 desktop 重写 `PlanetDialog` 状态机。
   - 已补 planet surface hover 的首段 ray picking 与 hover label 投影；后续继续补完整 numbered sector 选择/面板、sector 展开/选区实绘、launch cutscene。
   - 高频 UI 行为复核顺序：`HudFragment` → `ConsoleFragment` → `PlayerListFragment`。
2. 继续复核 graphics/g3d 行为深度：
   - `simplex_noise3d` 当前是本地确定性入口，仍需后续与 Arc `Simplex.noise3d` 做数值级对照。
   - `PlanetRenderer` 场景壳已完成，桌面 campaign route 已消费 `PlanetScenePlan` 并生成可见 preview primitives，且已补 surface hover ray picking 与 hover label 投影；仍需 OpenGL backend 将 scene step 真实落成 3D draw。
3. 对 `desktop/src/lib.rs` 中已有的菜单/HUD/对话框集中实现继续拆分映射，避免重复实现但保留逐文件 Rust 对应。
4. 跑桌面端最小启动/渲染路径验证，并继续保持 `cargo check --workspace --all-targets` 通过。
