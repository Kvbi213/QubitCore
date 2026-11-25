======================================================================
               QUBIT ECOSYSTEM -- INSTRUKCJA DLA WINDOWS 11           
======================================================================

Gratulacje! W tej paczce znajduje sie kompletny pakiet instalacyjny Qubit:

1. METODA GRAFICZNA (Zalecana dla Windows 11):
   - Uruchom bezposrednio: QubitInstaller.exe
   - Pojawi sie nowoczesne okno instalatora w trybie Dark Mode.
   - Kliknij "Instaluj". Program utworzy katalog roboczy w %LOCALAPPDATA%\Qubit,
     skonfiguruje sciezke PATH w rejestrze Windows, utworzy skrot
     "Qubit Mission Control" na Twoim Pulpicie i zainicjalizuje baze SQLite WAL.

2. METODA KONSOLOWA (Automatyczna):
   - Kliknij dwukrotnie plik "install.bat" LUB
   - W terminalu PowerShell wpisz: .\install.ps1

3. KORZYSTANIE Z EKOSYSTEMU:
   - Pulpit: Kliknij dwukrotnie skrot "Qubit Mission Control".
   - Przegladarka: Otworz panel pod adresem https://qubit.kvbi.pl/
   - Konsola: Wpisz polecenie `qubit status` lub `qubit chat "twoj prompt"`.

Specyfikacja:
- Zuzycie RAM: <60 MB (w pelni spelniony budzet)
- Bezpieczenstwo: Brama Zero-Trust MCP z twarda blokada tainted contexts
- Storage: Single-Writer Actor SQLite WAL + wyszukiwanie pelnotekstowe FTS5 BM25
- Konteneryzacja: Pelne wsparcie Dockerfile i docker-compose.yml z limitem 64MB RAM.
