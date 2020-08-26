use prometheus_exporter_base::{render_prometheus, MetricType, PrometheusMetric};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    render_prometheus(
        ([0, 0, 0, 0], 32221).into(),
        RwLock::new(HashMap::new()),
        |_request, data| async move {
            {
                let resp = reqwest::get("http://192.168.4.1/accounting/ip.cgi").await?;
                let resp = resp.text();
                let text = resp.await?;
                {
                    let mut write_hash_map = data.write().await;
                    for l in text.lines() {
                        let fields = l.split(" ").collect::<Vec<_>>();
                        let addresses = (
                            fields.get(0).unwrap().to_string(),
                            fields.get(1).unwrap().to_string(),
                        );
                        let value = fields.get(2).unwrap().parse::<u64>().unwrap();
                        *write_hash_map.entry(addresses).or_insert(value) += value;
                    }
                }
            }

            let bytes = PrometheusMetric::new(
                "mikrotik_accounting_bytes",
                MetricType::Counter,
                "Bytes transferred",
            );
            let mut s = bytes.render_header();
            for ((src, dst), value) in data.read().await.iter() {
                let labels: &[(&str, &str)] = &[("SRC-ADDRESS", src), ("DST-ADDRESS", dst)];
                s.push_str(&bytes.render_sample(Some(labels), *value, None));
            }

            Ok(s)
        },
    )
    .await;
}
