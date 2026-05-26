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
        if loaded && materialized {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        loaded && materialized,
        "desktop should load, confirm and materialize real world stream; loaded={loaded} materialized={materialized}; client: {last_client_status}; server: {last_server_status}; connect_error: {:?}",
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
    use mindustry_core::mindustry::net::{
        EntitySnapshotCallPacket, HiddenSnapshotCallPacket, StateSnapshotCallPacket,
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
    let mut first_entity_data = Vec::new();
    first_entity_data.extend_from_slice(&1001i32.to_be_bytes());
    first_entity_data.push(2);
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
    let hidden = HiddenSnapshotCallPacket { ids: vec![4, 5] };

    server
        .net_server
        .send_entity_sync_snapshot(
            connection_id,
            state_snapshot.clone(),
            vec![first_entity.clone(), second_entity.clone()],
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
                && state.entity_snapshot_packets_seen == 2
                && state.last_entity_snapshot.as_ref() == Some(&second_entity)
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
        if applied && desktop.game_state.wave == state_snapshot.wave {
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
        assert_eq!(state.last_entity_snapshot.as_ref(), Some(&second_entity));
        assert_eq!(state.entity_snapshot_packets_sent, 2);
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
        assert_eq!(state.entity_snapshot_packets_seen, 2);
        assert_eq!(state.last_entity_snapshot.as_ref(), Some(&second_entity));
        assert_eq!(state.entity_snapshot_mirrors.len(), 2);
        assert_eq!(state.entity_snapshot_mirrors[0].records.len(), 1);
        assert_eq!(state.entity_snapshot_mirrors[0].records[0].entity_id, 1001);
        assert_eq!(state.entity_snapshot_mirrors[0].records[0].type_id, 2);
        assert_eq!(state.entity_snapshot_mirrors[1].records.len(), 2);
        assert_eq!(state.entity_snapshot_mirrors[1].records[0].entity_id, 1002);
        assert_eq!(state.entity_snapshot_mirrors[1].records[0].type_id, 3);
        assert_eq!(state.entity_snapshot_mirrors[1].records[1].entity_id, 1003);
        assert_eq!(state.entity_snapshot_mirrors[1].records[1].type_id, 4);
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
