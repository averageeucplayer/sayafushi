$appName = "Sayafushi";
$signingPrivateKeyPath = ".\~\.tauri\sayafushi.key"
$tauriConfigPath = "app/tauri.windows.conf.json"
$secretsPath = ".secrets.json"
$certBase64Path = "cert.txt"

try {
    $cert = Get-ChildItem -Path Cert:\CurrentUser\My | Where-Object {
        $_.Subject -like "*CN=$appName*"
    }

    $json = Get-Content $tauriConfigPath -Raw | ConvertFrom-Json
    $json.bundle.windows.certificateThumbprint = $cert.Thumbprint
    $json | ConvertTo-Json -Depth 10 | Set-Content -Path $tauriConfigPath -Force
    Write-Output "certificateThumbprint: $($cert.Thumbprint)"

    $json = Get-Content -Path $secretsPath -Raw
    $secrets = $json | ConvertFrom-Json

    $signingPrivateKey = Get-Content -Path $signingPrivateKeyPath -Raw
    $certBase64 = Get-Content -Path $certBase64Path -Raw

    $env:TAURI_SIGNING_PRIVATE_KEY = $signingPrivateKey
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD
    $env:WINDOWS_CERTIFICATE = $certBase64
    $env:WINDOWS_CERTIFICATE_PASSWORD = $secrets.WINDOWS_CERTIFICATE_PASSWORD

    Write-Output "TAURI_SIGNING_PRIVATE_KEY_PASSWORD: $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD"
    Write-Output "WINDOWS_CERTIFICATE_PASSWORD: $env:WINDOWS_CERTIFICATE_PASSWORD"
    $env:RUST_LOG="tauri=debug"
    $env:TAURI_LOG_LEVEL="debug"

    cargo tauri build --verbose --debug --features "production"

} catch {
    Write-Error "An error occurred: $_"
}