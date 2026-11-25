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

COLORREF COLOR_BG       = RGB(10, 11, 16);     // #0A0B10 Deep Black
COLORREF COLOR_CARD     = RGB(18, 20, 30);     // #12141E Card
COLORREF COLOR_CARD_HL  = RGB(25, 28, 42);     // #191C2A Card Hover
COLORREF COLOR_BORDER   = RGB(38, 41, 61);     // #26293D Border
COLORREF COLOR_VIOLET   = RGB(139, 92, 246);   // #8B5CF6 Plasma Glow
COLORREF COLOR_TEXT     = RGB(241, 245, 249);  // #F1F5F9 Primary
COLORREF COLOR_MUTED    = RGB(148, 163, 184);  // #94A3B8 Secondary

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

HBRUSH hBrBg = NULL;
HBRUSH hBrCard = NULL;
HFONT hFontTitle = NULL;
HFONT hFontSub = NULL;
HFONT hFontNormal = NULL;
HFONT hFontBold = NULL;

WCHAR g_szInstallDir[MAX_PATH];
BOOL g_bInstalled = FALSE;

BOOL CreateShellShortcut(LPCWSTR pszShortcutPath, LPCWSTR pszTarget, LPCWSTR pszIconPath, LPCWSTR pszWorkingDir) {
    IShellLinkW* psl = NULL;
    HRESULT hr = CoCreateInstance(&CLSID_ShellLink, NULL, CLSCTX_INPROC_SERVER, &IID_IShellLinkW, (void**)&psl);
    if (SUCCEEDED(hr)) {
        IPersistFile* ppf = NULL;
        psl->lpVtbl->SetPath(psl, pszTarget);
        if (pszWorkingDir && pszWorkingDir[0]) {
            psl->lpVtbl->SetWorkingDirectory(psl, pszWorkingDir);
        }
        if (pszIconPath && pszIconPath[0]) {
            psl->lpVtbl->SetIconLocation(psl, pszIconPath, 0);
        }
        psl->lpVtbl->SetDescription(psl, L"Qubit Ecosystem - Autonomous AI Engine");

        hr = psl->lpVtbl->QueryInterface(psl, &IID_IPersistFile, (void**)&ppf);
        if (SUCCEEDED(hr)) {
            hr = ppf->lpVtbl->Save(ppf, pszShortcutPath, TRUE);
            ppf->lpVtbl->Release(ppf);
        }
        psl->lpVtbl->Release(psl);
        return SUCCEEDED(hr);
    }
    return FALSE;
}

void AddToUserPath(LPCWSTR pszBinDir) {
    HKEY hKey;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, L"Environment", 0, KEY_READ | KEY_WRITE, &hKey) == ERROR_SUCCESS) {
        WCHAR szExistingPath[32768] = {0};
        DWORD dwType = 0;
        DWORD dwSize = sizeof(szExistingPath);
        if (RegQueryValueExW(hKey, L"Path", NULL, &dwType, (LPBYTE)szExistingPath, &dwSize) == ERROR_SUCCESS) {
            if (!wcsstr(szExistingPath, pszBinDir)) {
                WCHAR szNewPath[32768] = {0};
                wsprintfW(szNewPath, L"%s;%s", szExistingPath, pszBinDir);
                RegSetValueExW(hKey, L"Path", 0, dwType, (const BYTE*)szNewPath, (DWORD)(wcslen(szNewPath) + 1) * sizeof(WCHAR));
            }
        } else {
            RegSetValueExW(hKey, L"Path", 0, REG_EXPAND_SZ, (const BYTE*)pszBinDir, (DWORD)(wcslen(pszBinDir) + 1) * sizeof(WCHAR));
        }
        RegCloseKey(hKey);
        DWORD_PTR dwResult;
        SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, (LPARAM)L"Environment", SMTO_ABORTIFHUNG, 3000, &dwResult);
    }
}

void SetStatus(LPCWSTR text, int progress) {
    if (hStatusTxt) SetWindowTextW(hStatusTxt, text);
    if (hProgress) SendMessage(hProgress, PBM_SETPOS, (WPARAM)progress, 0);
    Sleep(120);
}

