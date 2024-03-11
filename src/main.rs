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

        let addr = match hostname {
            "mus.sh" => {
                // serve static files. idk how
                ("127.0.0.1", 3000)
            }
            "crafty.mus.sh" => ("127.0.0.1", 8443),
            "short.mus.sh" => ("s.mus.sh", 443),
            "s.mus.sh" => ("127.0.0.1", 8001),
            "hompimpa.mus.sh" => ("127.0.0.1", 8082),
            "notion-note.mus.sh" => ("127.0.0.1", 8083),
            "scelefeed.mus.sh" => ("127.0.0.1", 8084),
            "sso.mus.sh" => ("127.0.0.1", 8085),
            "hahaha.mus.sh" => ("127.0.0.1", 8086),
            "blog.mus.sh" => ("127.0.0.1", 4321),
            "odoo.mus.sh" => ("127.0.0.1", 8069),
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
