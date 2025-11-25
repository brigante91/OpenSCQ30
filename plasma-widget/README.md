# OpenSCQ30 Plasma Widget

Un widget Plasma per KDE che mostra lo stato della batteria e informazioni sui dispositivi Soundcore connessi.

## Caratteristiche

- **Visualizzazione Batteria**: Mostra il livello di batteria per dispositivi single, dual (left/right) e case
- **Stato Connessione**: Indica se un dispositivo è connesso o meno
- **Aggiornamento Automatico**: Aggiorna i dati ogni 5 secondi quando connesso
- **Compatibilità**: Funziona con tutti i dispositivi Soundcore supportati da OpenSCQ30

## Installazione

1. **Copia il widget nella directory Plasma**:
   ```bash
   cp -r plasma-widget ~/.local/share/plasma/plasmoids/com.oppzippy.openscq30.widget
   ```

2. **Rendi eseguibile lo script backend**:
   ```bash
   chmod +x ~/.local/share/plasma/plasmoids/com.oppzippy.openscq30.widget/backend/openscq30-widget-backend.sh
   ```

3. **Aggiungi il widget al pannello o desktop**:
   - Clic destro sul pannello → "Aggiungi widget"
   - Cerca "OpenSCQ30 Battery Widget"
   - Aggiungi al pannello o desktop

## Requisiti

- **OpenSCQ30 CLI**: Deve essere compilato (lo script lo trova automaticamente)
  - Compila con: `cargo build --release -p openscq30-cli`
  - Lo script cerca automaticamente in:
    - `PROJECT_ROOT/target/release/openscq30`
    - `PROJECT_ROOT/target/debug/openscq30`
    - `/usr/local/bin/openscq30`
    - `/usr/bin/openscq30`
    - `~/.local/bin/openscq30`
    - PATH (se installato)

- **sqlite3**: Per leggere il database dei dispositivi accoppiati
  ```bash
  sudo dnf install sqlite  # Fedora
  sudo apt install sqlite3  # Debian/Ubuntu
  ```

- **jq** (opzionale ma consigliato): Per parsing JSON avanzato
  ```bash
  sudo dnf install jq  # Fedora
  sudo apt install jq  # Debian/Ubuntu
  ```

- **timeout** (opzionale): Per evitare che lo script si blocchi
  - Generalmente già installato su Linux

## Configurazione

### Configurazione Automatica

Il widget legge automaticamente i dispositivi accoppiati dal database OpenSCQ30:
- Database: `~/.config/openscq30/database.sqlite`
- Il widget tenta di connettersi al primo dispositivo accoppiato

### Configurazione Manuale

Clic destro sul widget → "Configura" per:
- **Update Interval**: Intervallo di aggiornamento (1-60 secondi, default: 5)
- **CLI Path**: Path personalizzato alla CLI (opzionale, lascia vuoto per auto-rilevamento)

## Troubleshooting

### Il widget mostra "Not Connected"

1. Verifica che almeno un dispositivo sia accoppiato:
   ```bash
   # Se hai la CLI nel PATH
   openscq30 paired-devices list
   
   # Oppure usa il path completo
   /path/to/target/release/openscq30 paired-devices list
   ```

2. Verifica che lo script backend funzioni:
   ```bash
   ~/.local/share/plasma/plasmoids/com.oppzippy.openscq30.widget/backend/openscq30-widget-backend.sh
   ```
   Dovrebbe restituire JSON con lo stato del dispositivo.

3. Verifica che il CLI sia trovato:
   - Lo script cerca automaticamente in più posizioni
   - Se non lo trova, configura il path manualmente nel widget

4. Controlla i log di Plasma:
   ```bash
   journalctl -f | grep openscq30
   ```

### L'icona non appare

- Assicurati che il widget sia stato copiato correttamente
- Riavvia Plasma: `killall plasmashell && kstart plasmashell`

### I dati non si aggiornano

- Verifica che il dispositivo sia effettivamente connesso
- Controlla che lo script backend sia eseguibile
- Verifica i permessi del database SQLite

## Sviluppo

Il widget è composto da:

- **metadata.json**: Metadati del widget Plasma
- **ui/main.qml**: File principale QML
- **ui/CompactRepresentation.qml**: Vista compatta per il pannello
- **ui/FullRepresentation.qml**: Vista completa per il desktop
- **backend/openscq30-widget-backend.sh**: Script che interroga il CLI

## Miglioramenti Futuri

- [ ] Controlli rapidi (toggle noise canceling, equalizer presets)
- [ ] Selezione dispositivo quando multipli sono accoppiati
- [ ] Notifiche per batteria bassa
- [ ] Personalizzazione colori e dimensioni
- [ ] Backend Rust nativo invece di script shell

## Licenza

GPL-3.0-or-later (stessa licenza di OpenSCQ30)

