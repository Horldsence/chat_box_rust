use file_finder::{FileFinder, SearchOptions};
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("文件查找器示例");
    println!("================");

    // 创建文件查找器实例
    let finder = FileFinder::new()?;

    println!("正在初始化并建立文件索引...");
    finder.initialize().await?;

    // 获取索引统计信息
    let file_count = finder.index.file_count();
    println!("索引建立完成，共索引了 {} 个文件", file_count);

    // 示例1: 简单的文件名搜索
    println!("\n示例1: 搜索包含 'config' 的文件");
    let options = SearchOptions {
        pattern: "config".to_string(),
        use_regex: false,
        include_hidden: false,
        max_results: Some(10),
        ..Default::default()
    };

    let results = finder.search_files(options)?;
    println!("找到 {} 个匹配的文件:", results.len());
    for file in &results {
        println!("  📄 {} ({}字节)", file.name, file.size);
    }

    // 示例2: 使用正则表达式搜索
    println!("\n示例2: 使用正则表达式搜索 .txt 文件");
    let regex_options = SearchOptions {
        pattern: r"\.txt$".to_string(),
        use_regex: true,
        include_hidden: false,
        max_results: Some(5),
        ..Default::default()
    };

    let regex_results = finder.search_files(regex_options)?;
    println!("找到 {} 个 .txt 文件:", regex_results.len());
    for file in &regex_results {
        println!("  📝 {}", file.path.display());
    }

    // 示例3: 按文件类型搜索
    println!("\n示例3: 搜索特定文件类型");
    let type_options = SearchOptions {
        pattern: "".to_string(),
        use_regex: false,
        include_hidden: false,
        file_types: Some(vec!["rs".to_string(), "toml".to_string()]),
        max_results: Some(10),
        ..Default::default()
    };

    let type_results = finder.search_files(type_options)?;
    println!("找到 {} 个 Rust 相关文件:", type_results.len());
    for file in &type_results {
        println!(
            "  🦀 {} ({})",
            file.name,
            file.extension.as_deref().unwrap_or("无扩展名")
        );
    }

    // 示例4: 内容搜索（较慢但更全面）
    println!("\n示例4: 在文件内容中搜索 'fn main'");
    let content_options = SearchOptions {
        pattern: "fn main".to_string(),
        use_regex: false,
        include_hidden: false,
        max_results: Some(5),
        search_content: true,
        ..Default::default()
    };

    let content_results = finder.search_with_content(content_options).await?;
    println!("在内容中找到 {} 个匹配的文件:", content_results.len());
    for file in &content_results {
        println!("  🔍 {}", file.path.display());
    }

    // 示例5: 复杂的正则表达式搜索
    println!("\n示例5: 使用正则表达式搜索图片文件");
    let image_options = SearchOptions {
        pattern: r"\.(jpg|jpeg|png|gif|bmp|svg)$".to_string(),
        use_regex: true,
        include_hidden: false,
        max_results: Some(8),
        ..Default::default()
    };

    let image_results = finder.search_files(image_options)?;
    println!("找到 {} 个图片文件:", image_results.len());
    for file in &image_results {
        println!("  🖼️  {} ({}字节)", file.name, file.size);
    }

    // 示例6: 获取特定文件信息
    println!("\n示例6: 获取特定文件的详细信息");
    if let Some(first_result) = results.first() {
        if let Some(file_info) = finder.get_file_info(&first_result.path).await? {
            println!("文件详细信息:");
            println!("  名称: {}", file_info.name);
            println!("  路径: {}", file_info.path.display());
            println!("  大小: {} 字节", file_info.size);
            println!("  修改时间: {}", file_info.modified);
            println!("  是否为目录: {}", file_info.is_dir);
            if let Some(ext) = &file_info.extension {
                println!("  扩展名: {}", ext);
            }
        }
    }

    // 示例7: 检查索引状态
    println!("\n示例7: 检查索引状态");
    let needs_update = finder.index.needs_update(Duration::from_secs(3600)); // 1小时
    println!(
        "索引是否需要更新: {}",
        if needs_update { "是" } else { "否" }
    );

    // 示例8: 刷新索引
    if needs_update {
        println!("正在刷新索引...");
        finder.refresh_index().await?;
        println!("索引刷新完成");
    }

    println!("\n所有示例执行完成！");

    Ok(())
}