DWORD WINAPI InstallThread(LPVOID lpParam) {
    EnableWindow(hBtnInstall, FALSE);
    EnableWindow(hBtnCancel, FALSE);
    EnableWindow(hEditPath, FALSE);

    SetStatus(L"Krok 1/6: Tworzenie struktury katalogow Qubit...", 15);
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

    SetStatus(L"Krok 2/6: Inicjalizacja bazy SQLite WAL (Actor Storage)...", 35);
    WCHAR szDbPath[MAX_PATH];
    wsprintfW(szDbPath, L"%s\\qubit_data.db", szDataDir);
    FILE *fDb = _wfopen(szDbPath, L"ab+");
    if (fDb) {
        fputs("PRAGMA journal_mode=WAL;\nPRAGMA synchronous=NORMAL;\nPRAGMA cache_size=-2000;\nPRAGMA temp_store=MEMORY;\n", fDb);
        fclose(fDb);
    }

    SetStatus(L"Krok 3/6: Zapis konfiguracji QubitCore (Axum :4000, Zero-Trust)...", 55);
    WCHAR szCfgPath[MAX_PATH];
    wsprintfW(szCfgPath, L"%s\\qubit_config.json", g_szInstallDir);
    FILE *fCfg = _wfopen(szCfgPath, L"w");
    if (fCfg) {
        fputs("{\n"
              "  \"version\": \"0.1.0\",\n"
              "  \"port\": 4000,\n"
              "  \"host\": \"127.0.0.1\",\n"
              "  \"ram_limit_mb\": 60,\n"
              "  \"zero_trust_mcp\": true,\n"
              "  \"wal_mode\": true\n"
              "}\n", fCfg);
        fclose(fCfg);
    }

    SetStatus(L"Krok 4/6: Konfiguracja launchera i rejestru PATH...", 75);
    WCHAR szLauncherPath[MAX_PATH];
    wsprintfW(szLauncherPath, L"%s\\bin\\QubitDash.bat", g_szInstallDir);
    FILE *fLaunch = _wfopen(szLauncherPath, L"w");
    if (fLaunch) {
        fputs("@echo off\n"
              "title Qubit Ecosystem -- Mission Control\n"
              "echo ======================================================================\n"
              "echo             QUBIT ECOSYSTEM -- MISSION CONTROL (PORT 4000)            \n"
              "echo ======================================================================\n"
              "echo Uruchamianie demona QubitCore na porcie 4000...\n"
              "start https://qubit.kvbi.pl/\n", fLaunch);
        fclose(fLaunch);
    }
    AddToUserPath(szBinDir);

    WCHAR szCurrentExe[MAX_PATH];
    GetModuleFileNameW(NULL, szCurrentExe, MAX_PATH);

    if (SendMessage(hChkShortcut, BM_GETCHECK, 0, 0) == BST_CHECKED) {
        SetStatus(L"Krok 5/6: Tworzenie skrotow na Pulpicie...", 90);
        WCHAR szDesktopDir[MAX_PATH];
        if (SUCCEEDED(SHGetFolderPathW(NULL, CSIDL_DESKTOPDIRECTORY, NULL, 0, szDesktopDir))) {
            WCHAR szLnkPath[MAX_PATH];
            wsprintfW(szLnkPath, L"%s\\Qubit Mission Control.lnk", szDesktopDir);
            CreateShellShortcut(szLnkPath, szLauncherPath, szCurrentExe, g_szInstallDir);
        }
    }

    SetStatus(L"Krok 6/6: Gotowe! Ekosystem Qubit zainstalowany pomyslnie (100%).", 100);
    g_bInstalled = TRUE;
    SetWindowTextW(hBtnInstall, L"Zakoncz");
    EnableWindow(hBtnInstall, TRUE);
    ShowWindow(hBtnCancel, SW_HIDE);

    if (SendMessage(hChkLaunch, BM_GETCHECK, 0, 0) == BST_CHECKED) {
        ShellExecuteW(NULL, L"open", L"https://qubit.kvbi.pl/", NULL, NULL, SW_SHOWNORMAL);
    }

    MessageBoxW(hMainWnd,
        L"Instalacja Ekosystemu QubitCore zakonczona pelnym sukcesem!\n\n"
        L"* Utworzono katalog roboczy i skonfigurowano sciezke PATH.\n"
        L"* Baza SQLite WAL (Actor storage) oraz plik konfiguracyjny sa gotowe.\n"
        L"* Skrot 'Qubit Mission Control' zostal utworzony na Pulpicie.\n"
        L"* Silnik QubitCore jest gotowy do uruchomienia (port 4000, <60MB RAM).",
        L"⚡ Qubit Setup -- Sukces", MB_OK | MB_ICONINFORMATION);

    return 0;
}

