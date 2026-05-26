#[test]
fn workspace_smoke() {
    assert_eq!(
        mindustry_core::mindustry::UPSTREAM_BASELINE,
        "mindustry-upstream-v158"
    );
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
    use std::net::{TcpListener, UdpSocket};
    use std::thread;
    use std::time::Duration;

    fn free_local_port() -> u16 {
        for _ in 0..32 {
            let tcp = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = tcp.local_addr().unwrap().port();
            if UdpSocket::bind(("127.0.0.1", port)).is_ok() {
                return port;
            }
        }
        panic!("could not reserve a local TCP/UDP port pair");
    }

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

    let mut loaded = false;
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
        if loaded
            && desktop
                .runtime
                .payload_runtime_states
                .contains_key(&loader_tile)
        {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        loaded,
        "desktop should load and confirm real world stream; client: {last_client_status}; server: {last_server_status}; connect_error: {:?}",
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
