$ErrorActionPreference = 'Continue'

function Get-WslServiceName {
    foreach ($name in @('WslService', 'LxssManager')) {
        if (Get-Service -Name $name -ErrorAction SilentlyContinue) {
            return $name
        }
    }

    return $null
}

function Wait-ServiceRunning {
    param([string] $Name, [int] $TimeoutSeconds = 30)

    $waitUntil = [datetime]::Now.AddSeconds($TimeoutSeconds)
    do {
        $service = Get-Service -Name $Name -ErrorAction SilentlyContinue
        if ($null -ne $service -and $service.Status -eq 'Running') {
            return $true
        }
        Start-Sleep -Milliseconds 500
    } until ([datetime]::Now -ge $waitUntil)

    return $false
}

$serviceName = Get-WslServiceName

if (-not [string]::IsNullOrWhiteSpace($serviceName)) {
    Write-Output "STEP 1: Starting $serviceName"
    Start-Service -Name $serviceName -ErrorAction SilentlyContinue

    if (-not (Wait-ServiceRunning -Name $serviceName)) {
        Write-Warning 'Error: wsl-service-timeout'
        exit 1
    }
}

Write-Output 'STEP 2: Validating wsl.exe'
wsl.exe --status 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Warning 'Error: wsl-status-failed'
    exit 1
}

Write-Output 'Continue'
