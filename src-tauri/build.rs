use log::info;

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("开始执行构建脚本");

    // // 配置 Vosk 库
    // configure_vosk();

    // 执行 tauri 构建
    tauri_build::build();
}

// fn configure_vosk() {
//     // Vosk版本和平台参数
//     let vosk_version = "0.3.45";
//     let target_os = env::var("CARGO_CFG_TARGET_OS").expect("无法获取目标操作系统");
//     let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("无法获取目标架构");

//     // 检查是否为静态链接，如果环境变量不存在，默认为动态链接
//     let is_static = env::var("CARGO_CFG_TARGET_FEATURE")
//         .map(|f| f.contains("static"))
//         .unwrap_or(false);

//     info!(
//         "目标平台: {}-{} (static={})",
//         target_os, target_arch, is_static
//     );

//     // 库文件下载地址
//     let (lib_url, lib_name) = match (&*target_os, &*target_arch) {
//         ("linux", "x86_64") => (
//             format!("https://github.com/alphacep/vosk-api/releases/download/v{}/vosk-linux-x86_64-{}.tar.gz", vosk_version, vosk_version),
//             format!("vosk-linux-x86_64-{}", vosk_version)
//         ),
//         ("macos", "aarch64") => (
//             format!("https://github.com/alphacep/vosk-api/releases/download/v{}/vosk-ios-universal-{}.tar.gz", vosk_version, vosk_version),
//             format!("vosk-ios-universal-{}", vosk_version)
//         ),
//         ("windows", "x86_64") => (
//             format!("https://github.com/alphacep/vosk-api/releases/download/v{}/vosk-win64-0.3.45.zip", vosk_version),
//             format!("vosk-win64-{}", vosk_version)
//         ),
//         ("linux", "riscv64") => (
//             format!("https://github.com/alphacep/vosk-api/releases/download/v{}/vosk-linux-riscv64-0.3.45.zip", vosk_version),
//             format!("vosk-linux-riscv64-{}", vosk_version)
//         ),
//         // 添加其他平台支持...
//         _ => panic!("不支持的平台: {}-{}", target_os, target_arch)
//     };

//     // 获取构建目录
//     let out_dir = PathBuf::from(env::var("OUT_DIR").expect("无法获取OUT_DIR"));
//     let vosk_dir = out_dir.join("vosk");
//     fs::create_dir_all(&vosk_dir).expect("无法创建vosk目录");

//     // 准备下载文件
//     let is_zip = lib_url.ends_with(".zip");
//     let archive_path = if is_zip {
//         vosk_dir.join("vosk.zip")
//     } else {
//         vosk_dir.join("vosk.tar.gz")
//     };

//     // 下载库文件
//     if !archive_path.exists() {
//         info!("从 {} 下载Vosk库", lib_url);

//         #[cfg(feature = "reqwest")]
//         {
//             let mut response = reqwest::blocking::get(&lib_url)
//                 .unwrap_or_else(|_| panic!("下载Vosk库失败，URL: {}", lib_url));
//             let mut file = fs::File::create(&archive_path).expect("无法创建下载文件");
//             copy(&mut response, &mut file).expect("无法保存下载文件");
//         }

//         #[cfg(not(feature = "reqwest"))]
//         {
//             use yoke::cartable_ptr::CartablePointerLike;
//             let response = ureq::get(&lib_url).call().expect("下载Vosk库失败");
//             let mut file = fs::File::create(&archive_path).expect("无法创建下载文件");
//             let mut reader = response.into_raw();
//             std::io::copy(&mut reader, &mut file).expect("无法保存下载文件");
//         }
//     }

//     // 解压文件
//     info!("解压Vosk库...");
//     let extract_dir = vosk_dir.join("extracted");
//     fs::create_dir_all(&extract_dir).expect("无法创建解压目录");

//     if is_zip {
//         // 解压ZIP文件
//         let file = fs::File::open(&archive_path).expect("无法打开ZIP文件");
//         let mut archive = zip::ZipArchive::new(file).expect("无法解析ZIP文件");

//         for i in 0..archive.len() {
//             let mut file = archive.by_index(i).expect("无法读取ZIP文件中的条目");
//             let outpath = match file.enclosed_name() {
//                 Some(path) => extract_dir.join(path),
//                 None => continue,
//             };

