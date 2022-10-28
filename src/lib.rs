use hyper::{Body, Request};
use lifec::{
    prelude::{AsyncContext, Plugin, ThunkContext, Value},
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

    fn compile(parser: &mut lifec::prelude::AttributeParser) {
        /*
        Example Usage: 
            : .header Accept 
            : Accept .symbol text/json
         */
        parser.add_custom_with("header", |p, c| {
            let child_entity = p.last_child_entity().expect("should have a child entity");

            p.define_child(child_entity, "header", Value::Symbol(c));
        })
    }

    fn call(context: &ThunkContext) -> Option<AsyncContext> {
        context.clone().task(|_| {
            let mut tc = context.clone();
            async move {
                if let Some(client) = tc.client() {
                    let mut request = Request::builder();

                    if let Some(uri) = tc.state().find_symbol("request") {
                        request = request.uri(uri);
                    } else if let Some(uri) = tc.search().find_symbol("uri") {
                        request = request.uri(uri);
                    }

                    if let Some(method) = tc.state().find_symbol("method") {
                        request = request.method(method.as_str());
                    }
              
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
                            Ok(resp) => {
                                tc.cache_response(resp);
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
