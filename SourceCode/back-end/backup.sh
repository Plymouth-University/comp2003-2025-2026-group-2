#!/bin/bash

BACKUP_DIR="/home/ubuntu/logsmart/backups"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/logsmartdb_$DATE.sql"

mkdir -p "$BACKUP_DIR"

docker exec postgres pg_dump -U admin logsmartdb > "$BA>

gzip "$BACKUP_FILE"

find "$BACKUP_DIR" -name "*.sql.gz" -mtime +7 -delete

echo "Backup completed: ${BACKUP_FILE}.gz"