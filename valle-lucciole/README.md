# Valle delle Lucciole

Gioco di fattoria 2.5D in pixel art, scritto in **Rust** con [macroquad](https://macroquad.rs).
Tutta la grafica (terreno, alberi, case, colture, personaggi, icone, font) è generata via codice:
non ci sono immagini esterne. Gli sprite vengono anche esportati in PNG in `assets/sprites/`.

![Tutti gli sprite](assets/sprites/_tutti_gli_sprite.png)

## Avviare il gioco (Windows, macOS, Linux)

1. Installa Rust da <https://rustup.rs> (su Windows serve anche "Desktop development with C++" di Visual Studio Build Tools, che l'installer propone).
2. Da questa cartella:

```bash
cargo run --release
```

Il primo avvio compila tutto (circa un minuto). Poi parte in pochi secondi.

## Comandi

| Tasto | Azione |
|---|---|
| WASD / frecce | Muoviti (Shift per correre) |
| Clic sinistro / C / Spazio | Usa l'attrezzo o pianta i semi |
| Clic destro / E / X | Raccogli, apri la bancarella, usa la cassa, mangia |
| 1-0, rotella, Q / Tab | Cambia oggetto |
| H | Mostra i comandi |

## Come si gioca

- **Zappa** il terreno, **pianta** i semi, **annaffia** ogni giorno: le colture crescono solo nei giorni bagnati.
- Riempi l'annaffiatoio al **pozzo** o allo **stagno**.
- **Ascia** per alberi, ceppi e rami; **piccone** per i sassi; **falce** per le erbacce (a volte trovi semi).
- Metti il raccolto nella **cassa** accanto a casa: lo paga di notte.
- Oltre il ponte c'è la piazza con la **bancarella di Mira** (aperta dalle 8:00 alle 20:00).
- Entra in **casa** per dormire: la partita si salva in `salvataggio.txt`. Dopo le 2:00 crolli dalla stanchezza.
- Il giorno passa con luce dell'alba, tramonto e notte; a volte piove (annaffia per te). Di sera si accendono le lucciole.

| Coltura | Giorni | Seme | Vendita |
|---|---|---|---|
| Carota | 4 | 20 | 38 |
| Pomodoro | 6 | 45 | 30 (2-4 frutti) |
| Girasole | 7 | 60 | 130 |
| Zucca | 9 | 110 | 340 |

## Come è fatto

| File | Contenuto |
|---|---|
| `src/sprites.rs` | Tutti gli sprite dipinti pixel per pixel (luce dall'alto a sinistra, contorni selettivi) |
| `src/world.rs` | Mappa, oggetti e "cottura" del terreno in una texture con bordi organici |
| `src/render.rs` | Rendering 2.5D: ordinamento per profondità, ombre, trasparenza dietro alberi e tetti, luci notturne |
| `src/game.rs` | Logica: attrezzi, colture, giorni, negozio, salvataggio |
| `src/ui.rs`, `src/font.rs` | Interfaccia e font bitmap 5x7 |

Il mondo viene disegnato a bassa risoluzione (circa 480x270) e ingrandito a scala intera, così i pixel restano nitidi.
Di notte una mappa di luce viene moltiplicata sulla scena (finestre, lampioni, lucciole).

### Strumenti per sviluppatori

```bash
cargo run --release -- --export-art
```

Riesporta tutti gli sprite in `assets/sprites/`.

```bash
VALLE_SHOT=prova.png,40,selftest cargo run --release
```

Esegue un test automatico del ciclo di gioco (zappa, semina, crescita, raccolto, vendita, alberi, negozio, salvataggio).

### Versione web

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/valle-lucciole.wasm web/
```

Poi servi la cartella `web/` con un server qualsiasi (es. `python -m http.server`) e apri `index.html`.
Nella versione web il salvataggio non è disponibile.
