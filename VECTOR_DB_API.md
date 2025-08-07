# Vector Database API Documentation

This document provides comprehensive documentation for the Vector Database functionality in the chat_box_rust application.

## Overview

The Vector Database system allows you to:
- Store and search through file embeddings using semantic similarity
- Add individual files or sync entire directories
- Perform filtered searches with metadata
- Manage indexed files and database statistics

## Architecture

### Backend (Rust)
- **VectorDb**: Core vector database implementation in `file_vec` crate
- **Commands**: Tauri commands in `src/commands/vector_db.rs`
- **State**: Vector database state managed in `AppState`

### Frontend (TypeScript/React)
- **Types**: TypeScript interfaces in `src/types/vectordb.ts`
- **Hook**: React hook in `src/hooks/useVectorDb.ts`
- **Component**: UI component in `src/components/VectorDbManager.tsx`

## Tauri Commands Reference

### Database Management

#### `create_vector_db(name: string) -> Result<(), String>`
Creates a new vector database with the specified collection name.

**Parameters:**
- `name`: Collection name for the vector database

**Example:**
```typescript
await invoke('create_vector_db', { name: 'my_documents' });
```

#### `delete_vector_db() -> Result<(), String>`
Deletes the vector database and all its data.

**Example:**
```typescript
await invoke('delete_vector_db');
```

#### `test_vector_db_connection() -> Result<(), String>`
Tests the connection to the vector database.

**Example:**
```typescript
try {
  await invoke('test_vector_db_connection');
  console.log('Connection successful');
} catch (error) {
  console.log('Connection failed:', error);
}
```

### File Operations

#### `add_file_to_vector_db(file_path: string) -> Result<AddFileResponse, String>`
Adds a single file to the vector database.

**Parameters:**
- `file_path`: Path to the file to add

**Returns:**
```typescript
interface AddFileResponse {
  point_id: string;
  file_path: string;
}
```

**Example:**
```typescript
const response = await invoke<AddFileResponse>('add_file_to_vector_db', {
  filePath: '/path/to/document.txt'
});
console.log('File added with ID:', response.point_id);
```

#### `sync_files_to_vector_db(directory_path: string) -> Result<SyncFilesResponse, String>`
Syncs all supported files from a directory to the vector database.

**Parameters:**
- `directory_path`: Path to the directory to sync

**Returns:**
```typescript
interface SyncFilesResponse {
  added_files: string[];
  count: number;
}
```

**Example:**
```typescript
const response = await invoke<SyncFilesResponse>('sync_files_to_vector_db', {
  directoryPath: '/path/to/documents'
});
console.log(`Added ${response.count} files`);
```

#### `update_file_in_vector_db(file_path: string) -> Result<(), String>`
Updates an existing file in the vector database.

**Parameters:**
- `file_path`: Path to the file to update

**Example:**
```typescript
await invoke('update_file_in_vector_db', { filePath: '/path/to/updated_file.txt' });
```

#### `delete_file_from_vector_db(file_path: string) -> Result<(), String>`
Removes a file from the vector database by its path.

**Parameters:**
- `file_path`: Path to the file to delete

**Example:**
```typescript
await invoke('delete_file_from_vector_db', { filePath: '/path/to/file.txt' });
```

#### `delete_point_from_vector_db(point_id: string) -> Result<(), String>`
Removes a specific point from the vector database by its ID.

**Parameters:**
- `point_id`: UUID of the point to delete

**Example:**
```typescript
await invoke('delete_point_from_vector_db', { pointId: 'uuid-string' });
```

#### `get_indexed_files() -> Result<Vec<String>, String>`
Returns a list of all file paths currently indexed in the database.

**Returns:**
Array of file paths

**Example:**
```typescript
const files = await invoke<string[]>('get_indexed_files');
console.log('Indexed files:', files);
```

### Search Operations

#### `search_vector_db(query: string, top_k?: number) -> Result<Vec<SearchResultResponse>, String>`
Performs a semantic search on the vector database.

