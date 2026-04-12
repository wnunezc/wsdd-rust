$ErrorActionPreference = 'Stop'

function Get-DockerDesktopPath {
    $candidates = @(
        "$env:ProgramFiles\Docker\Docker\Docker Desktop.exe",
        "${env:ProgramFiles(x86)}\Docker\Docker\Docker Desktop.exe"
    )

    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) {
            return $candidate
        }
    }

    return $null
}

function Wait-ServiceRunning {
    param([string] $Name, [int] $TimeoutSeconds = 120)

    $waitUntil = [datetime]::Now.AddSeconds($TimeoutSeconds)
    do {
        $service = Get-Service -Name $Name -ErrorAction SilentlyContinue
        if ($null -ne $service -and $service.Status -eq 'Running') {
            return $true
        }
        Start-Sleep -Seconds 1
    } until ([datetime]::Now -ge $waitUntil)

    return $false
}

function Wait-DockerReady {
    param([int] $TimeoutSeconds = 120)

    $waitUntil = [datetime]::Now.AddSeconds($TimeoutSeconds)
    do {
        $pipeOpen = Test-Path -LiteralPath \\.\pipe\docker_engine
        docker info 2>&1 | Out-Null
        if ($pipeOpen -and $LASTEXITCODE -eq 0) {
            return $true
        }
        Start-Sleep -Milliseconds 500
    } until ([datetime]::Now -ge $waitUntil)

    return $false
}

$dockerDesktopPath = Get-DockerDesktopPath
if ([string]::IsNullOrWhiteSpace($dockerDesktopPath)) {
    Write-Warning 'Error: docker-desktop-not-found'
    exit 1
}

Write-Output 'STEP 1: Starting com.docker.service'
Start-Service -Name 'com.docker.service' -ErrorAction SilentlyContinue

if (-not (Wait-ServiceRunning -Name 'com.docker.service')) {
    Write-Warning 'Error: service-timeout'
    exit 1
}

Write-Output 'STEP 2: Launching Docker Desktop'
Start-Process -FilePath $dockerDesktopPath -ErrorAction Stop

Write-Output 'STEP 3: Waiting for Docker daemon'
if (-not (Wait-DockerReady)) {
    Write-Warning 'Error: docker-timeout'
    exit 1
}

Write-Output 'Continue'
