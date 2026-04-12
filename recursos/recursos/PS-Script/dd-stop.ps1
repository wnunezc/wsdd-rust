$ErrorActionPreference = 'Continue'

function Wait-PipeClosed {
    param([int] $TimeoutSeconds = 60)

    $waitUntil = [datetime]::Now.AddSeconds($TimeoutSeconds)
    do {
        if (-not (Test-Path -LiteralPath \\.\pipe\docker_engine)) {
            return $true
        }
        Start-Sleep -Milliseconds 500
    } until ([datetime]::Now -ge $waitUntil)

    return $false
}

Write-Output 'STEP 1: Stopping Docker UI processes'
@('Docker Desktop', 'DockerDesktop', 'com.docker.backend', 'com.docker.extensions') | ForEach-Object {
    Get-Process -Name $_ -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

Write-Output 'STEP 2: Stopping com.docker.service'
Stop-Service -Name 'com.docker.service' -Force -ErrorAction SilentlyContinue

Write-Output 'STEP 3: Waiting for Docker pipe to close'
if (-not (Wait-PipeClosed)) {
    Write-Warning 'Error: pipe-still-open'
    exit 1
}

Write-Output 'Continue'
