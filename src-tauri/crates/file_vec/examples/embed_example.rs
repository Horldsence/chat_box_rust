use cb_config::{EmbedConfig, QdrantConfig};
use file_vec::embed::FastEmbedWrapper;
use file_vec::vec_db::{QdrantVectorDb, SearchQuery, VectorPoint};
use std::collections::HashMap;
use tokio;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🚀 FastEmbed + Qdrant 集成示例");

    // 步骤 1: 创建嵌入器
    println!("\n📝 步骤 1: 初始化 FastEmbed 嵌入器");
    let config = EmbedConfig {
        model_name: "BAAI/bge-small-en-v1.5".to_string(),
        max_length: 512,
        batch_size: 32,
        show_download_progress: true,
        cache_dir: None,
    };
    let embedder = FastEmbedWrapper::new(config);
    println!("✅ FastEmbed 嵌入器初始化成功");

    // 获取向量维度
    let vector_dim = embedder.get_vector_dimension().await?;
    println!("📏 向量维度: {}", vector_dim);

    // 步骤 2: 配置 Qdrant
    println!("\n📝 步骤 2: 配置 Qdrant 向量数据库");
    let config = QdrantConfig {
        enabled: true,
        server_url: "http://localhost".to_string(),
        server_port: 6334,
        collection_name: "chat_embeddings".to_string(),
        vector_size: vector_dim as u64,
        distance_metric: "Cosine".to_string(),
        timeout_seconds: 30,
        use_grpc: true,
    };

    let vector_db = QdrantVectorDb::new(config).await?;
    println!("✅ Qdrant 连接成功");

    // 测试连接
    vector_db.test_connection().await?;
    vector_db.ensure_collection().await?;
    println!("✅ 集合创建/验证成功");

    // 步骤 3: 准备文档数据
    println!("\n📝 步骤 3: 准备文档数据");
    let documents = vec![
        "北京是中国的首都，拥有悠久的历史和丰富的文化遗产。".to_string(),
        "上海是中国最大的城市，也是重要的经济中心。".to_string(),
        "深圳是中国改革开放的前沿城市，科技产业发达。".to_string(),
        "杭州以西湖美景闻名，也是电商巨头阿里巴巴的总部所在地。".to_string(),
        "成都是四川省的省会，以美食和大熊猫而著称。".to_string(),
        "广州是广东省的省会，是中国南方的重要门户城市。".to_string(),
        "西安是古代丝绸之路的起点，拥有兵马俑等世界文化遗产。".to_string(),
        "苏州以其精美的园林和丝绸文化而闻名于世。".to_string(),
    ];

    // 步骤 4: 生成嵌入向量
    println!("\n📝 步骤 4: 为文档生成嵌入向量");
    let embed_results = embedder.embed_passages(&documents).await?;
    println!("✅ 生成了 {} 个嵌入向量", embed_results.len());

    // 步骤 5: 创建向量点并插入数据库
    println!("\n📝 步骤 5: 插入向量到数据库");
    let mut vector_points = Vec::new();

    for (i, result) in embed_results.iter().enumerate() {
        let mut payload = HashMap::new();
        payload.insert(
            "text".to_string(),
            serde_json::Value::String(documents[i].clone()),
        );
        payload.insert(
            "city_type".to_string(),
            serde_json::Value::String("chinese_city".to_string()),
        );
        payload.insert(
            "index".to_string(),
            serde_json::Value::Number(serde_json::Number::from(i)),
        );

        let point = VectorPoint {
            id: Uuid::new_v4(),
            vector: result.vector.clone(),
            payload,
            timestamp: chrono::Utc::now().timestamp(),
        };

        vector_points.push(point);
    }

    vector_db.upsert_points(vector_points).await?;
    println!("✅ 成功插入 {} 个向量点", embed_results.len());

    // 验证数据插入
    let count = vector_db.count_points().await?;
    println!("📊 数据库中总共有 {} 个向量点", count);

    // 步骤 6: 执行语义搜索
    println!("\n📝 步骤 6: 执行语义搜索");

    // 查询 1: 关于历史文化的城市
    let query1 = "哪个城市有丰富的历史文化？";
    println!("🔍 查询 1: {}", query1);

    let query_result = embedder.embed_query(query1).await?;
    let search_query = SearchQuery {
        vector: query_result.vector,
        filter: None,
        limit: 3,
        with_payload: true,
        with_vector: false,
        score_threshold: Some(0.0),
    };

    let results = vector_db.search(search_query).await?;
    println!("找到 {} 个相关结果：", results.len());
    for (i, result) in results.iter().enumerate() {
        println!(
            "  {}. 相似度: {:.3}, 文本: {}",
            i + 1,
            result.score,
            result
                .payload
                .get("text")
                .unwrap_or(&serde_json::Value::Null)
        );
    }

    // 查询 2: 关于经济发展的城市
    let query2 = "哪个城市经济发达，科技产业比较强？";
    println!("\n🔍 查询 2: {}", query2);

    let query_result2 = embedder.embed_query(query2).await?;
    let search_query2 = SearchQuery {
        vector: query_result2.vector,
        filter: None,
        limit: 3,
        with_payload: true,
        with_vector: false,
        score_threshold: Some(0.0),
    };

    let results2 = vector_db.search(search_query2).await?;
    println!("找到 {} 个相关结果：", results2.len());
    for (i, result) in results2.iter().enumerate() {
        println!(
            "  {}. 相似度: {:.3}, 文本: {}",
            i + 1,
            result.score,
            result
                .payload
                .get("text")
                .unwrap_or(&serde_json::Value::Null)
        );
    }

    // 查询 3: 关于美食的城市
    let query3 = "哪个城市美食比较有名？";
    println!("\n🔍 查询 3: {}", query3);

    let query_result3 = embedder.embed_query(query3).await?;
    let search_query3 = SearchQuery {
        vector: query_result3.vector,
        filter: None,
        limit: 3,
        with_payload: true,
        with_vector: false,
        score_threshold: Some(0.0),
    };

    let results3 = vector_db.search(search_query3).await?;
    println!("找到 {} 个相关结果：", results3.len());
    for (i, result) in results3.iter().enumerate() {
        println!(
            "  {}. 相似度: {:.3}, 文本: {}",
            i + 1,
            result.score,
            result
                .payload
                .get("text")
                .unwrap_or(&serde_json::Value::Null)
        );
    }

    // 步骤 7: 测试相似度计算
    println!("\n📝 步骤 7: 测试文本相似度计算");

    let text1 = "北京是中国的首都";
    let text2 = "上海是中国最大的城市";
    let text3 = "Beijing is the capital of China";

    let result1 = embedder.embed_text(text1).await?;
    let result2 = embedder.embed_text(text2).await?;
    let result3 = embedder.embed_text(text3).await?;

    let similarity_1_2 = FastEmbedWrapper::cosine_similarity(&result1.vector, &result2.vector)?;
    let similarity_1_3 = FastEmbedWrapper::cosine_similarity(&result1.vector, &result3.vector)?;

    println!("文本相似度比较：");
    println!("  '{}' vs '{}': {:.3}", text1, text2, similarity_1_2);
    println!("  '{}' vs '{}': {:.3}", text1, text3, similarity_1_3);

    // 步骤 8: 测试自定义模型配置
    println!("\n📝 步骤 8: 测试自定义模型配置");

    let custom_config = EmbedConfig {
        model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        max_length: 256,
        batch_size: 16,
        show_download_progress: true,
        cache_dir: None,
    };

    // 注意：这里只是展示配置，实际使用时可能需要不同的向量维度
    println!("自定义配置: {:?}", custom_config);

    // 在实际应用中，你可以这样创建自定义嵌入器：
    // let custom_embedder = create_embedder(custom_config).await?;

    // 步骤 9: 批量处理示例
    println!("\n📝 步骤 9: 批量处理示例");

    let batch_texts = vec![
        "机器学习是人工智能的一个分支".to_string(),
        "深度学习使用神经网络进行学习".to_string(),
        "自然语言处理帮助计算机理解人类语言".to_string(),
        "计算机视觉让机器能够看懂图像".to_string(),
    ];

    let batch_results = embedder.embed_batch(&batch_texts).await?;
    println!("✅ 批量处理了 {} 个文本", batch_results.len());

    // 计算批量文本之间的相似度矩阵
    println!("\n相似度矩阵：");
    for i in 0..batch_results.len() {
        for j in 0..batch_results.len() {
            if i <= j {
                let similarity = FastEmbedWrapper::cosine_similarity(
                    &batch_results[i].vector,
                    &batch_results[j].vector,
                )?;
                println!("  文本{} vs 文本{}: {:.3}", i + 1, j + 1, similarity);
            }
        }
    }

    // 步骤 10: 清理和总结
    println!("\n📝 步骤 10: 总结");
    let final_count = vector_db.count_points().await?;
    println!("📊 最终数据库中的向量点数量: {}", final_count);

    println!("\n🎉 FastEmbed + Qdrant 集成示例完成！");
    println!("💡 主要功能展示：");
    println!("   ✅ FastEmbed 模型初始化");
    println!("   ✅ 文本嵌入向量生成");
    println!("   ✅ 批量嵌入处理");
    println!("   ✅ 向量数据库存储");
    println!("   ✅ 语义相似性搜索");
    println!("   ✅ 相似度计算");
    println!("   ✅ 查询和文档前缀处理");

    Ok(())
}
