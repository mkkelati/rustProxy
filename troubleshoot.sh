#!/bin/bash

# Rusty Proxy Troubleshooting Script
# Run this to diagnose installation issues

echo "🔍 Rusty Proxy Installation Troubleshooting"
echo "==========================================="
echo

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    echo "✓ Running as root/sudo"
else
    echo "❌ Not running as root - some checks may fail"
    echo "   Try: sudo ./troubleshoot.sh"
fi
echo

# Check if menu script exists
echo "📁 Checking menu script installation:"
if [[ -f "/usr/local/bin/menu" ]]; then
    echo "✓ Menu script exists at /usr/local/bin/menu"
    ls -la /usr/local/bin/menu
else
    echo "❌ Menu script not found at /usr/local/bin/menu"
fi

if [[ -f "/usr/local/bin/rusty-proxy-menu" ]]; then
    echo "✓ Main menu script exists at /usr/local/bin/rusty-proxy-menu"
    ls -la /usr/local/bin/rusty-proxy-menu
else
    echo "❌ Main menu script not found at /usr/local/bin/rusty-proxy-menu"
fi
echo

# Check PATH
echo "🛤️  Checking PATH:"
if echo $PATH | grep -q "/usr/local/bin"; then
    echo "✓ /usr/local/bin is in PATH"
else
    echo "❌ /usr/local/bin is NOT in PATH"
    echo "   Current PATH: $PATH"
fi
echo

# Check which command
echo "🔍 Testing command resolution:"
which menu 2>/dev/null && echo "✓ 'menu' command found" || echo "❌ 'menu' command not found"
which rusty-proxy-menu 2>/dev/null && echo "✓ 'rusty-proxy-menu' command found" || echo "❌ 'rusty-proxy-menu' command not found"
echo

# Check service status
echo "🚀 Checking Rusty Proxy service:"
if systemctl list-units --full -all | grep -q "rusty-proxy.service"; then
    echo "✓ Service exists"
    systemctl status rusty-proxy --no-pager -l
else
    echo "❌ Service not found"
fi
echo

# Check installation directory
echo "📂 Checking installation directory:"
if [[ -d "/opt/rusty-proxy" ]]; then
    echo "✓ Installation directory exists"
    ls -la /opt/rusty-proxy/
else
    echo "❌ Installation directory not found"
fi
echo

echo "🔧 Quick Fixes:"
echo "1. If menu script is missing, run the installation again:"
echo "   curl -sSL https://raw.githubusercontent.com/mkkelati/rustProxy/main/install.sh | sudo bash"
echo
echo "2. If PATH issue, try absolute path:"
echo "   sudo /usr/local/bin/menu"
echo
echo "3. If permission issue, fix permissions:"
echo "   sudo chmod +x /usr/local/bin/menu"
echo "   sudo chmod +x /usr/local/bin/rusty-proxy-menu"
echo
echo "4. Manually create menu shortcut (temporary fix):"
echo "   sudo ln -sf /usr/local/bin/rusty-proxy-menu /usr/local/bin/menu"
echo
echo "5. Use the full command instead:"
echo "   sudo rusty-proxy-menu"