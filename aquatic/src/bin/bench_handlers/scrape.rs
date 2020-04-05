use std::time::{Duration, Instant};
use std::net::SocketAddr;

use rand::Rng;
use rand_distr::Pareto;

use aquatic::bench_utils::*;
use aquatic::handler::*;
use aquatic::common::*;

use crate::common::*;


const SCRAPE_REQUESTS: usize = 1_000_000;
const SCRAPE_NUM_HASHES: usize = 10;


pub fn bench(
    rng: &mut impl Rng,
    state: &State,
    info_hashes: &Vec<InfoHash>
){
    println!("# benchmark: handle_scrape_requests\n");
    println!("generating data..");

    let mut responses = Vec::with_capacity(SCRAPE_REQUESTS);

    let mut scrape_requests = create_scrape_requests(rng, &info_hashes);

    let time = Time(Instant::now());

    for (request, src) in scrape_requests.iter() {
        let key = ConnectionKey {
            connection_id: request.connection_id,
            socket_addr: *src,
        };

        state.connections.insert(key, time);
    }

    let scrape_requests = scrape_requests.drain(..);

    ::std::thread::sleep(Duration::from_secs(1));

    let now = Instant::now();

    println!("running benchmark..");

    handle_scrape_requests(
        &state,
        &mut responses,
        scrape_requests,
    );

    let duration = Instant::now() - now;

    println!("\nrequests/second: {:.2}", SCRAPE_REQUESTS as f64 / (duration.as_millis() as f64 / 1000.0));
    println!("time per request: {:.2}ns", duration.as_nanos() as f64 / SCRAPE_REQUESTS as f64);

    let mut total_num_peers = 0.0f64;
    let mut num_responses: usize = 0;

    for (response, _src) in responses.drain(..){
        if let Response::Scrape(response) = response {
            for stats in response.torrent_stats {
                total_num_peers += f64::from(stats.seeders.0);
                total_num_peers += f64::from(stats.leechers.0);
            }

            num_responses += 1;
        }
    }

    if num_responses != SCRAPE_REQUESTS {
        println!("ERROR: only {} responses received", num_responses);
    }

    println!("avg num peers reported: {:.2}", total_num_peers / (SCRAPE_REQUESTS as f64 * SCRAPE_NUM_HASHES as f64));
}


fn create_scrape_requests(
    rng: &mut impl Rng,
    info_hashes: &Vec<InfoHash>
) -> Vec<(ScrapeRequest, SocketAddr)> {
    let pareto = Pareto::new(1., PARETO_SHAPE).unwrap();

    let max_index = info_hashes.len() - 1;

    let mut requests = Vec::new();

    for _ in 0..SCRAPE_REQUESTS {
        let mut request_info_hashes = Vec::new();

        for _ in 0..SCRAPE_NUM_HASHES {
            let info_hash_index = pareto_usize(rng, pareto, max_index);
            request_info_hashes.push(info_hashes[info_hash_index])
        }

        let request = ScrapeRequest {
            connection_id: ConnectionId(rng.gen()),
            transaction_id: TransactionId(rng.gen()),
            info_hashes: request_info_hashes,
        };

        let src = SocketAddr::from(([rng.gen(), rng.gen(), rng.gen(), rng.gen()], rng.gen()));

        requests.push((request, src));
    }

    requests
}