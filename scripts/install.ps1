[CmdletBinding()]
param(
  [ValidateSet('auto', 'exe', 'msi')]
  [string]$Method = 'auto',

  [string]$Version = 'latest',

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
  'User-Agent' = 'NotifyTrap-Windows-CLI-Installer'
}
$Release = Invoke-RestMethod -Uri $ReleaseUrl -Headers $Headers

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

$TemporaryDirectory = Join-Path ([System.IO.Path]::GetTempPath()) ("notifytrap-install-" + [System.Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $TemporaryDirectory -Force | Out-Null

try {
  $InstallerPath = Join-Path $TemporaryDirectory $SelectedAsset.name
  Write-Host "[NotifyTrap] Downloading $($SelectedAsset.name)..." -ForegroundColor Cyan
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

  Write-Host "[NotifyTrap] Installation flow finished." -ForegroundColor Green
} finally {
  if (Test-Path $TemporaryDirectory) {
    Remove-Item -Path $TemporaryDirectory -Recurse -Force -ErrorAction SilentlyContinue
  }
}
