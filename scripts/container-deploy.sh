#!/bin/bash
set -euo pipefail

echo "======================================================================"
echo "          QUBIT CORE -- AUTOMATED CONTAINER DEPLOYMENT                "
echo "======================================================================"

CONTAINER_CMD=""
if command -v docker >/dev/null 2>&1; then
    CONTAINER_CMD="docker"
elif command -v podman >/dev/null 2>&1; then
    CONTAINER_CMD="podman"
else
    echo "[-] Blad: Nie znaleziono Dockera ani Podmana w systemie."
    echo "[*] QubitCore moze zostac uruchomiony natywnie uzywajac: ./target/release/qubit-core"
    exit 1
fi

echo "[+] Wykryto runtime kontenerowy: $CONTAINER_CMD"
echo "[*] Budowanie obrazu qubit/qubit-core:latest..."
$CONTAINER_CMD build -t qubit/qubit-core:latest .

echo "[*] Uruchamianie kontenera z limitem RAM 64MB..."
$CONTAINER_CMD rm -f qubit-core 2>/dev/null || true
$CONTAINER_CMD run -d \
    --name qubit-core \
    --restart unless-stopped \
    -p 4000:4000 \
    -m 64m \
    --cpus 1.0 \
    -v qubit_data:/var/lib/qubit \
    qubit/qubit-core:latest

echo "[*] Oczekiwanie na healthcheck demona (Axum :4000)..."
for i in {1..15}; do
    if curl -s http://127.0.0.1:4000/health | grep -q "ok"; then
        echo "[+] SUKCES: Kontener QubitCore dziala stabilnie na porcie 4000!"
        exit 0
    fi
    sleep 1
done

echo "[!] Ostrzezenie: Kontener wystartowal, ale healthcheck nie odpowiedzial w ciagu 15s."
exit 0
