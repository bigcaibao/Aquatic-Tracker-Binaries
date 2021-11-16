pub mod common;
pub mod config;
pub mod handlers;
#[cfg(feature = "with-mio")]
pub mod network_mio;
#[cfg(feature = "with-io-uring")]
pub mod network_uring;
pub mod tasks;

use config::Config;

use std::collections::BTreeMap;
use std::sync::{atomic::AtomicUsize, Arc};
use std::thread::Builder;
use std::time::Duration;

use anyhow::Context;
#[cfg(feature = "cpu-pinning")]
use aquatic_common::cpu_pinning::{pin_current_if_configured_to, WorkerIndex};
use aquatic_common::privileges::drop_privileges_after_socket_binding;
use crossbeam_channel::unbounded;

use aquatic_common::access_list::update_access_list;
use signal_hook::consts::SIGUSR1;
use signal_hook::iterator::Signals;

use common::{ConnectedRequestSender, ConnectedResponseSender, SocketWorkerIndex, State};

pub const APP_NAME: &str = "aquatic_udp: UDP BitTorrent tracker";

pub fn run(config: Config) -> ::anyhow::Result<()> {
    let state = State::default();

    update_access_list(&config.access_list, &state.access_list)?;

    let mut signals = Signals::new(::std::iter::once(SIGUSR1))?;

    {
        let config = config.clone();
        let state = state.clone();

        ::std::thread::spawn(move || run_inner(config, state));
    }

    #[cfg(feature = "cpu-pinning")]
    pin_current_if_configured_to(
        &config.cpu_pinning,
        config.socket_workers,
        WorkerIndex::Other,
    );

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

pub fn run_inner(config: Config, state: State) -> ::anyhow::Result<()> {
    let num_bound_sockets = Arc::new(AtomicUsize::new(0));

    let mut request_senders = Vec::new();
    let mut request_receivers = BTreeMap::new();

    let mut response_senders = Vec::new();
    let mut response_receivers = BTreeMap::new();

    for i in 0..config.request_workers {
        let (request_sender, request_receiver) = unbounded();

        request_senders.push(request_sender);
        request_receivers.insert(i, request_receiver);
    }

    for i in 0..config.socket_workers {
        let (response_sender, response_receiver) = unbounded();

        response_senders.push(response_sender);
        response_receivers.insert(i, response_receiver);
    }

    for i in 0..config.request_workers {
        let config = config.clone();
        let request_receiver = request_receivers.remove(&i).unwrap().clone();
        let response_sender = ConnectedResponseSender::new(response_senders.clone());

        Builder::new()
            .name(format!("request-{:02}", i + 1))
            .spawn(move || {
                #[cfg(feature = "cpu-pinning")]
                pin_current_if_configured_to(
                    &config.cpu_pinning,
                    config.socket_workers,
                    WorkerIndex::RequestWorker(i),
                );

                handlers::run_request_worker(config, request_receiver, response_sender)
            })
            .with_context(|| "spawn request worker")?;
    }

    for i in 0..config.socket_workers {
        let state = state.clone();
        let config = config.clone();
        let request_sender =
            ConnectedRequestSender::new(SocketWorkerIndex(i), request_senders.clone());
        let response_receiver = response_receivers.remove(&i).unwrap();
        let num_bound_sockets = num_bound_sockets.clone();

        Builder::new()
            .name(format!("socket-{:02}", i + 1))
            .spawn(move || {
                #[cfg(feature = "cpu-pinning")]
                pin_current_if_configured_to(
                    &config.cpu_pinning,
                    config.socket_workers,
                    WorkerIndex::SocketWorker(i),
                );

                cfg_if::cfg_if!(
                    if #[cfg(feature = "with-io-uring")] {
                        network_uring::run_socket_worker(
                            state,
                            config,
                            request_sender,
                            response_receiver,
                            num_bound_sockets,
                        );
                    } else {
                        network_mio::run_socket_worker(
                            state,
                            config,
                            i,
                            request_sender,
                            response_receiver,
                            num_bound_sockets,
                        );
                    }
                );
            })
            .with_context(|| "spawn socket worker")?;
    }

    ::std::mem::drop(request_senders);
    ::std::mem::drop(request_receivers);

    ::std::mem::drop(response_senders);
    ::std::mem::drop(response_receivers);

    if config.statistics.interval != 0 {
        let state = state.clone();
        let config = config.clone();

        Builder::new()
            .name("statistics-collector".to_string())
            .spawn(move || {
                #[cfg(feature = "cpu-pinning")]
                pin_current_if_configured_to(
                    &config.cpu_pinning,
                    config.socket_workers,
                    WorkerIndex::Other,
                );

                loop {
                    ::std::thread::sleep(Duration::from_secs(config.statistics.interval));

                    tasks::gather_and_print_statistics(&state, &config);
                }
            })
            .with_context(|| "spawn statistics worker")?;
    }

    drop_privileges_after_socket_binding(
        &config.privileges,
        num_bound_sockets,
        config.socket_workers,
    )
    .unwrap();

    #[cfg(feature = "cpu-pinning")]
    pin_current_if_configured_to(
        &config.cpu_pinning,
        config.socket_workers,
        WorkerIndex::Other,
    );

    loop {
        ::std::thread::sleep(Duration::from_secs(
            config.cleaning.torrent_cleaning_interval,
        ));

        state.torrents.lock().clean(&config, &state.access_list);
    }
}
