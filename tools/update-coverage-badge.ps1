param(
    [string]$covSummary = ""
)

$currentDir = Get-Location
$appFolder = Join-Path $currentDir "app" 
$markdownPath = "README.md"

if ($covSummary) {
    $covSummaryPath = Resolve-Path $covSummary -ErrorAction SilentlyContinue
} else {
    $covSummaryPath = ""
}

Push-Location

try {
    Set-Location $appFolder
    Write-Host "Changed location to $appFolder"

    if ($covSummaryPath -and (Test-Path $covSummaryPath)) {
        Write-Host "Reading coverage summary from file: $covSummaryPath"
        $summary = Get-Content $covSummaryPath -Raw
    } else {
        Write-Host "No file specified, running 'cargo llvm-cov --summary-only'..."
        try {
            $summary = cargo llvm-cov --summary-only 2>&1
        } catch {
            Write-Error "Failed to run cargo llvm-cov: $_"
            exit 1
        }
    }
} finally {
    Pop-Location
    Write-Host "Returned to original location: $currentDir"
}

$totalPattern = 'TOTAL\s+\d+\s+\d+\s+([\d.]+)%\s+\d+\s+\d+\s+([\d.]+)%\s+\d+\s+\d+\s+([\d.]+)%'
$totalMatch   = [regex]::Match($summary, $totalPattern)

if ($totalMatch.Success) {
    $linesPct     = $totalMatch.Groups[1].Value
    $functionsPct = $totalMatch.Groups[2].Value
    $regionsPct   = $totalMatch.Groups[3].Value

    Write-Host "Lines: $linesPct%, Functions: $functionsPct%, Regions: $regionsPct%"
} else {
    Write-Warning "Could not parse TOTAL line from coverage summary"
}

$linesPctValue = [double]$linesPct

if ($linesPctValue -ge 90) {
    $color = "brightgreen"
} elseif ($linesPctValue -ge 80) {
    $color = "green"
} elseif ($linesPctValue -ge 70) {
    $color = "yellowgreen"
} elseif ($linesPctValue -ge 60) {
    $color = "yellow"
} elseif ($linesPctValue -ge 50) {
    $color = "orange"
} elseif ($linesPctValue -ge 30) {
    $color = "orange-red"
} else {
    $color = "red"
}

Write-Host "Coverage: $linesPctValue%, Color: $color"

$newBadge = "![coverage](https://img.shields.io/badge/coverage-$linesPct%25-$color)"

if (Test-Path $markdownPath) {
    $mdContent = Get-Content -Raw $markdownPath

    $updated = [regex]::Replace(
        $mdContent,
        '!\[coverage\]\(https://img\.shields\.io/badge/coverage-[0-9.]+%25-[a-zA-Z0-9-]+\)',
        $newBadge,
        [System.Text.RegularExpressions.RegexOptions]::IgnoreCase
    )

    Set-Content -Path $markdownPath -Value $updated -Encoding UTF8
    Write-Host "Coverage badge updated in $markdownPath"
} else {
    Write-Error "$markdownPath not found!"
}