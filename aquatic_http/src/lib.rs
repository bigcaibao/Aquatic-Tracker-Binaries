use aquatic_common::{
    access_list::update_access_list,
    cpu_pinning::{
        glommio::{get_worker_placement, set_affinity_for_util_worker},
        WorkerIndex,
    },
    privileges::drop_privileges_after_socket_binding,
    rustls_config::create_rustls_config,
};
use common::State;
use glommio::{channels::channel_mesh::MeshBuilder, prelude::*};
use signal_hook::{consts::SIGUSR1, iterator::Signals};
use std::sync::{atomic::AtomicUsize, Arc};

use crate::config::Config;

mod common;
pub mod config;
mod workers;

pub const APP_NAME: &str = "aquatic_http: BitTorrent tracker (HTTP over TLS)";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const SHARED_CHANNEL_SIZE: usize = 1024;

pub fn run(config: Config) -> ::anyhow::Result<()> {
    let state = State::default();

    update_access_list(&config.access_list, &state.access_list)?;

    let mut signals = Signals::new(::std::iter::once(SIGUSR1))?;

    {
        let config = config.clone();
        let state = state.clone();

        ::std::thread::spawn(move || run_inner(config, state));
    }

    if config.cpu_pinning.active {
        set_affinity_for_util_worker(
            &config.cpu_pinning,
            config.socket_workers,
            config.request_workers,
        )?;
    }

    for signal in &mut signals {
        match signal {
            SIGUSR1 => {
                let _ = update_access_list(&config.access_list, &state.access_list);
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

pub fn run_inner(config: Config, state: State) -> anyhow::Result<()> {
    let num_peers = config.socket_workers + config.request_workers;

    let request_mesh_builder = MeshBuilder::partial(num_peers, SHARED_CHANNEL_SIZE);
    let response_mesh_builder = MeshBuilder::partial(num_peers, SHARED_CHANNEL_SIZE);

    let num_bound_sockets = Arc::new(AtomicUsize::new(0));

    let tls_config = Arc::new(create_rustls_config(
        &config.network.tls_certificate_path,
        &config.network.tls_private_key_path,
    )?);

    let mut executors = Vec::new();

    for i in 0..(config.socket_workers) {
        let config = config.clone();
        let state = state.clone();
        let tls_config = tls_config.clone();
        let request_mesh_builder = request_mesh_builder.clone();
        let response_mesh_builder = response_mesh_builder.clone();
        let num_bound_sockets = num_bound_sockets.clone();

        let placement = get_worker_placement(
            &config.cpu_pinning,
            config.socket_workers,
            config.request_workers,
            WorkerIndex::SocketWorker(i),
        )?;
        let builder = LocalExecutorBuilder::new(placement).name("socket");

        let executor = builder.spawn(move || async move {
            workers::socket::run_socket_worker(
                config,
                state,
                tls_config,
                request_mesh_builder,
                response_mesh_builder,
                num_bound_sockets,
            )
            .await
        });

        executors.push(executor);
    }

    for i in 0..(config.request_workers) {
        let config = config.clone();
        let state = state.clone();
        let request_mesh_builder = request_mesh_builder.clone();
        let response_mesh_builder = response_mesh_builder.clone();

        let placement = get_worker_placement(
            &config.cpu_pinning,
            config.socket_workers,
            config.request_workers,
            WorkerIndex::RequestWorker(i),
        )?;
        let builder = LocalExecutorBuilder::new(placement).name("request");

        let executor = builder.spawn(move || async move {
            workers::request::run_request_worker(
                config,
                state,
                request_mesh_builder,
                response_mesh_builder,
            )
            .await
        });

        executors.push(executor);
    }

    drop_privileges_after_socket_binding(
        &config.privileges,
        num_bound_sockets,
        config.socket_workers,
    )
    .unwrap();

    if config.cpu_pinning.active {
        set_affinity_for_util_worker(
            &config.cpu_pinning,
            config.socket_workers,
            config.request_workers,
        )?;
    }

    for executor in executors {
        executor
            .expect("failed to spawn local executor")
            .join()
            .unwrap();
    }

    Ok(())
}
