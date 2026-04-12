$ErrorActionPreference = 'Stop'

function Get-DockerSettingsPath {
    $candidates = @(
        "$env:APPDATA\Docker\settings-store.json",
        "$env:APPDATA\Docker\settings.json"
    )

    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) {
            return $candidate
        }
    }

    return $null
}

function Test-SettingValue {
    param(
        [Parameter(Mandatory = $true)] $Object,
        [Parameter(Mandatory = $true)] [string] $Name,
        [Parameter(Mandatory = $true)] $Expected,
        [switch] $Optional
    )

    $property = $Object.PSObject.Properties[$Name]
    if ($null -eq $property) {
        return $Optional.IsPresent
    }

    return $property.Value -eq $Expected
}

try {
    $settingsPath = Get-DockerSettingsPath
    if ([string]::IsNullOrWhiteSpace($settingsPath)) {
        Write-Output ""
        return
    }

    $settingsObject = Get-Content $settingsPath -Raw | ConvertFrom-Json

    $required = @{
        exposeDockerAPIOnTCP2375 = $true
        updateHostsFile          = $true
        runWinServiceInWslMode   = $true
        useResourceSaver         = $false
        openUIOnStartupDisabled  = $true
    }

    foreach ($item in $required.GetEnumerator()) {
        if (-not (Test-SettingValue -Object $settingsObject -Name $item.Key -Expected $item.Value)) {
            Write-Output ""
            return
        }
    }

    if (-not (Test-SettingValue -Object $settingsObject -Name 'noWindowsContainers' -Expected $false -Optional)) {
        Write-Output ""
        return
    }

    Write-Output "Updated"
} catch {
    Write-Output ""
}
