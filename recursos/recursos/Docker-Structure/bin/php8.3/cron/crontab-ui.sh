#!/bin/bash

# Instalar nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.5/install.sh | bash
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"
nvm install 20.8.1

# Instalar crontab-ui
npm install -g crontab-ui

# Configurar la ruta de la base de datos
mkdir -p /var/data/cron
CRON_DB_PATH=/var/data/cron crontab-ui &

# Simular la pulsación de la tecla "Enter" para continuar
echo -ne '\n'
echo "crontab-ui installation finished"