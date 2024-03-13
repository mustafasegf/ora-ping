use async_trait::async_trait;
use http::header;
use http::StatusCode;
use pingora::prelude::*;
use pingora_core::protocols::http::SERVER_NAME;
use pingora_http::ResponseHeader;
use std::sync::Arc;

pub struct Context {
    pub subdomain: String,
}

pub struct GW {
    pub domain: String,
}

const LOCALHOST: &str = "127.0.0.1";

#[async_trait]
impl ProxyHttp for GW {
    type CTX = Context;
    fn new_ctx(&self) -> Self::CTX {
        Context {
            subdomain: String::new(),
        }
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        let hostname = session
            .req_header()
            .headers
            .get(header::HOST)
            .map_or("", |x| x.to_str().unwrap_or_default());

        let subdomain = match hostname.ends_with(self.domain.as_str()) {
            true => hostname
                .trim_end_matches(self.domain.as_str())
                .trim_end_matches('.'),
            false => {
                return Err(Error::explain(
                    HTTPStatus(StatusCode::BAD_REQUEST.into()),
                    "invalid domain",
                ))
            }
        };

        ctx.subdomain = subdomain.to_string();

        if subdomain.is_empty() {
            let base = "./static";

            let method = &session.req_header().method;
            if method != http::Method::GET && method != http::Method::HEAD {
                return Err(Error::explain(
                    HTTPStatus(StatusCode::METHOD_NOT_ALLOWED.into()),
                    "method not allowed",
                ));
            }

            let path_to_file = format!("{}{}", base, session.req_header().uri.path());
            let path_to_file = if path_to_file.ends_with('/') {
                format!("{}index.html", path_to_file)
            } else {
                path_to_file
            };

            let file = match std::fs::read(&path_to_file) {
                Ok(file) => file,
                Err(_) => {
                    return Err(Error::explain(
                        HTTPStatus(StatusCode::NOT_FOUND.into()),
                        "file not found",
                    ))
                }
            };

            let content_length = file.len();

            let mut resp = ResponseHeader::build(StatusCode::OK, Some(4)).unwrap();
            resp.insert_header(header::SERVER, &SERVER_NAME[..])
                .unwrap();
            resp.insert_header(header::CONTENT_LENGTH, content_length.to_string())
                .unwrap();
            resp.insert_header(header::CONTENT_TYPE, "text/html")
                .unwrap();

            session.write_response_header(Box::new(resp)).await.unwrap();
            session.set_keepalive(None);

            session.write_response_body(file.into()).await.unwrap();
            return Ok(true);
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let short = format!("s.{}", self.domain);

        let addr = match ctx.subdomain.as_str() {
            "crafty" => (LOCALHOST, 8443),
            "short" => (short.as_str(), 443),
            "s" => (LOCALHOST, 8001),
            "hompimpa" => (LOCALHOST, 8082),
            "notion-note" => (LOCALHOST, 8083),
            "scelefeed" => (LOCALHOST, 8084),
            "sso" => (LOCALHOST, 8085),
            "hahaha" => (LOCALHOST, 8086),
            "blog" => (LOCALHOST, 4321),
            "odoo" => (LOCALHOST, 8069),
            "local" => (LOCALHOST, 3000),
            _ => {
                return Err(Error::explain(
                    HTTPStatus(StatusCode::NOT_FOUND.into()),
                    "subdomain not found",
                ))
            }
        };

        let peer = Box::new(HttpPeer::new(addr, false, self.domain.to_string()));
        Ok(peer)
    }
}

pub struct RP(Arc<LoadBalancer<RoundRobin>>);

fn main() {
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let mut gw = http_proxy_service(
        &my_server.configuration,
        GW {
            domain: "localhost:6188".to_string(),
        },
    );

    gw.add_tcp("0.0.0.0:6188");
    my_server.add_service(gw);

    my_server.run_forever();
}
