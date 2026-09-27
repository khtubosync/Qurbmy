//! A connector lets go of everything it started when it is dropped.
//!
//! A phone builds one for every sync pass and drops it at the end
//! (decision 0020). What it started in the background -- the beacon sender and
//! listener, the loop that keeps reconnecting to the rendezvous service -- was
//! never stopped. Found on a Galaxy S23 a minute after a pass: three beacon
//! listeners, one per pass since the app started, a reconnect loop still
//! looking up a rendezvous address every twenty seconds, and beacons still
//! announcing an address the phone had stopped accepting connections on, so
//! the desktop dialled it every eight seconds for nothing.

use qurb_keys::MasterKey;
use qurb_peer::{Connector, Finding, Identity};
use std::time::{Duration, Instant};

fn alive() -> usize {
    tokio::runtime::Handle::current().metrics().num_alive_tasks()
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_dropped_connector_stops_everything_it_started() {
    let dir = tempfile::tempdir().unwrap();
    let identity = Identity::load_or_create(dir.path()).unwrap();

    // Nothing listens at this address, so the reconnect loop is running rather
    // than finished: the phone's case, whose rendezvous address did not resolve.
    let url = format!("ws://127.0.0.1:{}", free_port());

    let before = alive();
    let connector = Connector::start(
        "127.0.0.1:0".parse().unwrap(),
        identity,
        MasterKey::generate(),
        &qurb_peer::tls::TrustList::default(),
        &url,
        Finding::beacons_on(free_port()),
    )
    .await
    .unwrap();
    let port = connector.local_addr().unwrap().port();
    assert!(alive() > before, "the connector started nothing, so this tests nothing");

    drop(connector);

    // Aborted tasks finish on the runtime's time, not the caller's.
    let deadline = Instant::now() + Duration::from_secs(2);
    while alive() > before && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(alive(), before, "tasks outlived the connector that started them");

    // And the socket it was dialled on is closed, so nothing can go on
    // announcing an address that no longer answers.
    std::net::UdpSocket::bind(("127.0.0.1", port))
        .expect("the dropped connector's socket is still open");
}