**Parameters:**
- `query`: Search query string
- `top_k`: Maximum number of results (default: 10)

**Returns:**
```typescript
interface SearchResultResponse {
  id: string;
  score: number;
  payload: Record<string, any>;
}
```

**Example:**
```typescript
const results = await invoke<SearchResultResponse[]>('search_vector_db', {
  query: 'machine learning algorithms',
  topK: 5
});

results.forEach(result => {
  console.log(`Score: ${result.score}, File: ${result.payload.file_path}`);
});
```

#### `search_vector_db_with_filter(query: string, top_k?: number, filter: Record<string, any>) -> Result<Vec<SearchResultResponse>, String>`
Performs a filtered semantic search on the vector database.

**Parameters:**
- `query`: Search query string
- `top_k`: Maximum number of results
- `filter`: Filter criteria object

**Filter Options:**
```typescript
{
  extension?: string;           // File extension filter
  file_size_min?: number;      // Minimum file size
  file_size_max?: number;      // Maximum file size  
  modified_after?: number;     // Modified after timestamp
  modified_before?: number;    // Modified before timestamp
  file_path?: string;          // Partial file path match
}
```

**Example:**
```typescript
const results = await invoke<SearchResultResponse[]>('search_vector_db_with_filter', {
  query: 'python tutorial',
  topK: 10,
  filter: {
    extension: 'py',
    file_size_min: 1024
  }
});
```

### Statistics and Maintenance

#### `get_vector_db_stats() -> Result<VectorDbStats, String>`
Returns comprehensive statistics about the vector database.

**Returns:**
```typescript
interface VectorDbStats {
  collection_info: any;
  point_count: number;
  mapped_files: number;
  embedding_model: string;
}
```

**Example:**
```typescript
const stats = await invoke<VectorDbStats>('get_vector_db_stats');
console.log(`Database contains ${stats.point_count} points from ${stats.mapped_files} files`);
```

#### `delete_all_embeddings() -> Result<(), String>`
Clears all embeddings from the vector database.

**Example:**
```typescript
await invoke('delete_all_embeddings');
```

#### `get_embedding_dimension() -> Result<number, String>`
Returns the dimension of the embedding vectors.

**Example:**
```typescript
const dimension = await invoke<number>('get_embedding_dimension');
console.log('Embedding dimension:', dimension);
```

## React Hook Usage

The `useVectorDb` hook provides a convenient React interface:

```typescript
import { useVectorDb } from '../hooks/useVectorDb';

function MyComponent() {
  const {
    isInitialized,
    isLoading,
    error,
    stats,
    search,
    addFile,
    deleteFile,
    syncDirectory,
    refreshStats,
    testConnection,
    initializeDb,
    clearDatabase
  } = useVectorDb();

  // Initialize database
  const handleInit = async () => {
    await initializeDb('my_collection');
  };

  // Perform search
  const handleSearch = async (query: string) => {
    try {
      const results = await search(query, {
        topK: 10,
        scoreThreshold: 0.5,
        filter: {
          extension: 'txt'
        }
      });
      console.log('Search results:', results);
    } catch (error) {
      console.error('Search failed:', error);
    }
  };

  return (
    <div>
      {!isInitialized && (
        <button onClick={handleInit}>Initialize Database</button>
      )}
      
      {isInitialized && (
        <div>
          <button onClick={() => handleSearch('test query')}>
            Search
          </button>
          
          {stats && (
            <p>Files indexed: {stats.mapped_files}</p>
          )}
        </div>
      )}
      
      {error && <div>Error: {error}</div>}
    </div>
  );
}
```

## Component Usage

The `VectorDbManager` component provides a complete UI for vector database operations:

```typescript
import VectorDbManager from '../components/VectorDbManager';

function App() {
  const handleFileSelect = (filePath: string) => {
    console.log('Selected file:', filePath);
    // Handle file selection
  };

  return (
    <VectorDbManager onFileSelect={handleFileSelect} />
  );
}
```

