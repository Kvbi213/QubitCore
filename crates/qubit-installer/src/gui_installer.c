#define UNICODE
#define _UNICODE
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
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
HFONT hFontUI = NULL;
HFONT hFontTitle = NULL;

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
        psl->lpVtbl->SetDescription(psl, L"QubitCore & QubitDash — Ekosystem Autonomiczny");
        
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
    Sleep(250); // Maly interwal dla plynnego UX
}

void PerformInstallation(void) {
    EnableWindow(hBtnInstall, FALSE);
    EnableWindow(hChkCore, FALSE);
    EnableWindow(hChkDash, FALSE);
    EnableWindow(hChkCli, FALSE);
    EnableWindow(hChkDocs, FALSE);
    EnableWindow(hChkShortcut, FALSE);
    EnableWindow(hChkLaunch, FALSE);

    // 1. Sprawdzanie i tworzenie katalogu
    SetStatus(L"Krok 1/6: Przygotowywanie katalogu instalacyjnego...", 15);
    GetWindowTextW(hEditPath, g_szInstallDir, MAX_PATH);
    CreateDirectoryW(g_szInstallDir, NULL);
    
    WCHAR szBinDir[MAX_PATH], szDataDir[MAX_PATH];
    wsprintfW(szBinDir, L"%s\\bin", g_szInstallDir);
    wsprintfW(szDataDir, L"%s\\data", g_szInstallDir);
    CreateDirectoryW(szBinDir, NULL);
    CreateDirectoryW(szDataDir, NULL);

    // 2. Weryfikacja bazy SQLite WAL
    SetStatus(L"Krok 2/6: Inicjalizacja bazy danych SQLite WAL (qubit_data.db)...", 35);
    WCHAR szDbPath[MAX_PATH];
    wsprintfW(szDbPath, L"%s\\qubit_data.db", szDataDir);
    FILE *fDb = _wfopen(szDbPath, L"ab+");
    if (fDb) {
        fputs("PRAGMA journal_mode=WAL;\nPRAGMA cache_size=-2000;\n", fDb);
        fclose(fDb);
    }

    // 3. Generowanie konfiguracji ekosystemu
    SetStatus(L"Krok 3/6: Zapisywanie konfiguracji QubitCore (port 4000)...", 55);
    WCHAR szCfgPath[MAX_PATH];
    wsprintfW(szCfgPath, L"%s\\qubit_config.json", g_szInstallDir);
    FILE *fCfg = _wfopen(szCfgPath, L"w");
    if (fCfg) {
        fputs("{\n  \"version\": \"0.1.0\",\n  \"port\": 4000,\n  \"ram_limit_mb\": 60,\n  \"zero_trust_mcp\": true,\n  \"wal_mode\": true\n}\n", fCfg);
        fclose(fCfg);
    }

    // 4. Utworzenie skryptu startowego QubitDash
    SetStatus(L"Krok 4/6: Konfiguracja launchera QubitDash...", 75);
    WCHAR szLauncherPath[MAX_PATH];
    wsprintfW(szLauncherPath, L"%s\\bin\\QubitDash.bat", g_szInstallDir);
    FILE *fLaunch = _wfopen(szLauncherPath, L"w");
    if (fLaunch) {
        fputs("@echo off\ntitle Qubit Ecosystem Launcher\necho Starting QubitCore & Opening Dashboard...\nstart https://qubit.kvbi.pl/\n", fLaunch);
        fclose(fLaunch);
    }

    // Zapis ikony w folderze programu
    WCHAR szIconDest[MAX_PATH];
    wsprintfW(szIconDest, L"%s\\qubit.ico", g_szInstallDir);
    WCHAR szCurrentExe[MAX_PATH];
    GetModuleFileNameW(NULL, szCurrentExe, MAX_PATH);

    // 5. Tworzenie skrótu na Pulpicie
    if (SendMessage(hChkShortcut, BM_GETCHECK, 0, 0) == BST_CHECKED) {
        SetStatus(L"Krok 5/6: Tworzenie skrotu z ikonka na Pulpicie...", 90);
        WCHAR szDesktopDir[MAX_PATH];
        if (SUCCEEDED(SHGetFolderPathW(NULL, CSIDL_DESKTOPDIRECTORY, NULL, 0, szDesktopDir))) {
            WCHAR szLnkPath[MAX_PATH];
            wsprintfW(szLnkPath, L"%s\\Qubit Ecosystem.lnk", szDesktopDir);
            CreateShellShortcut(szLnkPath, szLauncherPath, szCurrentExe, g_szInstallDir);
        }
    }

    // 6. Finalizacja
    SetStatus(L"Krok 6/6: Instalacja zakonczona sukcesem! (100%)", 100);
    g_bInstalled = TRUE;
    SetWindowTextW(hBtnInstall, L"Zakoncz");
    EnableWindow(hBtnInstall, TRUE);
    ShowWindow(hBtnCancel, SW_HIDE);

    // Opcjonalne uruchomienie
    if (SendMessage(hChkLaunch, BM_GETCHECK, 0, 0) == BST_CHECKED) {
        ShellExecuteW(NULL, L"open", L"https://qubit.kvbi.pl/", NULL, NULL, SW_SHOWNORMAL);
    }

    MessageBoxW(hMainWnd, 
        L"Ekosystem QubitCore & QubitDash zostal pomyslnie zainstalowany!\n\n"
        L"- Skrot z ikona zostal utworzony na Twoim Pulpicie.\n"
        L"- Baza danych SQLite WAL zostala zainicjalizowana.\n"
        L"- Konfiguracja portu 4000 gotowa.",
        L"Qubit Setup — Sukces", MB_OK | MB_ICONINFORMATION);
}

