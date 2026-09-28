//! A device stays reachable across a rendezvous service restarting.
//!
//! Services restart — for a deploy, a reboot, a crash. Before this, a device
//! whose signalling connection went with it stayed silent until the daemon
//! itself was restarted: it kept syncing on its timer, so nothing looked
//! broken, and it simply stopped being reachable and stopped being able to say
//! it had news. Found by restarting the service during a test, which is the
//! only way it was ever going to be found.

use qurb_keys::MasterKey;
use qurb_peer::{Connector, Identity};
use qurb_signal::SignalServer;
use std::sync::Arc;

/// Start a rendezvous service on a chosen port, and return a handle that stops
/// it when dropped.
async fn service_on(port: u16) -> tokio::task::JoinHandle<()> {
    let server = Arc::new(
        SignalServer::bind(format!("127.0.0.1:{port}").parse().unwrap()).await.unwrap(),
    );
    tokio::spawn(async move { server.serve().await })
}

#[tokio::test(flavor = "multi_thread")]
async fn a_device_reconnects_after_the_service_restarts() {
    let dir = tempfile::tempdir().unwrap();
    let master = MasterKey::generate();
    let identity = Identity::load_or_create(dir.path()).unwrap();

    // A port of our choosing, so the service can be stopped and started again
    // at the same address — which is what a restart looks like to a device.
    let port = {
        let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        probe.local_addr().unwrap().port()
    };

    let first = service_on(port).await;
    let url = format!("ws://127.0.0.1:{port}");

    let connector = Connector::start(
        "127.0.0.1:0".parse().unwrap(),
        identity,
        master.clone(),
        &qurb_peer::tls::TrustList::default(),
        &url,
        qurb_peer::Finding::nothing(),
    )
    .await
    .expect("the first connection");

    // Saying something works while the service is up.
    let peer = qurb_peer::Fingerprint::from_bytes([7; 32]);
    connector.tell_waiting(peer).expect("the connection should be live");

    // The service goes away and comes back, as a deploy would.
    first.abort();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let _second = service_on(port).await;

    // Long enough for the backoff's first few attempts.
    tokio::time::sleep(std::time::Duration::from_secs(6)).await;

    // The test is that this still works. Before reconnection existed the
    // signalling task had returned and the channel was dead, so every
    // subsequent attempt to say anything failed for the life of the process.
    connector
        .tell_waiting(peer)
        .expect("the device never reconnected to the rendezvous service");

    // And it can still be introduced, which is the part that matters for
    // actually syncing rather than merely holding a socket.
    let reached = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        connector.reach(peer),
    )
    .await;
    assert!(reached.is_ok(), "asking for an introduction hung after the restart");
}

/// A relay that cannot be reached costs the fallback, not the device.
///
/// It used to stop a device starting at all -- so with a server of one's own
/// down, a phone could not sync with the laptop beside it on the same Wi-Fi,
/// which needs neither the relay nor the server.
#[tokio::test(flavor = "multi_thread")]
async fn a_relay_that_does_not_answer_does_not_stop_a_device() {
    let dir = tempfile::tempdir().unwrap();
    let identity = Identity::load_or_create(dir.path()).unwrap();
    let nobody = {
        let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        probe.local_addr().unwrap()
    };

    let connector = Connector::start(
        "127.0.0.1:0".parse().unwrap(),
        identity,
        MasterKey::generate(),
        &qurb_peer::tls::TrustList::default(),
        "ws://127.0.0.1:1",
        qurb_peer::Finding { stun: false, beacons: None, relay: Some(nobody) },
    )
    .await
    .expect("a relay that does not answer stopped the device starting");
    assert!(connector.relay_endpoint().is_none(), "claims a relay path it does not have");
}

/// When every address fails, the error says how each one did.
///
/// "Could not reach" alone was all the daemon could log when a phone on mobile
/// data was announced and never reached (2026-09-28): nothing to tell a path
/// the networks would not open from a device that answered and refused.
#[tokio::test(flavor = "multi_thread")]
async fn an_unreachable_device_is_reported_address_by_address() {
    let dir = tempfile::tempdir().unwrap();
    let port = {
        let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        probe.local_addr().unwrap().port()
    };
    let _service = service_on(port).await;
    let connector = Connector::start(
        "127.0.0.1:0".parse().unwrap(),
        Identity::load_or_create(dir.path()).unwrap(),
        MasterKey::generate(),
        &qurb_peer::tls::TrustList::default(),
        &format!("ws://127.0.0.1:{port}"),
        qurb_peer::Finding::nothing(),
    )
    .await
    .unwrap();

    // Bound and never read: packets to it vanish, as they do into a network
    // that will not open a path.
    let silent = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = silent.local_addr().unwrap();
    let endpoints = qurb_signal::Endpoints { public: None, local: vec![address] };

    let peer = qurb_peer::Fingerprint::from_bytes([7; 32]);
    let error = connector.race(peer, &endpoints).await.err().expect("nothing answers there");
    let said = error.to_string();
    assert!(said.contains(&format!("{address} timed out")), "{said}");
}