## Configuration

### Default Configuration

The system uses these default configurations:

**Embedding Config:**
```rust
EmbedConfig {
    model_name: "BAAI/bge-small-en-v1.5".to_string(),
    max_length: 512,
    batch_size: 32,
    show_download_progress: true,
    cache_dir: None,
}
```

**Qdrant Config:**
```rust
QdrantConfig {
    enabled: true,
    server_url: "http://localhost".to_string(),
    server_port: 6333,
    collection_name: "file_vectors".to_string(),
    vector_size: 384, // BGE-small dimension
    distance_metric: "cosine".to_string(),
    timeout_seconds: 30,
    use_grpc: false,
}
```

## Supported File Types

The system currently supports these file types by default:
- `.txt` - Text files
- `.md` - Markdown files
- `.rs` - Rust source files
- `.py` - Python source files

## Error Handling

All commands return `Result<T, String>` types. Common errors include:

- **"Vector database not initialized"**: Database hasn't been created yet
- **"Vector database feature not enabled"**: Feature flag not set during compilation
- **"Failed to connect to Qdrant server"**: Qdrant server is not running
- **"File not found"**: Specified file path doesn't exist
- **"Invalid UUID format"**: Point ID is not a valid UUID

## Best Practices

### Performance
- Use batch operations (sync directory) instead of adding files individually
- Set appropriate `topK` limits for searches to avoid performance issues
- Use filters to narrow down search space when possible

### Memory Management
- Regularly check database statistics to monitor growth
- Consider clearing old embeddings periodically
- Monitor file system space for embedding storage

### Search Quality
- Use descriptive queries that match the content style
- Experiment with different score thresholds
- Consider file types when searching (use filters)

### Error Handling
```typescript
try {
  const results = await search(query);
  // Handle results
} catch (error) {
  if (error.includes('not initialized')) {
    // Handle initialization error
    await initializeDb();
  } else {
    // Handle other errors
    console.error('Search failed:', error);
  }
}
```

## Prerequisites

### Backend Requirements
- Qdrant server running on localhost:6333 (or configured port)
- `vector_db` feature enabled in Cargo.toml
- Sufficient disk space for embeddings storage

### Model Downloads
The first run will download the embedding model (~90MB for BGE-small).
Ensure internet connection is available for initial setup.

## Troubleshooting

### Common Issues

**Database Not Initialized**
```typescript
// Check if initialized and initialize if needed
if (!isInitialized) {
  await initializeDb('my_collection');
}
```

**Connection Failed**
- Verify Qdrant server is running
- Check server URL and port configuration
- Ensure firewall allows connections

**Search Returns No Results**
- Check if files are actually indexed
- Verify search query matches content style
- Try lowering score threshold
- Check if filters are too restrictive

**Out of Memory**
- Reduce batch size in configuration
- Process files in smaller chunks
- Monitor system memory usage

## API Evolution

This API is designed to be extensible. Future versions may include:

- Custom embedding models
- Advanced filtering options  
- Batch operations for better performance
- Real-time file watching and updates
- Export/import functionality for embeddings

## Examples

### Complete Workflow Example

```typescript
import { useVectorDb } from '../hooks/useVectorDb';

async function completeWorkflow() {
  const vectorDb = useVectorDb();
  
  // 1. Initialize database
  await vectorDb.initializeDb('documents');
  
  // 2. Add some files
  await vectorDb.syncDirectory('/home/user/documents');
  
  // 3. Search for content
  const results = await vectorDb.search('machine learning', {
    topK: 5,
    scoreThreshold: 0.6
  });
  
  // 4. Process results
  results.forEach(result => {
    console.log(`Found: ${result.payload.file_name} (${result.score.toFixed(3)})`);
  });
  
  // 5. Get statistics
  const stats = await vectorDb.refreshStats();
  console.log(`Indexed ${stats.mapped_files} files`);
}
```

This documentation provides a complete reference for integrating and using the Vector Database functionality in your application.