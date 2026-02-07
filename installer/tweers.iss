#define MyAppName "TweeRS"
#ifndef MyAppVersion
  #error "MyAppVersion must be defined via /D flag: iscc /DMyAppVersion=x.y.z tweers.iss"
#endif
#define MyAppPublisher "Raven-Book"
#define MyAppURL "https://github.com/Raven-Book/TweeRS"
#define MyAppExeName "tweers.exe"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
DefaultDirName={localappdata}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputBaseFilename=TweeRS-{#MyAppVersion}-setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ChangesEnvironment=yes
PrivilegesRequired=lowest

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "tweers.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\test\story-format\*"; DestDir: "{app}\story-format"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"

[Registry]
Root: HKCU; Subkey: "Environment"; \
    ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; \
    Check: NeedsAddPath('{app}')

[Code]
function NeedsAddPath(Param: string): boolean;
var
    OrigPath: string;
begin
    if not RegQueryStringValue(HKCU,
        'Environment',
        'Path', OrigPath)
    then begin
        Result := True;
        exit;
    end;
    Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
end;
