# Emojis vs Zombies — rekonstrukcja w Rust

Natywna gra desktopowa (jeden plik binarny, bez assetów) odtwarzająca pętlę rozgrywki
z `Emojis vs Zombies` (Cowboy Crab Games, Unity/Mono) na podstawie analizy statycznej
`Assembly-CSharp.dll`. Wyniki analizy i źródła każdej formuły: [EVIDENCE.md](EVIDENCE.md).

## Uruchomienie

```bash
cd rust-game
cargo run --release
```

Gotowy binarny plik (po zbudowaniu):

```bash
./target/release/emojis-vs-zombies
```

Wymagania: Rust (testowane na 1.99), Linux z X11/Wayland. Brak innych zależności
systemowych — grafika jest rysowana piksel po pikselu, okno obsługuje `minifb`.

Dodatkowe tryby:

```bash
./target/release/emojis-vs-zombies --selftest        # asercje + symulacja 600 s
./target/release/emojis-vs-zombies --diagnose        # 15 min losowej gry + kontrola niezmiennikow
./target/release/emojis-vs-zombies --bench           # sredni czas rysowania klatki
./target/release/emojis-vs-zombies --shot a.ppm      # deterministyczna klatka do PPM
./target/release/emojis-vs-zombies --shot a.ppm --tab 2
```

## Sterowanie

| Wejście | Akcja |
| --- | --- |
| klik na kartę obrońcy | wybór obrońcy do postawienia (zakładka DEFENDERS) |
| klik na slot przy murze | postawienie / **podmiana** obrońcy — każda kopia kosztuje swoją cenę |
| klik na zajęty slot bez wyboru | wybór stojącego tam obrońcy (jego pole się podświetla) |
| prawy klik na slot | **usunięcie** obrońcy (sprzęt wraca do ekwipunku) |
| klik na kartę EQUIPPED (góra/dół) | zdjęcie broni / gadżetu z obrońcy |
| OPTIONS → FULL RESET (2 kliknięcia) | kasuje **wszystko**: postacie, rzadkości, ekwipunek, ulepszenia, prestiż, statystyki i zapis |
| klik na kartę | kupno ulepszenia / ulepszenia prestiżu |
| `1`–`4` | zakładki: DEFENDERS, UPGRADES, REBIRTH, OPTIONS |
| `Spacja` | pauza |
| `F5` | zapis stanu |
| `R` | restart przebiegu po przegranej |
| `B` | rebirth (prestiż) |
| `Esc` | wyjście z zapisem |

## Co jest w grze

- **Proceduralne fale** (`src/game/waves.rs`): numer fali wyznacza archetyp —
  `MIXED`, `SWARM` (tłum słabszych), `ELITE` (mniej, ale mocniejsi) lub `RUSH`
  (szybsi i liczni) — oraz tempo spawnu, liczebność partii i mnożniki HP/tempa.
  Wszystko jest deterministyczne (ta sama fala zawsze wygląda tak samo), ale kolejne
  fale się różnią, a trudność rośnie z numerem. Archetyp jest widoczny w HUD przy numerze fali.
- **Fale 30 s**; boss co 5. falę, a fala 5 jest falą wyłącznie bossową (jak w oryginale).
- **Trzy warianty wrogów** odblokowywane progami fal (szybki od 6, gruby od 11).
- **Obrońcy** z 7 typów broni i mnożnikami obrażeń z oryginału. Posiadanie postaci
  nie daje darmowych kopii: **każde postawienie kosztuje cenę karty**, a brak gotówki blokuje akcję.
- **Zakładki PACKS i GEAR są celowo usunięte** z interfejsu — zostają cztery ekrany:
  obrońcy, ulepszenia, prestiż i opcje. Logika paczek i ekwipunku została w `src/game/`,
  ale nie ma do niej wejścia w UI.
- **Rzadkości** Common–Mythic sterujące obrażeniami i szansą kryta.
- **Zbieracze monet**: monety leżą na ziemi, zbieracz je nosi (pojemność + prędkość)
  i opróżnia przy forcie, zamiast automatycznego naciągania gotówki. Ulepszenie
  `COLLECTOR` dodaje zbieracza (+2 pojemności i +8 tempa za poziom, limit 5).
