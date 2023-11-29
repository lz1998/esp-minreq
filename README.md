# esp-minreq
Async HTTP Client

```toml
esp-minreq = { git = "https://github.com/lz1998/esp-minreq.git", branch = "main", features = ["json"] }
```

```rust
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Default, Debug)]
    #[serde(default)]
    struct Response {
        pub ip: String,
        pub ip_decimal: i64,
        pub country: String,
        pub country_iso: String,
        pub country_eu: bool,
        pub region_name: String,
        pub region_code: String,
        pub city: String,
        pub latitude: f64,
        pub longitude: f64,
        pub time_zone: String,
        pub asn: String,
        pub asn_org: String,
    }
    let resp: Response = esp_minreq::get("https://ifconfig.co/json")
        .send::<esp_minreq::tcp::HttpStream>()
        .await
        .unwrap()
        .json()
        .unwrap();
    log::info!("{:?}", resp);
```
