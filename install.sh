#!/bin/bash

# Rusty Proxy Ubuntu Installation Script
# This script installs Rusty Proxy as a system service on Ubuntu

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SERVICE_NAME="rusty-proxy"
INSTALL_DIR="/opt/rusty-proxy"
SERVICE_USER="rusty-proxy"
CONFIG_DIR="/etc/rusty-proxy"
LOG_DIR="/var/log/rusty-proxy"
BINARY_NAME="rusty-proxy"

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

install_dependencies() {
    print_status "Installing dependencies..."
    
    # Update package list
    apt-get update
    
    # Install required packages
    apt-get install -y curl build-essential pkg-config libssl-dev
    
    print_success "Dependencies installed"
}

install_rust() {
    if command -v rustc &> /dev/null; then
        print_status "Rust is already installed"
        return
    fi
    
    print_status "Installing Rust..."
    
    # Install Rust using rustup (for the service user)
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    
    print_success "Rust installed"
}

create_user() {
    if id "$SERVICE_USER" &>/dev/null; then
        print_status "User $SERVICE_USER already exists"
    else
        print_status "Creating service user: $SERVICE_USER"
        useradd --system --shell /bin/false --home-dir $INSTALL_DIR --create-home $SERVICE_USER
        print_success "Service user created"
    fi
}

create_directories() {
    print_status "Creating directories..."
    
    mkdir -p $INSTALL_DIR
    mkdir -p $CONFIG_DIR
    mkdir -p $LOG_DIR
    mkdir -p $INSTALL_DIR/scripts
    
    chown -R $SERVICE_USER:$SERVICE_USER $INSTALL_DIR
    chown -R $SERVICE_USER:$SERVICE_USER $LOG_DIR
    
    print_success "Directories created"
}

build_application() {
    print_status "Building Rusty Proxy..."
    
    # Copy source files to install directory
    cp -r * $INSTALL_DIR/
    cd $INSTALL_DIR
    
    # Build the application
    sudo -u $SERVICE_USER bash -c 'source ~/.cargo/env && cargo build --release'
    
    # Copy binary to system location
    cp target/release/$BINARY_NAME /usr/local/bin/
    chmod +x /usr/local/bin/$BINARY_NAME
    
    print_success "Application built and installed"
}

create_config() {
    print_status "Creating default configuration..."
    
    cat > $CONFIG_DIR/config.toml << EOF
[proxy]
bind_address = "0.0.0.0"
port = 8080
upstream_timeout = 30
max_connections = 1000
buffer_size = 8192

[scripts]
directory = "$INSTALL_DIR/scripts"
enabled = true
max_execution_time = 5000
allowed_domains = ["*"]
blocked_domains = []

[logging]
level = "info"
file = "$LOG_DIR/rusty-proxy.log"
max_size = "10MB"
max_files = 5

[security]
require_auth = false
auth_token = ""
rate_limit = 100
whitelist_ips = []
blacklist_ips = []
EOF

    chown $SERVICE_USER:$SERVICE_USER $CONFIG_DIR/config.toml
    print_success "Configuration file created at $CONFIG_DIR/config.toml"
}

create_service() {
    print_status "Creating systemd service..."
    
    cat > /etc/systemd/system/$SERVICE_NAME.service << EOF
[Unit]
Description=Rusty Proxy HTTP Injector
Documentation=https://github.com/yourusername/rusty-proxy
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_USER
WorkingDirectory=$INSTALL_DIR
ExecStart=/usr/local/bin/$BINARY_NAME --config $CONFIG_DIR/config.toml start
ExecReload=/bin/kill -HUP \$MAINPID
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=$SERVICE_NAME

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$LOG_DIR $INSTALL_DIR/scripts
CapabilityBoundingSet=CAP_NET_BIND_SERVICE

# Environment
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    print_success "Systemd service created"
}

configure_firewall() {
    if command -v ufw &> /dev/null; then
        print_status "Configuring firewall..."
        ufw allow 8080/tcp comment "Rusty Proxy"
        print_success "Firewall configured"
    else
        print_warning "UFW not found, please configure firewall manually for port 8080"
    fi
}

create_management_scripts() {
    print_status "Creating management scripts..."
    
    # Start script
    cat > /usr/local/bin/rusty-proxy-start << EOF
#!/bin/bash
systemctl start $SERVICE_NAME
systemctl status $SERVICE_NAME
EOF

    # Stop script
    cat > /usr/local/bin/rusty-proxy-stop << EOF
#!/bin/bash
systemctl stop $SERVICE_NAME
EOF

    # Status script
    cat > /usr/local/bin/rusty-proxy-status << EOF
#!/bin/bash
systemctl status $SERVICE_NAME
EOF

    # Logs script
    cat > /usr/local/bin/rusty-proxy-logs << EOF
#!/bin/bash
journalctl -u $SERVICE_NAME -f
EOF

    chmod +x /usr/local/bin/rusty-proxy-*
    print_success "Management scripts created"
}

main() {
    print_status "Starting Rusty Proxy installation..."
    
    check_root
    install_dependencies
    install_rust
    create_user
    create_directories
    build_application
    create_config
    create_service
    configure_firewall
    create_management_scripts
    
    print_success "Installation completed successfully!"
    echo
    print_status "Next steps:"
    echo "  1. Edit configuration: sudo nano $CONFIG_DIR/config.toml"
    echo "  2. Start service: sudo systemctl start $SERVICE_NAME"
    echo "  3. Enable auto-start: sudo systemctl enable $SERVICE_NAME"
    echo "  4. Check status: sudo systemctl status $SERVICE_NAME"
    echo "  5. View logs: sudo journalctl -u $SERVICE_NAME"
    echo
    print_status "Management commands:"
    echo "  - rusty-proxy-start: Start the service"
    echo "  - rusty-proxy-stop: Stop the service"
    echo "  - rusty-proxy-status: Check service status"
    echo "  - rusty-proxy-logs: View service logs"
    echo
    print_status "The proxy will be available at http://localhost:8080"
    print_warning "Remember to configure your browser or applications to use this proxy!"
}

# Run main function
main "$@"