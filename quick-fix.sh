#!/bin/bash

# Quick Fix for Menu Command Issues
# Run this script to fix common menu problems

echo "🔧 Rusty Proxy Menu Quick Fix"
echo "============================="
echo

# Check if we're root
if [[ $EUID -ne 0 ]]; then
    echo "❌ This script must be run as root"
    echo "   Run: sudo ./quick-fix.sh"
    exit 1
fi

echo "🔍 Diagnosing and fixing menu issues..."
echo

# Create the menu scripts if they don't exist
if [[ ! -f "/usr/local/bin/rusty-proxy-menu" ]]; then
    echo "📝 Creating main menu script..."
    
    cat > /usr/local/bin/rusty-proxy-menu << 'EOF'
#!/bin/bash

# Service configuration
SERVICE_NAME="rusty-proxy"
CONFIG_DIR="/etc/rusty-proxy"
INSTALL_DIR="/opt/rusty-proxy"
LOG_DIR="/var/log/rusty-proxy"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

show_banner() {
    clear
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║                    🦀 RUSTY PROXY 🦀                    ║"
    echo "║              HTTP Injector Script Manager               ║"
    echo "║                      Version 0.1.0                      ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo ""
}

show_status() {
    echo -e "${BLUE}━━━ SERVICE STATUS ━━━${NC}"
    if systemctl is-active --quiet $SERVICE_NAME; then
        echo -e "${GREEN}✓ Service Status: RUNNING${NC}"
    else
        echo -e "${RED}✗ Service Status: STOPPED${NC}"
    fi
    
    if systemctl is-enabled --quiet $SERVICE_NAME; then
        echo -e "${GREEN}✓ Auto-start: ENABLED${NC}"
    else
        echo -e "${YELLOW}⚠ Auto-start: DISABLED${NC}"
    fi
    
    echo -e "${BLUE}Port: 8080${NC}"
    echo -e "${BLUE}Config: $CONFIG_DIR/config.toml${NC}"
    echo -e "${BLUE}Logs: $LOG_DIR/rusty-proxy.log${NC}"
    echo ""
}

show_menu() {
    echo -e "${PURPLE}━━━ MAIN MENU ━━━${NC}"
    echo -e "${GREEN}1.${NC} Start Service"
    echo -e "${GREEN}2.${NC} Stop Service"
    echo -e "${GREEN}3.${NC} Restart Service"
    echo -e "${GREEN}4.${NC} View Service Status"
    echo -e "${GREEN}5.${NC} View Live Logs"
    echo -e "${GREEN}6.${NC} Edit Configuration"
    echo -e "${GREEN}7.${NC} List Injection Scripts"
    echo -e "${GREEN}8.${NC} Enable/Disable Scripts"
    echo -e "${GREEN}9.${NC} Test Proxy Connection"
    echo -e "${GREEN}10.${NC} View Documentation"
    echo -e "${GREEN}11.${NC} Uninstall Rusty Proxy"
    echo -e "${RED}0.${NC} Exit"
    echo ""
}

