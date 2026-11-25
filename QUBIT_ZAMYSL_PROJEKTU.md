# QUBIT — Założenia Projektowe, Architektura i Plan Realizacji

> **Cel dokumentu:** Zwięzły, kompletny opis zamysłu projektu **Qubit** („Jeden Mózg, Wiele Oczu i Rąk”), podziału na moduły, architektury technicznej oraz planu budowy **QubitCore** od zera i rozwoju witryny **qubit.kvbi.pl**.

---

## 1. Zamysł i problem, który Qubit rozwiązuje

Współczesne frameworki agentowe (oparte głównie o Pythona / Node.js / Electrona) cierpią na trzy główne problemy:
1. **Ogromny narzut pamięciowy:** Setki megabajtów lub gigabajty RAM w spoczynku, co uniemożliwia tanie uruchamianie 24/7 na małych serwerach VPS (1 vCPU, 1 GB RAM).
2. **Koszty API i rozproszenie sesji:** Brak spójnej sesji między terminalem, IDE, aplikacją desktopową i komunikatorem oraz wysyłanie każdego prostego zapytania do płatnych API chmurowych.
3. **Brak twardych granic bezpieczeństwa:** Dawanie agentom AI dostępu do powłoki systemu bez izolacji i bez obowiązkowej bramki zatwierdzania operacji ryzykownych.

**Odpowiedź Qubit:**
- Centralny, ultralekki demon w języku **Rust** (**QubitCore**, cel: `< 60 MB RAM` w spoczynku) działający 24/7 jako **jeden mózg** z jedną wspólną sesją (`session_id`).
- Różne interfejsy (**QubitDash**, **QubitCLI**, wtyczki IDE, komunikatory) podłączone do tego samego demona przez szybkie IPC (Unix Domain Sockets / Named Pipes / WebSocket).
- **Hybrydowe zasilanie modelami:** Direct API (Gemini, Anthropic, OpenRouter, Ollama) + lokalne mostki CLI (`agy`, `claude`) + możliwość oddelegowania ciężkich zadań na domowy PC z GPU (**QubitNode**).
- **Bezpieczeństwo Zero-Trust + Human-In-The-Loop:** Izolacja katalogów (`MCP Roots`), oznaczanie sesji z niezaufanymi danymi (`TaintedContext`) oraz wstrzymywanie ryzykownych komend do akceptacji człowieka (**Elicitation Gate**).

---

## 2. Mapa Ekosystemu (Moduły i Role)

| Moduł | Rola | Technologia | Status obecny |
| :--- | :--- | :--- | :--- |
| **QubitCore** | Centralny demon orkiestracyjny, router trybów (`Reflex` / `Deep`), aktor SQLite WAL, serwer HTTP/WS (`:4000` / `:4001`), piaskownice i MCP. | Rust, Tokio, Axum 0.8, `rusqlite` (WAL + FTS5), `serde` / `ts-rs` | **Do zbudowania od zera** (istnieje `Cargo.toml` i specyfikacje, katalogi `crates/*/src` są puste) |
| **QubitCLI** | Lekki klient terminalowy do montowania katalogu roboczego (`qubit .`), podglądu statusu, start/stop demona i czatu z sesją. | Rust, `clap` 4.5, IPC / REST | **Plan / do spięcia z QubitCore** |
| **QubitDash** | Natywny kokpit desktopowy (Mission Control) ze wzorcem *Ephemeral WebView* (3–5 MB w trayu), bramką akceptacji HITL i modułami osobistymi (OmniDash: Librus, finanse, nawyki, zadania). | Tauri v2 + TypeScript / Tailwind | **Plan / specyfikacja** |
| **QubitDocs** | System bazy wiedzy w czystym Markdownie (`.md` + YAML Frontmatter) z wyszukiwaniem pełnotekstowym FTS5 + BM25 (< 15 MB RAM) w jednej bazie `qubit_data.db`. | Rust + SQLite FTS5 + `QubitDocs Agent` | **Plan / specyfikacja** |
| **QubitNode** | Zdalne węzły wykonawcze (Compute Offloading) do zadań ciężkich: lokalne LLM (Ollama / vLLM), analiza binarna (Ghidra Headless), izolowane testy po mTLS. | Rust / Docker, mTLS, MCP over TLS | **Koncept** |
| **QubitWeb** | Oficjalna witryna produktowa (`https://qubit.kvbi.pl`), dokumentacja architektury, mapa modułów i dystrybucja binarek. | React 18, Vite 6, Tailwind CSS, `wouter`, Framer Motion | **Aktywna** (wymaga poprawek kodu, linków i rozbudowy UX) |

---

## 3. Architektura QubitCore (Budowa od zera — podział na `crates/`)

Workspace Cargo w `QubitCore/` jest już podzielony na 8 docelowych skrzynek:

