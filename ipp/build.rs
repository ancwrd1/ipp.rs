fn main() {
    #[cfg(all(feature = "client", feature = "tls"))]
    println!("cargo:rustc-cfg=__sync_tls");

    #[cfg(all(feature = "async-client", feature = "tls"))]
    println!("cargo:rustc-cfg=__async_tls");
}
