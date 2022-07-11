use hyper::{Body, Request};
use lifec::{plugins::{Plugin, ThunkContext}, Value};

/// Type for installing a lifec plugin implementation. This plugin makes
/// https requests, with a hyper secure client.
/// 
#[derive(Default)]
pub struct HyperContext;

impl Plugin<ThunkContext> for HyperContext {
    fn symbol() -> &'static str {
        "request"
    }

    fn description() -> &'static str {
        r#"
        Creates a http request, and sends a request with a hyper client. Https only.
        "#
    }

    fn call_with_context(context: &mut ThunkContext) -> Option<lifec::plugins::AsyncContext> {
        context.clone().task(|_| {
            let mut tc = context.clone();
            let block_name = tc.block.block_name.to_string();
            async move {
                if let Some(client) = tc.client() {
                    let mut request = Request::builder();

                    if let Some(method) = tc.as_ref().find_text("method") {
                        request = request.method(method.as_str());
                    }

                    // ex -- define Accept header .text textt/javascript
                    for (name, value) in tc.as_ref().find_symbol_values("header") {
                        let header_name = name.trim_end_matches("::header").to_string();

                        if let Value::TextBuffer(header_value) = value {
                            request = request.header(header_name, header_value);
                        }
                    }

                    if let Some(body) = tc.as_ref().find_binary("body") {
                        match request.body(Body::from(body)) {
                            Ok(request) => match client.request(request).await {
                                Ok(mut resp) => {
                                    match hyper::body::to_bytes(resp.body_mut()).await {
                                        Ok(body) => {
                                            if let Some(project) = tc.project.as_mut() {
                                                *project = project.with_block(
                                                    block_name,
                                                    "response",
                                                    |a| {
                                                        a.add_binary_attr("body", body.to_vec());
                                                    },
                                                );
                                            }
                                        }
                                        Err(_) => {}
                                    }
                                }
                                Err(_) => {}
                            },
                            Err(_) => {}
                        }
                    }
                }

                Some(tc)
            }
        })
    }
}