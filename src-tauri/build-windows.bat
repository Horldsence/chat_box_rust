@echo off
echo Setting up environment for Windows build...

set VCPKGRS_DYNAMIC=1
set ORT_STRATEGY=system
set ESAXX_USE_SYSTEM_LIBS=1
set CFLAGS=/MD
set CXXFLAGS=/MD
set RUSTFLAGS=-C target-feature=-crt-static

echo Creating patched esaxx-rs...
mkdir -p ..\patches\esaxx-rs
echo diff --git a/build.rs b/build.rs > ..\patches\esaxx-rs\build.rs.patch
echo index abcdefg..hijklmn 100644 >> ..\patches\esaxx-rs\build.rs.patch
echo --- a/build.rs >> ..\patches\esaxx-rs\build.rs.patch
echo +++ b/build.rs >> ..\patches\esaxx-rs\build.rs.patch
echo @@ -10,6 +10,10 @@ fn main() { >> ..\patches\esaxx-rs\build.rs.patch
echo      let mut build = cc::Build::new(); >> ..\patches\esaxx-rs\build.rs.patch
echo      build.cpp(true); >> ..\patches\esaxx-rs\build.rs.patch
echo  >> ..\patches\esaxx-rs\build.rs.patch
echo +    #[cfg(target_os = "windows")] >> ..\patches\esaxx-rs\build.rs.patch
echo +    build.flag_if_supported("/MD") >> ..\patches\esaxx-rs\build.rs.patch
echo +         .static_crt(false); >> ..\patches\esaxx-rs\build.rs.patch
echo + >> ..\patches\esaxx-rs\build.rs.patch
echo      // Add the source files >> ..\patches\esaxx-rs\build.rs.patch
echo      build.file("esaxx/esa.cpp"); >> ..\patches\esaxx-rs\build.rs.patch
echo      build.file("esaxx/sais.cpp"); >> ..\patches\esaxx-rs\build.rs.patch

echo Building Tauri application with dynamic runtime libraries...
cargo build --release

echo Build completed.