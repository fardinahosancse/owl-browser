; Inno Setup script for Owl. Build with:  iscc installer\owl.iss
; The version comes from the OWL_VERSION environment variable (set by the release workflow).
#define AppName "Owl"
#define AppPublisher "fardinahosancse"
#define AppURL "https://github.com/fardinahosancse/owl-browser"
#define AppExe "Owl.exe"
#define AppVersion GetEnv("OWL_VERSION")
#if AppVersion == ""
  #define AppVersion "0.0.0"
#endif
; the Windows file-version field must be numeric (a.b.c.d): drop any "-beta" style suffix
#define DashPos Pos("-", AppVersion)
#if DashPos > 0
  #define NumVersion Copy(AppVersion, 1, DashPos - 1)
#else
  #define NumVersion AppVersion
#endif

[Setup]
; A fixed AppId lets new versions upgrade in place and uninstall cleanly.
AppId={{7C3B6F52-2A41-4C58-9D0E-5A1F6B8E3D27}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}/issues
AppUpdatesURL={#AppURL}/releases
VersionInfoVersion={#NumVersion}.0
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
; per-user by default (no administrator rights); the user may choose all users in the dialog
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
LicenseFile=..\LICENSE
SetupIconFile=..\assets\owl.ico
UninstallDisplayIcon={app}\{#AppExe}
OutputDir=Output
OutputBaseFilename=Owl-Setup-{#AppVersion}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\dist\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
; only present with the GNU toolchain; the MSVC build links the loader statically
Source: "..\dist\WebView2Loader.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent

[Code]
const
  WebView2Key = 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';
  WebView2Key32 = 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';

function WebView2Installed(): Boolean;
var
  V: String;
begin
  Result :=
    (RegQueryStringValue(HKLM, WebView2Key32, 'pv', V) and (V <> '') and (V <> '0.0.0.0')) or
    (RegQueryStringValue(HKLM, WebView2Key, 'pv', V) and (V <> '') and (V <> '0.0.0.0')) or
    (RegQueryStringValue(HKCU, WebView2Key, 'pv', V) and (V <> '') and (V <> '0.0.0.0'));
end;

function InitializeSetup(): Boolean;
var
  ErrCode: Integer;
begin
  Result := True;
  if not WebView2Installed() then
  begin
    if MsgBox('Owl needs the Microsoft Edge WebView2 Runtime, which was not found on this PC.' + #13#10 + #13#10 +
              'Open the official Microsoft download page now? Install the runtime, then run this setup again.',
              mbConfirmation, MB_YESNO) = IDYES then
      ShellExec('open', 'https://developer.microsoft.com/microsoft-edge/webview2/', '', '', SW_SHOWNORMAL, ewNoWait, ErrCode);
    Result := False;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  DataDir: String;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    DataDir := ExpandConstant('{localappdata}\Owl');
    // a silent uninstall (/VERYSILENT) keeps the data: never delete it without asking
    if DirExists(DataDir) and not UninstallSilent then
      if MsgBox('Also delete your Owl data (history, bookmarks, saved passwords, cookies and settings)?' + #13#10 +
                'This cannot be undone. Choose No to keep it for a future install.',
                mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES then
        DelTree(DataDir, True, True, True);
  end;
end;
