# certmgr.msc

$storePath = "Cert:\CurrentUser\My"
$tauriConfigPath = "app/tauri.conf.json"
$pfxPath = "cert.pfx"
$base64Path = "cert.txt"

Get-ChildItem -Path $storePath | Where-Object { $_.FriendlyName -eq $appName } | ForEach-Object {
    Write-Host "Removing certificate: $($_.Subject)"
    Remove-Item -Path $_.PSPath
}

$tauriConfig = Get-Content $tauriConfigPath | ConvertFrom-Json
$appName = $tauriConfig.productName
Write-Host "App Name: $appName"

$chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
$passwordStr = -join ((1..20) | ForEach-Object { $chars[(Get-Random -Maximum $chars.Length)] })
Write-Host "Generated Password: $passwordStr"

$gitName = git config user.name
$gitEmail = git config user.email
$subject = "CN=$appName, O=$gitName, E=$gitEmail"
Write-Host $subject

$cert = New-SelfSignedCertificate -Type CodeSigningCert `
    -Subject $subject `
    -CertStoreLocation $storePath `
    -FriendlyName $appName

$password = ConvertTo-SecureString -String $passwordStr -Force -AsPlainText

Export-PfxCertificate -Cert $cert -FilePath $pfxPath -Password $password | Out-Null
Write-Host "Exported PFX to $pfxPath"

certutil -encode $pfxPath $base64Path
Write-Host "Base64 certificate saved to $base64Path"