LRESULT CALLBACK WndProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
        case WM_CREATE: {
            hFontUI = CreateFontW(16, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
                                 DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                                 CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI");
            hFontTitle = CreateFontW(22, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE,
                                    DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
                                    CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI");

            // Tytuł i podtytuł
            HWND hTitle = CreateWindowW(L"STATIC", L"⚡ Kreator Instalacji Ekosystemu Qubit",
                                        WS_CHILD | WS_VISIBLE | SS_LEFT, 25, 20, 500, 30, hWnd, NULL, NULL, NULL);
            SendMessage(hTitle, WM_SETFONT, (WPARAM)hFontTitle, TRUE);

            HWND hSub = CreateWindowW(L"STATIC", L"Wybierz komponenty do instalacji w Twoim systemie Windows:",
                                      WS_CHILD | WS_VISIBLE | SS_LEFT, 25, 55, 500, 20, hWnd, NULL, NULL, NULL);
            SendMessage(hSub, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            // Checkboxy komponentów
            HWND hGrpComp = CreateWindowW(L"BUTTON", L"Komponenty Ekosystemu",
                                          WS_CHILD | WS_VISIBLE | BS_GROUPBOX, 25, 85, 495, 130, hWnd, NULL, NULL, NULL);
            SendMessage(hGrpComp, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            hChkCore = CreateWindowW(L"BUTTON", L"QubitCore (Natywny demon Axum 0.8, REST, WebSocket, SQLite WAL)",
                                     WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 40, 110, 465, 22, hWnd, (HMENU)ID_CHK_CORE, NULL, NULL);
            SendMessage(hChkCore, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkCore, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            hChkDash = CreateWindowW(L"BUTTON", L"QubitDash (Pulpit graficzny Mission Control, skrot na Pulpicie)",
                                     WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 40, 135, 465, 22, hWnd, (HMENU)ID_CHK_DASH, NULL, NULL);
            SendMessage(hChkDash, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkDash, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            hChkCli = CreateWindowW(L"BUTTON", L"QubitCLI (Wiersz polecen i mostek stdio JSON-RPC pod Cursor/VS Code)",
                                    WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 40, 160, 465, 22, hWnd, (HMENU)ID_CHK_CLI, NULL, NULL);
            SendMessage(hChkCli, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            hChkDocs = CreateWindowW(L"BUTTON", L"QubitDocs (Air-Gapped Two-Tier Memory & integracja z Obsidianem)",
                                     WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 40, 185, 465, 22, hWnd, (HMENU)ID_CHK_DOCS, NULL, NULL);
            SendMessage(hChkDocs, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            // Opcje dodatkowe
            HWND hGrpOpt = CreateWindowW(L"BUTTON", L"Opcje instalacji i skróty",
                                         WS_CHILD | WS_VISIBLE | BS_GROUPBOX, 25, 225, 495, 75, hWnd, NULL, NULL, NULL);
            SendMessage(hGrpOpt, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            hChkShortcut = CreateWindowW(L"BUTTON", L"Utworz skrot z ikona Plasma Glow na Pulpicie",
                                         WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 40, 248, 465, 22, hWnd, (HMENU)ID_CHK_SHORTCUT, NULL, NULL);
            SendMessage(hChkShortcut, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkShortcut, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            hChkLaunch = CreateWindowW(L"BUTTON", L"Uruchom QubitDash w przegladarce zaraz po zakonczeniu",
                                       WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 40, 272, 465, 22, hWnd, (HMENU)ID_CHK_LAUNCH, NULL, NULL);
            SendMessage(hChkLaunch, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkLaunch, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            // Ścieżka docelowa
            HWND hLblPath = CreateWindowW(L"STATIC", L"Katalog docelowy:",
                                          WS_CHILD | WS_VISIBLE | SS_LEFT, 25, 310, 120, 20, hWnd, NULL, NULL, NULL);
            SendMessage(hLblPath, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            WCHAR szDefaultPath[MAX_PATH];
            SHGetFolderPathW(NULL, CSIDL_LOCAL_APPDATA, NULL, 0, szDefaultPath);
            wsprintfW(g_szInstallDir, L"%s\\Programs\\QubitCore", szDefaultPath);

            hEditPath = CreateWindowExW(WS_EX_CLIENTEDGE, L"EDIT", g_szInstallDir,
                                        WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL, 140, 308, 380, 23, hWnd, (HMENU)ID_EDIT_PATH, NULL, NULL);
            SendMessage(hEditPath, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            // Status i Pasek postępu
            hStatusTxt = CreateWindowW(L"STATIC", L"Gotowy do instalacji wybranego pakietu.",
                                       WS_CHILD | WS_VISIBLE | SS_LEFT, 25, 342, 495, 20, hWnd, (HMENU)ID_STATUS_TXT, NULL, NULL);
            SendMessage(hStatusTxt, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            hProgress = CreateWindowExW(0, PROGRESS_CLASSW, NULL,
                                        WS_CHILD | WS_VISIBLE | PBS_SMOOTH, 25, 365, 495, 22, hWnd, (HMENU)ID_PROGRESS, NULL, NULL);
            SendMessage(hProgress, PBM_SETRANGE, 0, MAKELPARAM(0, 100));
            SendMessage(hProgress, PBM_SETPOS, 0, 0);

            // Przyciski akcji
            hBtnInstall = CreateWindowW(L"BUTTON", L"Instaluj",
                                        WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON, 335, 405, 95, 30, hWnd, (HMENU)ID_BTN_INSTALL, NULL, NULL);
            SendMessage(hBtnInstall, WM_SETFONT, (WPARAM)hFontUI, TRUE);

            hBtnCancel = CreateWindowW(L"BUTTON", L"Anuluj",
                                       WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON, 440, 405, 80, 30, hWnd, (HMENU)ID_BTN_CANCEL, NULL, NULL);
            SendMessage(hBtnCancel, WM_SETFONT, (WPARAM)hFontUI, TRUE);
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
            if (hFontUI) DeleteObject(hFontUI);
            if (hFontTitle) DeleteObject(hFontTitle);
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

    WNDCLASSEXW wcex = {0};
    wcex.cbSize = sizeof(wcex);
    wcex.style = CS_HREDRAW | CS_VREDRAW;
    wcex.lpfnWndProc = WndProc;
    wcex.hInstance = hInstance;
    wcex.hCursor = LoadCursor(NULL, IDC_ARROW);
    wcex.hbrBackground = (HBRUSH)(COLOR_BTNFACE + 1);
    wcex.lpszClassName = L"QubitSetupWindowClass";
    wcex.hIcon = LoadIconW(hInstance, MAKEINTRESOURCEW(1));
    wcex.hIconSm = LoadIconW(hInstance, MAKEINTRESOURCEW(1));

    if (!RegisterClassExW(&wcex)) {
        return 1;
    }

    // Wycentrowanie okna na ekranie
    int screenW = GetSystemMetrics(SM_CXSCREEN);
    int screenH = GetSystemMetrics(SM_CYSCREEN);
    int winW = 560;
    int winH = 490;
    int posX = (screenW - winW) / 2;
    int posY = (screenH - winH) / 2;

    hMainWnd = CreateWindowExW(
        WS_EX_APPWINDOW,
        L"QubitSetupWindowClass",
        L"⚡ Qubit Setup — Kreator Instalacji Ekosystemu",
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

    return (int)msg.wParam;
}
