#!/bin/bash
set -euo pipefail

INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.qubit"
mkdir -p "${INSTALL_DIR}" "${CONFIG_DIR}" "${CONFIG_DIR}/data"

echo "======================================================================"
echo "          QUBIT CORE -- LINUX / MACOS UNIFIED INSTALLER               "
echo "======================================================================"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

if [ -f "${ROOT_DIR}/target/release/qubit-core" ]; then
    cp "${ROOT_DIR}/target/release/qubit-core" "${INSTALL_DIR}/qubit-core"
    cp "${ROOT_DIR}/target/release/qubit-cli" "${INSTALL_DIR}/qubit-cli"
    chmod +x "${INSTALL_DIR}/qubit-core" "${INSTALL_DIR}/qubit-cli"
    echo "[+] Skopiowano qubit-core i qubit-cli do ${INSTALL_DIR}"
fi

cat << 'EOF' > "${CONFIG_DIR}/qubit_config.json"
{
  "version": "0.1.0",
  "port": 4000,
  "host": "127.0.0.1",
  "ram_limit_mb": 60,
  "zero_trust_mcp": true,
  "wal_mode": true
}
EOF
echo "[+] Zapisano konfiguracje w ${CONFIG_DIR}/qubit_config.json"

if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    echo "[!] Wskazowka: Dodaj ${INSTALL_DIR} do zmiennej PATH w ~/.bashrc lub ~/.zshrc:"
    echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

echo "[+] SUKCES: QubitCore gotowy do uzycia! Uruchom: qubit-core"
