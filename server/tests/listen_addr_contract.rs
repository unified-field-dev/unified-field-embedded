//! Integration contracts for host listen address.

use std::net::SocketAddr;

use server::listen_addr;

#[test]
fn listen_addr_parses_loopback_happy_path() {
    std::env::remove_var("LEPTOS_SITE_ADDR");
    std::env::set_var("SITE_ADDR", "127.0.0.1:3999");
    let addr = listen_addr().expect("valid SITE_ADDR");
    assert_eq!(addr, "127.0.0.1:3999".parse::<SocketAddr>().unwrap());
    std::env::remove_var("SITE_ADDR");
}

#[test]
fn listen_addr_default_loopback_when_unset() {
    std::env::remove_var("SITE_ADDR");
    std::env::remove_var("LEPTOS_SITE_ADDR");
    let addr = listen_addr().expect("default listen");
    assert_eq!(addr, SocketAddr::from(([127, 0, 0, 1], 3000)));
}

#[test]
fn listen_addr_garbage_env_errors_sad() {
    std::env::remove_var("LEPTOS_SITE_ADDR");
    std::env::set_var("SITE_ADDR", ":::bad");
    assert!(
        listen_addr().is_err(),
        "malformed SITE_ADDR must fail closed"
    );
    std::env::remove_var("SITE_ADDR");
}
