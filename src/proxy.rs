use anyhow::{anyhow, Result};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, Uri};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::http_injector::HttpInjector;
use crate::script_manager::ScriptManager;

pub struct ProxyServer {
    port: u16,
    config: Config,
    injector: Arc<HttpInjector>,
    client: Client<hyper::client::HttpConnector>,
}

impl ProxyServer {
    pub fn new(port: u16, config: Config, script_manager: ScriptManager) -> Self {
        let injector = Arc::new(HttpInjector::new(script_manager, config.clone()));
        let client = Client::new();

        ProxyServer {
            port,
            config,
            injector,
            client,
        }
    }

    pub async fn run(self) -> Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let injector = self.injector.clone();
        let client = self.client.clone();
        let config = Arc::new(self.config.clone());

        let make_svc = make_service_fn(move |_conn| {
            let injector = injector.clone();
            let client = client.clone();
            let config = config.clone();

            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    Self::handle_request(req, injector.clone(), client.clone(), config.clone())
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);

        info!("Rusty Proxy listening on http://{}", addr);
        info!("Proxy configuration:");
        info!("  - Scripts enabled: {}", self.config.scripts.enabled);
        info!("  - Max connections: {}", self.config.proxy.max_connections);
        info!("  - Upstream timeout: {}s", self.config.proxy.upstream_timeout);
        info!("  - Rate limit: {} req/min", self.config.security.rate_limit);

        if let Err(e) = server.await {
            error!("Server error: {}", e);
        }

        Ok(())
    }

    async fn handle_request(
        req: Request<Body>,
        injector: Arc<HttpInjector>,
        client: Client<hyper::client::HttpConnector>,
        config: Arc<Config>,
    ) -> Result<Response<Body>, Infallible> {
        let client_ip = "127.0.0.1"; // In a real implementation, extract from connection
        
        // Check IP whitelist/blacklist
        if !config.is_ip_allowed(client_ip) {
            warn!("Blocked request from IP: {}", client_ip);
            return Ok(injector.create_blocked_response("IP address not allowed"));
        }

        let uri = req.uri().clone();
        let method = req.method().clone();
        
        info!("{} {}", method, uri);
        debug!("Processing request for: {}", uri);

        // Process the request through the injector
        let processed_req = match injector.process_request(req).await {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to process request: {}", e);
                return Ok(injector.create_error_response(&e.to_string()));
            }
        };

        // Handle CONNECT method for HTTPS tunneling
        if processed_req.method() == hyper::Method::CONNECT {
            return Self::handle_connect(processed_req).await;
        }

        // Forward the request to the target server
        let response = match Self::forward_request(processed_req, &client, &config).await {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to forward request: {}", e);
                return Ok(injector.create_error_response(&e.to_string()));
            }
        };

        // Extract domain for response processing
        let domain = uri.host().unwrap_or("unknown").to_string();

        // Process the response through the injector
        let processed_res = match injector.process_response(response, &domain).await {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to process response: {}", e);
                return Ok(injector.create_error_response(&e.to_string()));
            }
        };

        Ok(processed_res)
    }

    async fn forward_request(
        mut req: Request<Body>,
        client: &Client<hyper::client::HttpConnector>,
        config: &Config,
    ) -> Result<Response<Body>> {
        // Ensure the request has a proper scheme
        let uri = req.uri();
        let new_uri = if uri.scheme().is_none() {
            let scheme = if uri.port() == Some(443) { "https" } else { "http" };
            Uri::builder()
                .scheme(scheme)
                .authority(uri.authority().unwrap().as_str())
                .path_and_query(uri.path_and_query().map(|x| x.as_str()).unwrap_or("/"))
                .build()?
        } else {
            uri.clone()
        };

        *req.uri_mut() = new_uri;

        // Set timeout
        let timeout = std::time::Duration::from_secs(config.proxy.upstream_timeout);
        
        // Forward the request
        let response = tokio::time::timeout(timeout, client.request(req)).await??;
        
        Ok(response)
    }

    async fn handle_connect(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        // For HTTPS tunneling, we need to establish a TCP connection
        // This is a simplified implementation
        let uri = req.uri();
        let host_port = uri.authority().map(|auth| auth.as_str()).unwrap_or("");
        
        match Self::establish_tunnel(host_port).await {
            Ok(_) => {
                // Return 200 Connection Established
                let response = Response::builder()
                    .status(200)
                    .body(Body::empty())
                    .unwrap();
                Ok(response)
            }
            Err(e) => {
                error!("Failed to establish tunnel to {}: {}", host_port, e);
                let response = Response::builder()
                    .status(500)
                    .body(Body::from("Failed to establish tunnel"))
                    .unwrap();
                Ok(response)
            }
        }
    }

    async fn establish_tunnel(host_port: &str) -> Result<()> {
        // Parse host and port
        let parts: Vec<&str> = host_port.split(':').collect();
        let host = parts.get(0).ok_or_else(|| anyhow!("Invalid host"))?;
        let port: u16 = parts.get(1).unwrap_or(&"443").parse()?;

        // Establish TCP connection
        let _stream = tokio::net::TcpStream::connect((host.to_string(), port)).await?;
        
        // In a full implementation, you would bridge the client and server connections
        info!("Established tunnel to {}:{}", host, port);
        Ok(())
    }
}