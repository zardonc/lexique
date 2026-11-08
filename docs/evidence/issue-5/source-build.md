# Windows speech source materials and build steps

The separate source companion retains the full pinned software archives, original ONNX and HF configuration, the author's configuration, build controls, conversion scripts and notices. [The resource manifest](resources.json) gives exact URLs, sizes and hashes; [the source research](source-delivery-research.md) explains the GPL / AGPL delivery route and remaining model evidence gap.

Run the following from the extracted source companion root in PowerShell 7. Keep the matching Windows runtime ZIP beside it. These commands use new working directories and preserve the supplied archives.

## Reproduce the model preparation

Download the three preparation wheels plus the pinned NumPy and protobuf wheels from the manifest. These are tools for ONNX metadata preparation; the runtime ZIP uses the prepared model.

```powershell
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
$manifest = Get-Content project/docs/evidence/issue-5/resources.json -Raw | ConvertFrom-Json
New-Item -ItemType Directory prepare-downloads | Out-Null
$tools = @($manifest.preparation_downloads) + @($manifest.downloads | Where-Object name -In 'numpy', 'protobuf')
foreach ($item in $tools) {
  $path = Join-Path prepare-downloads $item.file
  Invoke-WebRequest $item.url -OutFile $path
  if ((Get-Item $path).Length -ne $item.bytes -or
      (Get-FileHash $path -Algorithm SHA256).Hash.ToLower() -ne $item.sha256) {
    throw "resource checksum mismatch: $($item.file)"
  }
}
uv venv --python 3.14.10 prepare-python
uv pip install --python prepare-python/Scripts/python.exe --no-index --find-links prepare-downloads `
  onnx==1.23.2 ml-dtypes==0.6.0 numpy==2.5.4 protobuf==7.36.2 typing-extensions==4.16.0
prepare-python/Scripts/python.exe -B -X utf8 project/docs/evidence/issue-5/prepare.py model prepared-assets
```

The three output hashes must match `assets` in the manifest. The script preserves the serialized ONNX graph, copies the HF config, exports the phoneme ID map, and adds the nine Sherpa metadata entries. The author config is retained separately because it also contains dataset and descriptive metadata. Modification date: 2026-11-07; weights, graph, inference settings and speaker mapping are unchanged.

To reconstruct the runtime directory, acquire and validate every `downloads` item into a new directory using the same loop, then run:

```powershell
prepare-python/Scripts/python.exe -B -X utf8 project/docs/evidence/issue-5/bundle.py `
  runtime-downloads prepared-assets windows-x64-rebuilt --sources ../windows-x64-sources-v3.zip
```

Its README gives the network-denied generation and playback command. The output must not exist. The offline VC prerequisite installer remains separate from source conversion and is not executed by these commands.

## Build the Piper native frontend

Use Windows x64 MSVC / Windows SDK and a Python development environment. The pinned upstream `pyproject.toml` requires scikit-build, setuptools, wheel, CMake and Ninja; root CMake requires CMake 3.26 or later. The upstream wheel workflow uses cibuildwheel 2.23.2 and `CIBW_BUILD=cp39-*`; `setup.cfg` selects the `cp39` stable ABI. The following direct wheel build uses Python 3.9 with that same ABI selection.

```powershell
New-Item -ItemType Directory rebuild | Out-Null
tar -xf upstream/piper-639388b6317fc4731e91d53da42aea68fd4166ff.tar.gz -C rebuild
tar -xf upstream/sonic-fbf75c3d6d846bad3bb3d456cbc5d07d9fd8c104.tar.gz -C rebuild
Copy-Item upstream/espeak-ng-724808c5a83f9ef95fdd0db886ba7ba537ff224a.tar.gz rebuild
$piperSource = (Resolve-Path rebuild/piper1-gpl-639388b6317fc4731e91d53da42aea68fd4166ff).Path
$localPatch = (Resolve-Path project/docs/evidence/issue-5/piper-local-sources.patch).Path
git -C $piperSource apply $localPatch
uv venv --python 3.9 rebuild/python
uv pip install --python rebuild/python/Scripts/python.exe build==1.2.2
rebuild/python/Scripts/python.exe -m build --wheel --outdir rebuilt-wheels $piperSource
```

The supplied patch points Piper's eSpeak ExternalProject at the local complete archive and eSpeak's Sonic FetchContent at the extracted local source. It keeps upstream static linking and feature choices: ASYNC, MBROLA, LIBSONIC, LIBPCAUDIO, KLATT and SPEECHPLAYER off; EXTRA_cmn / EXTRA_ru on. The eSpeak archive already includes ucd-tools. This patch changes source acquisition only; the measured runtime uses the original upstream wheel. General build tools must be acquired before an offline rebuild.

The resulting wheel contains `espeakbridge.pyd` and generated `espeak-ng-data`; install it using `uv pip install --python <development-python> --no-deps <wheel>`. The pinned PyPI sdist lacks the native source and CMake files, so use the full archive supplied here. Preserve the original archive, local patch and per-file notices with any modified build.

## Sherpa and native dependency sources

Extract `upstream/sherpa-onnx-af2d8d81dbfa61c4cea4c568da78f3982edca005.tar.gz` to a new directory. Its `setup.py`, `cmake/` and `.github/workflows/` retain the upstream Python and native build controls. Put the supplied `eigen-5.0.1.tar.gz` in that source root to satisfy its documented local CMake search. Other permissive build dependencies follow that fixed tree's acquisition scripts; their notices are preserved in the runtime and [license directory](licenses/ATTRIBUTION.md).

The measured Sherpa binary reports 2.0.1, Git SHA `af2d8d81` and native ONNX Runtime 1.28.2. The separately installed Python ONNX Runtime wheel is 1.31.1. Published source controls establish the build route, while the wheel identities and native imports establish the tested binary set.

## Limitations

- The model author has not identified its preferred source form for modification. Original ONNX / configs and this conversion are available, but they do not establish that AGPL §1 source-form fact. Public model redistribution clearance remains open.
- Native source rebuilds and exact upstream wheel build provenance have not been verified. The local patch is checked against the pinned source; its bridge build has not been run.
- Other-platform execution and clean-machine installation are deferred. Generic compiler / build-tool acquisition is separate from the measured speech resources.
