$certTempDirectory = "certificate"
$base64CertPath = Join-Path $certTempDirectory "cert.txt"
$pfxCertPath = Join-Path $certTempDirectory "cert.pfx"
$tauriConfigPath = "app/tauri.conf.json"
$certStoreLocation = "Cert:\CurrentUser\My"

if (-not (Test-Path $certTempDirectory)) {
    New-Item -ItemType Directory -Path $certTempDirectory | Out-Null
}

try {
    Set-Content -Path $base64CertPath -Value $env:WINDOWS_CERTIFICATE -Force

    certutil -decode $base64CertPath $pfxCertPath | Out-Null
    $password = ConvertTo-SecureString -String $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force

    $cert = Import-PfxCertificate -FilePath $pfxCertPath `
        -CertStoreLocation $certStoreLocation `
        -Password $password

    Remove-Item -Path $certTempDirectory -Recurse -Force

    $json = Get-Content $tauriConfigPath -Raw | ConvertFrom-Json
    $json.bundle.windows.certificateThumbprint = $cert.Thumbprint
    $json | ConvertTo-Json -Depth 10 | Set-Content -Path $tauriConfigPath -Force
}
catch {
    Write-Error "An error occurred: $_"
    exit 1
}