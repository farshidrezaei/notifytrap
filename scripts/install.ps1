[CmdletBinding()]
param(
  [ValidateSet('auto', 'exe', 'msi', 'cli')]
  [string]$Method = 'auto',

  [string]$Version = 'latest',

  [switch]$CliOnly,

  [switch]$Silent
)

$ErrorActionPreference = 'Stop'

$Repository = 'farshidrezaei/notifytrap'
$ApiBaseUrl = "https://api.github.com/repos/$Repository/releases"

if ($Version -eq 'latest') {
  $ReleaseUrl = "$ApiBaseUrl/latest"
} else {
  $NormalizedTag = if ($Version.StartsWith('v')) { $Version } else { "v$Version" }
  $ReleaseUrl = "$ApiBaseUrl/tags/$NormalizedTag"
}

Write-Host "[NotifyTrap] Fetching release metadata ($Version)..." -ForegroundColor Cyan
$Headers = @{
  'Accept'     = 'application/vnd.github+json'
  'User-Agent' = 'NotifyTrap-Windows-Installer'
}
$Release = Invoke-RestMethod -Uri $ReleaseUrl -Headers $Headers

$TemporaryDirectory = Join-Path ([System.IO.Path]::GetTempPath()) ("notifytrap-install-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $TemporaryDirectory -Force | Out-Null

try {
  # CLI-only mode: Download and unpack notifytrap-cli.exe
  if ($CliOnly -or ($Method -eq 'cli')) {
    $CliAsset = $Release.assets | Where-Object { $_.name -match 'notifytrap-cli-windows.*\.zip$' } | Select-Object -First 1
    if (-not $CliAsset) {
      throw "No notifytrap-cli Windows archive found in release '$($Release.tag_name)'."
    }

    $ZipPath = Join-Path $TemporaryDirectory $CliAsset.name
    Write-Host "[NotifyTrap] Downloading $($CliAsset.name)..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $CliAsset.browser_download_url -OutFile $ZipPath -UseBasicParsing

    $TargetDir = Join-Path $env:LOCALAPPDATA "Programs\NotifyTrap\bin"
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
    Expand-Archive -Path $ZipPath -DestinationPath $TargetDir -Force

    # Ensure TargetDir is in User PATH
    $UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($UserPath -notlike "*$TargetDir*") {
      [Environment]::SetEnvironmentVariable('Path', "$UserPath;$TargetDir", 'User')
      $env:Path = "$env:Path;$TargetDir"
      Write-Host "[NotifyTrap] Added $TargetDir to User PATH." -ForegroundColor Yellow
    }

    Write-Host "[NotifyTrap] CLI installed successfully to $TargetDir\notifytrap-cli.exe" -ForegroundColor Green
    return
  }

  # Desktop + CLI Mode
  $Assets = $Release.assets | Where-Object {
    -not $_.name.EndsWith('.sig') -and -not $_.name.EndsWith('.zip')
  }

  $SelectedAsset = switch ($Method) {
    'msi' {
      $Assets | Where-Object { $_.name -match '\.msi$' } | Select-Object -First 1
    }
    'exe' {
      $Assets | Where-Object { $_.name -match '\.exe$' } | Select-Object -First 1
    }
    default {
      $ExeAsset = $Assets | Where-Object { $_.name -match '-setup\.exe$|\.exe$' } | Select-Object -First 1
      if ($ExeAsset) { $ExeAsset } else { $Assets | Where-Object { $_.name -match '\.msi$' } | Select-Object -First 1 }
    }
  }

  if (-not $SelectedAsset) {
    throw "No suitable Windows installer (.exe or .msi) was found in release '$($Release.tag_name)'."
  }

  $InstallerPath = Join-Path $TemporaryDirectory $SelectedAsset.name
  Write-Host "[NotifyTrap] Downloading desktop installer $($SelectedAsset.name)..." -ForegroundColor Cyan
  Invoke-WebRequest -Uri $SelectedAsset.browser_download_url -OutFile $InstallerPath -UseBasicParsing

  Write-Host "[NotifyTrap] Launching installer..." -ForegroundColor Cyan
  if ($InstallerPath.EndsWith('.msi')) {
    $MsiArguments = if ($Silent) { "/i `"$InstallerPath`" /passive" } else { "/i `"$InstallerPath`"" }
    Start-Process -FilePath 'msiexec.exe' -ArgumentList $MsiArguments -Wait
  } else {
    $ExeArguments = if ($Silent) { '/S' } else { '' }
    if ($ExeArguments) {
      Start-Process -FilePath $InstallerPath -ArgumentList $ExeArguments -Wait
    } else {
      Start-Process -FilePath $InstallerPath -Wait
    }
  }

  # Install standalone notifytrap-cli binary alongside desktop
  $CliAsset = $Release.assets | Where-Object { $_.name -match 'notifytrap-cli-windows.*\.zip$' } | Select-Object -First 1
  if ($CliAsset) {
    $ZipPath = Join-Path $TemporaryDirectory $CliAsset.name
    Invoke-WebRequest -Uri $CliAsset.browser_download_url -OutFile $ZipPath -UseBasicParsing
    $TargetDir = Join-Path $env:LOCALAPPDATA "Programs\NotifyTrap\bin"
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
    Expand-Archive -Path $ZipPath -DestinationPath $TargetDir -Force

    $UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($UserPath -notlike "*$TargetDir*") {
      [Environment]::SetEnvironmentVariable('Path', "$UserPath;$TargetDir", 'User')
      $env:Path = "$env:Path;$TargetDir"
    }
    Write-Host "[NotifyTrap] Both Desktop App and 'notifytrap-cli' installed." -ForegroundColor Green
  } else {
    Write-Host "[NotifyTrap] Desktop installation finished." -ForegroundColor Green
  }
} finally {
  if (Test-Path $TemporaryDirectory) {
    Remove-Item -Path $TemporaryDirectory -Recurse -Force -ErrorAction SilentlyContinue
  }
}
