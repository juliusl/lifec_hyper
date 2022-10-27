use hyper::{Body, Request};
use lifec::{
    prelude::{AsyncContext, Plugin, ThunkContext},
    state::AttributeIndex,
};

/// Type for installing a lifec plugin implementation. This plugin makes
/// https requests, with a hyper secure client.
///
#[derive(Default)]
pub struct HyperContext;

impl Plugin for HyperContext {
    fn symbol() -> &'static str {
        "request"
    }

    fn description() -> &'static str {
        "Creates a http request, and sends a request with a hyper client. HTTPS only"
    }

    fn call(context: &ThunkContext) -> Option<AsyncContext> {
        context.clone().task(|_| {
            let mut tc = context.clone();
            async move {
                if let Some(client) = tc.client() {
                    let mut request = Request::builder();

                    if let Some(uri) = tc.state().find_text("uri") {
                        request = request.uri(uri);
                    }

                    if let Some(method) = tc.state().find_text("method") {
                        request = request.method(method.as_str());
                    }

                    // ex -- define Accept header .text textt/javascript
                    for name in tc.search().find_symbol_values("header") {
                        if let Some(header_value) = tc.state().find_symbol(&name) {
                            request = request.header(name, header_value);
                        }
                    }

                    let body = tc
                        .state()
                        .find_binary("body")
                        .and_then(|b| Some(Body::from(b)))
                        .unwrap_or(Body::empty());

                    match request.body(Body::from(body)) {
                        Ok(request) => match client.request(request).await {
                            Ok(mut resp) => match hyper::body::to_bytes(resp.body_mut()).await {
                                Ok(body) => {
                                    tc.with_binary("body", body.to_vec());
                                }
                                Err(err) => {
                                    eprintln!("request: error getting body {err}");
                                }
                            },
                            Err(err) => {
                                eprintln!("request: error sending request {err}");
                            }
                        },
                        Err(err) => {
                            eprintln!("request: error creating request {err}");
                        }
                    }
                }

                Some(tc)
            }
        })
    }
}