//             if file.name().ends_with('/') {
//                 fs::create_dir_all(&outpath).expect("无法创建目录");
//             } else {
//                 if let Some(p) = outpath.parent() {
//                     if !p.exists() {
//                         fs::create_dir_all(p).expect("无法创建父目录");
//                     }
//                 }
//                 let mut outfile = fs::File::create(&outpath).expect("无法创建文件");
//                 std::io::copy(&mut file, &mut outfile).expect("无法写入文件");
//             }
//         }
//     } else {
//         // 解压TAR.GZ文件
//         let tar_gz = fs::File::open(&archive_path).expect("无法打开TAR.GZ文件");
//         let tar = flate2::read::GzDecoder::new(tar_gz);
//         let mut archive = tar::Archive::new(tar);
//         archive.unpack(&extract_dir).expect("无法解压TAR.GZ文件");
//     }

//     // 寻找lib目录
//     let lib_dir = if extract_dir.join(&lib_name).exists() {
//         extract_dir.join(&lib_name)
//     } else {
//         extract_dir.clone()
//     };

//     // 配置链接参数
//     info!("配置链接参数，库路径: {}", lib_dir.display());
//     println!("cargo:rustc-link-search=native={}", lib_dir.display());

//     if is_static {
//         println!("cargo:rustc-link-lib=static=vosk");
//         // iOS额外配置
//         if target_os == "ios" {
//             println!("cargo:rustc-link-lib=framework=Accelerate");
//             println!("cargo:rustc-link-lib=c++");
//         }
//     } else {
//         println!("cargo:rustc-link-lib=dylib=vosk");
//     }

//     // 复制库文件到运行时目录
//     // 获取输出目录（即可执行文件所在的目录）
//     let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("无法获取CARGO_MANIFEST_DIR");
//     let bin_dir = Path::new(&manifest_dir).parent().unwrap().join("target");

//     // 创建release和debug目录
//     let release_dir = bin_dir.join("release");
//     let debug_dir = bin_dir.join("debug");

//     fs::create_dir_all(&release_dir).unwrap_or_else(|_| {});
//     fs::create_dir_all(&debug_dir).unwrap_or_else(|_| {});

//     // 复制库文件
//     if !is_static {
//         let lib_files = match target_os.as_str() {
//             "windows" => vec![
//                 "libvosk.dll",
//                 "libgcc_s_seh-1.dll",
//                 "libstdc++-6.dll",
//                 "libwinpthread-1.dll",
//             ],
//             "linux" => vec!["libvosk.so"],
//             "macos" => vec!["libvosk.dylib"],
//             _ => vec![],
//         };

//         for lib_file in lib_files {
//             // 查找库文件
//             let src_path = find_file(&lib_dir, lib_file);
//             if src_path.exists() {
//                 // 复制到debug和release目录
//                 let debug_path = debug_dir.join(lib_file);
//                 let release_path = release_dir.join(lib_file);

//                 info!(
//                     "复制库文件: {} -> {}",
//                     src_path.display(),
//                     debug_path.display()
//                 );
//                 let _ = fs::copy(&src_path, &debug_path);

//                 info!(
//                     "复制库文件: {} -> {}",
//                     src_path.display(),
//                     release_path.display()
//                 );
//                 let _ = fs::copy(&src_path, &release_path);
//             } else {
//                 info!("未找到库文件: {}", lib_file);
//             }
//         }
//     }

//     // 设置重新运行条件
//     println!("cargo:rerun-if-changed=build.rs");
// }

// // 递归查找文件
// fn find_file(dir: &Path, filename: &str) -> PathBuf {
//     let result = PathBuf::new();

//     if let Ok(entries) = fs::read_dir(dir) {
//         for entry in entries.flatten() {
//             let path = entry.path();
//             if path.is_file() && path.file_name().map_or(false, |name| name == filename) {
//                 return path;
//             } else if path.is_dir() {
//                 let found = find_file(&path, filename);
//                 if found.exists() {
//                     return found;
//                 }
//             }
//         }
//     }
//     result
// }