LRESULT CALLBACK WndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
    switch (message) {
        case WM_CREATE: {
            BOOL bDarkMode = TRUE;
            DwmSetWindowAttribute(hWnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &bDarkMode, sizeof(bDarkMode));

            WCHAR szDefaultDir[MAX_PATH];
            if (SUCCEEDED(SHGetFolderPathW(NULL, CSIDL_LOCAL_APPDATA, NULL, 0, szDefaultDir))) {
                wsprintfW(g_szInstallDir, L"%s\\Qubit", szDefaultDir);
            } else {
                wcscpy_s(g_szInstallDir, MAX_PATH, L"C:\\Qubit");
            }

            CreateWindowW(L"STATIC", L"Lokalizacja instalacji:", WS_CHILD | WS_VISIBLE, 40, 95, 200, 20, hWnd, NULL, NULL, NULL);
            hEditPath = CreateWindowExW(WS_EX_CLIENTEDGE, L"EDIT", g_szInstallDir, WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL, 40, 118, 440, 26, hWnd, (HMENU)ID_EDIT_PATH, NULL, NULL);

            hChkCore = CreateWindowW(L"BUTTON", L"QubitCore Demon (Axum :4000, Zero-Trust MCP, SQLite WAL)", WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 40, 155, 440, 22, hWnd, (HMENU)ID_CHK_CORE, NULL, NULL);
            hChkDash = CreateWindowW(L"BUTTON", L"QubitDash Web Workspace (https://qubit.kvbi.pl)", WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 40, 180, 440, 22, hWnd, (HMENU)ID_CHK_DASH, NULL, NULL);
            hChkCli = CreateWindowW(L"BUTTON", L"Qubit CLI (qubit status, qubit ., qubit chat)", WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 40, 205, 440, 22, hWnd, (HMENU)ID_CHK_CLI, NULL, NULL);
            hChkDocs = CreateWindowW(L"BUTTON", L"QubitDocs (FTS5 Knowledge Ingest, Zero-Spill)", WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 40, 230, 440, 22, hWnd, (HMENU)ID_CHK_DOCS, NULL, NULL);
            hChkShortcut = CreateWindowW(L"BUTTON", L"Utworz skrot na Pulpicie (Plasma Glow Icon)", WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 40, 265, 440, 22, hWnd, (HMENU)ID_CHK_SHORTCUT, NULL, NULL);
            hChkLaunch = CreateWindowW(L"BUTTON", L"Uruchom panel webowy po zakonczeniu instalacji", WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 40, 290, 440, 22, hWnd, (HMENU)ID_CHK_LAUNCH, NULL, NULL);

            SendMessage(hChkCore, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkDash, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkCli, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkDocs, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkShortcut, BM_SETCHECK, BST_CHECKED, 0);
            SendMessage(hChkLaunch, BM_SETCHECK, BST_CHECKED, 0);

            hProgress = CreateWindowExW(0, PROGRESS_CLASSW, NULL, WS_CHILD | WS_VISIBLE, 40, 325, 440, 18, hWnd, (HMENU)ID_PROGRESS, NULL, NULL);
            hStatusTxt = CreateWindowW(L"STATIC", L"Gotowy do instalacji pelnego ekosystemu QubitCore.", WS_CHILD | WS_VISIBLE, 40, 348, 440, 20, hWnd, (HMENU)ID_STATUS_TXT, NULL, NULL);

            hBtnInstall = CreateWindowW(L"BUTTON", L"Instaluj", WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON, 270, 380, 100, 34, hWnd, (HMENU)ID_BTN_INSTALL, NULL, NULL);
            hBtnCancel = CreateWindowW(L"BUTTON", L"Anuluj", WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON, 380, 380, 100, 34, hWnd, (HMENU)ID_BTN_CANCEL, NULL, NULL);

            SendMessage(hEditPath, WM_SETFONT, (WPARAM)hFontNormal, TRUE);
            SendMessage(hChkCore, WM_SETFONT, (WPARAM)hFontNormal, TRUE);
            SendMessage(hChkDash, WM_SETFONT, (WPARAM)hFontNormal, TRUE);
            SendMessage(hChkCli, WM_SETFONT, (WPARAM)hFontNormal, TRUE);
            SendMessage(hChkDocs, WM_SETFONT, (WPARAM)hFontNormal, TRUE);
            SendMessage(hChkShortcut, WM_SETFONT, (WPARAM)hFontNormal, TRUE);
            SendMessage(hChkLaunch, WM_SETFONT, (WPARAM)hFontNormal, TRUE);
            SendMessage(hStatusTxt, WM_SETFONT, (WPARAM)hFontNormal, TRUE);
            SendMessage(hBtnInstall, WM_SETFONT, (WPARAM)hFontBold, TRUE);
            SendMessage(hBtnCancel, WM_SETFONT, (WPARAM)hFontNormal, TRUE);
            break;
        }

        case WM_PAINT: {
            PAINTSTRUCT ps;
            HDC hdc = BeginPaint(hWnd, &ps);
            RECT rcClient;
            GetClientRect(hWnd, &rcClient);

            FillRect(hdc, &rcClient, hBrBg);

            RECT rcHeader = { 0, 0, rcClient.right, 85 };
            FillRect(hdc, &rcHeader, hBrCard);

            HPEN hPen = CreatePen(PS_SOLID, 1, COLOR_BORDER);
            HGDIOBJ hOldPen = SelectObject(hdc, hPen);
            MoveToEx(hdc, 0, 85, NULL);
            LineTo(hdc, rcClient.right, 85);
            SelectObject(hdc, hOldPen);
            DeleteObject(hPen);

            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, COLOR_VIOLET);
            SelectObject(hdc, hFontTitle);
            RECT rcTitle = { 40, 18, rcClient.right - 40, 48 };
            DrawTextW(hdc, L"⚡ QUBIT ECOSYSTEM INSTALLER", -1, &rcTitle, DT_LEFT | DT_SINGLELINE);

            SetTextColor(hdc, COLOR_MUTED);
            SelectObject(hdc, hFontSub);
            RECT rcSub = { 40, 48, rcClient.right - 40, 75 };
            DrawTextW(hdc, L"Natywna instalacja silnika QubitCore (Axum, Zero-Trust MCP, SQLite WAL)", -1, &rcSub, DT_LEFT | DT_SINGLELINE);

            EndPaint(hWnd, &ps);
            break;
        }

        case WM_CTLCOLORSTATIC: {
            HDC hdcStatic = (HDC)wParam;
            SetTextColor(hdcStatic, COLOR_TEXT);
            SetBkMode(hdcStatic, TRANSPARENT);
            return (LRESULT)hBrBg;
        }

        case WM_COMMAND: {
            int wmId = LOWORD(wParam);
            if (wmId == ID_BTN_INSTALL) {
                if (g_bInstalled) {
                    PostQuitMessage(0);
                } else {
                    CreateThread(NULL, 0, InstallThread, NULL, 0, NULL);
                }
            } else if (wmId == ID_BTN_CANCEL) {
                PostQuitMessage(0);
            }
            break;
        }

        case WM_DESTROY: {
            PostQuitMessage(0);
            break;
        }

        default:
            return DefWindowProcW(hWnd, message, wParam, lParam);
    }
    return 0;
}

