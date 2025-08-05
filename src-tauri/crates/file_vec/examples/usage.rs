use cb_config::QdrantConfig;
use file_vec::vec_db::{
    QdrantVectorDb, SearchQuery, create_filtered_search_query, create_search_query,
    create_vector_point,
};
use std::collections::HashMap;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Create Qdrant configuration
    let config = QdrantConfig {
        enabled: true,
        server_url: "http://localhost".to_string(),
        server_port: 6334,
        collection_name: "chat_vectors".to_string(),
        vector_size: 4, // Using 4 dimensions for this example
        distance_metric: "Cosine".to_string(),
        timeout_seconds: 30,
        use_grpc: true,
    };

    println!("🚀 Starting Qdrant Vector Database Example");

    // Create vector database instance
    let vector_db = QdrantVectorDb::new(config).await?;
    println!("✅ Connected to Qdrant");

    // Test connection
    match vector_db.test_connection().await {
        Ok(_) => println!("✅ Connection test successful"),
        Err(e) => {
            println!("❌ Connection test failed: {}", e);
            return Err(e.into());
        }
    }

    // Ensure collection exists
    vector_db.ensure_collection().await?;
    println!("✅ Collection ensured");

    // Example 1: Insert single vector
    println!("\n📝 Example 1: Inserting single vector");
    let mut payload1 = HashMap::new();
    payload1.insert(
        "text".to_string(),
        serde_json::Value::String("This is a document about Berlin".to_string()),
    );
    payload1.insert(
        "category".to_string(),
        serde_json::Value::String("travel".to_string()),
    );
    payload1.insert(
        "language".to_string(),
        serde_json::Value::String("en".to_string()),
    );

    let point1 = create_vector_point(vec![0.05, 0.61, 0.76, 0.74], payload1);
    vector_db.upsert_point(point1).await?;
    println!("✅ Single vector inserted");

    // Example 2: Insert multiple vectors
    println!("\n📝 Example 2: Inserting multiple vectors");
    let vectors = vec![
        create_vector_point(
            vec![0.19, 0.81, 0.75, 0.11],
            HashMap::from([
                (
                    "text".to_string(),
                    serde_json::Value::String("London is a great city".to_string()),
                ),
                (
                    "category".to_string(),
                    serde_json::Value::String("travel".to_string()),
                ),
                (
                    "language".to_string(),
                    serde_json::Value::String("en".to_string()),
                ),
            ]),
        ),
        create_vector_point(
            vec![0.36, 0.55, 0.47, 0.94],
            HashMap::from([
                (
                    "text".to_string(),
                    serde_json::Value::String("Moscow weather is cold".to_string()),
                ),
                (
                    "category".to_string(),
                    serde_json::Value::String("weather".to_string()),
                ),
                (
                    "language".to_string(),
                    serde_json::Value::String("en".to_string()),
                ),
            ]),
        ),
        create_vector_point(
            vec![0.18, 0.01, 0.85, 0.80],
            HashMap::from([
                (
                    "text".to_string(),
                    serde_json::Value::String("New York never sleeps".to_string()),
                ),
                (
                    "category".to_string(),
                    serde_json::Value::String("travel".to_string()),
                ),
                (
                    "language".to_string(),
                    serde_json::Value::String("en".to_string()),
                ),
            ]),
        ),
        create_vector_point(
            vec![0.24, 0.18, 0.22, 0.44],
            HashMap::from([
                (
                    "text".to_string(),
                    serde_json::Value::String("Beijing has amazing food".to_string()),
                ),
                (
                    "category".to_string(),
                    serde_json::Value::String("food".to_string()),
                ),
                (
                    "language".to_string(),
                    serde_json::Value::String("en".to_string()),
                ),
            ]),
        ),
        create_vector_point(
            vec![0.35, 0.08, 0.11, 0.44],
            HashMap::from([
                (
                    "text".to_string(),
                    serde_json::Value::String("Mumbai is very crowded".to_string()),
                ),
                (
                    "category".to_string(),
                    serde_json::Value::String("travel".to_string()),
                ),
                (
                    "language".to_string(),
                    serde_json::Value::String("en".to_string()),
                ),
            ]),
        ),
    ];

    vector_db.upsert_points(vectors).await?;
    println!("✅ Multiple vectors inserted");

    // Check collection info
    let count = vector_db.count_points().await?;
    println!("📊 Total points in collection: {}", count);

    // Example 3: Basic search
    println!("\n🔍 Example 3: Basic search");
    let search_vector = vec![0.2, 0.1, 0.9, 0.7];
    let query = create_search_query(search_vector, 3);

    let results = vector_db.search(query).await?;
    println!("Found {} results:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!(
            "  {}. Score: {:.3}, Text: {}",
            i + 1,
            result.score,
            result
                .payload
                .get("text")
                .unwrap_or(&serde_json::Value::Null)
        );
    }

    // Example 4: Filtered search
    println!("\n🔍 Example 4: Filtered search (travel category only)");
    let filter = HashMap::from([(
        "category".to_string(),
        serde_json::Value::String("travel".to_string()),
    )]);
    let filtered_query = create_filtered_search_query(vec![0.2, 0.1, 0.9, 0.7], filter, 5);

    let filtered_results = vector_db.search(filtered_query).await?;
    println!("Found {} travel-related results:", filtered_results.len());
    for (i, result) in filtered_results.iter().enumerate() {
        println!(
            "  {}. Score: {:.3}, Text: {}, Category: {}",
            i + 1,
            result.score,
            result
                .payload
                .get("text")
                .unwrap_or(&serde_json::Value::Null),
            result
                .payload
                .get("category")
                .unwrap_or(&serde_json::Value::Null)
        );
    }

    // Example 5: Query with score threshold
    println!("\n🔍 Example 5: Query with score threshold");
    let threshold_query = SearchQuery {
        vector: vec![0.2, 0.1, 0.9, 0.7],
        filter: None,
        limit: 10,
        with_payload: true,
        with_vector: false,
        score_threshold: Some(0.5), // Only return results with score >= 0.5
    };

    let threshold_results = vector_db.query(threshold_query).await?;
    println!(
        "Found {} results with score >= 0.5:",
        threshold_results.len()
    );
    for (i, result) in threshold_results.iter().enumerate() {
        println!(
            "  {}. Score: {:.3}, Text: {}",
            i + 1,
            result.score,
            result
                .payload
                .get("text")
                .unwrap_or(&serde_json::Value::Null)
        );
    }

    // Example 6: Complex filter (multiple conditions)
    println!("\n🔍 Example 6: Complex filter (travel + English)");
    let complex_filter = HashMap::from([
        (
            "category".to_string(),
            serde_json::Value::String("travel".to_string()),
        ),
        (
            "language".to_string(),
            serde_json::Value::String("en".to_string()),
        ),
    ]);
    let complex_query = create_filtered_search_query(vec![0.2, 0.1, 0.9, 0.7], complex_filter, 5);

    let complex_results = vector_db.search(complex_query).await?;
    println!("Found {} travel + English results:", complex_results.len());
    for (i, result) in complex_results.iter().enumerate() {
        println!(
            "  {}. Score: {:.3}, Text: {}",
            i + 1,
            result.score,
            result
                .payload
                .get("text")
                .unwrap_or(&serde_json::Value::Null)
        );
    }

    // Example 7: Get collection information
    println!("\n📊 Example 7: Collection information");
    match vector_db.get_collection_info().await {
        Ok(info) => {
            println!("Collection info:");
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        Err(e) => println!("Failed to get collection info: {}", e),
    }

    // Example 8: Delete points by filter
    println!("\n🗑️  Example 8: Delete points by filter (food category)");
    let delete_filter = HashMap::from([(
        "category".to_string(),
        serde_json::Value::String("food".to_string()),
    )]);

    vector_db.delete_points_by_filter(delete_filter).await?;
    println!("✅ Deleted food category points");

    let final_count = vector_db.count_points().await?;
    println!("📊 Final count after deletion: {}", final_count);

    println!("\n🎉 Example completed successfully!");
    Ok(())
}
