import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  VectorDbStats,
  SearchResultResponse,
  AddFileResponse,
  SyncFilesResponse,
  VectorSearchOptions,
  VectorDbError,
  UseVectorDbReturn,
} from "../types/vectordb";

export const useVectorDb = (): UseVectorDbReturn => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [stats, setStats] = useState<VectorDbStats | null>(null);

  const clearError = useCallback(() => setError(null), []);

  const handleError = useCallback((err: any) => {
    const errorMessage =
      typeof err === "string" ? err : err.message || "Unknown error";
    setError(errorMessage);
    console.error("VectorDb error:", err);
  }, []);

  // Check if vector database is initialized
  const checkInitialization = useCallback(async () => {
    try {
      clearError();
      await invoke("test_vector_db_connection");
      setIsInitialized(true);
      return true;
    } catch (err) {
      setIsInitialized(false);
      // Don't set error here as it's normal for db to not be initialized
      return false;
    }
  }, [clearError]);

  // Initialize vector database
  const initializeDb = useCallback(
    async (collectionName: string = "default_collection") => {
      try {
        setIsLoading(true);
        clearError();

        await invoke("create_vector_db", { name: collectionName });
        setIsInitialized(true);
        await refreshStats();

        return true;
      } catch (err) {
        handleError(err);
        setIsInitialized(false);
        return false;
      } finally {
        setIsLoading(false);
      }
    },
    [clearError, handleError],
  );

  // Refresh statistics
  const refreshStats = useCallback(async () => {
    if (!isInitialized) return;

    try {
      clearError();
      const dbStats = await invoke<VectorDbStats>("get_vector_db_stats");
      setStats(dbStats);
    } catch (err) {
      handleError(err);
    }
  }, [isInitialized, clearError, handleError]);

  // Search vector database
  const search = useCallback(
    async (
      query: string,
      options?: Partial<VectorSearchOptions>,
    ): Promise<SearchResultResponse[]> => {
      if (!isInitialized) {
        throw new VectorDbError("Vector database not initialized");
      }

      try {
        setIsLoading(true);
        clearError();

        let results: SearchResultResponse[];

        if (options?.filter && Object.keys(options.filter).length > 0) {
          // Convert our filter format to the backend format
          const backendFilter: Record<string, any> = {};

          if (options.filter.extension) {
            backendFilter.extension = options.filter.extension;
          }
          if (options.filter.filePath) {
            backendFilter.file_path = options.filter.filePath;
          }
          if (options.filter.fileSize) {
            if (options.filter.fileSize.min !== undefined) {
              backendFilter.file_size_min = options.filter.fileSize.min;
            }
            if (options.filter.fileSize.max !== undefined) {
              backendFilter.file_size_max = options.filter.fileSize.max;
            }
          }
          if (options.filter.modified) {
            if (options.filter.modified.after !== undefined) {
              backendFilter.modified_after = options.filter.modified.after;
            }
            if (options.filter.modified.before !== undefined) {
              backendFilter.modified_before = options.filter.modified.before;
            }
          }

          results = await invoke<SearchResultResponse[]>(
            "search_vector_db_with_filter",
            {
              query,
              topK: options?.topK || 10,
              filter: backendFilter,
            },
          );
        } else {
          results = await invoke<SearchResultResponse[]>("search_vector_db", {
            query,
            topK: options?.topK || 10,
          });
        }

        // Apply score threshold if specified
        if (options?.scoreThreshold !== undefined) {
          results = results.filter(
            (result) => result.score >= options.scoreThreshold!,
          );
        }

        return results;
      } catch (err) {
        handleError(err);
        throw new VectorDbError(`Search failed: ${err}`);
      } finally {
        setIsLoading(false);
      }
    },
    [isInitialized, clearError, handleError],
  );

  // Add file to vector database
  const addFile = useCallback(
    async (filePath: string): Promise<AddFileResponse> => {
      if (!isInitialized) {
        throw new VectorDbError("Vector database not initialized");
      }

      try {
        setIsLoading(true);
        clearError();

        const response = await invoke<AddFileResponse>(
          "add_file_to_vector_db",
          {
            filePath,
          },
        );

        // Refresh stats after adding file
        await refreshStats();

        return response;
      } catch (err) {
        handleError(err);
        throw new VectorDbError(`Failed to add file: ${err}`);
      } finally {
        setIsLoading(false);
      }
    },
    [isInitialized, clearError, handleError, refreshStats],
  );

  // Delete file from vector database
  const deleteFile = useCallback(
    async (filePath: string): Promise<void> => {
      if (!isInitialized) {
        throw new VectorDbError("Vector database not initialized");
      }

      try {
        setIsLoading(true);
        clearError();

        await invoke("delete_file_from_vector_db", { filePath });

        // Refresh stats after deletion
        await refreshStats();
      } catch (err) {
        handleError(err);
        throw new VectorDbError(`Failed to delete file: ${err}`);
      } finally {
        setIsLoading(false);
      }
    },
    [isInitialized, clearError, handleError, refreshStats],
  );

  // Sync directory with vector database
  const syncDirectory = useCallback(
    async (directoryPath: string): Promise<SyncFilesResponse> => {
      if (!isInitialized) {
        throw new VectorDbError("Vector database not initialized");
      }

      try {
        setIsLoading(true);
        clearError();

        const response = await invoke<SyncFilesResponse>(
          "sync_files_to_vector_db",
          {
            directoryPath,
          },
        );

        // Refresh stats after sync
        await refreshStats();

        return response;
      } catch (err) {
        handleError(err);
        throw new VectorDbError(`Failed to sync directory: ${err}`);
      } finally {
        setIsLoading(false);
      }
    },
    [isInitialized, clearError, handleError, refreshStats],
  );

  // Update file in vector database
  const updateFile = useCallback(
    async (filePath: string): Promise<void> => {
      if (!isInitialized) {
        throw new VectorDbError("Vector database not initialized");
      }

      try {
        setIsLoading(true);
        clearError();

        await invoke("update_file_in_vector_db", { filePath });

        // Refresh stats after update
        await refreshStats();
      } catch (err) {
        handleError(err);
        throw new VectorDbError(`Failed to update file: ${err}`);
      } finally {
        setIsLoading(false);
      }
    },
    [isInitialized, clearError, handleError, refreshStats],
  );

  // Get indexed files
  const getIndexedFiles = useCallback(async (): Promise<string[]> => {
    if (!isInitialized) {
      throw new VectorDbError("Vector database not initialized");
    }

    try {
      clearError();
      const files = await invoke<string[]>("get_indexed_files");
      return files;
    } catch (err) {
      handleError(err);
      throw new VectorDbError(`Failed to get indexed files: ${err}`);
    }
  }, [isInitialized, clearError, handleError]);

  // Test connection
  const testConnection = useCallback(async (): Promise<boolean> => {
    try {
      clearError();
      await invoke("test_vector_db_connection");
      return true;
    } catch (err) {
      handleError(err);
      return false;
    }
  }, [clearError, handleError]);

  // Delete all embeddings
  const clearDatabase = useCallback(async (): Promise<void> => {
    if (!isInitialized) {
      throw new VectorDbError("Vector database not initialized");
    }

    try {
      setIsLoading(true);
      clearError();

      await invoke("delete_all_embeddings");

      // Refresh stats after clearing
      await refreshStats();
    } catch (err) {
      handleError(err);
      throw new VectorDbError(`Failed to clear database: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }, [isInitialized, clearError, handleError, refreshStats]);

  // Get embedding dimension
  const getEmbeddingDimension = useCallback(async (): Promise<number> => {
    if (!isInitialized) {
      throw new VectorDbError("Vector database not initialized");
    }

    try {
      clearError();
      const dimension = await invoke<number>("get_embedding_dimension");
      return dimension;
    } catch (err) {
      handleError(err);
      throw new VectorDbError(`Failed to get embedding dimension: ${err}`);
    }
  }, [isInitialized, clearError, handleError]);

  // Check initialization on mount
  useEffect(() => {
    checkInitialization();
  }, [checkInitialization]);

  // Load stats when initialized
  useEffect(() => {
    if (isInitialized) {
      refreshStats();
    }
  }, [isInitialized, refreshStats]);

  return {
    // State
    isInitialized,
    isLoading,
    error,
    stats,

    // Core operations
    search,
    addFile,
    deleteFile,
    syncDirectory,
    refreshStats,
    testConnection,

    // Extended operations
    initializeDb,
    updateFile,
    getIndexedFiles,
    clearDatabase,
    getEmbeddingDimension,
    clearError,
    checkInitialization,
  };
};

export default useVectorDb;