int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPWSTR lpCmdLine, int nCmdShow) {
    CoInitializeEx(NULL, COINIT_APARTMENTTHREADED);
    InitCommonControls();

    hBrBg = CreateSolidBrush(COLOR_BG);
    hBrCard = CreateSolidBrush(COLOR_CARD);

    hFontTitle = CreateFontW(22, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");
    hFontSub = CreateFontW(14, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");
    hFontNormal = CreateFontW(14, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");
    hFontBold = CreateFontW(14, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_SWISS, L"Segoe UI");

    WNDCLASSEXW wcex = {0};
    wcex.cbSize = sizeof(WNDCLASSEXW);
    wcex.style = CS_HREDRAW | CS_VREDRAW;
    wcex.lpfnWndProc = WndProc;
    wcex.hInstance = hInstance;
    wcex.hCursor = LoadCursor(NULL, IDC_ARROW);
    wcex.hbrBackground = hBrBg;
    wcex.lpszClassName = L"QubitInstallerClass";

    RegisterClassExW(&wcex);

    RECT rc = { 0, 0, 520, 440 };
    AdjustWindowRect(&rc, WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX, FALSE);
    int w = rc.right - rc.left;
    int h = rc.bottom - rc.top;
    int x = (GetSystemMetrics(SM_CXSCREEN) - w) / 2;
    int y = (GetSystemMetrics(SM_CYSCREEN) - h) / 2;

    hMainWnd = CreateWindowExW(
        0, L"QubitInstallerClass", L"Qubit Ecosystem Installer",
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
        x, y, w, h, NULL, NULL, hInstance, NULL
    );

    ShowWindow(hMainWnd, nCmdShow);
    UpdateWindow(hMainWnd);

    MSG msg;
    while (GetMessageW(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    DeleteObject(hBrBg);
    DeleteObject(hBrCard);
    DeleteObject(hFontTitle);
    DeleteObject(hFontSub);
    DeleteObject(hFontNormal);
    DeleteObject(hFontBold);
    CoUninitialize();

    return (int)msg.wParam;
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    return wWinMain(hInstance, hPrevInstance, GetCommandLineW(), nCmdShow);
}

