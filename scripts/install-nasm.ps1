$ErrorActionPreference = 'Stop'
# Handle native failures ourselves so transient package-feed errors can be retried.
$PSNativeCommandUseErrorActionPreference = $false

$maximumAttempts = 3
for ($attempt = 1; $attempt -le $maximumAttempts; $attempt++) {
    choco install nasm -y --no-progress
    $installExitCode = $LASTEXITCODE
    if ($installExitCode -eq 0 -or $installExitCode -eq 3010) {
        break
    }
    if ($attempt -eq $maximumAttempts) {
        throw "NASM installation failed after $maximumAttempts attempts (Chocolatey exit $installExitCode)."
    }
    Write-Warning "Chocolatey exited with $installExitCode; retrying NASM installation."
    Start-Sleep -Seconds (10 * $attempt)
}

$nasmDirectory = Join-Path $env:ProgramFiles 'NASM'
$nasmExecutable = Join-Path $nasmDirectory 'nasm.exe'
if (-not (Test-Path -LiteralPath $nasmExecutable -PathType Leaf)) {
    throw "Chocolatey completed, but NASM is missing: $nasmExecutable"
}
& $nasmExecutable -v
if ($LASTEXITCODE -ne 0) {
    throw "NASM version check failed with exit code $LASTEXITCODE."
}
if (-not $env:GITHUB_PATH) {
    throw 'GITHUB_PATH is required to make NASM available to subsequent workflow steps.'
}
Add-Content -LiteralPath $env:GITHUB_PATH -Value $nasmDirectory -Encoding utf8
