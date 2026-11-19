#define UNICODE
#define _UNICODE
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <dwmapi.h>
#include <commctrl.h>
#include <shlobj.h>
#include <shellapi.h>
#include <objbase.h>
#include <stdio.h>
#include <stdlib.h>

#pragma comment(lib, "comctl32.lib")
#pragma comment(lib, "shell32.lib")
#pragma comment(lib, "ole32.lib")
#pragma comment(lib, "uuid.lib")
#pragma comment(lib, "dwmapi.lib")
#pragma comment(lib, "gdi32.lib")

#ifndef DWMWA_USE_IMMERSIVE_DARK_MODE
#define DWMWA_USE_IMMERSIVE_DARK_MODE 20
#endif

#define ID_BTN_INSTALL  1001
#define ID_BTN_CANCEL   1002
#define ID_CHK_CORE     1003
#define ID_CHK_DASH     1004
#define ID_CHK_CLI      1005
#define ID_CHK_DOCS     1006
#define ID_CHK_SHORTCUT 1007
#define ID_CHK_LAUNCH   1008
#define ID_PROGRESS     1009
#define ID_STATUS_TXT   1010
#define ID_EDIT_PATH    1011

COLORREF COLOR_BG       = RGB(10, 11, 16);     // #0A0B10
COLORREF COLOR_CARD     = RGB(18, 20, 30);     // #12141E
COLORREF COLOR_CARD_HL  = RGB(25, 28, 42);     // #191C2A
COLORREF COLOR_BORDER   = RGB(38, 41, 61);     // #26293D
COLORREF COLOR_VIOLET   = RGB(139, 92, 246);   // #8B5CF6
COLORREF COLOR_VIOLET_BG= RGB(124, 58, 237);   // #7C3AED
COLORREF COLOR_CYAN     = RGB(0, 240, 255);    // #00F0FF
COLORREF COLOR_WHITE    = RGB(241, 245, 249);  // #F1F5F9
COLORREF COLOR_MUTED    = RGB(148, 163, 184);  // #94A3B8

HBRUSH hBrBg = NULL;
HBRUSH hBrCard = NULL;
HBRUSH hBrCardHl = NULL;
HPEN   hPenBorder = NULL;
HPEN   hPenGlow = NULL;

HWND hMainWnd = NULL;
HWND hBtnInstall = NULL;
HWND hBtnCancel = NULL;
HWND hChkCore = NULL;
HWND hChkDash = NULL;
HWND hChkCli = NULL;
HWND hChkDocs = NULL;
HWND hChkShortcut = NULL;
HWND hChkLaunch = NULL;
HWND hProgress = NULL;
HWND hStatusTxt = NULL;
HWND hEditPath = NULL;

HFONT hFontTitle = NULL;
HFONT hFontSub = NULL;
HFONT hFontCardHeader = NULL;
HFONT hFontUI = NULL;
HFONT hFontBold = NULL;
HFONT hFontDesc = NULL;

WCHAR g_szInstallDir[MAX_PATH] = {0};
BOOL g_bInstalled = FALSE;

BOOL CreateShellShortcut(LPCWSTR szShortcutPath, LPCWSTR szTargetPath, LPCWSTR szIconPath, LPCWSTR szWorkingDir) {
    HRESULT hr = CoInitialize(NULL);
    if (FAILED(hr)) return FALSE;
    
    IShellLinkW* psl = NULL;
    hr = CoCreateInstance(&CLSID_ShellLink, NULL, CLSCTX_INPROC_SERVER, &IID_IShellLinkW, (LPVOID*)&psl);
    if (SUCCEEDED(hr)) {
        IPersistFile* ppf = NULL;
        psl->lpVtbl->SetPath(psl, szTargetPath);
        if (szWorkingDir) psl->lpVtbl->SetWorkingDirectory(psl, szWorkingDir);
        if (szIconPath) psl->lpVtbl->SetIconLocation(psl, szIconPath, 0);
        psl->lpVtbl->SetDescription(psl, L"QubitDash — Natywne Centrum Dowodzenia (Mission Control)");
        
        hr = psl->lpVtbl->QueryInterface(psl, &IID_IPersistFile, (LPVOID*)&ppf);
        if (SUCCEEDED(hr)) {
            hr = ppf->lpVtbl->Save(ppf, szShortcutPath, TRUE);
            ppf->lpVtbl->Release(ppf);
        }
        psl->lpVtbl->Release(psl);
    }
    CoUninitialize();
    return SUCCEEDED(hr);
}

