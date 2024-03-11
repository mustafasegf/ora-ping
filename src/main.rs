use async_trait::async_trait;
use http::header;
use http::StatusCode;
use pingora::prelude::*;
use std::sync::Arc;

pub struct GW;
#[async_trait]
impl ProxyHttp for GW {
    type CTX = ();
    fn new_ctx(&self) -> () {
        ()
    }

    async fn upstream_peer(&self, session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let hostname = session
            .req_header()
            .headers
            .get(header::HOST)
            .map_or("", |x| x.to_str().unwrap_or_default());

        let domain = "localhost:6188";

        let subdomain = match hostname.ends_with(domain) {
            true => hostname.trim_end_matches(domain),
            false => {
                return Err(Error::explain(
                    HTTPStatus(StatusCode::BAD_REQUEST.into()),
                    "invalid domain",
                ))
            }
        };
        println!("subdomain: {}", subdomain);
        let localhost = "127.0.0.1".to_string();

        let addr = match subdomain {
            "" => {
                // serve static files. idk how
                (localhost, 3000)
            }
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
            _ => {
                return Err(Error::explain(
                    HTTPStatus(StatusCode::NOT_FOUND.into()),
                    "subdomain not found",
                ))
            }
        };

        let peer = Box::new(HttpPeer::new(addr, false, "".to_string()));
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