- **Licznik DPS**, questy z nagrodami i tempo gry x1/x2/x3.
- **Mur** z HP, pęknięciami i przegraną po wybiciu.
- **Sklep ulepszeń** (6 kart: siła, tempo, łup, mur, slot, zbieracze)
  z ceną `base * growth^(lvl-1) * 5^floor((lvl-1)/25)` i **bez górnego limitu poziomów**:
  każdy poziom PLATFORMY dokłada 4 pola (siatka 12 kolumn = 48 pozycji), a COLLECTOR
  dokłada zbieracza bez limitu sztuk.
- **Prestiż** z punktami `floor(wave/5)` za bossa i dodatkiem `2*(i+1)` za każde 10 fal.
- **Autosave co 5 s** do `~/.local/share/emojis_vs_zombies/save.txt`.

## Assety

Wszystkie obrazki są wkompilowane w binarkę (`include_bytes!`), więc plik binarny
jest samowystarczalny. Źródła skopiowano do `assets/`:

- **UI** — `Complete_UI_Essential_Pack_Free` (Flat Theme): ramki, przyciski, paski, sloty (9-slice).
- **Bronie** — `guns_gameassets`: ikony pistoletu, strzelby, karabinu, snajperki, SMG, granatu.
- **Twarze** — `16x16-Emoji-Pack_v1.1`: arkusz 8x8, wycinane kafle 16 px (obrońcy i potwory).

Licencje tych paczek leżą w `assets/LICENSE_*.txt`.

## Czego świadomie nie odtworzono

- Przezroczystości okna, zmiany rozdzielczości i lokalizacji PL/EN z `SettingsUI`
  (wymagają ingerencji w backend okna, nie tylko w rysowanie).
- Tutoriala opartego o `GameFlagType` oraz drzewa ulepszeń prestiżu w wersji graficznej.
- Dokładnych wartości ScriptableObjectów (bazowe HP wrogów, koszty, sprite'y) — leżą
  w `data.unity3d`, a nie w kodzie C#, więc nie były dostępne analizą assembly.
  Tam, gdzie trzeba było wybrać liczbę, w `src/config.rs` jest komentarz `[D]`.

## Struktura

```
src/config.rs        stałe i formuły odtworzone z IL (z odwołaniami do metod)
src/game/mod.rs      typy stanu, statystyki pochodne, losowość, log
src/game/combat.rs   pętla symulacji: spawn, ruch, pociski, obrażenia, monety
src/game/economy.rs  obrońcy, ekwipunek, ulepszenia, paczki, prestiż, resety
src/game/quests.rs   questy i nagrody
src/game/waves.rs    proceduralny generator fal (archetypy, tempo, mnożniki)
src/render/mod.rs    Rect/Layout, paleta, helpery paneli, orkiestracja klatki
src/render/field.rs  tło, mur, sloty, obrońcy, wrogowie, pociski, monety
src/render/hud.rs    górny HUD, quest, podpowiedzi, nakładki
src/render/panel.rs  zakładki i karty czterech ekranów
src/art.rs           pixel-art procedurowany: monety, naboje, błysk lufy
src/assets.rs        wkompilowane PNG (UI, bronie, emoji) + cache tła
src/gfx.rs           framebuffer, prymitywy, blit sprite'ów, font 3x5
src/save.rs          zapis/odczyt stanu
src/main.rs          okno, wejście, tryby --selftest, --diagnose, --bench, --shot
```

Nowy wygląd zmienia się w `src/art.rs` (wzory pikseli) i `src/render/mod.rs`
(paleta), a nowa mechanika w `src/game/*` — bez ruszania reszty.

## Wydajność

`--bench` mierzy średni czas klatki (300 klatek). Kolejne kroki optymalizacji,
minimalne z 9 serii (na maszynie z innymi procesami w tle pomiary potrafią się różnić):

| Wersja | ms/klatkę |
| --- | --- |
| wyjściowa (gradient + 9-slice co klatkę) | 7,0 |
| cache tła (niebo, księżyc, ziemia) | 6,4 |
| szybka ścieżka blitu dla nieprzezroczystych pikseli | 3,3 |
| cache oprawy UI (pasek HUD + dolny panel) | 2,6 |

Budżet 60 FPS to 16,7 ms, więc zostaje ok. 6× zapasu na efekty.
