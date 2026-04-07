## Check for Docker Desktop (File Detection Method)
$DockerDesktopExe = (Get-ChildItem -Path "C:\Program Files\Docker\Docker\Docker Desktop.exe","C:\Program Files (x86)\Docker\Docker\Docker Desktop.exe" -ErrorAction SilentlyContinue)

if ($null -eq $DockerDesktopExe) {
    Write-Host ""
    return;
}

$DockerDesktopPath = $($DockerDesktopExe.FullName).Replace("C:\Program Files\","").Replace("C:\Program Files (x86)\","")
$FileVersion = (Get-Item -Path "$($DockerDesktopExe.FullName)" -ErrorAction SilentlyContinue).VersionInfo.FileVersion

## Check if FileVersion is null
if ($null -eq $FileVersion) {
    Write-Host ""
    return;
}

## Execute Detection Logic
If([String](Get-Item -Path "$Env:ProgramFiles\$DockerDesktopPath","${Env:ProgramFiles(x86)}\$DockerDesktopPath" -ErrorAction SilentlyContinue).VersionInfo.FileVersion -ge "$FileVersion"){
    $dockerIsRunning = (docker ps 2>&1) -match '^(?!error)'
    if($dockerIsRunning)
    {
        Write-Host "Running"
    }
    else
    {
        Write-Host ""
        return;
    }
}
else 
{
    Write-Host ""
    return;
}
