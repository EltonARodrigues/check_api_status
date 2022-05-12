
use reqwest::{header::HeaderMap, header::HeaderName, Client, Method, Response, Url};
use serde_json::Value;
use std::collections::HashMap;

use crate::utils::yarn::{ConfigMethod, Depends, ReqHash, Request};
use crate::utils::notify::send_notify;

use crate::Value::Null;

// pub mod requests {

pub async fn request_api(
    request: &Request,
    headers: HeaderMap,
    body: &Option<ReqHash>,
) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;

    // Missing all methods
    let method: &'static [u8] = if request.method == ConfigMethod::POST {
        b"POST"
    } else {
        b"GET"
    };

    // TODO
    let resp: Response;

    resp = match body {
        Some(req_body) => {
            client
                .request(Method::from_bytes(method)?, Url::parse(&request.url)?)
                .headers(headers)
                .json(&req_body)
                .send()
                .await?
        }
        None => {
            client
                .request(Method::from_bytes(method)?, Url::parse(&request.url)?)
                .headers(headers)
                .send()
                .await?
        }
    };

    Ok(resp)
}

fn get_field<'a>(value: &'a Value, fields: Vec<&str>, size: usize, start: usize) -> &'a Value {
    let next_value = &value[fields[start]];

    if start == (size - 1) {
        return next_value;
    }

    return get_field(next_value, fields, size, start + 1);
}

pub async fn get_depends_result(depends: &Depends, system_notify: bool) -> HashMap<String, Vec<ReqHash>> {
    let mut headers_map = HeaderMap::new();

    for header in &depends.request.headers {
        for (key, value) in header {
            headers_map.insert(
                HeaderName::from_lowercase(key.as_bytes()).unwrap(),
                value.parse().unwrap(),
            );
        }
    }
    let response = request_api(&depends.request, headers_map, &depends.request.body).await;

    return match response {
        Ok(r) => {
            let body = r.text().await;

            let result: Value = match body {
                Ok(r) => {
                    let body_json = serde_json::from_str(&r).unwrap();
                    body_json
                }
                Err(e) => panic!("TODO: {:?}", e),
            };

            let mut depends_headers: Vec<ReqHash> = Vec::new();

            for header_field in &depends.header_fields {
                let fields: Vec<_> = header_field.split('.').collect();
                let size = fields.len();
                let result_field = get_field(&result, fields, size, 0);

                let mut fields_values = HashMap::new();
                if result_field != &Null {
                    fields_values.insert(header_field.to_string(), result_field.to_string());
                }

                depends_headers.push(fields_values);
            }

            let mut depends_body: Vec<ReqHash> = Vec::new();

            for body_field in &depends.body_fields {
                let fields: Vec<_> = body_field.split('.').collect();
                let size = fields.len();
                let result_field = get_field(&result, fields, size, 0);

                let mut fields_values = HashMap::new();
                if result_field != &Null {
                    fields_values.insert(
                        body_field.to_string(),
                        serde_json::to_string(result_field).unwrap(),
                    );
                }

                depends_body.push(fields_values);
            }

            let mut depends_results: HashMap<String, Vec<ReqHash>> = HashMap::new();
            depends_results.insert("depends_body".to_string(), depends_body);
            depends_results.insert("depends_headers".to_string(), depends_headers);

            depends_results
        }
        Err(_) => {
            if system_notify == true {
                let msg = format!("Error to request depends");

                send_notify(
                    depends.name.as_str(),
                    "dialog-error",
                    &msg,
                )
                .unwrap();
            }
            HashMap::new()
        }
    };
}
// }
