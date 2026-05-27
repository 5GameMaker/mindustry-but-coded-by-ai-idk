#[test]
fn workspace_smoke() {
    assert_eq!(
        mindustry_core::mindustry::UPSTREAM_BASELINE,
        "mindustry-upstream-v158"
    );
}

#[cfg(test)]
fn free_local_port() -> u16 {
    use std::net::{TcpListener, UdpSocket};

    for _ in 0..128 {
        let tcp = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = tcp.local_addr().unwrap().port();
        if UdpSocket::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    panic!("could not reserve a local TCP/UDP port pair");
}

#[cfg(test)]
fn pump_real_server_desktop_until(
    server: &mut mindustry_server::ServerLauncher,
    desktop: &mut mindustry_desktop::DesktopLauncher,
    ready: impl Fn(&mindustry_desktop::DesktopLauncher) -> bool,
) {
    use std::thread;
    use std::time::Duration;

    let mut loaded = false;
    let mut materialized = false;
    let mut server_confirmed = false;
    let mut last_client_status = String::new();
    let mut last_server_status = String::new();
    for _ in 0..80 {
        desktop.update();
        server.update();
        desktop.update();
        server.update();

        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        loaded = state.last_world_data_error.is_none()
            && state.last_loaded_world_data.is_some()
            && state.connect_confirm_sent;
        last_client_status = format!(
            "attempts={} connect_events={} connect_packet_sent={} connected={} loading={} world_stream_events={} confirm={} world_data={} error={:?} provider_events={:?}",
            state.connection_attempts,
            state.connect_events,
            state.connect_packet_sent,
            state.connected,
            state.world_data_loading,
            state.world_stream_events,
            state.connect_confirm_sent,
            state.last_loaded_world_data.is_some(),
            state.last_world_data_error,
            state.last_provider_events,
        );
        drop(state);
        {
            let state = server.net_server.state();
            let state = state.lock().unwrap();
            server_confirmed = state.last_connect_confirm_connection_id.is_some();
            last_server_status = format!(
                "accepted={} rejected={} pending={:?} streams={} last_world_conn={:?} confirm_conn={:?} network_error={:?}",
                state.connect_packets_accepted,
                state.connect_packets_rejected,
                state.pending_world_data_connections,
                state.world_streams_sent,
                state.last_world_data_connection_id,
                state.last_connect_confirm_connection_id,
                server.network_error,
            );
        }

        materialized = ready(desktop);
        if loaded && materialized && server_confirmed {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        loaded && materialized && server_confirmed,
        "desktop should load, confirm, server should receive confirm, and materialize real world stream; loaded={loaded} materialized={materialized} server_confirmed={server_confirmed}; client: {last_client_status}; server: {last_server_status}; connect_error: {:?}",
        desktop.connect_error
    );
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert!(state.connected);
        assert!(!state.world_data_loading);
        assert_eq!(state.world_stream_events, 1);
        assert!(state.last_world_data_error.is_none());
        assert!(state.last_loaded_world_data.is_some());
        assert!(state.connect_confirm_sent);
    }
    {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.connect_packets_accepted, 1);
        assert!(state.pending_world_data_connections.is_empty());
        assert_eq!(state.world_streams_sent, 1);
        assert!(state.last_world_data_connection_id.is_some());
        assert!(state.last_connect_confirm_connection_id.is_some());
    }
}

