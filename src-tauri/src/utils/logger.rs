use chrono::Local;
use env_logger::Env;
use log::info;
use log::LevelFilter;
use std::io::Write;

pub fn init_logger() {
    print_magic();
    let env = Env::default().filter_or("MY_LOG_LEVEL", "debug");
    // 设置日志打印格式
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} | {} | {}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or("<unnamed>"),
                record.args()
            )
        })
        .write_style(env_logger::WriteStyle::Always) // Enable built-in coloring
        .filter(None, LevelFilter::Debug)
        .init();
    info!("env_logger initialized.");
}

pub fn print_magic() {
    println!(
        r#"
==============================================================================
  /$$$$$$  /$$   /$$  /$$$$$$  /$$$$$$$$         /$$$$$$$   /$$$$$$  /$$   /$$
 /$$__  $$| $$  | $$ /$$__  $$|__  $$__/        | $$__  $$ /$$__  $$| $$  / $$
| $$  \__/| $$  | $$| $$  \ $$   | $$           | $$  \ $$| $$  \ $$|  $$/ $$/
| $$      | $$$$$$$$| $$$$$$$$   | $$    /$$$$$$| $$$$$$$ | $$  | $$ \  $$$$/ 
| $$      | $$__  $$| $$__  $$   | $$   |______/| $$__  $$| $$  | $$  >$$  $$ 
| $$    $$| $$  | $$| $$  | $$   | $$           | $$  \ $$| $$  | $$ /$$/\  $$
|  $$$$$$/| $$  | $$| $$  | $$   | $$           | $$$$$$$/|  $$$$$$/| $$  \ $$
 \______/ |__/  |__/|__/  |__/   |__/           |_______/  \______/ |__/  |__/
                佛祖保佑                               永无BUG
==============================================================================
                                starting up...
    "#
    );
}
