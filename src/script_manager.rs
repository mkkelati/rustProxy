use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionScript {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub target_domains: Vec<String>,
    pub inject_type: InjectType,
    pub script_content: String,
    pub headers: HashMap<String, String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InjectType {
    Header,
    Body,
    ResponseHeader,
    ResponseBody,
    JavaScript,
    CSS,
}

#[derive(Debug, Clone)]
pub struct InjectionResult {
    pub modified: bool,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub javascript: Option<String>,
    pub css: Option<String>,
}

pub struct ScriptManager {
    scripts_dir: PathBuf,
    scripts: HashMap<String, InjectionScript>,
}

impl ScriptManager {
    pub fn new<P: AsRef<Path>>(scripts_dir: P) -> Result<Self> {
        let scripts_dir = scripts_dir.as_ref().to_path_buf();
        
        // Create scripts directory if it doesn't exist
        if !scripts_dir.exists() {
            fs::create_dir_all(&scripts_dir)?;
            info!("Created scripts directory: {:?}", scripts_dir);
        }

        let mut manager = ScriptManager {
            scripts_dir,
            scripts: HashMap::new(),
        };

        manager.load_scripts()?;
        manager.create_example_scripts()?;
        
        Ok(manager)
    }

    pub fn load_scripts(&mut self) -> Result<()> {
        self.scripts.clear();
        
        for entry in fs::read_dir(&self.scripts_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_script(&path) {
                    Ok(script) => {
                        info!("Loaded script: {}", script.name);
                        self.scripts.insert(script.name.clone(), script);
                    }
                    Err(e) => {
                        error!("Failed to load script {:?}: {}", path, e);
                    }
                }
            }
        }

        info!("Loaded {} injection scripts", self.scripts.len());
        Ok(())
    }

    fn load_script<P: AsRef<Path>>(&self, path: P) -> Result<InjectionScript> {
        let content = fs::read_to_string(path)?;
        let script: InjectionScript = serde_json::from_str(&content)?;
        Ok(script)
    }

    pub fn list_scripts(&self) -> Vec<String> {
        self.scripts.keys().cloned().collect()
    }

    pub fn get_scripts_for_domain(&self, domain: &str) -> Vec<&InjectionScript> {
        self.scripts
            .values()
            .filter(|script| {
                script.enabled && self.domain_matches(domain, &script.target_domains)
            })
            .collect()
    }

    fn domain_matches(&self, domain: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if pattern == "*" || pattern == domain {
                return true;
            }
            
            if pattern.starts_with("*.") {
                let suffix = &pattern[2..];
                if domain.ends_with(suffix) {
                    return true;
                }
            }
            
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(domain) {
                    return true;
                }
            }
        }
        false
    }

    pub fn apply_request_injections(&self, domain: &str, headers: &mut HashMap<String, String>, body: &mut String) -> Result<InjectionResult> {
        let scripts = self.get_scripts_for_domain(domain);
        let mut result = InjectionResult {
            modified: false,
            headers: None,
            body: None,
            javascript: None,
            css: None,
        };

        for script in scripts {
            match script.inject_type {
                InjectType::Header => {
                    for (key, value) in &script.headers {
                        headers.insert(key.clone(), value.clone());
                        result.modified = true;
                    }
                }
                InjectType::Body => {
                    if !script.script_content.is_empty() {
                        body.push_str(&script.script_content);
                        result.modified = true;
                    }
                }
                InjectType::JavaScript => {
                    result.javascript = Some(script.script_content.clone());
                    result.modified = true;
                }
                InjectType::CSS => {
                    result.css = Some(script.script_content.clone());
                    result.modified = true;
                }
                _ => {} // Response injections handled separately
            }
            
            debug!("Applied script: {} for domain: {}", script.name, domain);
        }

        Ok(result)
    }

    pub fn apply_response_injections(&self, domain: &str, headers: &mut HashMap<String, String>, body: &mut String) -> Result<InjectionResult> {
        let scripts = self.get_scripts_for_domain(domain);
        let mut result = InjectionResult {
            modified: false,
            headers: None,
            body: None,
            javascript: None,
            css: None,
        };

        for script in scripts {
            match script.inject_type {
                InjectType::ResponseHeader => {
                    for (key, value) in &script.headers {
                        headers.insert(key.clone(), value.clone());
                        result.modified = true;
                    }
                }
                InjectType::ResponseBody => {
                    if !script.script_content.is_empty() {
                        // Inject before closing body tag if HTML
                        if body.contains("</body>") {
                            *body = body.replace("</body>", &format!("{}</body>", script.script_content));
                        } else {
                            body.push_str(&script.script_content);
                        }
                        result.modified = true;
                    }
                }
                InjectType::JavaScript => {
                    if body.contains("</head>") {
                        let js_injection = format!("<script>{}</script>", script.script_content);
                        *body = body.replace("</head>", &format!("{}</head>", js_injection));
                        result.modified = true;
                    }
                }
                InjectType::CSS => {
                    if body.contains("</head>") {
                        let css_injection = format!("<style>{}</style>", script.script_content);
                        *body = body.replace("</head>", &format!("{}</head>", css_injection));
                        result.modified = true;
                    }
                }
                _ => {} // Request injections handled separately
            }
        }

        Ok(result)
    }

    fn create_example_scripts(&self) -> Result<()> {
        let examples = vec![
            InjectionScript {
                name: "custom-headers".to_string(),
                description: "Inject custom headers for debugging".to_string(),
                version: "1.0.0".to_string(),
                author: "Rusty Proxy".to_string(),
                target_domains: vec!["*.example.com".to_string()],
                inject_type: InjectType::Header,
                script_content: String::new(),
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("X-Debug".to_string(), "true".to_string());
                    headers.insert("X-Proxy".to_string(), "rusty-proxy".to_string());
                    headers
                },
                enabled: false,
            },
            InjectionScript {
                name: "debug-console".to_string(),
                description: "Inject debug console for web debugging".to_string(),
                version: "1.0.0".to_string(),
                author: "Rusty Proxy".to_string(),
                target_domains: vec!["*".to_string()],
                inject_type: InjectType::JavaScript,
                script_content: r#"
console.log('Rusty Proxy Debug Console Loaded');
window.rustyProxy = {
    version: '0.1.0',
    debug: function(msg) {
        console.log('[RUSTY-PROXY]', msg);
    },
    getInfo: function() {
        return {
            userAgent: navigator.userAgent,
            url: window.location.href,
            timestamp: new Date().toISOString()
        };
    }
};
"#.to_string(),
                headers: HashMap::new(),
                enabled: false,
            },
            InjectionScript {
                name: "cors-bypass".to_string(),
                description: "Add CORS headers to responses".to_string(),
                version: "1.0.0".to_string(),
                author: "Rusty Proxy".to_string(),
                target_domains: vec!["*".to_string()],
                inject_type: InjectType::ResponseHeader,
                script_content: String::new(),
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
                    headers.insert("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string());
                    headers.insert("Access-Control-Allow-Headers".to_string(), "Content-Type, Authorization".to_string());
                    headers
                },
                enabled: false,
            },
        ];

        for script in examples {
            let script_path = self.scripts_dir.join(format!("{}.json", script.name));
            if !script_path.exists() {
                let script_json = serde_json::to_string_pretty(&script)?;
                fs::write(script_path, script_json)?;
                info!("Created example script: {}", script.name);
            }
        }

        Ok(())
    }
}