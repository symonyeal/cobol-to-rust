# Windows GnuCOBOL reference profile

## Identity and provenance

The execution host is `x86_64-windows`; the reference compiler and generated executables are `i686-windows`. Keep these architectures distinct in the local profile. The pinned compiler reports GnuCOBOL 3.2.0, MinGW GCC 9.2.0, 32-bit mode, GMP, cJSON, PDCurses, and VBISAM.

The official [GnuCOBOL 3.2 Windows archive](https://downloads.sourceforge.net/project/gnucobol/gnucobol/3.2/gnucobol-3.2_win.zip) is a source tree, not a ready-to-run compiler. Its SHA-256 is `98e16ec53d1389de9a392f1c565493acd610dc0de15a68af779b0f6d5283b62a`. Its `build_windows/README.txt` describes a native Visual Studio build and external native dependencies.

The runnable reference compiler is the VBI binary package published by [Arnold Trembley](https://www.arnoldtrembley.com/GnuCOBOL.htm), who identifies it as GnuCOBOL 3.2.0 built with MinGW GCC 9.2.0 and including VBISAM 2.1.1, GMP 6.2.1, and PDCursesMod 4.3.7. The package is `GC32-VBI-SP1-rename-7z-to-exe.7z`, SHA-256 `546523b293cd6a3dabb7ae173af2bee8a11d850db3f91ef3201d8f6b2e4e9d72`. This is a contributor-built Windows distribution, not an upstream-produced binary. It is pinned because the upstream Windows archive contains source only and this package supplies the compatible compiler and runtime while retaining GnuCOBOL 3.2.0.

The package contains a 7-Zip self-extracting archive despite its `.7z` suffix. The provisioner copies it to `.exe` and extracts it with the versioned official [7-Zip 26.04 console extractor](https://sourceforge.net/projects/sevenzip/files/7-Zip/26.04/7zr.exe/), SHA-256 `256feca8e274e5da655e2a284fabafd9f554365eb164862089dacd4e8276d282`. All three downloads are retained outside the repository in the selected `reference-env` directory.

## Provisioning

On Windows 10 or later with Python and `curl.exe`, run:

```powershell
python scripts/provision-reference.py --reference-env "<path to cobol-to-rust-work\reference-env>"
```

The script uses `curl.exe --ssl-no-revoke` for Windows hosts where certificate revocation checks cannot reach their responder. It verifies every download hash, safely extracts the source archive, verifies the compiler tree, captures `cobc --info`, builds and runs a COBOL date smoke program, then writes the local `profile.json`. Download, extraction, compiler, and smoke outputs are retained under `reference-env\logs`. The local profile contains an absolute root and must remain outside the public repository.

## Runner profile

`profile.json` conforms to the runner's profile schema. It pins every file under the extracted compiler root by SHA-256 and records a hash of the exact raw `cobc --info` stdout. `root` is an absolute local path; `cobc` and `files` paths are relative and use `/` separators.

The profile environment is applied with a cleared inherited environment. `${ROOT}` expands to the local compiler root:

| Variable | Value |
| --- | --- |
| `COB_MAIN_DIR` | `${ROOT}\` |
| `COB_CONFIG_DIR` | `${ROOT}\config` |
| `COB_COPY_DIR` | `${ROOT}\copy` |
| `COB_CFLAGS` | `-I"${ROOT}\include"` |
| `COB_LDFLAGS` | `-L"${ROOT}\lib"` |
| `COB_LIBRARY_PATH` | `${ROOT}\extras` |
| `COB_RUNTIME_CONFIG` | `${ROOT}\config\runtime.cfg` |
| `COB_CURRENT_DATE` | `2026/11/06 12:34:56` |
| `LOCALEDIR` | `${ROOT}\locale` |
| `TZ` | `UTC0` |
| `LANG`, `LC_ALL` | `C` |
| `PATH` | `${ROOT}\bin;C:\Windows\System32;C:\Windows` |
| `SystemRoot` | `C:\Windows` |
| `TEMP`, `TMP` | `${ROOT}\..\tmp` |

## Verification evidence

Captured on 2026-11-06 on Windows 11 x64:

- `cobc --info`: exit 0, stderr empty; reports GnuCOBOL 3.2.0, MinGW GCC 9.2.0, and `i686-pc-mingw32`; raw stdout SHA-256 `9073ad0b47693fc618680c3d39274e3e9e488b59bb6167be0ccbdee2fb3859f1`.
- Compile/link command: `cobc.exe -x -fixed -std=default -o smoke-date.exe smoke-date.cob`; exit 0, stdout and stderr empty.
- Smoke executable: exit 0, stderr empty. With `COB_CURRENT_DATE=2026/11/06 12:34:56` and `TZ=UTC0`, `ACCEPT ... FROM DATE` returned `261106` and `FUNCTION CURRENT-DATE` returned a value beginning `20261106123456` with offset `+0000`.
- GnuCOBOL 3.2's `COB_CURRENT_DATE` accepts a date/time override, as documented in the bundled `config/runtime.cfg`. It fixes the date and seconds, but the two fractional digits in `FUNCTION CURRENT-DATE` still vary with wall time. The selected COBOL source uses `ACCEPT ... FROM DATE`; do not treat the full `FUNCTION CURRENT-DATE` value as deterministic under this profile.

The local profile and raw smoke captures are retained in `reference-env`; no absolute local path is included in this document.