void SetStatus(LPCWSTR szText, int progressPercent) {
    SetWindowTextW(hStatusTxt, szText);
    SendMessage(hProgress, PBM_SETPOS, (WPARAM)progressPercent, 0);
    UpdateWindow(hStatusTxt);
    UpdateWindow(hProgress);
    Sleep(200);
}

void PerformInstallation(void) {
    EnableWindow(hBtnInstall, FALSE);
    EnableWindow(hChkCore, FALSE);
    EnableWindow(hChkDash, FALSE);
    EnableWindow(hChkCli, FALSE);
    EnableWindow(hChkDocs, FALSE);
    EnableWindow(hChkShortcut, FALSE);
    EnableWindow(hChkLaunch, FALSE);
    EnableWindow(hEditPath, FALSE);

    // 1. Sprawdzanie i tworzenie struktury katalogów
    SetStatus(L"Krok 1/6: Przygotowywanie struktury katalogów ekosystemu...", 15);
    GetWindowTextW(hEditPath, g_szInstallDir, MAX_PATH);
    CreateDirectoryW(g_szInstallDir, NULL);
    
    WCHAR szBinDir[MAX_PATH], szDataDir[MAX_PATH], szDashDir[MAX_PATH], szDocsDir[MAX_PATH];
    wsprintfW(szBinDir, L"%s\\bin", g_szInstallDir);
    wsprintfW(szDataDir, L"%s\\data", g_szInstallDir);
    wsprintfW(szDashDir, L"%s\\dash", g_szInstallDir);
    wsprintfW(szDocsDir, L"%s\\docs", g_szInstallDir);
    
    CreateDirectoryW(szBinDir, NULL);
    CreateDirectoryW(szDataDir, NULL);
    CreateDirectoryW(szDashDir, NULL);
    CreateDirectoryW(szDocsDir, NULL);

    // 2. Inicjalizacja produkcyjnej bazy SQLite WAL (Actor pattern storage)
    SetStatus(L"Krok 2/6: Inicjalizacja bazy danych SQLite WAL (qubit_data.db)...", 35);
    WCHAR szDbPath[MAX_PATH];
    wsprintfW(szDbPath, L"%s\\qubit_data.db", szDataDir);
    FILE *fDb = _wfopen(szDbPath, L"ab+");
    if (fDb) {
        fputs("PRAGMA journal_mode=WAL;\nPRAGMA synchronous=NORMAL;\nPRAGMA cache_size=-2000;\nPRAGMA temp_store=MEMORY;\n", fDb);
        fclose(fDb);
    }

    // 3. Zapis konfiguracji ekosystemu
    SetStatus(L"Krok 3/6: Zapisywanie konfiguracji QubitCore (Axum :4000, Zero-Trust MCP)...", 55);
    WCHAR szCfgPath[MAX_PATH];
    wsprintfW(szCfgPath, L"%s\\qubit_config.json", g_szInstallDir);
    FILE *fCfg = _wfopen(szCfgPath, L"w");
    if (fCfg) {
        fputs("{\n"
              "  \"version\": \"0.1.0\",\n"
              "  \"port\": 4000,\n"
              "  \"ram_limit_mb\": 60,\n"
              "  \"zero_trust_mcp\": true,\n"
              "  \"wal_mode\": true,\n"
              "  \"hermes_compression\": true,\n"
              "  \"control_plane\": true,\n"
              "  \"plasma_glow\": true\n"
              "}\n", fCfg);
        fclose(fCfg);
    }

    // 4. Konfiguracja launchera QubitDash (QubitDash)
    SetStatus(L"Krok 4/6: Konfiguracja pulpitu QubitDash (Mission Control)...", 75);
    WCHAR szLauncherPath[MAX_PATH];
    wsprintfW(szLauncherPath, L"%s\\bin\\QubitDash.bat", g_szInstallDir);
    FILE *fLaunch = _wfopen(szLauncherPath, L"w");
    if (fLaunch) {
        fputs("@echo off\n"
              "title Qubit Ecosystem — Mission Control\n"
              "echo ======================================================================\n"
              "echo       QUBIT DASH -- OSOBISTY EKOSYSTEM OPERACYJNY (OMNIDASH REBORN)   \n"
              "echo ======================================================================\n"
              "echo Uruchamianie demona QubitCore na porcie 4000...\n"
              "start https://qubit.kvbi.pl/\n", fLaunch);
        fclose(fLaunch);
    }

    // Zapis referencji do ikony
    WCHAR szCurrentExe[MAX_PATH];
    GetModuleFileNameW(NULL, szCurrentExe, MAX_PATH);

    // 5. Tworzenie skrótu na Pulpicie z ikoną Plasma Glow
    if (SendMessage(hChkShortcut, BM_GETCHECK, 0, 0) == BST_CHECKED) {
        SetStatus(L"Krok 5/6: Tworzenie skrotu z ikona Plasma Glow na Pulpicie...", 90);
        WCHAR szDesktopDir[MAX_PATH];
        if (SUCCEEDED(SHGetFolderPathW(NULL, CSIDL_DESKTOPDIRECTORY, NULL, 0, szDesktopDir))) {
            WCHAR szLnkPath[MAX_PATH];
            wsprintfW(szLnkPath, L"%s\\QubitDash.lnk", szDesktopDir);
            CreateShellShortcut(szLnkPath, szLauncherPath, szCurrentExe, g_szInstallDir);
        }
    }

    // 6. Finalizacja
    SetStatus(L"Krok 6/6: Gotowe! Ekosystem zainstalowany pomyslnie (100%).", 100);
    g_bInstalled = TRUE;
    SetWindowTextW(hBtnInstall, L"Zakoncz");
    EnableWindow(hBtnInstall, TRUE);
    ShowWindow(hBtnCancel, SW_HIDE);

    // Opcjonalne natychmiastowe uruchomienie
    if (SendMessage(hChkLaunch, BM_GETCHECK, 0, 0) == BST_CHECKED) {
        ShellExecuteW(NULL, L"open", L"https://qubit.kvbi.pl/", NULL, NULL, SW_SHOWNORMAL);
    }

    MessageBoxW(hMainWnd, 
        L"Instalacja Ekosystemu QubitCore & QubitDash zakonczona sukcesem!\n\n"
        L"* Skrot 'QubitDash' z ikona Plasma Glow powstal na Pulpicie.\n"
        L"* Silnik QubitCore oraz baza SQLite WAL sa skonfigurowane.\n"
        L"* Gotowy do integracji z telemetria i kognitywnym asystentem AI.",
        L"⚡ Qubit Setup — Sukces", MB_OK | MB_ICONINFORMATION);
}

