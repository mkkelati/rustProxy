use anyhow::Result;
use hyper::{Request, Response, Body, Uri, Method};
use hyper::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{debug, error, info, warn};
use crate::script_manager::{ScriptManager, InjectionResult};
use crate::config::Config;

pub struct HttpInjector {
    script_manager: ScriptManager,
    config: Config,
}

impl HttpInjector {
    pub fn new(script_manager: ScriptManager, config: Config) -> Self {
        HttpInjector {
            script_manager,
            config,
        }
    }

    pub async fn process_request(&self, mut req: Request<Body>) -> Result<Request<Body>> {
        let uri = req.uri().clone();
        let domain = self.extract_domain(&uri);
        
        if !self.config.is_domain_allowed(&domain) {
            warn!("Domain {} is not allowed", domain);
            return Ok(req);
        }

        // Convert headers to HashMap for easier manipulation
        let mut headers_map = self.headers_to_map(req.headers());
        let mut body_string = String::new();

        // Read body if present
        if req.method() == Method::POST || req.method() == Method::PUT {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
            body_string = String::from_utf8_lossy(&body_bytes).to_string();
        }

        // Apply request injections
        if self.config.scripts.enabled {
            match self.script_manager.apply_request_injections(&domain, &mut headers_map, &mut body_string) {
                Ok(injection_result) => {
                    if injection_result.modified {
                        info!("Applied request injections for domain: {}", domain);
                    }
                }
                Err(e) => {
                    error!("Failed to apply request injections: {}", e);
                }
            }
        }

        // Rebuild request with modified headers
        let (mut parts, _) = Request::from(req).into_parts();
        parts.headers = self.map_to_headers(&headers_map)?;
        
        let new_body = if body_string.is_empty() {
            Body::empty()
        } else {
            Body::from(body_string)
        };

        Ok(Request::from_parts(parts, new_body))
    }

    pub async fn process_response(&self, mut res: Response<Body>, domain: &str) -> Result<Response<Body>> {
        if !self.config.is_domain_allowed(domain) {
            return Ok(res);
        }

        // Convert headers to HashMap for easier manipulation
        let mut headers_map = self.headers_to_map(res.headers());
        
        // Read response body
        let body_bytes = hyper::body::to_bytes(res.into_body()).await?;
        let mut body_string = String::from_utf8_lossy(&body_bytes).to_string();

        // Apply response injections
        if self.config.scripts.enabled {
            match self.script_manager.apply_response_injections(domain, &mut headers_map, &mut body_string) {
                Ok(injection_result) => {
                    if injection_result.modified {
                        info!("Applied response injections for domain: {}", domain);
                        
                        // Update content length if body was modified
                        headers_map.insert("content-length".to_string(), body_string.len().to_string());
                    }
                }
                Err(e) => {
                    error!("Failed to apply response injections: {}", e);
                }
            }
        }

        // Rebuild response with modified headers and body
        let (mut parts, _) = Response::from(res).into_parts();
        parts.headers = self.map_to_headers(&headers_map)?;
        
        Ok(Response::from_parts(parts, Body::from(body_string)))
    }

    fn extract_domain(&self, uri: &Uri) -> String {
        uri.host().unwrap_or("unknown").to_string()
    }

    fn headers_to_map(&self, headers: &HeaderMap) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (name, value) in headers {
            if let Ok(value_str) = value.to_str() {
                map.insert(name.as_str().to_lowercase(), value_str.to_string());
            }
        }
        map
    }

    fn map_to_headers(&self, map: &HashMap<String, String>) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        for (name, value) in map {
            let header_name = HeaderName::from_str(name)?;
            let header_value = HeaderValue::from_str(value)?;
            headers.insert(header_name, header_value);
        }
        Ok(headers)
    }

    pub fn create_blocked_response(&self, reason: &str) -> Response<Body> {
        let body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Blocked by Rusty Proxy</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .error {{ color: #d32f2f; }}
        .info {{ color: #1976d2; }}
    </style>
</head>
<body>
    <h1 class="error">Access Blocked</h1>
    <p class="info">This request was blocked by Rusty Proxy.</p>
    <p><strong>Reason:</strong> {}</p>
    <p><em>Powered by Rusty Proxy v0.1.0</em></p>
</body>
</html>"#,
            reason
        );

        Response::builder()
            .status(403)
            .header("content-type", "text/html")
            .header("content-length", body.len())
            .body(Body::from(body))
            .unwrap()
    }

    pub fn create_error_response(&self, error: &str) -> Response<Body> {
        let body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Proxy Error</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .error {{ color: #d32f2f; }}
    </style>
</head>
<body>
    <h1 class="error">Proxy Error</h1>
    <p>An error occurred while processing your request:</p>
    <p><strong>{}</strong></p>
    <p><em>Powered by Rusty Proxy v0.1.0</em></p>
</body>
</html>"#,
            error
        );

        Response::builder()
            .status(500)
            .header("content-type", "text/html")
            .header("content-length", body.len())
            .body(Body::from(body))
            .unwrap()
    }
}