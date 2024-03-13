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

pub struct GW;
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

        let domain = "localhost:6188";
        let subdomain = match hostname.ends_with(domain) {
            true => hostname.trim_end_matches(domain).trim_end_matches('.'),
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

            println!("writen: {:?}", session.response_written());
            return Ok(true);
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let domain = "localhost:6188";

        let localhost = "127.0.0.1".to_string();

        println!("subdomain: {}", ctx.subdomain);

        let addr = match ctx.subdomain.as_str() {
            "crafty" => (localhost, 8443),
            "short" => (format!("s.{domain}"), 443),
            "s" => (localhost, 8001),
            "hompimpa" => (localhost, 8082),
            "notion-note" => (localhost, 8083),
            "scelefeed" => (localhost, 8084),
            "sso" => (localhost, 8085),
            "hahaha" => (localhost, 8086),
            "blog" => (localhost, 4321),
            "odoo" => (localhost, 8069),
            "local" => (localhost, 3000),
            _ => {
                return Err(Error::explain(
                    HTTPStatus(StatusCode::NOT_FOUND.into()),
                    "subdomain not found",
                ))
            }
        };

        let peer = Box::new(HttpPeer::new(addr, false, domain.to_string()));
        Ok(peer)
    }
}

pub struct RP(Arc<LoadBalancer<RoundRobin>>);

fn main() {
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let mut gw = http_proxy_service(&my_server.configuration, GW);

    gw.add_tcp("0.0.0.0:6188");
    my_server.add_service(gw);

    my_server.run_forever();
}
