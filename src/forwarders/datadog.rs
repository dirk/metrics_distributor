//! Reports metrics to Datadog through their HTTPS API.

use chrono::UTC;
use hyper::client::{Client, RequestBuilder};
use hyper::header::ContentType;
use rustc_serialize::json::{self, Json, ToJson};
use std::collections::BTreeMap;

use super::Forwarder;
use super::super::metrics::AggregatedMetrics;

/// Forwards metrics to Datadog via its HTTPS API.
///
/// See [their documentation][] for more details.
///
/// [their documentation]: http://docs.datadoghq.com/api/
pub struct DatadogForwarder {
    pub api_key: String,
    pub base_url: String,
}

impl DatadogForwarder {
    pub fn new(api_key: &str) -> DatadogForwarder {
        DatadogForwarder {
            api_key: api_key.to_owned(),
            base_url: "https://app.datadoghq.com/api".to_owned(),
        }
    }

    fn serialize_metrics(metrics: AggregatedMetrics) -> Json {
        use super::super::metrics::AggregatedMetricType::*;

        let timestamp = UTC::now().timestamp();

        let series: Vec<Json> = metrics
            .iter()
            .map(|metric| {
                let mut object: BTreeMap<String, Json> = BTreeMap::new();

                let (ref metric_type, ref dim, ref value) = *metric;

                let api_type = match *metric_type {
                    Count   => "count",
                    Measure => "gauge",
                    Sample  => "gauge",
                };

                object.insert("metric".to_owned(), dim.name.to_json());
                object.insert("type".to_owned(), api_type.to_json());
                object.insert("points".to_owned(), Json::Array(vec![
                    Json::Array(vec![ timestamp.to_json(), value.to_json() ]),
                ]));

                object.to_json()
            })
            .collect();

        let mut data: BTreeMap<String, Json> = BTreeMap::new();
        data.insert("series".to_owned(), Json::Array(series));

        // Convert it to a `Json::Object`.
        data.to_json()
    }

    fn post<'a>(&'a self, client: &'a Client, path: &str) -> RequestBuilder {
        let path = format!("{}{}?api_key={}", self.base_url, path, self.api_key);

        client.post(&path)
            .header(ContentType::json())
    }
}

impl Forwarder for DatadogForwarder {
    fn forward_metrics(&self, metrics: AggregatedMetrics) {
        let body = json::encode(&DatadogForwarder::serialize_metrics(metrics)).unwrap();
        let client = Client::new();

        let res = self.post(&client, "/v1/series")
            .body(&body)
            .send();

        match res {
            Err(err) => {
                println!("Datadog HTTP Error: {:#?}", err)
            },
            Ok(res) => {
                if !res.status.is_success() {
                    println!("Datadog API Error: {:#?}", res);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DatadogForwarder;
    use super::super::super::metrics::{
        AggregatedMetrics,
        AggregatedMetricType,
    };

    use rustc_serialize::json::ToJson;

    #[test]
    fn datadog_forwarder_serializes_metrics() {
        let metrics = AggregatedMetrics::with_metrics(vec![
            (AggregatedMetricType::Count, "test_count".to_owned(), 1.0),
        ]);
        let json = DatadogForwarder::serialize_metrics(metrics);

        let series = json.find("series").and_then(|s| s.as_array());
        assert_eq!(series.is_some(), true);

        let series = series.unwrap();
        assert_eq!(series.len(), 1);

        let item = series[0].as_object().unwrap();
        assert_eq!(item.get("metric"), Some(&"test_count".to_json()));
        assert_eq!(item.get("type"),   Some(&"count".to_json()));

        let points = item.get("points").unwrap().as_array().unwrap();
        assert_eq!(points.len(), 1);
        let point = points[0].as_array().unwrap();
        assert_eq!(point.len(), 2);
        let ref value = point[1];
        assert_eq!(value, &1.0.to_json());
    }
}
