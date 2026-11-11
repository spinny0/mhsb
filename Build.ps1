param([switch]$SkipSkies)
$ErrorActionPreference='Stop'
$taskRoot=$PSScriptRoot
& cargo build --manifest-path "$taskRoot/native/Cargo.toml" --target-dir "$taskRoot/native/target" --release --locked -j 2
if($LASTEXITCODE -ne 0){throw 'Build failed'}
Copy-Item -LiteralPath "$taskRoot/native/target/release/mhsb.exe" -Destination "$taskRoot/MHSB.exe" -Force
if(!$SkipSkies){
    $taskNames=@('mud-0.env.gz','mud-1.env.gz','mud-2.env.gz','mud-3.env.gz','citrus-0.env.gz','citrus-1.env.gz','citrus-2.env.gz','citrus-3.env.gz')
    $taskMissing=@($taskNames | Where-Object {!(Test-Path -LiteralPath "$taskRoot/environments/$_")})
    if($taskMissing.Count){
        Write-Host 'Downloading the skies from the existing release...'
        $taskTemporary=Join-Path ([IO.Path]::GetTempPath()) ([IO.Path]::GetRandomFileName()+'.zip')
        try {
            Invoke-WebRequest -Uri 'https://github.com/spinny0/mhsb/releases/download/v0.2.1/MHSB-0.2.1-Windows-x64.zip' -OutFile $taskTemporary -UseBasicParsing
            if((Get-FileHash -LiteralPath $taskTemporary -Algorithm SHA256).Hash.ToLowerInvariant() -ne '742f5b3b3754a67d01fe2d7d84d809fa0735ee7d2b7d64ab0e51c969097b0502'){throw 'Sky download checksum mismatch'}
            Add-Type -AssemblyName System.IO.Compression.FileSystem
            $taskArchive=[IO.Compression.ZipFile]::OpenRead($taskTemporary)
            try {foreach($taskName in $taskMissing){
                $taskEntry=$taskArchive.GetEntry("MHSB/environments/$taskName")
                if(!$taskEntry){throw "Missing sky: $taskName"}
                [IO.Compression.ZipFileExtensions]::ExtractToFile($taskEntry,"$taskRoot/environments/$taskName",$true)
            }} finally {$taskArchive.Dispose()}
        } finally {if(Test-Path -LiteralPath $taskTemporary){Remove-Item -LiteralPath $taskTemporary -Force}}
    }
}
Write-Host 'Built MHSB.exe. Run it from this folder.'
