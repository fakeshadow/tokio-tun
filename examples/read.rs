use std::{net::Ipv4Addr, os::unix::io::AsRawFd};

use pnet::packet::{
    icmp::{IcmpPacket, IcmpType},
    ipv4::Ipv4Packet,
    Packet,
};
use tokio::io::AsyncReadExt;
use tokio_tun::{Builder, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut tun = Builder::new()
        .name("")
        .tap(false)
        .packet_info(false)
        .mtu(1350)
        .up()
        .address(Ipv4Addr::new(10, 21, 22, 1))
        .destination(Ipv4Addr::new(10, 22, 23, 1))
        .broadcast(Ipv4Addr::BROADCAST)
        .netmask(Ipv4Addr::new(255, 255, 255, 0))
        .build()?;

    println!("-----------");
    println!("tun created");
    println!("-----------");

    println!(
        "┌ name: {}\n├ fd: {}\n├ mtu: {}\n├ flags: {}\n├ address: {}\n├ destination: {}\n├ broadcast: {}\n└ netmask: {}",
        tun.name(),
        tun.as_raw_fd(),
        tun.mtu().unwrap(),
        tun.flags().unwrap(),
        tun.address().unwrap(),
        tun.destination().unwrap(),
        tun.broadcast().unwrap(),
        tun.netmask().unwrap(),
    );

    println!("---------------------");
    println!("ping 10.22.23.2 to test");
    println!("---------------------");

    let mut buf = [0u8; 1024];
    loop {
        let n = tun.read(&mut buf).await?;

        if let Some(ip) = Ipv4Packet::new(&mut buf[..n]) {
            if let Some(icmp) = IcmpPacket::new(ip.payload()) {
                if icmp.get_icmp_type() == IcmpType::new(8) {
                    println!("{icmp:?}");
                }
            }
        }
    }
}