handle_choice() {
    case $1 in
        1)
            echo -e "${BLUE}Starting Rusty Proxy...${NC}"
            systemctl start $SERVICE_NAME
            systemctl status $SERVICE_NAME --no-pager -l
            ;;
        2)
            echo -e "${BLUE}Stopping Rusty Proxy...${NC}"
            systemctl stop $SERVICE_NAME
            echo -e "${GREEN}Service stopped${NC}"
            ;;
        3)
            echo -e "${BLUE}Restarting Rusty Proxy...${NC}"
            systemctl restart $SERVICE_NAME
            systemctl status $SERVICE_NAME --no-pager -l
            ;;
        4)
            systemctl status $SERVICE_NAME --no-pager -l
            ;;
        5)
            echo -e "${BLUE}Viewing live logs (Press Ctrl+C to exit)...${NC}"
            journalctl -u $SERVICE_NAME -f
            ;;
        6)
            echo -e "${BLUE}Opening configuration file...${NC}"
            ${EDITOR:-nano} $CONFIG_DIR/config.toml
            echo -e "${YELLOW}Restart service to apply changes${NC}"
            ;;
        7)
            echo -e "${BLUE}Available injection scripts:${NC}"
            ls -la $INSTALL_DIR/scripts/
            ;;
        8)
            echo -e "${BLUE}Script management:${NC}"
            echo "Scripts are located in: $INSTALL_DIR/scripts/"
            echo "Edit script files to enable/disable (set 'enabled': true/false)"
            ;;
        9)
            echo -e "${BLUE}Testing proxy connection...${NC}"
            if curl -x localhost:8080 -s -o /dev/null -w "%{http_code}" http://httpbin.org/ip | grep -q "200"; then
                echo -e "${GREEN}✓ Proxy is working correctly${NC}"
            else
                echo -e "${RED}✗ Proxy test failed${NC}"
            fi
            ;;
        10)
            echo -e "${BLUE}Opening documentation...${NC}"
            echo "Documentation: https://github.com/mkkelati/rustProxy"
            echo "Local README: $INSTALL_DIR/README.md"
            ;;
        11)
            echo -e "${RED}Uninstalling Rusty Proxy...${NC}"
            read -p "Are you sure? (y/N): " confirm
            if [[ $confirm =~ ^[Yy]$ ]]; then
                systemctl stop $SERVICE_NAME
                systemctl disable $SERVICE_NAME
                rm -f /etc/systemd/system/$SERVICE_NAME.service
                rm -rf $INSTALL_DIR
                rm -rf $CONFIG_DIR
                rm -f /usr/local/bin/rusty-proxy*
                rm -f /usr/local/bin/menu
                echo -e "${GREEN}Rusty Proxy uninstalled${NC}"
                exit 0
            fi
            ;;
        0)
            echo -e "${CYAN}Thanks for using Rusty Proxy! 🦀${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}Invalid option. Please try again.${NC}"
            ;;
    esac
}

# Main menu loop
while true; do
    show_banner
    show_status
    show_menu
    read -p "Enter your choice [0-11]: " choice
    echo ""
    handle_choice $choice
    echo ""
    read -p "Press Enter to continue..."
done
EOF

    chmod +x /usr/local/bin/rusty-proxy-menu
    echo "✓ Main menu script created"
else
    echo "✓ Main menu script already exists"
fi

# Create the menu shortcut
if [[ ! -f "/usr/local/bin/menu" ]]; then
    echo "📝 Creating menu shortcut..."
    cat > /usr/local/bin/menu << 'EOF'
#!/bin/bash
rusty-proxy-menu
EOF
    chmod +x /usr/local/bin/menu
    echo "✓ Menu shortcut created"
else
    echo "✓ Menu shortcut already exists"
fi

# Fix permissions
echo "🔒 Fixing permissions..."
chmod +x /usr/local/bin/rusty-proxy-menu
chmod +x /usr/local/bin/menu
echo "✓ Permissions fixed"

# Test the commands
echo "🧪 Testing commands..."
if command -v menu >/dev/null 2>&1; then
    echo "✓ 'menu' command works"
else
    echo "❌ 'menu' command still not working"
fi

if command -v rusty-proxy-menu >/dev/null 2>&1; then
    echo "✓ 'rusty-proxy-menu' command works"
else
    echo "❌ 'rusty-proxy-menu' command still not working"
fi

echo
echo "🎉 Quick fix completed!"
echo
echo "📋 You can now use these commands:"
echo "  menu                - Launch interactive menu"
echo "  rusty-proxy-menu    - Full menu command"
echo
echo "💡 If 'menu' still doesn't work, try:"
echo "  sudo menu"
echo "  /usr/local/bin/menu"
echo "  rusty-proxy-menu"