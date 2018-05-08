#!/bin/bash
cargo build --release
sudo mkdir -p /usr/local/bin

NAME="update-caches"
sudo cp target/release/update-caches-rust "/usr/local/bin/${NAME}.bin"
sudo cp "init/${NAME}.service" /etc/systemd/system

cat <<EOF | sudo tee "/usr/local/bin/${NAME}"
#!/bin/bash
. /etc/${NAME}.conf
export PAUSE_SECONDS
export LOOP
export DATABASE_URL
${NAME}.bin
EOF

CONFIG_FILE="/etc/${NAME}.conf"
if [ ! -f "${CONFIG_FILE}" ]; then
    cat <<EOF | sudo tee "/etc/${NAME}.conf"
# pause between database checks if no stations to check are available
PAUSE_SECONDS=60
# continue after STATIONS stations have been checked
LOOP=true
# database connection string (mysql, mariadb)
DATABASE_URL=mysql://myuser:mypassword@localhost/radio
EOF
fi

sudo chmod ugo+x "/usr/local/bin/${NAME}"
sudo groupadd --system radiobrowser
sudo useradd --system --no-create-home --gid radiobrowser radiobrowser

sudo systemctl daemon-reload

echo "Enable service with:"
echo " - systemctl enable ${NAME}"
echo "Start service with:"
echo " - systemctl start ${NAME}"
echo "Logs:"
echo " - journalctl log -uf ${NAME}"
echo "Edit /etc/${NAME}.conf according to your needs."