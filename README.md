# Rusty Proxy

A high-performance HTTP proxy script manager written in Rust for traffic injection and manipulation. Designed for network testing, debugging, and development purposes.

## Features

- 🚀 **High Performance**: Built with Rust and Tokio for maximum performance
- 🔧 **Script Management**: Dynamic loading and execution of injection scripts
- 🛡️ **Security**: IP whitelisting, rate limiting, and domain filtering
- 🔄 **HTTP Injection**: Support for header, body, JavaScript, and CSS injection
- ⚙️ **Configurable**: TOML-based configuration with hot reloading
- 🐧 **Ubuntu Ready**: Easy installation with systemd service integration
- 📝 **Comprehensive Logging**: Structured logging with configurable levels

## Quick Start

### Prerequisites

- Ubuntu 18.04+ (other Linux distributions may work but are not officially supported)
- Internet connection for downloading Rust and dependencies

### Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/rusty-proxy.git
   cd rusty-proxy
   ```

2. **Run the installation script:**
   ```bash
   chmod +x install.sh
   sudo ./install.sh
   ```

3. **Start the service:**
   ```bash
   sudo systemctl start rusty-proxy
   sudo systemctl enable rusty-proxy
   ```

4. **Configure your browser or application to use the proxy:**
   - Proxy address: `localhost`
   - Proxy port: `8080`

### Manual Installation

If you prefer to install manually:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Build the application
cargo build --release

# Run the proxy
./target/release/rusty-proxy start
```

## Configuration

The configuration file is located at `/etc/rusty-proxy/config.toml` (system installation) or `config.toml` (manual installation).

### Basic Configuration

```toml
[proxy]
bind_address = "127.0.0.1"  # Listen address
port = 8080                 # Listen port
upstream_timeout = 30       # Upstream request timeout in seconds
max_connections = 1000      # Maximum concurrent connections
buffer_size = 8192         # Buffer size for data transfer

[scripts]
directory = "scripts"       # Directory containing injection scripts
enabled = true             # Enable/disable script execution
max_execution_time = 5000  # Maximum script execution time in ms
allowed_domains = ["*"]    # Domains where scripts can run
blocked_domains = []       # Explicitly blocked domains

[logging]
level = "info"             # Log level: trace, debug, info, warn, error
file = "rusty-proxy.log"   # Log file path
max_size = "10MB"          # Maximum log file size
max_files = 5              # Number of log files to keep

[security]
require_auth = false       # Require authentication
auth_token = ""           # Authentication token (if required)
rate_limit = 100          # Requests per minute per IP
whitelist_ips = []        # Allowed IP addresses (empty = allow all)
blacklist_ips = []        # Blocked IP addresses
```

## Injection Scripts

Rusty Proxy supports various types of injection scripts for modifying HTTP traffic:

### Script Format

Scripts are JSON files placed in the `scripts` directory:

```json
{
  "name": "example-script",
  "description": "Example injection script",
  "version": "1.0.0",
  "author": "Your Name",
  "target_domains": ["example.com", "*.example.org"],
  "inject_type": "JavaScript",
  "script_content": "console.log('Injected by Rusty Proxy');",
  "headers": {},
  "enabled": true
}
```

### Injection Types

1. **Header**: Inject custom HTTP headers into requests
2. **Body**: Inject content into request body
3. **ResponseHeader**: Inject headers into responses
4. **ResponseBody**: Inject content into response body
5. **JavaScript**: Inject JavaScript code into HTML pages
6. **CSS**: Inject CSS styles into HTML pages

### Example Scripts

#### Debug Console Injection
```json
{
  "name": "debug-console",
  "description": "Inject debug console for web debugging",
  "version": "1.0.0",
  "author": "Rusty Proxy",
  "target_domains": ["*"],
  "inject_type": "JavaScript",
  "script_content": "console.log('Debug mode enabled'); window.rustyProxy = { debug: true };",
  "headers": {},
  "enabled": false
}
```

#### CORS Headers
```json
{
  "name": "cors-bypass",
  "description": "Add CORS headers to responses",
  "version": "1.0.0",
  "author": "Rusty Proxy",
  "target_domains": ["*"],
  "inject_type": "ResponseHeader",
  "script_content": "",
  "headers": {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, OPTIONS"
  },
  "enabled": false
}
```

## Usage

### Command Line Interface

```bash
# Start the proxy server
rusty-proxy start

# List available scripts
rusty-proxy list-scripts

# Use custom configuration
rusty-proxy --config /path/to/config.toml start

# Use custom port
rusty-proxy --port 9090 start

# Install as system service
rusty-proxy install
```

### Management Commands (System Installation)

```bash
# Start the service
rusty-proxy-start

# Stop the service
rusty-proxy-stop

# Check service status
rusty-proxy-status

# View logs in real-time
rusty-proxy-logs
```

### Browser Configuration

#### Firefox
1. Open Settings → General → Network Settings
2. Select "Manual proxy configuration"
3. HTTP Proxy: `localhost`, Port: `8080`
4. Check "Use this proxy server for all protocols"

#### Chrome
```bash
google-chrome --proxy-server="localhost:8080"
```

#### System-wide (Ubuntu)
```bash
export http_proxy=http://localhost:8080
export https_proxy=http://localhost:8080
```

## Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/yourusername/rusty-proxy.git
cd rusty-proxy

# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run start
```

### Project Structure

```
rusty-proxy/
├── src/
│   ├── main.rs           # Application entry point
│   ├── config.rs         # Configuration management
│   ├── proxy.rs          # Core proxy server
│   ├── script_manager.rs # Script loading and execution
│   └── http_injector.rs  # HTTP traffic modification
├── scripts/              # Injection scripts directory
├── config.toml          # Default configuration
├── install.sh           # Ubuntu installation script
└── Cargo.toml          # Rust dependencies
```

## Troubleshooting

### Common Issues

1. **Permission Denied**: Make sure the service user has proper permissions
   ```bash
   sudo chown -R rusty-proxy:rusty-proxy /opt/rusty-proxy
   ```

2. **Port Already in Use**: Change the port in configuration or stop conflicting services
   ```bash
   sudo netstat -tlnp | grep :8080
   ```

3. **Scripts Not Loading**: Check script directory permissions and JSON syntax
   ```bash
   sudo -u rusty-proxy ls -la /opt/rusty-proxy/scripts/
   ```

4. **Service Won't Start**: Check the logs for detailed error messages
   ```bash
   sudo journalctl -u rusty-proxy -n 50
   ```

### Log Locations

- System installation: `/var/log/rusty-proxy/rusty-proxy.log`
- Manual installation: `./rusty-proxy.log`
- Systemd journal: `journalctl -u rusty-proxy`

## Security Considerations

⚠️ **Important Security Notes:**

- This tool is designed for **development and testing purposes**
- **Do not use in production** without proper security review
- Be careful when enabling script injection on untrusted domains
- Always use IP whitelisting in sensitive environments
- Regularly update and review injection scripts
- Monitor logs for suspicious activity

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues and questions:
- Create an issue on GitHub
- Check the troubleshooting section
- Review the logs for error messages

## Changelog

### v0.1.0
- Initial release
- Basic HTTP proxy functionality
- Script injection system
- Ubuntu installation support
- Systemd service integration