#[test]
fn real_server_desktop_preview_snapshot_forwarding_updates_remote_player_cache_after_world_stream()
{
    use mindustry_core::mindustry::io::{BuildPlanWire, Vec2};
    use mindustry_core::mindustry::net::{
        ClientPlanSnapshotCallPacket, ClientPlanSnapshotReceivedCallPacket, NetConnection,
        PacketKind,
    };
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    server.runtime.state.world.resize(12, 12);
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context
            == mindustry_core::mindustry::core::GameRuntimeNetworkContext::client()
    });

    let (source_connection_id, target_connection_id, target_team) = {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        let target_connection_id = state
            .last_connect_confirm_connection_id
            .expect("target desktop should confirm before preview smoke");
        let target_team = state
            .connection_states
            .get(&target_connection_id)
            .expect("target connection should be tracked")
            .team;
        (
            target_connection_id + 10_000,
            target_connection_id,
            target_team,
        )
    };
    {
        let state = server.net_server.state();
        let mut state = state.lock().unwrap();
        let mut source_connection = NetConnection::new("simulated-preview-source");
        source_connection.name = "preview-source".into();
        source_connection.team = target_team;
        source_connection.has_connected = true;
        source_connection.player_added = true;
        state
            .connection_states
            .insert(source_connection_id, source_connection);
    }
    {
        let mut net = server.net_server.net_mut();
        net.handle_server_received_from_connection(
            Some(source_connection_id),
            true,
            PacketKind::ClientPlanSnapshotCallPacket(ClientPlanSnapshotCallPacket {
                group_id: 12,
                plans: Some(vec![BuildPlanWire::new_place(4, 5, 1, "router")]),
            }),
        );
    }
    assert_eq!(server.apply_new_network_server_events(), 1);
    server
        .net_server
        .send_client_plan_snapshot_received(
            target_connection_id,
            ClientPlanSnapshotReceivedCallPacket {
                player_id: source_connection_id,
                group_id: 12,
                plans: Some(vec![BuildPlanWire::new_place(4, 5, 1, "router")]),
            },
        )
        .expect("real server should send forwarded preview packet to target desktop");

    let mut delivered = false;
    let mut last_client_status = String::new();
    for _ in 0..120 {
        server.update();
        desktop.update();
        server.update();
        desktop.update();

        {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            delivered = state
                .client_plan_snapshot_received_packets
                .iter()
                .any(|packet| {
                    packet.player_id == source_connection_id
                        && packet.plans.as_ref().is_some_and(|plans| plans.len() == 1)
                });
            last_client_status = format!(
                "received={} last={:?} events={:?}",
                state.client_plan_snapshot_received_packets.len(),
                state.last_client_plan_snapshot_received,
                state.last_provider_events,
            );
        }

        if delivered && desktop.remote_players.contains_key(&source_connection_id) {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        delivered,
        "target desktop should receive forwarded preview snapshot; {last_client_status}"
    );
    assert!(desktop.remote_players.contains_key(&source_connection_id));
    let remote = desktop.remote_players.get(&source_connection_id).unwrap();
    assert_eq!(remote.preview_plans_assembling.len(), 1);
    assert_eq!(remote.team, desktop.player.team);

    let overlay_count = desktop.rebuild_other_player_preview_overlays_at(
        i64::MAX / 4,
        1.0,
        Some(Vec2::new(32.0, 40.0)),
    );
    assert_eq!(overlay_count, 1);
    let overlay = &desktop.other_player_preview_overlays[0];
    assert_eq!(overlay.player_id, source_connection_id);
    assert_eq!(overlay.entries.len(), 1);
    assert_eq!(overlay.entries[0].block, "router");
    assert_eq!(overlay.entries[0].world_pos, Vec2::new(32.0, 40.0));

    {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.client_plan_snapshot_packets_seen, 1);
        assert_eq!(state.client_plan_snapshots_forwarded, 1);
        assert_eq!(
            state.last_client_plan_snapshot_forwarded_connection_id,
            Some(target_connection_id)
        );
    }
    assert_eq!(server.server_preview_plan_packets_applied, 1);
    assert!(server
        .server_preview_players
        .contains_key(&source_connection_id));

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_world_stream_materializes_payload_sidecar() {
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimePayloadBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::world::blocks::payloads::{
        PayloadBlockBuildState, PayloadLoaderState, PayloadRef,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    let loader_tile = point2_pack(4, 4);
    let loader_def = server
        .content_loader
        .block_by_name("payload-loader")
        .expect("base content should include payload-loader");
    let container_def = server
        .content_loader
        .block_by_name("container")
        .expect("base content should include container");
    let container_id = container_def.base().id;

    let mut payload_bytes = Vec::new();
    BuildingComp::new(point2_pack(0, 0), container_def.base().clone(), TeamId(6))
        .write_base(&mut payload_bytes, false)
        .unwrap();
    let mut loader_building = BuildingComp::new(loader_tile, loader_def.base().clone(), TeamId(6));
    loader_building.set_rotation(2);

    server.runtime.state.world.resize(12, 12);
    server.runtime.add_building(loader_building);
    server.runtime.payload_runtime_states.insert(
        loader_tile,
        GameRuntimePayloadBlockState::Loader {
            common: PayloadBlockBuildState {
                payload: Some(PayloadRef::Block {
                    block: container_id,
                    version: 0,
                    build_bytes: payload_bytes,
                }),
                ..PayloadBlockBuildState::default()
            },
            loader: PayloadLoaderState {
                exporting: true,
                ..PayloadLoaderState::default()
            },
        },
    );
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);

    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop
            .runtime
            .payload_runtime_states
            .contains_key(&loader_tile)
    });

    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );
    let Some(GameRuntimePayloadBlockState::Loader { common, loader }) =
        desktop.runtime.payload_runtime_states.get(&loader_tile)
    else {
        panic!("payload loader sidecar should be materialized from real world stream");
    };
    assert!(loader.exporting);
    assert!(matches!(
        common.payload.as_ref(),
        Some(PayloadRef::Block { block, .. }) if *block == container_id
    ));

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_unit_cargo_loader_tether_spawn_syncs_to_client_runtime() {
    use mindustry_core::mindustry::content::blocks::BlockDef;
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimeDistributionBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::world::blocks::units::UnitCargoLoaderState;
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    let loader_tile = point2_pack(4, 4);
    let power_source_tile = point2_pack(6, 4);
    let loader_def = server
        .content_loader
        .block_by_name("unit-cargo-loader")
        .expect("base content should include unit-cargo-loader");
    let power_source_def = server
        .content_loader
        .block_by_name("power-source")
        .expect("base content should include power-source");
    let BlockDef::Distribution(loader_block) = loader_def else {
        panic!("unit-cargo-loader should be a distribution block");
    };
    let loader_base = loader_def.base().clone();
    let unit_build_time = loader_block.unit_build_time;
    let nitrogen = server
        .content_loader
        .liquid_by_name("nitrogen")
        .unwrap()
        .base
        .mappable
        .base
        .id;

    server.runtime.state.world.resize(12, 12);
    server
        .runtime
        .add_building(BuildingComp::new(loader_tile, loader_base, TeamId(6)));
    server.runtime.add_building(BuildingComp::new(
        power_source_tile,
        power_source_def.base().clone(),
        TeamId(6),
    ));
    if let Some(power) = server.runtime.buildings[0].power.as_mut() {
        power.status = 1.0;
    }
    if let Some(liquids) = server.runtime.buildings[0].liquids.as_mut() {
        liquids.set(nitrogen, 20.0);
    }
    server.runtime.distribution_runtime_states.insert(
        loader_tile,
        GameRuntimeDistributionBlockState::UnitCargoLoader(UnitCargoLoaderState::default()),
    );
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);

    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
            && matches!(
                desktop
                    .runtime
                    .distribution_runtime_states
                    .get(&loader_tile),
                Some(GameRuntimeDistributionBlockState::UnitCargoLoader(_))
            )
    });

    server
        .runtime
        .state
        .set(mindustry_core::mindustry::core::GameStateState::Playing);
    if let Some(GameRuntimeDistributionBlockState::UnitCargoLoader(state)) = server
        .runtime
        .distribution_runtime_states
        .get_mut(&loader_tile)
    {
        state.build_progress = 1.0 - 1.0 / unit_build_time;
        state.has_unit = false;
        state.read_unit_id = -1;
    }

    let mut client_read_unit_id = -1;
    let mut last_client_status = String::new();
    for _ in 0..80 {
        server.update();
        desktop.update();
        desktop.update();

        if let Some(GameRuntimeDistributionBlockState::UnitCargoLoader(state)) = desktop
            .runtime
            .distribution_runtime_states
            .get(&loader_tile)
        {
            client_read_unit_id = state.read_unit_id;
        }
        {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            last_client_status = format!(
                "unit_tether_seen={} last={:?} applied={}",
                state.unit_tether_block_spawned_packets_seen,
                state.last_unit_tether_block_spawned,
                desktop
                    .runtime
                    .client_unit_tether_block_spawned_packets_applied,
            );
        }
        if client_read_unit_id >= 0 {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        client_read_unit_id >= 0,
        "desktop should apply real UnitTetherBlockSpawnedCallPacket to unit cargo loader; client: {last_client_status}"
    );
    let Some(GameRuntimeDistributionBlockState::UnitCargoLoader(server_state)) =
        server.runtime.distribution_runtime_states.get(&loader_tile)
    else {
        panic!("server unit cargo loader state should remain present");
    };
    assert_eq!(client_read_unit_id, server_state.read_unit_id);
    assert!(server.server_units.contains_key(&server_state.read_unit_id));
    let desktop_unit = desktop
        .runtime
        .client_unit_snapshot_entities
        .get(&server_state.read_unit_id)
        .expect("desktop should materialize cargo unit snapshot from tether packet");
    assert_eq!(desktop_unit.type_info.name(), "manifold");
    assert_eq!(desktop_unit.team_id(), TeamId(6));
    assert!(desktop_unit.controller.is_cargo());
    assert_eq!(
        desktop_unit
            .cargo_ai
            .as_ref()
            .and_then(|cargo| cargo.tether_tile_pos),
        Some(loader_tile)
    );
    assert_eq!(
        server
            .last_runtime_item_transport_report
            .as_ref()
            .map(|report| report.unit_cargo_loader_built_units),
        Some(1)
    );
    assert_eq!(
        desktop
            .runtime
            .client_unit_tether_block_spawned_packets_applied,
        1
    );
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.unit_tether_block_spawned_packets_seen, 1);
        let packet = state
            .last_unit_tether_block_spawned
            .as_ref()
            .expect("desktop should record real unit tether spawned packet");
        assert_eq!(packet.tile, Some(loader_tile));
        assert_eq!(packet.id, server_state.read_unit_id);
    }

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_unit_cargo_transfer_syncs_item_mirrors_to_client_runtime() {
    use mindustry_core::mindustry::content::blocks::BlockDef;
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimeDistributionBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::world::blocks::units::{
        UnitCargoLoaderState, UnitCargoUnloadPointState,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    let loader_tile = point2_pack(4, 4);
    let unload_tile = point2_pack(9, 4);
    let power_source_tile = point2_pack(6, 4);
    let loader_def = server
        .content_loader
        .block_by_name("unit-cargo-loader")
        .expect("base content should include unit-cargo-loader");
    let unload_def = server
        .content_loader
        .block_by_name("unit-cargo-unload-point")
        .expect("base content should include unit-cargo-unload-point");
    let power_source_def = server
        .content_loader
        .block_by_name("power-source")
        .expect("base content should include power-source");
    let BlockDef::Distribution(loader_block) = loader_def else {
        panic!("unit-cargo-loader should be a distribution block");
    };
    let unit_build_time = loader_block.unit_build_time;
    let copper = server
        .content_loader
        .item_by_name("copper")
        .unwrap()
        .base
        .mappable
        .base
        .id;
    let nitrogen = server
        .content_loader
        .liquid_by_name("nitrogen")
        .unwrap()
        .base
        .mappable
        .base
        .id;

    server.runtime.state.world.resize(16, 12);
    let mut loader = BuildingComp::new(loader_tile, loader_def.base().clone(), TeamId(6));
    loader.items.as_mut().unwrap().add(copper, 12);
    if let Some(power) = loader.power.as_mut() {
        power.status = 1.0;
    }
    if let Some(liquids) = loader.liquids.as_mut() {
        liquids.set(nitrogen, 20.0);
    }
    server.runtime.add_building(loader);
    server.runtime.add_building(BuildingComp::new(
        unload_tile,
        unload_def.base().clone(),
        TeamId(6),
    ));
    server.runtime.add_building(BuildingComp::new(
        power_source_tile,
        power_source_def.base().clone(),
        TeamId(6),
    ));
    server.runtime.distribution_runtime_states.insert(
        loader_tile,
        GameRuntimeDistributionBlockState::UnitCargoLoader(UnitCargoLoaderState {
            build_progress: 1.0 - 1.0 / unit_build_time,
            ..UnitCargoLoaderState::default()
        }),
    );
    server.runtime.distribution_runtime_states.insert(
        unload_tile,
        GameRuntimeDistributionBlockState::UnitCargoUnload(UnitCargoUnloadPointState {
            item_id: Some(copper as i32),
            stale_timer: 0.0,
            stale: false,
        }),
    );
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
            && matches!(
                desktop
                    .runtime
                    .distribution_runtime_states
                    .get(&unload_tile),
                Some(GameRuntimeDistributionBlockState::UnitCargoUnload(_))
            )
    });

    server
        .runtime
        .state
        .set(mindustry_core::mindustry::core::GameStateState::Playing);

    let mut last_status = String::new();
    for _ in 0..120 {
        server.update();
        desktop.update();
        desktop.update();

        let server_unload_amount = server
            .runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == unload_tile)
            .and_then(|building| building.items.as_ref())
            .map(|items| items.get(copper))
            .unwrap_or(0);
        let desktop_unload_amount = desktop
            .runtime
            .buildings()
            .iter()
            .find(|building| building.tile_pos == unload_tile)
            .and_then(|building| building.items.as_ref())
            .map(|items| items.get(copper))
            .unwrap_or(0);
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        last_status = format!(
            "take_seen={} transfer_seen={} server_unload={} desktop_unload={} unit_mirrors={:?}",
            state.take_items_packets_seen,
            state.transfer_item_to_packets_seen,
            server_unload_amount,
            desktop_unload_amount,
            state.unit_item_mirrors
        );
        if state.take_items_packets_seen > 0
            && state.transfer_item_to_packets_seen > 0
            && server_unload_amount == 12
            && desktop_unload_amount == 12
        {
            break;
        }
        drop(state);
        thread::sleep(Duration::from_millis(20));
    }

    let Some(GameRuntimeDistributionBlockState::UnitCargoLoader(server_loader_state)) =
        server.runtime.distribution_runtime_states.get(&loader_tile)
    else {
        panic!("server loader state should remain present");
    };
    let desktop_unit = desktop
        .runtime
        .client_unit_snapshot_entities
        .get(&server_loader_state.read_unit_id)
        .expect("desktop should keep materialized cargo unit");
    assert_eq!(
        desktop_unit.items.stack.amount, 0,
        "cargo unit should have unloaded all mirrored copper; {last_status}"
    );
    let server_unload_building = server
        .runtime
        .buildings()
        .iter()
        .find(|building| building.tile_pos == unload_tile)
        .expect("server unload point should exist");
    assert_eq!(
        (desktop_unit.x(), desktop_unit.y()),
        (server_unload_building.x, server_unload_building.y),
        "desktop cargo unit should apply server entity snapshot position; {last_status}"
    );
    let desktop_unload_amount = desktop
        .runtime
        .buildings()
        .iter()
        .find(|building| building.tile_pos == unload_tile)
        .and_then(|building| building.items.as_ref())
        .map(|items| items.get(copper))
        .unwrap_or(0);
    assert_eq!(
        desktop_unload_amount, 12,
        "desktop unload point should mirror server cargo deposit; {last_status}"
    );

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_world_stream_materializes_multiple_payload_sidecars() {
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimePayloadBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::ctype::ContentType;
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::world::blocks::payloads::{
        PayloadBlockBuildState, PayloadConveyorState, PayloadDeconstructorState,
        PayloadDriverState, PayloadMassDriverState, PayloadRef, PayloadSortKey,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);

    let payload_router_def = server
        .content_loader
        .block_by_name("payload-router")
        .expect("base content should include payload-router");
    let mass_driver_def = server
        .content_loader
        .block_by_name("payload-mass-driver")
        .expect("base content should include payload-mass-driver");
    let deconstructor_def = server
        .content_loader
        .block_by_name("small-deconstructor")
        .expect("base content should include small-deconstructor");
    let router_def = server.content_loader.block_by_name("router").unwrap();
    let router_id = router_def.base().id;
    let router_tile = point2_pack(4, 4);
    let mass_driver_tile = point2_pack(9, 4);
    let deconstructor_tile = point2_pack(14, 4);
    let mut payload_bytes = Vec::new();
    BuildingComp::new(point2_pack(0, 0), router_def.base().clone(), TeamId(6))
        .write_base(&mut payload_bytes, false)
        .unwrap();
    let router_payload = PayloadRef::Block {
        block: router_id,
        version: 0,
        build_bytes: payload_bytes,
    };
    let sorted_router = PayloadSortKey {
        content_type: ContentType::Block.ordinal() as i8,
        id: router_id,
    };

    server.runtime.state.world.resize(24, 10);
    server.runtime.add_building(BuildingComp::new(
        router_tile,
        payload_router_def.base().clone(),
        TeamId(6),
    ));
    server.runtime.add_building(BuildingComp::new(
        mass_driver_tile,
        mass_driver_def.base().clone(),
        TeamId(6),
    ));
    server.runtime.add_building(BuildingComp::new(
        deconstructor_tile,
        deconstructor_def.base().clone(),
        TeamId(6),
    ));
    server.runtime.payload_runtime_states.insert(
        router_tile,
        GameRuntimePayloadBlockState::Router {
            conveyor: PayloadConveyorState {
                item: Some(router_payload.clone()),
                step: 1,
                step_accepted: 0,
                item_rotation: 45.0,
                ..PayloadConveyorState::default()
            },
            sorted: Some(sorted_router),
            rec_dir: 2,
            matches: true,
            smooth_rot: 180.0,
            control_time: -1.0,
        },
    );
    server.runtime.payload_runtime_states.insert(
        mass_driver_tile,
        GameRuntimePayloadBlockState::MassDriver {
            common: PayloadBlockBuildState::default(),
            driver: PayloadMassDriverState {
                link: -1,
                turret_rotation: 45.0,
                state: PayloadDriverState::Shooting,
                reload_counter: 0.25,
                charge: 0.5,
                loaded: true,
                charging: true,
                ..PayloadMassDriverState::default()
            },
        },
    );
    server.runtime.payload_runtime_states.insert(
        deconstructor_tile,
        GameRuntimePayloadBlockState::Deconstructor {
            common: PayloadBlockBuildState::default(),
            deconstructor: PayloadDeconstructorState {
                progress: 0.5,
                has_deconstructing: true,
                deconstructing: Some(router_payload),
                accum: Some(vec![1.0, 2.0]),
                ..PayloadDeconstructorState::default()
            },
        },
    );
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);

    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop
            .runtime
            .payload_runtime_states
            .contains_key(&router_tile)
            && desktop
                .runtime
                .payload_runtime_states
                .contains_key(&mass_driver_tile)
            && desktop
                .runtime
                .payload_runtime_states
                .contains_key(&deconstructor_tile)
    });

    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );
    let Some(GameRuntimePayloadBlockState::Router {
        conveyor,
        sorted,
        rec_dir,
        matches,
        ..
    }) = desktop.runtime.payload_runtime_states.get(&router_tile)
    else {
        panic!("payload router sidecar should materialize through real world stream");
    };
    assert!(matches);
    assert!(matches!(
        conveyor.item.as_ref(),
        Some(PayloadRef::Block { block, .. }) if *block == router_id
    ));
    assert_eq!(*sorted, Some(sorted_router));
    assert_eq!(*rec_dir, 2);

    let Some(GameRuntimePayloadBlockState::MassDriver { driver, .. }) = desktop
        .runtime
        .payload_runtime_states
        .get(&mass_driver_tile)
    else {
        panic!("payload mass driver sidecar should materialize through real world stream");
    };
    assert_eq!(driver.turret_rotation, 45.0);
    assert_eq!(driver.state, PayloadDriverState::Shooting);
    assert_eq!(driver.reload_counter, 0.25);
    assert_eq!(driver.charge, 0.5);
    assert!(driver.loaded);
    assert!(driver.charging);

    let Some(GameRuntimePayloadBlockState::Deconstructor { deconstructor, .. }) = desktop
        .runtime
        .payload_runtime_states
        .get(&deconstructor_tile)
    else {
        panic!("payload deconstructor sidecar should materialize through real world stream");
    };
    assert_eq!(deconstructor.progress, 0.5);
    assert_eq!(deconstructor.accum.as_deref(), Some(&[1.0, 2.0][..]));
    assert!(matches!(
        deconstructor.deconstructing.as_ref(),
        Some(PayloadRef::Block { block, .. }) if *block == router_id
    ));

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_state_snapshot_updates_runtime_after_world_stream() {
    use mindustry_core::mindustry::core::GameRuntimeNetworkContext;
    use mindustry_core::mindustry::net::StateSnapshotCallPacket;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    server.runtime.state.world.resize(8, 8);
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
    });

    let connection_id = {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        state
            .last_connect_confirm_connection_id
            .expect("server should receive connect confirm before state snapshot")
    };
    let snapshot = StateSnapshotCallPacket {
        wave_time: 33.5,
        wave: 12,
        enemies: 4,
        paused: true,
        game_over: true,
        time_data: 789,
        tps: 58,
        rand0: 1234,
        rand1: 5678,
        core_data: Vec::new(),
    };

    server
        .net_server
        .send_state_snapshot(connection_id, snapshot.clone())
        .expect("real server should send state snapshot");

    let mut applied = false;
    let mut last_client_status = String::new();
    for _ in 0..80 {
        desktop.update();
        server.update();
        {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            applied = state.last_state_snapshot.as_ref() == Some(&snapshot);
            last_client_status = format!(
                "state_snapshots={} last_snapshot={} last_server_snapshot={:?} provider_events={:?}",
                state.state_snapshot_packets_seen,
                state.last_state_snapshot.is_some(),
                state.last_server_snapshot_at,
                state.last_provider_events,
            );
        }
        if applied && desktop.game_state.wave == snapshot.wave {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        applied,
        "desktop should receive real state snapshot after world stream; client: {last_client_status}"
    );
    {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.last_state_snapshot_connection_id, Some(connection_id));
        assert_eq!(state.last_state_snapshot.as_ref(), Some(&snapshot));
        assert_eq!(state.state_snapshot_packets_sent, 1);
        assert!(state.last_state_snapshot_error.is_none());
    }
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.state_snapshot_packets_seen, 1);
        assert_eq!(state.last_state_snapshot.as_ref(), Some(&snapshot));
        assert!(state.last_state_snapshot_mirror.is_some());
    }
    assert_eq!(desktop.game_state.wavetime, snapshot.wave_time);
    assert_eq!(desktop.game_state.wave, snapshot.wave);
    assert_eq!(desktop.game_state.enemies, snapshot.enemies);
    assert_eq!(desktop.game_state.game_over, snapshot.game_over);
    assert_eq!(desktop.game_state.server_tps, snapshot.tps as i32);
    assert_eq!(desktop.runtime.state.server_tps, snapshot.tps as i32);
    assert_eq!(desktop.game_state.rand_seed0, snapshot.rand0);
    assert_eq!(desktop.game_state.rand_seed1, snapshot.rand1);
    assert_eq!(
        desktop.game_state.universe.seconds(true),
        snapshot.time_data
    );
    assert!(desktop.game_state.is_paused());
    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_entity_sync_snapshot_updates_net_client_after_world_stream() {
    use mindustry_core::mindustry::core::GameRuntimeNetworkContext;
    use mindustry_core::mindustry::ctype::Content;
    use mindustry_core::mindustry::entities::{
        BULLET_CLASS_ID, DECAL_CLASS_ID, EFFECT_STATE_CLASS_ID, FIRE_CLASS_ID, PLAYER_CLASS_ID,
        PUDDLE_CLASS_ID, WEATHER_STATE_CLASS_ID, WORLD_LABEL_CLASS_ID,
    };
    use mindustry_core::mindustry::io::{type_io, TeamId, UnitRef, Vec2 as IoVec2};
    use mindustry_core::mindustry::net::{
        EntitySnapshotCallPacket, HiddenSnapshotCallPacket, NetworkPlayerSyncData,
        StateSnapshotCallPacket,
    };
    use mindustry_core::mindustry::r#type::ItemStack;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    server.runtime.state.world.resize(8, 8);
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
    });

    let connection_id = {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        state
            .last_connect_confirm_connection_id
            .expect("server should receive connect confirm before entity sync")
    };
    let state_snapshot = StateSnapshotCallPacket {
        wave_time: 44.0,
        wave: 21,
        enemies: 8,
        paused: false,
        game_over: false,
        time_data: 901,
        tps: 60,
        rand0: 101,
        rand1: 202,
        core_data: Vec::new(),
    };
    let dagger_id = server
        .content_loader
        .unit_by_name("dagger")
        .expect("base content should include dagger")
        .base
        .mappable
        .base
        .id;
    let flare_id = server
        .content_loader
        .unit_by_name("flare")
        .expect("base content should include flare")
        .base
        .mappable
        .base
        .id;
    let unit_sync = type_io::UnitSyncWire {
        abilities: Vec::new(),
        ammo: 7.0,
        controller: type_io::ControllerWire::Ground,
        elevation: 0.25,
        flag: 42.5,
        health: 88.0,
        is_shooting: true,
        mine_tile: None,
        mounts: Vec::new(),
        plans: None,
        rotation: 270.0,
        shield: 9.0,
        spawned_by_core: true,
        stack: ItemStack::new("copper", 4),
        statuses: Vec::new(),
        team: TeamId(3),
        type_id: dagger_id,
        update_building: false,
        vel: IoVec2 { x: 1.5, y: -2.0 },
        x: 48.0,
        y: 56.0,
    };
    let mut first_entity_sync_bytes = Vec::new();
    type_io::write_unit_sync(
        &mut first_entity_sync_bytes,
        &server.content_loader,
        &unit_sync,
    )
    .unwrap();
    let mut first_entity_data = Vec::new();
    first_entity_data.extend_from_slice(&1001i32.to_be_bytes());
    first_entity_data.push(2);
    first_entity_data.extend_from_slice(&first_entity_sync_bytes);
    let first_entity = EntitySnapshotCallPacket {
        amount: 1,
        data: first_entity_data,
    };
    let mut second_entity_data = Vec::new();
    second_entity_data.extend_from_slice(&1002i32.to_be_bytes());
    second_entity_data.push(3);
    second_entity_data.extend_from_slice(&1003i32.to_be_bytes());
    second_entity_data.push(4);
    let second_entity = EntitySnapshotCallPacket {
        amount: 2,
        data: second_entity_data,
    };
    let multi_first_sync = type_io::UnitSyncWire {
        abilities: Vec::new(),
        ammo: 3.0,
        controller: type_io::ControllerWire::Ground,
        elevation: 0.5,
        flag: 11.0,
        health: 55.0,
        is_shooting: false,
        mine_tile: None,
        mounts: Vec::new(),
        plans: None,
        rotation: 45.0,
        shield: 1.0,
        spawned_by_core: false,
        stack: ItemStack::new("", 0),
        statuses: Vec::new(),
        team: TeamId(4),
        type_id: dagger_id,
        update_building: false,
        vel: IoVec2 { x: 0.5, y: 0.25 },
        x: 64.0,
        y: 72.0,
    };
    let multi_second_sync = type_io::UnitSyncWire {
        abilities: Vec::new(),
        ammo: 5.0,
        controller: type_io::ControllerWire::Ground,
        elevation: 0.75,
        flag: 22.0,
        health: 66.0,
        is_shooting: true,
        mine_tile: None,
        mounts: Vec::new(),
        plans: None,
        rotation: 135.0,
        shield: 2.0,
        spawned_by_core: true,
        stack: ItemStack::new("", 0),
        statuses: Vec::new(),
        team: TeamId(5),
        type_id: flare_id,
        update_building: false,
        vel: IoVec2 { x: -0.5, y: 1.25 },
        x: 80.0,
        y: 88.0,
    };
    let player_sync = NetworkPlayerSyncData {
        admin: true,
        boosting: true,
        color: 0x33_44_55_66,
        mouse_x: 700.0,
        mouse_y: 701.0,
        name: Some("real-snapshot-player".into()),
        selected_block_id: None,
        selected_rotation: 1,
        shooting: true,
        team: TeamId(6),
        typing: true,
        unit: UnitRef::Unit { id: 1004 },
        x: 123.0,
        y: 456.0,
    };
    let mut player_bytes = Vec::new();
    player_sync.write_to(&mut player_bytes).unwrap();
    let effect_sync = type_io::EffectStateSyncWire {
        color: type_io::RgbaColor::new(0x336699cc),
        data: type_io::TypeValue::String("spark".into()),
        effect_id: 7,
        lifetime: 50.0,
        offset_pos: 1.25,
        offset_rot: -2.5,
        offset_x: 3.0,
        offset_y: 4.0,
        parent_id: Some(1234),
        rot_with_parent: true,
        rotation: 90.0,
        time: 12.0,
        x: 100.0,
        y: 200.0,
    };
    let mut effect_bytes = Vec::new();
    type_io::write_effect_state_sync(&mut effect_bytes, &effect_sync).unwrap();
    let bullet_sync = type_io::BulletSyncWire {
        collided: vec![7, 9],
        damage: 33.0,
        data: type_io::TypeValue::String("spark-bullet".into()),
        fdata: 2.5,
        lifetime: 120.0,
        owner: type_io::EntityRef::new(1004),
        rotation: 180.0,
        team: TeamId(6),
        time: 10.0,
        bullet_type_id: 1,
        vel: IoVec2 { x: -0.25, y: 1.5 },
        x: 20.0,
        y: 40.0,
    };
    let mut bullet_bytes = Vec::new();
    type_io::write_bullet_sync(&mut bullet_bytes, &bullet_sync).unwrap();
    let decal_sync = type_io::DecalSyncWire {
        color: type_io::RgbaColor::new(0x11223344),
        lifetime: 30.0,
        rotation: 15.0,
        time: 2.0,
        x: 12.0,
        y: 24.0,
    };
    let mut decal_bytes = Vec::new();
    type_io::write_decal_sync(&mut decal_bytes, &decal_sync).unwrap();
    let fire_sync = type_io::FireSyncWire {
        lifetime: 150.0,
        tile_pos: Some(mindustry_core::mindustry::world::point2_pack(2, 3)),
        time: 45.0,
        x: 16.0,
        y: 24.0,
    };
    let mut fire_bytes = Vec::new();
    type_io::write_fire_sync(&mut fire_bytes, &fire_sync).unwrap();
    let oil_id = server
        .content_loader
        .liquid_by_name("oil")
        .expect("base content should include oil")
        .base
        .mappable
        .base
        .id;
    let puddle_sync = type_io::PuddleSyncWire {
        amount: 36.5,
        liquid_id: Some(oil_id),
        tile_pos: Some(mindustry_core::mindustry::world::point2_pack(4, 5)),
        x: 32.0,
        y: 40.0,
    };
    let mut puddle_bytes = Vec::new();
    type_io::write_puddle_sync(&mut puddle_bytes, &puddle_sync).unwrap();
    let rain_id = server
        .content_loader
        .weather_by_name("rain")
        .expect("base content should include rain")
        .id();
    let weather_sync = type_io::WeatherStateSyncWire {
        effect_timer: 12.0,
        intensity: 0.75,
        life: 600.0,
        opacity: 0.5,
        weather_id: Some(rain_id),
        wind_vector: IoVec2 { x: -0.25, y: 0.75 },
        x: 10.0,
        y: 20.0,
    };
    let mut weather_bytes = Vec::new();
    type_io::write_weather_state_sync(&mut weather_bytes, &weather_sync).unwrap();
    let label_sync = type_io::WorldLabelSyncWire {
        flags: 1 | 8,
        font_size: 1.5,
        parent_id: Some(1004),
        text: Some("rally".into()),
        x: 72.0,
        y: 96.0,
        z: 155.0,
    };
    let mut label_bytes = Vec::new();
    type_io::write_world_label_sync(&mut label_bytes, &label_sync).unwrap();
    let mut multi_first_bytes = Vec::new();
    type_io::write_unit_sync(
        &mut multi_first_bytes,
        &server.content_loader,
        &multi_first_sync,
    )
    .unwrap();
    let mut multi_second_bytes = Vec::new();
    type_io::write_unit_sync(
        &mut multi_second_bytes,
        &server.content_loader,
        &multi_second_sync,
    )
    .unwrap();
    let mut multi_entity_data = Vec::new();
    multi_entity_data.extend_from_slice(&connection_id.to_be_bytes());
    multi_entity_data.push(PLAYER_CLASS_ID);
    multi_entity_data.extend_from_slice(&player_bytes);
    multi_entity_data.extend_from_slice(&1009i32.to_be_bytes());
    multi_entity_data.push(EFFECT_STATE_CLASS_ID);
    multi_entity_data.extend_from_slice(&effect_bytes);
    multi_entity_data.extend_from_slice(&1011i32.to_be_bytes());
    multi_entity_data.push(BULLET_CLASS_ID);
    multi_entity_data.extend_from_slice(&bullet_bytes);
    multi_entity_data.extend_from_slice(&1010i32.to_be_bytes());
    multi_entity_data.push(DECAL_CLASS_ID);
    multi_entity_data.extend_from_slice(&decal_bytes);
    multi_entity_data.extend_from_slice(&1004i32.to_be_bytes());
    multi_entity_data.push(2);
    multi_entity_data.extend_from_slice(&multi_first_bytes);
    multi_entity_data.extend_from_slice(&1005i32.to_be_bytes());
    multi_entity_data.push(2);
    multi_entity_data.extend_from_slice(&multi_second_bytes);
    multi_entity_data.extend_from_slice(&1006i32.to_be_bytes());
    multi_entity_data.push(FIRE_CLASS_ID);
    multi_entity_data.extend_from_slice(&fire_bytes);
    multi_entity_data.extend_from_slice(&1007i32.to_be_bytes());
    multi_entity_data.push(PUDDLE_CLASS_ID);
    multi_entity_data.extend_from_slice(&puddle_bytes);
    multi_entity_data.extend_from_slice(&1008i32.to_be_bytes());
    multi_entity_data.push(WEATHER_STATE_CLASS_ID);
    multi_entity_data.extend_from_slice(&weather_bytes);
    multi_entity_data.extend_from_slice(&1012i32.to_be_bytes());
    multi_entity_data.push(WORLD_LABEL_CLASS_ID);
    multi_entity_data.extend_from_slice(&label_bytes);
    let multi_entity = EntitySnapshotCallPacket {
        amount: 10,
        data: multi_entity_data,
    };
    let hidden = HiddenSnapshotCallPacket { ids: vec![4, 5] };

    server
        .net_server
        .send_entity_sync_snapshot(
            connection_id,
            state_snapshot.clone(),
            vec![
                first_entity.clone(),
                second_entity.clone(),
                multi_entity.clone(),
            ],
            Some(hidden.clone()),
        )
        .expect("real server should send entity sync snapshot");

    let mut applied = false;
    let mut last_client_status = String::new();
    for _ in 0..80 {
        desktop.update();
        server.update();
        {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            applied = state.last_state_snapshot.as_ref() == Some(&state_snapshot)
                && state.entity_snapshot_packets_seen == 3
                && state.last_entity_snapshot.as_ref() == Some(&multi_entity)
                && state.last_hidden_snapshot.as_ref() == Some(&hidden);
            last_client_status = format!(
                "state={} entity={} hidden={} last_entity={:?} last_hidden={:?} provider_events={:?}",
                state.state_snapshot_packets_seen,
                state.entity_snapshot_packets_seen,
                state.hidden_snapshot_packets_seen,
                state.last_entity_snapshot,
                state.last_hidden_snapshot,
                state.last_provider_events,
            );
        }
        if applied
            && desktop.game_state.wave == state_snapshot.wave
            && desktop
                .runtime
                .client_unit_snapshot_entities
                .contains_key(&1005)
            && desktop
                .runtime
                .client_player_snapshot_entities
                .contains_key(&connection_id)
            && desktop
                .runtime
                .client_effect_snapshot_entities
                .contains_key(&1009)
            && desktop
                .runtime
                .client_bullet_snapshot_entities
                .contains_key(&1011)
            && desktop
                .runtime
                .client_decal_snapshot_entities
                .contains_key(&1010)
            && desktop
                .runtime
                .client_fire_snapshot_entities
                .contains_key(&1006)
            && desktop
                .runtime
                .client_puddle_snapshot_entities
                .contains_key(&1007)
            && desktop
                .runtime
                .client_weather_snapshot_entities
                .contains_key(&1008)
            && desktop
                .runtime
                .client_world_label_snapshot_entities
                .contains_key(&1012)
        {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        applied,
        "desktop should receive real entity sync snapshot after world stream; client: {last_client_status}"
    );
    {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.last_state_snapshot_connection_id, Some(connection_id));
        assert_eq!(state.last_state_snapshot.as_ref(), Some(&state_snapshot));
        assert_eq!(state.state_snapshot_packets_sent, 1);
        assert_eq!(
            state.last_entity_snapshot_connection_id,
            Some(connection_id)
        );
        assert_eq!(state.last_entity_snapshot.as_ref(), Some(&multi_entity));
        assert_eq!(state.entity_snapshot_packets_sent, 3);
        assert_eq!(
            state.last_hidden_snapshot_connection_id,
            Some(connection_id)
        );
        assert_eq!(state.last_hidden_snapshot.as_ref(), Some(&hidden));
        assert_eq!(state.hidden_snapshot_packets_sent, 1);
        assert!(state.last_entity_snapshot_error.is_none());
        assert!(state.last_hidden_snapshot_error.is_none());
    }
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.state_snapshot_packets_seen, 1);
        assert_eq!(state.last_state_snapshot.as_ref(), Some(&state_snapshot));
        assert_eq!(state.entity_snapshot_packets_seen, 3);
        assert_eq!(state.last_entity_snapshot.as_ref(), Some(&multi_entity));
        assert_eq!(state.entity_snapshot_mirrors.len(), 3);
        assert_eq!(state.entity_snapshot_mirrors[0].records.len(), 1);
        assert_eq!(state.entity_snapshot_mirrors[0].records[0].entity_id, 1001);
        assert_eq!(state.entity_snapshot_mirrors[0].records[0].type_id, 2);
        assert_eq!(state.entity_snapshot_mirrors[1].records.len(), 2);
        assert_eq!(state.entity_snapshot_mirrors[1].records[0].entity_id, 1002);
        assert_eq!(state.entity_snapshot_mirrors[1].records[0].type_id, 3);
        assert_eq!(state.entity_snapshot_mirrors[1].records[1].entity_id, 1003);
        assert_eq!(state.entity_snapshot_mirrors[1].records[1].type_id, 4);
        assert!(state.entity_snapshot_mirrors[2].records.is_empty());
        assert!(state.entity_snapshot_mirrors[2].parse_error.is_some());
        assert_eq!(state.hidden_snapshot_packets_seen, 1);
        assert_eq!(state.last_hidden_snapshot.as_ref(), Some(&hidden));
        assert_eq!(
            state
                .last_hidden_snapshot_mirror
                .as_ref()
                .map(|mirror| mirror.ids.as_slice()),
            Some(&[4, 5][..])
        );
        assert!(state.last_server_snapshot_at.is_some());
    }
    assert_eq!(desktop.game_state.wave, state_snapshot.wave);
    assert_eq!(desktop.game_state.server_tps, state_snapshot.tps as i32);
    assert_eq!(desktop.runtime.state.server_tps, state_snapshot.tps as i32);
    assert_eq!(
        desktop
            .runtime
            .client_entity_snapshot_records
            .get(&1001)
            .map(|record| record.type_id),
        Some(2)
    );
    let typed_unit = desktop
        .runtime
        .client_unit_snapshot_entities
        .get(&1001)
        .expect("real entity snapshot should materialize typed unit runtime");
    assert_eq!(typed_unit.id(), 1001);
    assert_eq!(typed_unit.type_info.base.mappable.base.id, dagger_id);
    assert_eq!(typed_unit.team_id(), TeamId(3));
    assert_eq!(typed_unit.x(), 48.0);
    assert_eq!(typed_unit.y(), 56.0);
    assert_eq!(typed_unit.rotation(), 270.0);
    assert_eq!(typed_unit.health.health, 88.0);
    assert_eq!(typed_unit.weapons.ammo, 7.0);
    assert!(typed_unit.entity.is_added());
    assert_eq!(typed_unit.sync.hooks.read_sync_calls, 1);
    assert_eq!(typed_unit.sync.hooks.snap_sync_calls, 1);
    let multi_first = desktop
        .runtime
        .client_unit_snapshot_entities
        .get(&1004)
        .expect("multi-record entity snapshot fallback should materialize first unit");
    assert_eq!(multi_first.type_info.base.mappable.base.id, dagger_id);
    assert_eq!(multi_first.team_id(), TeamId(4));
    assert_eq!(multi_first.x(), 64.0);
    assert_eq!(multi_first.y(), 72.0);
    assert_eq!(multi_first.rotation(), 45.0);
    let multi_second = desktop
        .runtime
        .client_unit_snapshot_entities
        .get(&1005)
        .expect("multi-record entity snapshot fallback should materialize second unit");
    assert_eq!(multi_second.type_info.base.mappable.base.id, flare_id);
    assert_eq!(multi_second.team_id(), TeamId(5));
    assert_eq!(multi_second.x(), 80.0);
    assert_eq!(multi_second.y(), 88.0);
    assert_eq!(multi_second.rotation(), 135.0);
    assert!(multi_second.weapons.is_shooting);
    assert_eq!(
        desktop
            .runtime
            .client_player_snapshot_entities
            .get(&connection_id),
        Some(&player_sync)
    );
    assert_eq!(desktop.player.name, "real-snapshot-player");
    assert!(desktop.player.admin);
    assert_eq!(desktop.player.color, 0x33_44_55_66);
    assert_eq!(desktop.player.team, TeamId(6));
    assert_eq!(desktop.player.unit_ref(), Some(UnitRef::Unit { id: 1004 }));
    assert_eq!(
        desktop
            .runtime
            .client_entity_snapshot_records
            .get(&connection_id)
            .map(|record| (record.type_id, record.sync_bytes.as_slice())),
        Some((PLAYER_CLASS_ID, player_bytes.as_slice()))
    );
    assert_eq!(
        desktop
            .runtime
            .client_entity_snapshot_records
            .get(&1009)
            .map(|record| (record.type_id, record.sync_bytes.as_slice())),
        Some((EFFECT_STATE_CLASS_ID, effect_bytes.as_slice()))
    );
    assert_eq!(
        desktop
            .runtime
            .client_entity_snapshot_records
            .get(&1006)
            .map(|record| (record.type_id, record.sync_bytes.as_slice())),
        Some((FIRE_CLASS_ID, fire_bytes.as_slice()))
    );
    let effect = desktop
        .runtime
        .client_effect_snapshot_entities
        .get(&1009)
        .expect("real mixed entity snapshot should materialize typed effect runtime");
    assert_eq!(effect.effect_id, Some(7));
    assert_eq!(effect.data, type_io::TypeValue::String("spark".into()));
    assert_eq!(effect.lifetime, 50.0);
    assert_eq!(effect.parent_id, Some(1234));
    assert!(effect.rot_with_parent);
    assert_eq!(effect.rotation, 90.0);
    assert_eq!(effect.time, 12.0);
    assert_eq!(effect.x, 100.0);
    assert_eq!(effect.y, 200.0);
    assert_eq!(
        desktop
            .runtime
            .client_entity_snapshot_records
            .get(&1011)
            .map(|record| (record.type_id, record.sync_bytes.as_slice())),
        Some((BULLET_CLASS_ID, bullet_bytes.as_slice()))
    );
    let bullet = desktop
        .runtime
        .client_bullet_snapshot_entities
        .get(&1011)
        .expect("real mixed entity snapshot should materialize typed bullet runtime");
    assert_eq!(bullet.bullet_type_id, 1);
    assert_eq!(bullet.team, TeamId(6));
    assert_eq!(bullet.owner, type_io::EntityRef::new(1004));
    assert_eq!(bullet.collided_ids, vec![7, 9]);
    assert_eq!(bullet.damage, 33.0);
    assert_eq!(
        bullet.data,
        type_io::TypeValue::String("spark-bullet".into())
    );
    assert_eq!(bullet.fdata, 2.5);
    assert_eq!(bullet.lifetime, 120.0);
    assert_eq!(bullet.rotation, 180.0);
    assert_eq!(bullet.time, 10.0);
    assert_eq!(bullet.velocity, IoVec2 { x: -0.25, y: 1.5 });
    assert_eq!((bullet.x, bullet.y), (20.0, 40.0));
    assert_eq!(
        desktop
            .runtime
            .client_entity_snapshot_records
            .get(&1010)
            .map(|record| (record.type_id, record.sync_bytes.as_slice())),
        Some((DECAL_CLASS_ID, decal_bytes.as_slice()))
    );
    let decal = desktop
        .runtime
        .client_decal_snapshot_entities
        .get(&1010)
        .expect("real mixed entity snapshot should materialize typed decal runtime");
    assert_eq!(decal.lifetime, 30.0);
    assert_eq!(decal.rotation, 15.0);
    assert_eq!(decal.time, 2.0);
    assert_eq!(decal.x, 12.0);
    assert_eq!(decal.y, 24.0);
    assert_eq!(
        desktop
            .runtime
            .client_entity_snapshot_records
            .get(&1007)
            .map(|record| (record.type_id, record.sync_bytes.as_slice())),
        Some((PUDDLE_CLASS_ID, puddle_bytes.as_slice()))
    );
    assert_eq!(
        desktop
            .runtime
            .client_entity_snapshot_records
            .get(&1008)
            .map(|record| (record.type_id, record.sync_bytes.as_slice())),
        Some((WEATHER_STATE_CLASS_ID, weather_bytes.as_slice()))
    );
    let fire = desktop
        .runtime
        .client_fire_snapshot_entities
        .get(&1006)
        .expect("real mixed entity snapshot should materialize typed fire runtime");
    assert_eq!(fire.lifetime, 150.0);
    assert_eq!(fire.time, 45.0);
    assert_eq!(fire.x, 16.0);
    assert_eq!(fire.y, 24.0);
    assert_eq!(fire.tile.unwrap().x, 2);
    assert_eq!(fire.tile.unwrap().y, 3);
    assert!(fire.registered);
    let puddle = desktop
        .runtime
        .client_puddle_snapshot_entities
        .get(&1007)
        .expect("real mixed entity snapshot should materialize typed puddle runtime");
    assert_eq!(puddle.amount, 36.5);
    assert_eq!(puddle.x, 32.0);
    assert_eq!(puddle.y, 40.0);
    assert_eq!(puddle.tile.unwrap().x, 4);
    assert_eq!(puddle.tile.unwrap().y, 5);
    assert_eq!(puddle.liquid.unwrap().flammability, 1.2);
    assert!(puddle.registered);
    let weather = desktop
        .runtime
        .client_weather_snapshot_entities
        .get(&1008)
        .expect("real mixed entity snapshot should materialize typed weather runtime");
    assert_eq!(weather.weather_name, "rain");
    assert_eq!(weather.effect_timer, 12.0);
    assert_eq!(weather.intensity, 0.75);
    assert_eq!(weather.life, 600.0);
    assert_eq!(weather.opacity, 0.5);
    assert_eq!(weather.wind_vector, (-0.25, 0.75));
    assert_eq!(weather.x, 10.0);
    assert_eq!(weather.y, 20.0);
    assert!(weather.added);
    assert_eq!(
        desktop
            .runtime
            .client_entity_snapshot_records
            .get(&1012)
            .map(|record| (record.type_id, record.sync_bytes.as_slice())),
        Some((WORLD_LABEL_CLASS_ID, label_bytes.as_slice()))
    );
    let label = desktop
        .runtime
        .client_world_label_snapshot_entities
        .get(&1012)
        .expect("real mixed entity snapshot should materialize typed world-label runtime");
    assert_eq!(label.flags, 1 | 8);
    assert_eq!(label.font_size, 1.5);
    assert_eq!(label.parent_id, Some(1004));
    assert_eq!(label.text, "rally");
    assert_eq!((label.x, label.y, label.z), (72.0, 96.0, 155.0));
    assert_eq!(
        desktop
            .runtime
            .client_entity_snapshot_records
            .get(&1003)
            .map(|record| record.type_id),
        Some(4)
    );
    assert!(desktop.runtime.client_hidden_entity_ids.contains(&4));
    assert!(desktop.runtime.client_hidden_entity_ids.contains(&5));
    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_block_snapshot_updates_net_client_after_world_stream() {
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimeDistributionBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::net::BlockSnapshotCallPacket;
    use mindustry_core::mindustry::world::blocks::distribution::{
        write_conveyor_state, ConveyorItemState, ConveyorState,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    server.runtime.state.world.resize(8, 8);
    let conveyor_base = server
        .content_loader
        .block_by_name("conveyor")
        .expect("base content should include conveyor")
        .base()
        .clone();
    let conveyor_tile = point2_pack(2, 2);
    let conveyor_id = conveyor_base.id;
    let copper_id = server
        .content_loader
        .item_by_name("copper")
        .expect("base content should include copper")
        .base
        .mappable
        .base
        .id;
    server.runtime.add_building(BuildingComp::new(
        conveyor_tile,
        conveyor_base.clone(),
        TeamId(6),
    ));
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
    });

    let connection_id = {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        state
            .last_connect_confirm_connection_id
            .expect("server should receive connect confirm before block snapshot")
    };
    let mut synced_conveyor = BuildingComp::new(conveyor_tile, conveyor_base, TeamId(6));
    synced_conveyor.health = 31.0;
    synced_conveyor.set_rotation(3);
    let conveyor_state = ConveyorState {
        items: vec![ConveyorItemState {
            item: copper_id,
            x: 0.25,
            y: 0.5,
        }],
    };
    let mut block_sync_bytes = Vec::new();
    synced_conveyor
        .write_base(&mut block_sync_bytes, false)
        .unwrap();
    write_conveyor_state(&mut block_sync_bytes, &conveyor_state).unwrap();
    let snapshot = BlockSnapshotCallPacket {
        amount: 1,
        data: {
            let mut data = Vec::new();
            data.extend_from_slice(&conveyor_tile.to_be_bytes());
            data.extend_from_slice(&conveyor_id.to_be_bytes());
            data.extend_from_slice(&block_sync_bytes);
            data
        },
    };
    server
        .net_server
        .send_block_snapshot(connection_id, snapshot.clone())
        .expect("real server should send block snapshot");

    let mut applied = false;
    let mut last_client_status = String::new();
    for _ in 0..80 {
        desktop.update();
        server.update();
        {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            applied = state.last_block_snapshot.as_ref() == Some(&snapshot);
            last_client_status = format!(
                "block_snapshots={} last_block={:?} last_server_snapshot={:?} provider_events={:?}",
                state.block_snapshot_packets_seen,
                state.last_block_snapshot,
                state.last_server_snapshot_at,
                state.last_provider_events,
            );
        }
        if applied {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        applied,
        "desktop should receive real block snapshot after world stream; client: {last_client_status}"
    );
    {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        assert_eq!(state.last_block_snapshot_connection_id, Some(connection_id));
        assert_eq!(state.last_block_snapshot.as_ref(), Some(&snapshot));
        assert_eq!(state.block_snapshot_packets_sent, 1);
        assert!(state.last_block_snapshot_sent_at.is_some());
        assert!(state.last_block_snapshot_error.is_none());
    }
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.block_snapshot_packets_seen, 1);
        assert_eq!(state.last_block_snapshot.as_ref(), Some(&snapshot));
        assert!(state.last_block_snapshot_at.is_some());
        let mirror = state
            .last_block_snapshot_mirror
            .as_ref()
            .expect("block snapshot should materialize into lightweight mirror");
        assert_eq!(mirror.records.len(), 1);
        assert_eq!(mirror.records[0].tile_pos, conveyor_tile);
        assert_eq!(mirror.records[0].block_id, conveyor_id);
        assert_eq!(mirror.records[0].sync_bytes, block_sync_bytes);
        assert!(mirror.parse_error.is_none());
        assert!(state.last_server_snapshot_at.is_some());
    }
    let runtime_record = desktop
        .runtime
        .client_block_snapshot_records
        .get(&conveyor_tile)
        .expect("real block snapshot should apply to client runtime sidecar");
    assert_eq!(runtime_record.block_id, conveyor_id);
    assert_eq!(runtime_record.sync_bytes, block_sync_bytes);
    let runtime_building = desktop
        .runtime
        .buildings()
        .iter()
        .find(|building| building.tile_pos == conveyor_tile)
        .expect("conveyor building should remain materialized");
    assert_eq!(runtime_building.health, 31.0);
    assert_eq!(runtime_building.rotation, 3);
    let Some(GameRuntimeDistributionBlockState::Conveyor(applied_conveyor)) = desktop
        .runtime
        .distribution_runtime_states
        .get(&conveyor_tile)
    else {
        panic!("real block snapshot should apply conveyor child state to runtime");
    };
    assert_eq!(applied_conveyor.items.len(), 1);
    assert_eq!(applied_conveyor.items[0].item, copper_id);
    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_payload_block_snapshot_updates_runtime_after_world_stream() {
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimePayloadBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::ctype::ContentType;
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::net::BlockSnapshotCallPacket;
    use mindustry_core::mindustry::world::blocks::payloads::{
        write_payload_conveyor_extra, write_payload_router_extra, PayloadConveyorState,
        PayloadSortKey,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    server.runtime.state.world.resize(10, 10);
    let payload_router_base = server
        .content_loader
        .block_by_name("payload-router")
        .expect("base content should include payload-router")
        .base()
        .clone();
    let router_id = server
        .content_loader
        .block_by_name("router")
        .expect("base content should include router")
        .base()
        .id;
    let payload_router_tile = point2_pack(3, 3);
    let payload_router_id = payload_router_base.id;
    server.runtime.add_building(BuildingComp::new(
        payload_router_tile,
        payload_router_base.clone(),
        TeamId(6),
    ));
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
            && desktop
                .runtime
                .buildings()
                .iter()
                .any(|building| building.tile_pos == payload_router_tile)
    });

    let connection_id = {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        state
            .last_connect_confirm_connection_id
            .expect("server should receive connect confirm before payload block snapshot")
    };
    let mut synced_router = BuildingComp::new(payload_router_tile, payload_router_base, TeamId(6));
    synced_router.health = 23.0;
    synced_router.set_rotation(2);
    let conveyor_state = PayloadConveyorState {
        item: None,
        item_rotation: 90.0,
        ..PayloadConveyorState::default()
    };
    let sorted_router = Some(PayloadSortKey {
        content_type: ContentType::Block.ordinal() as i8,
        id: router_id,
    });
    let rec_dir = 1;
    let mut block_sync_bytes = Vec::new();
    synced_router
        .write_base(&mut block_sync_bytes, false)
        .unwrap();
    write_payload_conveyor_extra(
        &mut block_sync_bytes,
        6.0,
        conveyor_state.item_rotation,
        conveyor_state.item.as_ref(),
    )
    .unwrap();
    write_payload_router_extra(&mut block_sync_bytes, sorted_router, rec_dir).unwrap();
    let snapshot = BlockSnapshotCallPacket {
        amount: 1,
        data: {
            let mut data = Vec::new();
            data.extend_from_slice(&payload_router_tile.to_be_bytes());
            data.extend_from_slice(&payload_router_id.to_be_bytes());
            data.extend_from_slice(&block_sync_bytes);
            data
        },
    };
    server
        .net_server
        .send_block_snapshot(connection_id, snapshot.clone())
        .expect("real server should send payload block snapshot");

    let mut applied = false;
    let mut last_client_status = String::new();
    for _ in 0..80 {
        desktop.update();
        server.update();
        let received = {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            last_client_status = format!(
                "block_snapshots={} last_block={:?} last_server_snapshot={:?} provider_events={:?}",
                state.block_snapshot_packets_seen,
                state.last_block_snapshot,
                state.last_server_snapshot_at,
                state.last_provider_events,
            );
            state.last_block_snapshot.as_ref() == Some(&snapshot)
        };
        let materialized = matches!(
            desktop.runtime.payload_runtime_states.get(&payload_router_tile),
            Some(GameRuntimePayloadBlockState::Router {
                conveyor,
                sorted,
                rec_dir: applied_rec_dir,
                matches,
                ..
            }) if conveyor.item_rotation == conveyor_state.item_rotation
                && *sorted == sorted_router
                && *applied_rec_dir == rec_dir
                && !*matches
        );
        applied = received && materialized;
        if applied {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        applied,
        "desktop should receive and apply real payload block snapshot after world stream; client: {last_client_status}"
    );
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.block_snapshot_packets_seen, 1);
        assert_eq!(state.last_block_snapshot.as_ref(), Some(&snapshot));
        let mirror = state
            .last_block_snapshot_mirror
            .as_ref()
            .expect("payload block snapshot should materialize into lightweight mirror");
        assert_eq!(mirror.records.len(), 1);
        assert_eq!(mirror.records[0].tile_pos, payload_router_tile);
        assert_eq!(mirror.records[0].block_id, payload_router_id);
        assert_eq!(mirror.records[0].sync_bytes, block_sync_bytes);
        assert!(mirror.parse_error.is_none());
    }
    let runtime_record = desktop
        .runtime
        .client_block_snapshot_records
        .get(&payload_router_tile)
        .expect("real payload block snapshot should apply to client runtime sidecar");
    assert_eq!(runtime_record.block_id, payload_router_id);
    assert_eq!(runtime_record.sync_bytes, block_sync_bytes);
    let runtime_building = desktop
        .runtime
        .buildings()
        .iter()
        .find(|building| building.tile_pos == payload_router_tile)
        .expect("payload-router building should remain materialized");
    assert_eq!(runtime_building.health, 23.0);
    assert_eq!(runtime_building.rotation, 2);
    let Some(GameRuntimePayloadBlockState::Router {
        conveyor,
        sorted,
        rec_dir: applied_rec_dir,
        matches,
        ..
    }) = desktop
        .runtime
        .payload_runtime_states
        .get(&payload_router_tile)
    else {
        panic!("real block snapshot should apply payload-router child state to runtime");
    };
    assert_eq!(conveyor.item_rotation, conveyor_state.item_rotation);
    assert_eq!(*sorted, sorted_router);
    assert_eq!(*applied_rec_dir, rec_dir);
    assert!(!matches);
    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_payload_mass_driver_block_snapshot_updates_runtime_after_world_stream() {
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimePayloadBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::net::BlockSnapshotCallPacket;
    use mindustry_core::mindustry::world::blocks::payloads::{
        write_payload_block_build_common, write_payload_mass_driver_extra, PayloadBlockBuildState,
        PayloadDriverState, PayloadMassDriverState, Vec2,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    server.runtime.state.world.resize(10, 10);
    let driver_base = server
        .content_loader
        .block_by_name("payload-mass-driver")
        .expect("base content should include payload-mass-driver")
        .base()
        .clone();
    let driver_tile = point2_pack(4, 4);
    let driver_id = driver_base.id;
    server.runtime.add_building(BuildingComp::new(
        driver_tile,
        driver_base.clone(),
        TeamId(6),
    ));
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
            && desktop
                .runtime
                .buildings()
                .iter()
                .any(|building| building.tile_pos == driver_tile)
    });

    let connection_id = {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        state
            .last_connect_confirm_connection_id
            .expect("server should receive connect confirm before payload mass driver snapshot")
    };
    let mut synced_driver = BuildingComp::new(driver_tile, driver_base, TeamId(6));
    synced_driver.health = 23.0;
    synced_driver.set_rotation(1);
    let common = PayloadBlockBuildState {
        payload: None,
        pay_vector: Vec2 { x: 1.5, y: -2.25 },
        pay_rotation: 45.0,
        carried: false,
    };
    let driver_state = PayloadMassDriverState {
        link: point2_pack(7, 4),
        turret_rotation: 135.0,
        state: PayloadDriverState::Shooting,
        reload_counter: 0.5,
        charge: 12.0,
        loaded: true,
        charging: true,
        ..PayloadMassDriverState::default()
    };
    let mut block_sync_bytes = Vec::new();
    synced_driver
        .write_base(&mut block_sync_bytes, false)
        .unwrap();
    write_payload_block_build_common(&mut block_sync_bytes, &common).unwrap();
    write_payload_mass_driver_extra(&mut block_sync_bytes, &driver_state).unwrap();
    let snapshot = BlockSnapshotCallPacket {
        amount: 1,
        data: {
            let mut data = Vec::new();
            data.extend_from_slice(&driver_tile.to_be_bytes());
            data.extend_from_slice(&driver_id.to_be_bytes());
            data.extend_from_slice(&block_sync_bytes);
            data
        },
    };
    server
        .net_server
        .send_block_snapshot(connection_id, snapshot.clone())
        .expect("real server should send payload mass driver snapshot");

    let mut applied = false;
    let mut last_client_status = String::new();
    for _ in 0..80 {
        desktop.update();
        server.update();
        let received = {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            last_client_status = format!(
                "block_snapshots={} last_block={:?} last_server_snapshot={:?} provider_events={:?}",
                state.block_snapshot_packets_seen,
                state.last_block_snapshot,
                state.last_server_snapshot_at,
                state.last_provider_events,
            );
            state.last_block_snapshot.as_ref() == Some(&snapshot)
        };
        let materialized = matches!(
            desktop.runtime.payload_runtime_states.get(&driver_tile),
            Some(GameRuntimePayloadBlockState::MassDriver {
                common: applied_common,
                driver,
            }) if *applied_common == common && *driver == driver_state
        );
        applied = received && materialized;
        if applied {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        applied,
        "desktop should receive and apply real payload mass driver snapshot after world stream; client: {last_client_status}"
    );
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.block_snapshot_packets_seen, 1);
        assert_eq!(state.last_block_snapshot.as_ref(), Some(&snapshot));
        let mirror = state
            .last_block_snapshot_mirror
            .as_ref()
            .expect("payload mass driver snapshot should materialize into lightweight mirror");
        assert_eq!(mirror.records.len(), 1);
        assert_eq!(mirror.records[0].tile_pos, driver_tile);
        assert_eq!(mirror.records[0].block_id, driver_id);
        assert_eq!(mirror.records[0].sync_bytes, block_sync_bytes);
        assert!(mirror.parse_error.is_none());
    }
    let runtime_record = desktop
        .runtime
        .client_block_snapshot_records
        .get(&driver_tile)
        .expect("real payload mass driver snapshot should apply to client runtime sidecar");
    assert_eq!(runtime_record.block_id, driver_id);
    assert_eq!(runtime_record.sync_bytes, block_sync_bytes);
    let runtime_building = desktop
        .runtime
        .buildings()
        .iter()
        .find(|building| building.tile_pos == driver_tile)
        .expect("payload-mass-driver building should remain materialized");
    assert_eq!(runtime_building.health, 23.0);
    assert_eq!(runtime_building.rotation, 1);
    assert_eq!(
        desktop.runtime.payload_runtime_states.get(&driver_tile),
        Some(&GameRuntimePayloadBlockState::MassDriver {
            common,
            driver: driver_state,
        })
    );
    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_payload_loader_block_snapshot_updates_runtime_after_world_stream() {
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimePayloadBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::net::BlockSnapshotCallPacket;
    use mindustry_core::mindustry::world::blocks::payloads::{
        write_payload_block_build_common, write_payload_loader_extra, PayloadBlockBuildState,
        PayloadLoaderState, Vec2,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    server.runtime.state.world.resize(10, 10);
    let loader_base = server
        .content_loader
        .block_by_name("payload-loader")
        .expect("base content should include payload-loader")
        .base()
        .clone();
    let loader_tile = point2_pack(5, 4);
    let loader_id = loader_base.id;
    server.runtime.add_building(BuildingComp::new(
        loader_tile,
        loader_base.clone(),
        TeamId(6),
    ));
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
            && desktop
                .runtime
                .buildings()
                .iter()
                .any(|building| building.tile_pos == loader_tile)
    });

    let connection_id = {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        state
            .last_connect_confirm_connection_id
            .expect("server should receive connect confirm before payload loader snapshot")
    };
    let mut synced_loader = BuildingComp::new(loader_tile, loader_base, TeamId(6));
    synced_loader.health = 23.0;
    synced_loader.set_rotation(3);
    let common = PayloadBlockBuildState {
        payload: None,
        pay_vector: Vec2 { x: -1.0, y: 2.0 },
        pay_rotation: 90.0,
        carried: false,
    };
    let loader_state = PayloadLoaderState {
        has_payload: false,
        exporting: true,
        ..PayloadLoaderState::default()
    };
    let mut block_sync_bytes = Vec::new();
    synced_loader
        .write_base(&mut block_sync_bytes, false)
        .unwrap();
    write_payload_block_build_common(&mut block_sync_bytes, &common).unwrap();
    write_payload_loader_extra(&mut block_sync_bytes, loader_state.exporting).unwrap();
    let snapshot = BlockSnapshotCallPacket {
        amount: 1,
        data: {
            let mut data = Vec::new();
            data.extend_from_slice(&loader_tile.to_be_bytes());
            data.extend_from_slice(&loader_id.to_be_bytes());
            data.extend_from_slice(&block_sync_bytes);
            data
        },
    };
    server
        .net_server
        .send_block_snapshot(connection_id, snapshot.clone())
        .expect("real server should send payload loader snapshot");

    let mut applied = false;
    let mut last_client_status = String::new();
    for _ in 0..80 {
        desktop.update();
        server.update();
        let received = {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            last_client_status = format!(
                "block_snapshots={} last_block={:?} last_server_snapshot={:?} provider_events={:?}",
                state.block_snapshot_packets_seen,
                state.last_block_snapshot,
                state.last_server_snapshot_at,
                state.last_provider_events,
            );
            state.last_block_snapshot.as_ref() == Some(&snapshot)
        };
        let materialized = matches!(
            desktop.runtime.payload_runtime_states.get(&loader_tile),
            Some(GameRuntimePayloadBlockState::Loader {
                common: applied_common,
                loader,
            }) if *applied_common == common && *loader == loader_state
        );
        applied = received && materialized;
        if applied {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        applied,
        "desktop should receive and apply real payload loader snapshot after world stream; client: {last_client_status}"
    );
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.block_snapshot_packets_seen, 1);
        assert_eq!(state.last_block_snapshot.as_ref(), Some(&snapshot));
        let mirror = state
            .last_block_snapshot_mirror
            .as_ref()
            .expect("payload loader snapshot should materialize into lightweight mirror");
        assert_eq!(mirror.records.len(), 1);
        assert_eq!(mirror.records[0].tile_pos, loader_tile);
        assert_eq!(mirror.records[0].block_id, loader_id);
        assert_eq!(mirror.records[0].sync_bytes, block_sync_bytes);
        assert!(mirror.parse_error.is_none());
    }
    let runtime_record = desktop
        .runtime
        .client_block_snapshot_records
        .get(&loader_tile)
        .expect("real payload loader snapshot should apply to client runtime sidecar");
    assert_eq!(runtime_record.block_id, loader_id);
    assert_eq!(runtime_record.sync_bytes, block_sync_bytes);
    let runtime_building = desktop
        .runtime
        .buildings()
        .iter()
        .find(|building| building.tile_pos == loader_tile)
        .expect("payload-loader building should remain materialized");
    assert_eq!(runtime_building.health, 23.0);
    assert_eq!(runtime_building.rotation, 3);
    assert_eq!(
        desktop.runtime.payload_runtime_states.get(&loader_tile),
        Some(&GameRuntimePayloadBlockState::Loader {
            common,
            loader: loader_state,
        })
    );
    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_payload_source_block_snapshot_updates_runtime_after_world_stream() {
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimePayloadBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::net::BlockSnapshotCallPacket;
    use mindustry_core::mindustry::world::blocks::payloads::{
        write_payload_block_build_common, write_payload_source_extra, PayloadBlockBuildState,
        PayloadSourceState, Vec2,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    server.runtime.state.world.resize(10, 10);
    let source_base = server
        .content_loader
        .block_by_name("payload-source")
        .expect("base content should include payload-source")
        .base()
        .clone();
    let source_tile = point2_pack(5, 5);
    let source_id = source_base.id;
    server.runtime.add_building(BuildingComp::new(
        source_tile,
        source_base.clone(),
        TeamId(6),
    ));
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
            && desktop
                .runtime
                .buildings()
                .iter()
                .any(|building| building.tile_pos == source_tile)
    });

    let connection_id = {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        state
            .last_connect_confirm_connection_id
            .expect("server should receive connect confirm before payload source snapshot")
    };
    let mut synced_source = BuildingComp::new(source_tile, source_base, TeamId(6));
    synced_source.health = 23.0;
    synced_source.set_rotation(1);
    let common = PayloadBlockBuildState {
        payload: None,
        pay_vector: Vec2 { x: 0.25, y: 0.5 },
        pay_rotation: 90.0,
        carried: false,
    };
    let source_state = PayloadSourceState {
        unit: Some(0),
        config_block: None,
        command_pos: Some(Vec2 { x: 8.0, y: 16.0 }),
        has_payload: false,
        ..PayloadSourceState::default()
    };
    let mut block_sync_bytes = Vec::new();
    synced_source
        .write_base(&mut block_sync_bytes, false)
        .unwrap();
    write_payload_block_build_common(&mut block_sync_bytes, &common).unwrap();
    write_payload_source_extra(
        &mut block_sync_bytes,
        source_state.unit,
        source_state.config_block,
        source_state.command_pos,
    )
    .unwrap();
    let snapshot = BlockSnapshotCallPacket {
        amount: 1,
        data: {
            let mut data = Vec::new();
            data.extend_from_slice(&source_tile.to_be_bytes());
            data.extend_from_slice(&source_id.to_be_bytes());
            data.extend_from_slice(&block_sync_bytes);
            data
        },
    };
    server
        .net_server
        .send_block_snapshot(connection_id, snapshot.clone())
        .expect("real server should send payload source snapshot");

    let mut applied = false;
    let mut last_client_status = String::new();
    for _ in 0..80 {
        desktop.update();
        server.update();
        let received = {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            last_client_status = format!(
                "block_snapshots={} last_block={:?} last_server_snapshot={:?} provider_events={:?}",
                state.block_snapshot_packets_seen,
                state.last_block_snapshot,
                state.last_server_snapshot_at,
                state.last_provider_events,
            );
            state.last_block_snapshot.as_ref() == Some(&snapshot)
        };
        let materialized = matches!(
            desktop.runtime.payload_runtime_states.get(&source_tile),
            Some(GameRuntimePayloadBlockState::Source {
                common: applied_common,
                source,
            }) if *applied_common == common && *source == source_state
        );
        applied = received && materialized;
        if applied {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        applied,
        "desktop should receive and apply real payload source snapshot after world stream; client: {last_client_status}"
    );
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.block_snapshot_packets_seen, 1);
        assert_eq!(state.last_block_snapshot.as_ref(), Some(&snapshot));
        let mirror = state
            .last_block_snapshot_mirror
            .as_ref()
            .expect("payload source snapshot should materialize into lightweight mirror");
        assert_eq!(mirror.records.len(), 1);
        assert_eq!(mirror.records[0].tile_pos, source_tile);
        assert_eq!(mirror.records[0].block_id, source_id);
        assert_eq!(mirror.records[0].sync_bytes, block_sync_bytes);
        assert!(mirror.parse_error.is_none());
    }
    let runtime_record = desktop
        .runtime
        .client_block_snapshot_records
        .get(&source_tile)
        .expect("real payload source snapshot should apply to client runtime sidecar");
    assert_eq!(runtime_record.block_id, source_id);
    assert_eq!(runtime_record.sync_bytes, block_sync_bytes);
    let runtime_building = desktop
        .runtime
        .buildings()
        .iter()
        .find(|building| building.tile_pos == source_tile)
        .expect("payload-source building should remain materialized");
    assert_eq!(runtime_building.health, 23.0);
    assert_eq!(runtime_building.rotation, 1);
    assert_eq!(
        desktop.runtime.payload_runtime_states.get(&source_tile),
        Some(&GameRuntimePayloadBlockState::Source {
            common,
            source: source_state,
        })
    );
    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_payload_deconstructor_block_snapshot_updates_runtime_after_world_stream() {
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimePayloadBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::net::BlockSnapshotCallPacket;
    use mindustry_core::mindustry::world::blocks::payloads::{
        write_deconstructor_extra, write_payload_block_build_common, write_payload_ref,
        PayloadBlockBuildState, PayloadDeconstructorState, PayloadRef, Vec2,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    server.runtime.state.world.resize(10, 10);
    let deconstructor_base = server
        .content_loader
        .block_by_name("small-deconstructor")
        .expect("base content should include small-deconstructor")
        .base()
        .clone();
    let router_base = server
        .content_loader
        .block_by_name("router")
        .expect("base content should include router")
        .base()
        .clone();
    let deconstructor_tile = point2_pack(5, 5);
    let deconstructor_id = deconstructor_base.id;
    server.runtime.add_building(BuildingComp::new(
        deconstructor_tile,
        deconstructor_base.clone(),
        TeamId(6),
    ));
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
            && desktop
                .runtime
                .buildings()
                .iter()
                .any(|building| building.tile_pos == deconstructor_tile)
    });

    let connection_id = {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        state
            .last_connect_confirm_connection_id
            .expect("server should receive connect confirm before payload deconstructor snapshot")
    };
    let mut payload_build_bytes = Vec::new();
    BuildingComp::new(point2_pack(0, 0), router_base.clone(), TeamId(6))
        .write_base(&mut payload_build_bytes, false)
        .unwrap();
    let deconstructing = PayloadRef::Block {
        block: router_base.id,
        version: 0,
        build_bytes: payload_build_bytes,
    };
    let mut synced_deconstructor =
        BuildingComp::new(deconstructor_tile, deconstructor_base, TeamId(6));
    synced_deconstructor.health = 23.0;
    let common = PayloadBlockBuildState {
        payload: None,
        pay_vector: Vec2 { x: -0.5, y: 0.75 },
        pay_rotation: 15.0,
        carried: false,
    };
    let deconstructor_state = PayloadDeconstructorState {
        progress: 0.4,
        accum: Some(vec![1.0, 2.5, 0.25]),
        has_payload: false,
        has_deconstructing: true,
        deconstructing: Some(deconstructing.clone()),
    };
    let mut block_sync_bytes = Vec::new();
    synced_deconstructor
        .write_base(&mut block_sync_bytes, false)
        .unwrap();
    write_payload_block_build_common(&mut block_sync_bytes, &common).unwrap();
    write_deconstructor_extra(
        &mut block_sync_bytes,
        deconstructor_state.progress,
        deconstructor_state.accum.as_deref(),
    )
    .unwrap();
    write_payload_ref(
        &mut block_sync_bytes,
        deconstructor_state.deconstructing.as_ref(),
    )
    .unwrap();
    let snapshot = BlockSnapshotCallPacket {
        amount: 1,
        data: {
            let mut data = Vec::new();
            data.extend_from_slice(&deconstructor_tile.to_be_bytes());
            data.extend_from_slice(&deconstructor_id.to_be_bytes());
            data.extend_from_slice(&block_sync_bytes);
            data
        },
    };
    server
        .net_server
        .send_block_snapshot(connection_id, snapshot.clone())
        .expect("real server should send payload deconstructor snapshot");

    let mut applied = false;
    let mut last_client_status = String::new();
    for _ in 0..80 {
        desktop.update();
        server.update();
        let received = {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            last_client_status = format!(
                "block_snapshots={} last_block={:?} last_server_snapshot={:?} provider_events={:?}",
                state.block_snapshot_packets_seen,
                state.last_block_snapshot,
                state.last_server_snapshot_at,
                state.last_provider_events,
            );
            state.last_block_snapshot.as_ref() == Some(&snapshot)
        };
        let materialized = matches!(
            desktop
                .runtime
                .payload_runtime_states
                .get(&deconstructor_tile),
            Some(GameRuntimePayloadBlockState::Deconstructor {
                common: applied_common,
                deconstructor,
            }) if *applied_common == common && *deconstructor == deconstructor_state
        );
        applied = received && materialized;
        if applied {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        applied,
        "desktop should receive and apply real payload deconstructor snapshot after world stream; client: {last_client_status}"
    );
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.block_snapshot_packets_seen, 1);
        assert_eq!(state.last_block_snapshot.as_ref(), Some(&snapshot));
        let mirror = state
            .last_block_snapshot_mirror
            .as_ref()
            .expect("payload deconstructor snapshot should materialize into lightweight mirror");
        assert_eq!(mirror.records.len(), 1);
        assert_eq!(mirror.records[0].tile_pos, deconstructor_tile);
        assert_eq!(mirror.records[0].block_id, deconstructor_id);
        assert_eq!(mirror.records[0].sync_bytes, block_sync_bytes);
        assert!(mirror.parse_error.is_none());
    }
    let runtime_record = desktop
        .runtime
        .client_block_snapshot_records
        .get(&deconstructor_tile)
        .expect("real payload deconstructor snapshot should apply to client runtime sidecar");
    assert_eq!(runtime_record.block_id, deconstructor_id);
    assert_eq!(runtime_record.sync_bytes, block_sync_bytes);
    let runtime_building = desktop
        .runtime
        .buildings()
        .iter()
        .find(|building| building.tile_pos == deconstructor_tile)
        .expect("payload-deconstructor building should remain materialized");
    assert_eq!(runtime_building.health, 23.0);
    assert_eq!(
        desktop
            .runtime
            .payload_runtime_states
            .get(&deconstructor_tile),
        Some(&GameRuntimePayloadBlockState::Deconstructor {
            common,
            deconstructor: deconstructor_state,
        })
    );
    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_payload_constructor_block_snapshot_updates_runtime_after_world_stream() {
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimePayloadBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::net::BlockSnapshotCallPacket;
    use mindustry_core::mindustry::world::blocks::payloads::{
        write_block_producer_progress, write_constructor_recipe, write_payload_block_build_common,
        BlockProducerState, PayloadBlockBuildState, Vec2,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    server.runtime.state.world.resize(10, 10);
    let constructor_base = server
        .content_loader
        .block_by_name("constructor")
        .expect("base content should include constructor")
        .base()
        .clone();
    let recipe = server
        .content_loader
        .block_by_name("router")
        .map(|block| block.base().id);
    let constructor_tile = point2_pack(4, 5);
    let constructor_id = constructor_base.id;
    server.runtime.add_building(BuildingComp::new(
        constructor_tile,
        constructor_base.clone(),
        TeamId(6),
    ));
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
            && desktop
                .runtime
                .buildings()
                .iter()
                .any(|building| building.tile_pos == constructor_tile)
    });

    let connection_id = {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        state
            .last_connect_confirm_connection_id
            .expect("server should receive connect confirm before payload constructor snapshot")
    };
    let mut synced_constructor = BuildingComp::new(constructor_tile, constructor_base, TeamId(6));
    synced_constructor.health = 23.0;
    synced_constructor.set_rotation(2);
    let common = PayloadBlockBuildState {
        payload: None,
        pay_vector: Vec2 { x: 0.0, y: -1.0 },
        pay_rotation: 180.0,
        carried: false,
    };
    let producer = BlockProducerState {
        progress: 3.5,
        has_payload: false,
        ..BlockProducerState::default()
    };
    let mut block_sync_bytes = Vec::new();
    synced_constructor
        .write_base(&mut block_sync_bytes, false)
        .unwrap();
    write_payload_block_build_common(&mut block_sync_bytes, &common).unwrap();
    write_block_producer_progress(&mut block_sync_bytes, producer.progress).unwrap();
    write_constructor_recipe(&mut block_sync_bytes, recipe).unwrap();
    let snapshot = BlockSnapshotCallPacket {
        amount: 1,
        data: {
            let mut data = Vec::new();
            data.extend_from_slice(&constructor_tile.to_be_bytes());
            data.extend_from_slice(&constructor_id.to_be_bytes());
            data.extend_from_slice(&block_sync_bytes);
            data
        },
    };
    server
        .net_server
        .send_block_snapshot(connection_id, snapshot.clone())
        .expect("real server should send payload constructor snapshot");

    let mut applied = false;
    let mut last_client_status = String::new();
    for _ in 0..80 {
        desktop.update();
        server.update();
        let received = {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            last_client_status = format!(
                "block_snapshots={} last_block={:?} last_server_snapshot={:?} provider_events={:?}",
                state.block_snapshot_packets_seen,
                state.last_block_snapshot,
                state.last_server_snapshot_at,
                state.last_provider_events,
            );
            state.last_block_snapshot.as_ref() == Some(&snapshot)
        };
        let materialized = matches!(
            desktop
                .runtime
                .payload_runtime_states
                .get(&constructor_tile),
            Some(GameRuntimePayloadBlockState::Constructor {
                common: applied_common,
                producer: applied_producer,
                recipe: applied_recipe,
            }) if *applied_common == common
                && *applied_producer == producer
                && *applied_recipe == recipe
        );
        applied = received && materialized;
        if applied {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        applied,
        "desktop should receive and apply real payload constructor snapshot after world stream; client: {last_client_status}"
    );
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.block_snapshot_packets_seen, 1);
        assert_eq!(state.last_block_snapshot.as_ref(), Some(&snapshot));
        let mirror = state
            .last_block_snapshot_mirror
            .as_ref()
            .expect("payload constructor snapshot should materialize into lightweight mirror");
        assert_eq!(mirror.records.len(), 1);
        assert_eq!(mirror.records[0].tile_pos, constructor_tile);
        assert_eq!(mirror.records[0].block_id, constructor_id);
        assert_eq!(mirror.records[0].sync_bytes, block_sync_bytes);
        assert!(mirror.parse_error.is_none());
    }
    let runtime_record = desktop
        .runtime
        .client_block_snapshot_records
        .get(&constructor_tile)
        .expect("real payload constructor snapshot should apply to client runtime sidecar");
    assert_eq!(runtime_record.block_id, constructor_id);
    assert_eq!(runtime_record.sync_bytes, block_sync_bytes);
    let runtime_building = desktop
        .runtime
        .buildings()
        .iter()
        .find(|building| building.tile_pos == constructor_tile)
        .expect("payload-constructor building should remain materialized");
    assert_eq!(runtime_building.health, 23.0);
    assert_eq!(runtime_building.rotation, 2);
    assert_eq!(
        desktop
            .runtime
            .payload_runtime_states
            .get(&constructor_tile),
        Some(&GameRuntimePayloadBlockState::Constructor {
            common,
            producer,
            recipe,
        })
    );
    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_payload_void_block_snapshot_updates_runtime_after_world_stream() {
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimePayloadBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::net::BlockSnapshotCallPacket;
    use mindustry_core::mindustry::world::blocks::payloads::{
        write_payload_block_build_common, PayloadBlockBuildState, Vec2,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    server.runtime.state.world.resize(10, 10);
    let void_base = server
        .content_loader
        .block_by_name("payload-void")
        .expect("base content should include payload-void")
        .base()
        .clone();
    let void_tile = point2_pack(4, 5);
    let void_id = void_base.id;
    server
        .runtime
        .add_building(BuildingComp::new(void_tile, void_base.clone(), TeamId(6)));
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
            && desktop
                .runtime
                .buildings()
                .iter()
                .any(|building| building.tile_pos == void_tile)
    });

    let connection_id = {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        state
            .last_connect_confirm_connection_id
            .expect("server should receive connect confirm before payload void snapshot")
    };
    let mut synced_void = BuildingComp::new(void_tile, void_base, TeamId(6));
    synced_void.health = 23.0;
    let common = PayloadBlockBuildState {
        payload: None,
        pay_vector: Vec2 { x: 0.25, y: -0.75 },
        pay_rotation: 270.0,
        carried: false,
    };
    let mut block_sync_bytes = Vec::new();
    synced_void
        .write_base(&mut block_sync_bytes, false)
        .unwrap();
    write_payload_block_build_common(&mut block_sync_bytes, &common).unwrap();
    let snapshot = BlockSnapshotCallPacket {
        amount: 1,
        data: {
            let mut data = Vec::new();
            data.extend_from_slice(&void_tile.to_be_bytes());
            data.extend_from_slice(&void_id.to_be_bytes());
            data.extend_from_slice(&block_sync_bytes);
            data
        },
    };
    server
        .net_server
        .send_block_snapshot(connection_id, snapshot.clone())
        .expect("real server should send payload void snapshot");

    let mut applied = false;
    let mut last_client_status = String::new();
    for _ in 0..80 {
        desktop.update();
        server.update();
        let received = {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            last_client_status = format!(
                "block_snapshots={} last_block={:?} last_server_snapshot={:?} provider_events={:?}",
                state.block_snapshot_packets_seen,
                state.last_block_snapshot,
                state.last_server_snapshot_at,
                state.last_provider_events,
            );
            state.last_block_snapshot.as_ref() == Some(&snapshot)
        };
        let materialized = matches!(
            desktop.runtime.payload_runtime_states.get(&void_tile),
            Some(GameRuntimePayloadBlockState::Void(applied_common))
                if *applied_common == common
        );
        applied = received && materialized;
        if applied {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        applied,
        "desktop should receive and apply real payload void snapshot after world stream; client: {last_client_status}"
    );
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.block_snapshot_packets_seen, 1);
        assert_eq!(state.last_block_snapshot.as_ref(), Some(&snapshot));
        let mirror = state
            .last_block_snapshot_mirror
            .as_ref()
            .expect("payload void snapshot should materialize into lightweight mirror");
        assert_eq!(mirror.records.len(), 1);
        assert_eq!(mirror.records[0].tile_pos, void_tile);
        assert_eq!(mirror.records[0].block_id, void_id);
        assert_eq!(mirror.records[0].sync_bytes, block_sync_bytes);
        assert!(mirror.parse_error.is_none());
    }
    let runtime_record = desktop
        .runtime
        .client_block_snapshot_records
        .get(&void_tile)
        .expect("real payload void snapshot should apply to client runtime sidecar");
    assert_eq!(runtime_record.block_id, void_id);
    assert_eq!(runtime_record.sync_bytes, block_sync_bytes);
    let runtime_building = desktop
        .runtime
        .buildings()
        .iter()
        .find(|building| building.tile_pos == void_tile)
        .expect("payload-void building should remain materialized");
    assert_eq!(runtime_building.health, 23.0);
    assert_eq!(
        desktop.runtime.payload_runtime_states.get(&void_tile),
        Some(&GameRuntimePayloadBlockState::Void(common))
    );
    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}

