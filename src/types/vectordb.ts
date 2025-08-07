// TypeScript interfaces for Vector Database Tauri commands

export interface SearchResultResponse {
  id: string;
  score: number;
  payload: Record<string, any>;
}

export interface VectorDbStats {
  collection_info: any;
  point_count: number;
  mapped_files: number;
  embedding_model: string;
}

export interface AddFileResponse {
  point_id: string;
  file_path: string;
}

export interface SyncFilesResponse {
  added_files: string[];
  count: number;
}

export interface VectorDbCommands {
  /**
   * Create a new vector database with the given collection name
   */
  create_vector_db: (name: string) => Promise<void>;

  /**
   * Delete the vector database and all its data
   */
  delete_vector_db: () => Promise<void>;

  /**
   * Add a file to the vector database
   */
  add_file_to_vector_db: (file_path: string) => Promise<AddFileResponse>;

  /**
   * Search the vector database for similar content
   */
  search_vector_db: (
    query: string,
    top_k?: number,
  ) => Promise<SearchResultResponse[]>;

  /**
   * Search the vector database with additional filters
   */
  search_vector_db_with_filter: (
    query: string,
    top_k: number | undefined,
    filter: Record<string, any>,
  ) => Promise<SearchResultResponse[]>;

  /**
   * Delete a file from the vector database by its path
   */
  delete_file_from_vector_db: (file_path: string) => Promise<void>;

  /**
   * Delete a specific point from the vector database by its ID
   */
  delete_point_from_vector_db: (point_id: string) => Promise<void>;

  /**
   * Delete all embeddings from the vector database
   */
  delete_all_embeddings: () => Promise<void>;

  /**
   * Get statistics about the vector database
   */
  get_vector_db_stats: () => Promise<VectorDbStats>;

  /**
   * Sync files from a directory to the vector database
   */
  sync_files_to_vector_db: (
    directory_path: string,
  ) => Promise<SyncFilesResponse>;

  /**
   * Update an existing file in the vector database
   */
  update_file_in_vector_db: (file_path: string) => Promise<void>;

  /**
   * Get a list of all indexed files in the vector database
   */
  get_indexed_files: () => Promise<string[]>;

  /**
   * Test the vector database connection
   */
  test_vector_db_connection: () => Promise<void>;

  /**
   * Get the embedding dimension used by the model
   */
  get_embedding_dimension: () => Promise<number>;
}

// Helper type for invoke calls
export type VectorDbCommandName = keyof VectorDbCommands;

// Search options interface for more complex queries
export interface VectorSearchOptions {
  query: string;
  topK?: number;
  scoreThreshold?: number;
  filter?: {
    extension?: string;
    fileSize?: { min?: number; max?: number };
    modified?: { after?: number; before?: number };
    filePath?: string;
  };
}

// File metadata interface
export interface FileMetadata {
  file_path: string;
  file_name: string;
  file_size?: number;
  modified?: number;
  extension?: string;
  content_preview?: string;
}

// Enhanced search result with typed payload
export interface TypedSearchResult {
  id: string;
  score: number;
  metadata: FileMetadata;
}

// Vector database status
export enum VectorDbStatus {
  NotInitialized = "not_initialized",
  Initializing = "initializing",
  Ready = "ready",
  Error = "error",
}

// Configuration interfaces
export interface EmbedConfig {
  model_name: string;
  max_length: number;
  batch_size: number;
  show_download_progress: boolean;
  cache_dir?: string;
}

export interface QdrantConfig {
  enabled: boolean;
  server_url: string;
  server_port: number;
  collection_name: string;
  vector_size: number;
  distance_metric: string;
  timeout_seconds: number;
  use_grpc: boolean;
}

export interface VectorDbConfig {
  qdrant_config: QdrantConfig;
  embed_config: EmbedConfig;
}

// Error types
export class VectorDbError extends Error {
  constructor(
    message: string,
    public readonly code?: string,
    public readonly details?: any,
  ) {
    super(message);
    this.name = "VectorDbError";
  }
}

// Utility functions for working with search results
export class VectorDbUtils {
  /**
   * Convert search result to typed format
   */
  static toTypedResult(result: SearchResultResponse): TypedSearchResult {
    return {
      id: result.id,
      score: result.score,
      metadata: {
        file_path: result.payload.file_path as string,
        file_name: result.payload.file_name as string,
        file_size: result.payload.file_size as number,
        modified: result.payload.modified as number,
        extension: result.payload.extension as string,
        content_preview: result.payload.content_preview as string,
      },
    };
  }

  /**
   * Filter results by minimum score
   */
  static filterByScore(
    results: SearchResultResponse[],
    minScore: number,
  ): SearchResultResponse[] {
    return results.filter((result) => result.score >= minScore);
  }

  /**
   * Group results by file extension
   */
  static groupByExtension(
    results: TypedSearchResult[],
  ): Record<string, TypedSearchResult[]> {
    return results.reduce(
      (groups, result) => {
        const ext = result.metadata.extension || "unknown";
        if (!groups[ext]) {
          groups[ext] = [];
        }
        groups[ext].push(result);
        return groups;
      },
      {} as Record<string, TypedSearchResult[]>,
    );
  }

  /**
   * Sort results by relevance score (descending)
   */
  static sortByRelevance(
    results: SearchResultResponse[],
  ): SearchResultResponse[] {
    return [...results].sort((a, b) => b.score - a.score);
  }

  /**
   * Extract unique file paths from results
   */
  static extractFilePaths(results: SearchResultResponse[]): string[] {
    return [...new Set(results.map((r) => r.payload.file_path as string))];
  }
}

// React hook type for vector database operations
export interface UseVectorDbReturn {
  // State
  isInitialized: boolean;
  isLoading: boolean;
  error: string | null;
  stats: VectorDbStats | null;

  // Core operations
  search: (
    query: string,
    options?: Partial<VectorSearchOptions>,
  ) => Promise<SearchResultResponse[]>;
  addFile: (filePath: string) => Promise<AddFileResponse>;
  deleteFile: (filePath: string) => Promise<void>;
  syncDirectory: (directoryPath: string) => Promise<SyncFilesResponse>;
  refreshStats: () => Promise<void>;
  testConnection: () => Promise<boolean>;

  // Extended operations
  initializeDb: (collectionName?: string) => Promise<boolean>;
  updateFile: (filePath: string) => Promise<void>;
  getIndexedFiles: () => Promise<string[]>;
  clearDatabase: () => Promise<void>;
  getEmbeddingDimension: () => Promise<number>;
  clearError: () => void;
  checkInitialization: () => Promise<boolean>;
}

export default VectorDbCommands;