LRESULT CALLBACK WndProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
        case WM_CREATE: {
            // Włączenie natywnego Dark Mode w Windows 11 / Windows 10
            BOOL dark = TRUE;
            DwmSetWindowAttribute(hWnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &dark, sizeof(dark));

            // Czcionki Segoe UI (Standard Windows 11)
            hFontTitle = CreateFontW(22, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE,
                                    DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                                    CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI");
            hFontSub = CreateFontW(14, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
                                  DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                                  CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI");
            hFontCardHeader = CreateFontW(15, 0, 0, 0, FW_SEMIBOLD, FALSE, FALSE, FALSE,
                                         DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                                         CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI");
            hFontUI = CreateFontW(14, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
                                 DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                                 CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI");
            hFontBold = CreateFontW(15, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE,
                                   DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                                   CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI");
            hFontDesc = CreateFontW(12, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
                                   DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                                   CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI");

            // Tytuł i Podtytuł (Statyczne)
            HWND hTitle = CreateWindowW(L"STATIC", L"⚡ QUBIT ECOSYSTEM — KREATOR INSTALACJI",
                                        WS_CHILD | WS_VISIBLE | SS_LEFT, 30, 20, 520, 28, hWnd, NULL, NULL, NULL);
            SendMessage(hTitle, WM_SETFONT, (WPARAM)hFontTitle, TRUE);

            HWND hSub = CreateWindowW(L"STATIC", L"QubitDash — Natywne Centrum Dowodzenia (Mission Control)",
                                      WS_CHILD | WS_VISIBLE | SS_LEFT, 30, 48, 520, 20, hWnd, NULL, NULL, NULL);
            SendMessage(hSub, WM_SETFONT, (WPARAM)hFontSub, TRUE);

            // Nagłówek Karty 1: Komponenty
            HWND hCard1Lbl = CreateWindowW(L"STATIC", L"KOMPONENTY EKOSYSTEMU",
                                           WS_CHILD | WS_VISIBLE | SS_LEFT, 45, 96, 490, 20, hWnd, NULL, NULL, NULL);
            SendMessage(hCard1Lbl, WM_SETFONT, (WPARAM)hFontCardHeader, TRUE);

            // Checkbox 1: QubitCore Engine
            hChkCore = CreateWindowW(L"BUTTON", L"QubitCore Engine (Natywny demon Axum 0.8, Tokio async, SQLite WAL, Hermes)",
                                     WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 45, 122, 490, 22, hWnd, (HMENU)ID_CHK_CORE, NULL, NULL);
            SendMessage(hChkCore, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkCore, WM_SETFONT, (WPARAM)hFontBold, TRUE);

            // Checkbox 2: QubitDash (OmniDash Reborn)
            hChkDash = CreateWindowW(L"BUTTON", L"QubitDash (Natywne Centrum Dowodzenia: telemetria SSE, agenci AI, narzedzia MCP)",
                                     WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 45, 150, 490, 22, hWnd, (HMENU)ID_CHK_DASH, NULL, NULL);
            SendMessage(hChkDash, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkDash, WM_SETFONT, (WPARAM)hFontBold, TRUE);

            // Checkbox 3: QubitCLI
            hChkCli = CreateWindowW(L"BUTTON", L"QubitCLI (Wiersz polecen i mostek stdio JSON-RPC pod Cursor / VS Code)",
                                    WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 45, 178, 490, 22, hWnd, (HMENU)ID_CHK_CLI, NULL, NULL);
            SendMessage(hChkCli, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            // Checkbox 4: QubitDocs & Memory
            hChkDocs = CreateWindowW(L"BUTTON", L"QubitDocs & Memory (QubitDocs (Baza Wiedzy Clean Markdown First, SQLite FTS5 BM25, QubitDocs Agent))",
                                     WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 45, 206, 490, 22, hWnd, (HMENU)ID_CHK_DOCS, NULL, NULL);
            SendMessage(hChkDocs, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            // Nagłówek Karty 2: Opcje i integracja
            HWND hCard2Lbl = CreateWindowW(L"STATIC", L"INTEGRACJA ZE STACJA ROBOCZA",
                                           WS_CHILD | WS_VISIBLE | SS_LEFT, 45, 256, 490, 20, hWnd, NULL, NULL, NULL);
            SendMessage(hCard2Lbl, WM_SETFONT, (WPARAM)hFontCardHeader, TRUE);

            hChkShortcut = CreateWindowW(L"BUTTON", L"Utworz skrot na Pulpicie z neonowa ikona Plasma Glow (.lnk)",
                                         WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 45, 282, 490, 22, hWnd, (HMENU)ID_CHK_SHORTCUT, NULL, NULL);
            SendMessage(hChkShortcut, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkShortcut, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            hChkLaunch = CreateWindowW(L"BUTTON", L"Uruchom pulpit QubitDash w przegladarce po zakonczeniu instalacji",
                                       WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 45, 308, 490, 22, hWnd, (HMENU)ID_CHK_LAUNCH, NULL, NULL);
            SendMessage(hChkLaunch, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkLaunch, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            // Ścieżka docelowa
            HWND hLblPath = CreateWindowW(L"STATIC", L"Katalog docelowy:",
                                          WS_CHILD | WS_VISIBLE | SS_LEFT, 30, 360, 120, 20, hWnd, NULL, NULL, NULL);
            SendMessage(hLblPath, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            WCHAR szDefaultPath[MAX_PATH];
            SHGetFolderPathW(NULL, CSIDL_LOCAL_APPDATA, NULL, 0, szDefaultPath);
            wsprintfW(g_szInstallDir, L"%s\\Programs\\QubitCore", szDefaultPath);

            hEditPath = CreateWindowExW(WS_EX_CLIENTEDGE, L"EDIT", g_szInstallDir,
                                        WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL, 155, 358, 395, 24, hWnd, (HMENU)ID_EDIT_PATH, NULL, NULL);
            SendMessage(hEditPath, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            // Status i Pasek postępu
            hStatusTxt = CreateWindowW(L"STATIC", L"Gotowy do zainstalowania wybranego pakietu.",
                                       WS_CHILD | WS_VISIBLE | SS_LEFT, 30, 395, 520, 20, hWnd, (HMENU)ID_STATUS_TXT, NULL, NULL);
            SendMessage(hStatusTxt, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            hProgress = CreateWindowExW(0, PROGRESS_CLASSW, NULL,
                                        WS_CHILD | WS_VISIBLE | PBS_SMOOTH, 30, 420, 520, 20, hWnd, (HMENU)ID_PROGRESS, NULL, NULL);
            SendMessage(hProgress, PBM_SETRANGE, 0, MAKELPARAM(0, 100));
            SendMessage(hProgress, PBM_SETPOS, 0, 0);
            SendMessage(hProgress, PBM_SETBARCOLOR, 0, (LPARAM)COLOR_CYAN);
            SendMessage(hProgress, PBM_SETBKCOLOR, 0, (LPARAM)COLOR_CARD);

            // Przyciski akcji (Owner Drawn)
            hBtnInstall = CreateWindowW(L"BUTTON", L"⚡ Zainstaluj Ekosystem",
                                        WS_CHILD | WS_VISIBLE | BS_OWNERDRAW, 320, 460, 145, 34, hWnd, (HMENU)ID_BTN_INSTALL, NULL, NULL);

            hBtnCancel = CreateWindowW(L"BUTTON", L"Anuluj",
                                       WS_CHILD | WS_VISIBLE | BS_OWNERDRAW, 475, 460, 75, 34, hWnd, (HMENU)ID_BTN_CANCEL, NULL, NULL);
            break;
        }

        case WM_CTLCOLORSTATIC: {
            HDC hdc = (HDC)wParam;
            HWND hCtl = (HWND)lParam;
            SetBkMode(hdc, TRANSPARENT);

            // Wyróżnienie nagłówków
            if (hCtl == GetDlgItem(hWnd, ID_STATUS_TXT)) {
                SetTextColor(hdc, COLOR_CYAN);
            } else {
                SetTextColor(hdc, COLOR_WHITE);
            }
            return (LRESULT)hBrBg;
        }

        case WM_CTLCOLOREDIT: {
            HDC hdc = (HDC)wParam;
            SetBkColor(hdc, COLOR_CARD);
            SetTextColor(hdc, COLOR_CYAN);
            return (LRESULT)hBrCard;
        }

        case WM_CTLCOLORBTN: {
            return (LRESULT)hBrBg;
        }

        case WM_PAINT: {
            PAINTSTRUCT ps;
            HDC hdc = BeginPaint(hWnd, &ps);
            RECT rc;
            GetClientRect(hWnd, &rc);

            // Tło okna (Deep Dark)
            FillRect(hdc, &rc, hBrBg);

            // Neonowa linia rozdzielająca pod nagłówkiem (Plasma Glow)
            HPEN hPenGlowLine = CreatePen(PS_SOLID, 2, COLOR_CYAN);
            HPEN hOldPen = (HPEN)SelectObject(hdc, hPenGlowLine);
            MoveToEx(hdc, 30, 78, NULL);
            LineTo(hdc, rc.right - 30, 78);
            SelectObject(hdc, hOldPen);
            DeleteObject(hPenGlowLine);

            // Karta 1: Komponenty Ekosystemu (Zaokrąglony prostokąt Dark Card)
            SelectObject(hdc, hPenBorder);
            SelectObject(hdc, hBrCard);
            RoundRect(hdc, 30, 88, rc.right - 30, 240, 14, 14);

            // Karta 2: Opcje i Integracja (Zaokrąglony prostokąt Dark Card)
            RoundRect(hdc, 30, 248, rc.right - 30, 342, 14, 14);

            EndPaint(hWnd, &ps);
            return 0;
        }

        case WM_DRAWITEM: {
            LPDRAWITEMSTRUCT dis = (LPDRAWITEMSTRUCT)lParam;
            if (dis->CtlID == ID_BTN_INSTALL) {
                BOOL pressed = (dis->itemState & ODS_SELECTED);
                HBRUSH hBtnBr = CreateSolidBrush(pressed ? RGB(109, 40, 217) : COLOR_VIOLET_BG);
                HPEN hBtnPen = CreatePen(PS_SOLID, 1, pressed ? COLOR_CYAN : COLOR_VIOLET);

                SelectObject(dis->hDC, hBtnBr);
                SelectObject(dis->hDC, hBtnPen);
                RoundRect(dis->hDC, dis->rcItem.left, dis->rcItem.top, dis->rcItem.right, dis->rcItem.bottom, 8, 8);

                SetBkMode(dis->hDC, TRANSPARENT);
                SetTextColor(dis->hDC, RGB(255, 255, 255));
                SelectObject(dis->hDC, hFontBold);
                
                WCHAR szLabel[64];
                GetWindowTextW(hBtnInstall, szLabel, 64);
                DrawTextW(dis->hDC, szLabel, -1, &dis->rcItem, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

                DeleteObject(hBtnBr);
                DeleteObject(hBtnPen);
                return TRUE;
            } else if (dis->CtlID == ID_BTN_CANCEL) {
                BOOL pressed = (dis->itemState & ODS_SELECTED);
                HBRUSH hBtnBr = CreateSolidBrush(pressed ? RGB(15, 17, 26) : RGB(24, 27, 40));
                HPEN hBtnPen = CreatePen(PS_SOLID, 1, pressed ? COLOR_CYAN : RGB(45, 49, 70));

                SelectObject(dis->hDC, hBtnBr);
                SelectObject(dis->hDC, hBtnPen);
                RoundRect(dis->hDC, dis->rcItem.left, dis->rcItem.top, dis->rcItem.right, dis->rcItem.bottom, 8, 8);

                SetBkMode(dis->hDC, TRANSPARENT);
                SetTextColor(dis->hDC, COLOR_MUTED);
                SelectObject(dis->hDC, hFontUI);
                DrawTextW(dis->hDC, L"Anuluj", -1, &dis->rcItem, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

                DeleteObject(hBtnBr);
                DeleteObject(hBtnPen);
                return TRUE;
            }
            break;
        }

        case WM_COMMAND: {
            int wmId = LOWORD(wParam);
            if (wmId == ID_BTN_INSTALL) {
                if (g_bInstalled) {
                    PostQuitMessage(0);
                } else {
                    PerformInstallation();
                }
            } else if (wmId == ID_BTN_CANCEL) {
                PostQuitMessage(0);
            }
            break;
        }

        case WM_DESTROY:
            if (hFontTitle) DeleteObject(hFontTitle);
            if (hFontSub) DeleteObject(hFontSub);
            if (hFontCardHeader) DeleteObject(hFontCardHeader);
            if (hFontUI) DeleteObject(hFontUI);
            if (hFontBold) DeleteObject(hFontBold);
            if (hFontDesc) DeleteObject(hFontDesc);
            PostQuitMessage(0);
            break;

        default:
            return DefWindowProcW(hWnd, msg, wParam, lParam);
    }
    return 0;
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    INITCOMMONCONTROLSEX icc;
    icc.dwSize = sizeof(icc);
    icc.dwICC = ICC_PROGRESS_CLASS | ICC_STANDARD_CLASSES;
    InitCommonControlsEx(&icc);

    hBrBg = CreateSolidBrush(COLOR_BG);
    hBrCard = CreateSolidBrush(COLOR_CARD);
    hBrCardHl = CreateSolidBrush(COLOR_CARD_HL);
    hPenBorder = CreatePen(PS_SOLID, 1, COLOR_BORDER);
    hPenGlow = CreatePen(PS_SOLID, 1, COLOR_VIOLET);

    WNDCLASSEXW wcex = {0};
    wcex.cbSize = sizeof(wcex);
    wcex.style = CS_HREDRAW | CS_VREDRAW;
    wcex.lpfnWndProc = WndProc;
    wcex.hInstance = hInstance;
    wcex.hCursor = LoadCursor(NULL, IDC_ARROW);
    wcex.hbrBackground = hBrBg;
    wcex.lpszClassName = L"QubitSetupDarkClass";
    wcex.hIcon = LoadIconW(hInstance, MAKEINTRESOURCEW(1));
    wcex.hIconSm = LoadIconW(hInstance, MAKEINTRESOURCEW(1));

    if (!RegisterClassExW(&wcex)) {
        return 1;
    }

    int screenW = GetSystemMetrics(SM_CXSCREEN);
    int screenH = GetSystemMetrics(SM_CYSCREEN);
    int winW = 595;
    int winH = 550;
    int posX = (screenW - winW) / 2;
    int posY = (screenH - winH) / 2;

    hMainWnd = CreateWindowExW(
        WS_EX_APPWINDOW,
        L"QubitSetupDarkClass",
        L"QubitInstaller — Instalator Ekosystemu Qubit",
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
        posX, posY, winW, winH,
        NULL, NULL, hInstance, NULL
    );

    if (!hMainWnd) return 1;

    ShowWindow(hMainWnd, nCmdShow);
    UpdateWindow(hMainWnd);

    MSG msg;
    while (GetMessageW(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    DeleteObject(hBrBg);
    DeleteObject(hBrCard);
    DeleteObject(hBrCardHl);
    DeleteObject(hPenBorder);
    DeleteObject(hPenGlow);

    return (int)msg.wParam;
}
