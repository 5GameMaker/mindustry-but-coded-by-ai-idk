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