#[test]
fn real_server_desktop_item_turret_block_snapshot_preserves_rotation_reload_after_world_stream() {
    use mindustry_core::mindustry::core::{
        game_runtime::GameRuntimeTurretBlockState, GameRuntimeNetworkContext,
    };
    use mindustry_core::mindustry::entities::comp::BuildingComp;
    use mindustry_core::mindustry::io::TeamId;
    use mindustry_core::mindustry::net::BlockSnapshotCallPacket;
    use mindustry_core::mindustry::world::blocks::defense::turrets::{
        item_turret_write_ammo, turret_write_child, ItemAmmoEntry, TurretState,
    };
    use mindustry_core::mindustry::world::point2_pack;
    use mindustry_server::ServerLauncher;
    use std::thread;
    use std::time::Duration;

    let port = free_local_port();
    let mut server = ServerLauncher::new(vec![
        "mindustry-server".into(),
        "--port".into(),
        port.to_string(),
    ]);
    server.runtime.state.world.resize(10, 10);
    let duo_base = server
        .content_loader
        .block_by_name("duo")
        .expect("base content should include duo")
        .base()
        .clone();
    let copper_id = server
        .content_loader
        .item_by_name("copper")
        .expect("base content should include copper")
        .base
        .mappable
        .base
        .id;
    let duo_tile = point2_pack(4, 5);
    let duo_id = duo_base.id;
    server
        .runtime
        .add_building(BuildingComp::new(duo_tile, duo_base.clone(), TeamId(6)));
    let existing_turret = TurretState {
        reload_counter: 1.25,
        rotation: 12.0,
        ..TurretState::default()
    };
    server.runtime.turret_runtime_states.insert(
        duo_tile,
        GameRuntimeTurretBlockState::Item {
            turret: existing_turret.clone(),
            ammo: vec![ItemAmmoEntry {
                item_id: copper_id,
                amount: 1,
            }],
        },
    );
    server.init();

    let mut desktop = mindustry_desktop::run(vec![
        "mindustry-desktop".into(),
        "--connect".into(),
        format!("127.0.0.1:{port}"),
    ]);
    pump_real_server_desktop_until(&mut server, &mut desktop, |desktop| {
        desktop.runtime.network_context == GameRuntimeNetworkContext::client()
            && matches!(
                desktop.runtime.turret_runtime_states.get(&duo_tile),
                Some(GameRuntimeTurretBlockState::Item { turret, .. })
                    if turret.rotation == existing_turret.rotation
                        && turret.reload_counter == existing_turret.reload_counter
            )
    });

    let connection_id = {
        let state = server.net_server.state();
        let state = state.lock().unwrap();
        state
            .last_connect_confirm_connection_id
            .expect("server should receive connect confirm before item turret snapshot")
    };
    let mut synced_duo = BuildingComp::new(duo_tile, duo_base, TeamId(6));
    synced_duo.health = 23.0;
    synced_duo.set_rotation(1);
    let incoming_turret = TurretState {
        reload_counter: 9.0,
        rotation: 90.0,
        ..TurretState::default()
    };
    let new_ammo = vec![ItemAmmoEntry {
        item_id: copper_id,
        amount: 7,
    }];
    let mut block_sync_bytes = Vec::new();
    synced_duo.write_base(&mut block_sync_bytes, false).unwrap();
    turret_write_child(&mut block_sync_bytes, &incoming_turret).unwrap();
    item_turret_write_ammo(&mut block_sync_bytes, &new_ammo).unwrap();
    let snapshot = BlockSnapshotCallPacket {
        amount: 1,
        data: {
            let mut data = Vec::new();
            data.extend_from_slice(&duo_tile.to_be_bytes());
            data.extend_from_slice(&duo_id.to_be_bytes());
            data.extend_from_slice(&block_sync_bytes);
            data
        },
    };
    server
        .net_server
        .send_block_snapshot(connection_id, snapshot.clone())
        .expect("real server should send item turret snapshot");

    let mut expected_turret = incoming_turret.clone();
    expected_turret.rotation = existing_turret.rotation;
    expected_turret.reload_counter = existing_turret.reload_counter;
    expected_turret.total_ammo = 7;
    let mut applied = false;
    let mut last_client_status = String::new();
    for _ in 0..80 {
        desktop.update();
        server.update();
        let received = {
            let state = desktop.net_client.state();
            let state = state.lock().unwrap();
            last_client_status = format!(
                "block_snapshots={} last_block={:?} last_server_snapshot={:?} provider_events={:?}",
                state.block_snapshot_packets_seen,
                state.last_block_snapshot,
                state.last_server_snapshot_at,
                state.last_provider_events,
            );
            state.last_block_snapshot.as_ref() == Some(&snapshot)
        };
        let materialized = matches!(
            desktop.runtime.turret_runtime_states.get(&duo_tile),
            Some(GameRuntimeTurretBlockState::Item { turret, ammo })
                if turret == &expected_turret && ammo == &new_ammo
        );
        applied = received && materialized;
        if applied {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        applied,
        "desktop should receive and apply real item turret snapshot after world stream while preserving local rotation/reload; client: {last_client_status}"
    );
    {
        let state = desktop.net_client.state();
        let state = state.lock().unwrap();
        assert_eq!(state.block_snapshot_packets_seen, 1);
        assert_eq!(state.last_block_snapshot.as_ref(), Some(&snapshot));
        let mirror = state
            .last_block_snapshot_mirror
            .as_ref()
            .expect("item turret snapshot should materialize into lightweight mirror");
        assert_eq!(mirror.records.len(), 1);
        assert_eq!(mirror.records[0].tile_pos, duo_tile);
        assert_eq!(mirror.records[0].block_id, duo_id);
        assert_eq!(mirror.records[0].sync_bytes, block_sync_bytes);
        assert!(mirror.parse_error.is_none());
    }
    let runtime_record = desktop
        .runtime
        .client_block_snapshot_records
        .get(&duo_tile)
        .expect("real item turret snapshot should apply to client runtime sidecar");
    assert_eq!(runtime_record.block_id, duo_id);
    assert_eq!(runtime_record.sync_bytes, block_sync_bytes);
    let runtime_building = desktop
        .runtime
        .buildings()
        .iter()
        .find(|building| building.tile_pos == duo_tile)
        .expect("duo building should remain materialized");
    assert_eq!(runtime_building.health, 23.0);
    assert_eq!(runtime_building.rotation, 1);
    assert_eq!(
        desktop.runtime.turret_runtime_states.get(&duo_tile),
        Some(&GameRuntimeTurretBlockState::Item {
            turret: expected_turret,
            ammo: new_ammo,
        })
    );
    assert_eq!(
        desktop.runtime.network_context,
        GameRuntimeNetworkContext::client()
    );

    desktop.net_client.net_mut().disconnect();
    server.close_network();
}
