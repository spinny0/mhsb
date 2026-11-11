# MHSB

Making Hair Sims Better.

**[Download for Windows](https://github.com/spinny0/mhsb/releases/download/v0.2.1/MHSB-0.2.1-Windows-x64.zip)**

Extract the ZIP and run **MHSB.exe**. Keep the `environments` folder beside it.

Paint hair anywhere on the human model, change its length and curl, comb it into shape, and see how it moves in the wind.

## Build it

You need Rust, the Windows C++ build tools and Windows SDK. From this folder, run:

```powershell
.\Build.ps1
```

The script builds the app and downloads the sky files from the existing release. Run the resulting `MHSB.exe` in this folder. Use `-SkipSkies` if you already have the sky files or only want to build.

The repository contains the native app source and the assets it needs to build. The included icons, model data and hairstyle thumbnails are used inside the app. Large sky files are kept in the release download. There is no browser version or old-version clutter here.

Asset credits are in [NOTICE.txt](NOTICE.txt).
