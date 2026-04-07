function Stop-Docker {
    param()

    $ErrorActionPreference = 'Continue'
    Stop-Service -Name "com.docker.*" -ErrorAction SilentlyContinue
    Set-Service -Name "com.docker.service" -StartupType Automatic -ErrorAction SilentlyContinue

    $processesToStop = @('Docker Desktop', 'com.docker.backend', 'com.docker.extensions')
    $processesToStop | ForEach-Object {
        Get-Process -Name $_ -ErrorAction Ignore | Stop-Process -Force -ErrorAction Ignore
    }

    Write-Host 'Continue'
}


Stop-Docker