1. **`qubit-protocol`** — Wspólne typy zdarzeń, żądań/odpowiedzi IPC, DTO oraz automatyczny eksport typów do TypeScript (`ts-rs`).
2. **`qubit-storage`** — Jednowątkowy aktor zapisu **SQLite WAL** (`qubit_data.db`): sesje, wiadomości, checkpointy, pamięć semantyczna, graf powiązań oraz indeks **FTS5 (BM25)** dla QubitDocs.
3. **`qubit-mcp`** — Warstwa narzędzi oparta o **Model Context Protocol (JSON-RPC 2.0)**: `MCP Roots` (blokada ścieżek poza projektem), rejestr narzędzi oraz `Elicitation` (kolejka akceptacji HITL) i flaga `TaintedContext`.
4. **`qubit-sandbox`** — Izolacja procesów i limitowanie zasobów (`cgroups v2` / kontenery), wykonywanie komend shellowych z timeoutami i limitem pamięci.
5. **`qubit-agents`** — Router trybów **Reflex** (szybka odpowiedź, niski koszt) vs **Deep** (planowanie, dekompozycja, pod-agenci), warstwa dostawców modeli (`ModelProvider`: API + mostki CLI) oraz kompresja kontekstu.
6. **`qubit-core`** — Główna binarka demona: runtime **Tokio**, serwer **Axum** (`127.0.0.1:4000` API/WS + wbudowany lekki panel diagnostyczny), pętla Heartbeat i obsługa IPC.
7. **`qubit-cli`** — Klient terminalowy (`qubit start`, `qubit status`, `qubit .`, `qubit chat`) komunikujący się z demonem przez IPC/HTTP.
8. **`qubit-installer`** — Natywny instalator/bootstrapper (docelowo pobierający binarki z GitHub Releases i inicjujący katalogi + `qubit_config.json`).

---

## 4. Plan budowy QubitCore (Etapami — MVP najpierw)

- **Etap 1 (Fundament + Minimalny Demon):**
  - `qubit-protocol` (wiadomości, statusy, błędy, typy sesji).
  - `qubit-storage` (inicjalizacja `qubit_data.db` w trybie WAL, tabele sesji, wiadomości, zadań i FTS5 dla notatek).
  - `qubit-core` (uruchomienie Tokio + Axum na `127.0.0.1:4000`, endpoint `/health`, `/api/sessions`, WebSocket stream zdarzeń).
  - `qubit-cli` (`qubit status`, `qubit chat "prompt"`, `qubit .`).
- **Etap 2 (Narzędzia, Bezpieczeństwo i Pierwszy Agent):**
  - `qubit-mcp` + podstawowe narzędzia plikowe z `MCP Roots` i kolejką `Elicitation` (akceptacja ryzykownych akcji).
  - `qubit-agents`: integracja z wybranym providerem LLM (Direct API + lokalny mostek CLI) i rozdział `Reflex` vs `Deep`.
- **Etap 3 (Piaskownice, QubitDocs i QubitDash):**
  - `qubit-sandbox` (limity RAM/CPU i bezpieczne wykonanie komend).
  - Silnik dokumentacji `QubitDocs` (indeksowanie `.md` do FTS5).
  - Podpięcie GUI **QubitDash** (Tauri v2) i zdalnych węzłów **QubitNode**.

---

## 5. Audyt i plan rozwoju strony `qubit.kvbi.pl` (`QubitWeb`)

### Co wymaga naprawy od razu:
1. **Błąd w `Caddyfile` na Mikrusie:** literówki w cudzysłowach (`X-Frame-Options "SAMEORIGIN""` i apostrof na końcu `Content-Security-Policy`) — powodują wysyłanie uszkodzonych znaków w nagłówkach HTTP.
2. **Surowy HTML w `Home.tsx` (`HOME_HTML` + `dangerouslySetInnerHTML`):**
   - Linki `<a href="/docs">` przeładowują całą stronę zamiast działać jak płynne SPA (`wouter`).
   - Martwe linki `href="#"` przy przyciskach pobierania (Windows/macOS/Linux).
   - Niespójność linku instalatora (`qubit-setup.exe` vs `QubitInstaller.exe`, domena `curl -sSL https://qubit.run | sh` nie należy do projektu).
   - Sprzeczne dane między `Home.tsx` a `data.ts` (np. `<18 MB` vs `<60 MB`, status `GOTOWY` dla modułu, który jeszcze nie ma kodu w `crates/`).
   - Zewnętrzne fonty `Material Symbols` mogą mrugać jako tekst przy ładowaniu (lepiej użyć SVG z `lucide-react` / `ModuleIcons.tsx`).
3. **SEO i konfiguracja:** brak tagów OpenGraph (`og:title`, `og:description`), kanonicznych linków oraz brak `favicon.ico` (jest tylko SVG).

### Co warto rozwinąć na `qubit.kvbi.pl`:
1. **Przebudowa `Home.tsx` na prawdziwe komponenty Reacta** z zachowaniem stylu *Plasma Glow* (bez `dangerouslySetInnerHTML`).
2. **Uczciwe i spójne oznaczenia statusów** na wszystkich kartach (`AKTYWNY` / `W BUDOWIE` / `PLAN`) oraz ujednolicenie metryk z `data.ts`.
3. **Sekcja „Jak to działa” (Flow):** wizualna ścieżka 4 kroków: *Zlecenie (CLI/Dash) → QubitCore (Reflex/Deep) → Piaskownica + Bramka HITL → Wynik*.
4. **Sekcja „Dla kogo / Use Cases” + „Dlaczego Qubit”:** konkretne scenariusze (VPS 1 GB RAM, lokalne CLI bez opłat za tokeny, bezpieczeństwo RE/sandbox) oraz porównanie zużycia RAM.
5. **Sekcja Quick Start + FAQ:** realna instrukcja uruchomienia (dopasowana do aktualnego stanu repozytorium) i odpowiedzi na pytania o prywatność, bezpieczeństwo i wymagania.
6. **Live Status / Roadmapa zsynchronizowana z postępem QubitCore.**
