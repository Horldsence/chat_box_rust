import React, { useState, useCallback, useEffect } from "react";
import {
  Box,
  Button,
  TextField,
  Typography,
  Paper,
  List,
  ListItem,
  ListItemText,
  ListItemSecondaryAction,
  IconButton,
  Chip,
  Alert,
  CircularProgress,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Tabs,
  Tab,
  Card,
  CardContent,
  GridLegacy as Grid,
  Divider,
  Collapse,
  FormControlLabel,
  Checkbox,
  Slider,
} from "@mui/material";
import {
  Search as SearchIcon,
  Add as AddIcon,
  Delete as DeleteIcon,
  Sync as SyncIcon,
  Storage as StorageIcon,
  Assessment as StatsIcon,
  Folder as FolderIcon,
  Settings as SettingsIcon,
  Refresh as RefreshIcon,
  Clear as ClearIcon,
} from "@mui/icons-material";
import { useVectorDb } from "../hooks/useVectorDb";
import { SearchResultResponse, VectorSearchOptions } from "../types/vectordb";

interface VectorDbManagerProps {
  onFileSelect?: (filePath: string) => void;
}

const VectorDbManager: React.FC<VectorDbManagerProps> = ({ onFileSelect }) => {
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
    clearDatabase,
    getIndexedFiles,
    clearError,
  } = useVectorDb();

  // UI State
  const [currentTab, setCurrentTab] = useState(0);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<SearchResultResponse[]>(
    [],
  );
  const [filePath, setFilePath] = useState("");
  const [directoryPath, setDirectoryPath] = useState("");
  const [collectionName, setCollectionName] = useState("default_collection");
  const [indexedFiles, setIndexedFiles] = useState<string[]>([]);
  const [showAdvancedSearch, setShowAdvancedSearch] = useState(false);
  const [searchOptions, setSearchOptions] = useState<
    Partial<VectorSearchOptions>
  >({
    topK: 10,
    scoreThreshold: 0.5,
  });

  // Dialog states
  const [showInitDialog, setShowInitDialog] = useState(false);
  const [showClearDialog, setShowClearDialog] = useState(false);
  const [showFileDialog, setShowFileDialog] = useState(false);
  const [showDirDialog, setShowDirDialog] = useState(false);

  // Load indexed files when initialized
  useEffect(() => {
    if (isInitialized) {
      loadIndexedFiles();
    }
  }, [isInitialized]);

  const loadIndexedFiles = useCallback(async () => {
    try {
      const files = await getIndexedFiles();
      setIndexedFiles(files);
    } catch (err) {
      console.error("Failed to load indexed files:", err);
    }
  }, [getIndexedFiles]);

  const handleSearch = useCallback(async () => {
    if (!searchQuery.trim()) return;

    try {
      const results = await search(searchQuery, searchOptions);
      setSearchResults(results);
    } catch (err) {
      console.error("Search failed:", err);
    }
  }, [search, searchQuery, searchOptions]);

  const handleAddFile = useCallback(async () => {
    if (!filePath.trim()) return;

    try {
      await addFile(filePath);
      setFilePath("");
      setShowFileDialog(false);
      await loadIndexedFiles();
    } catch (err) {
      console.error("Failed to add file:", err);
    }
  }, [addFile, filePath, loadIndexedFiles]);

  const handleSyncDirectory = useCallback(async () => {
    if (!directoryPath.trim()) return;

    try {
      const result = await syncDirectory(directoryPath);
      setDirectoryPath("");
      setShowDirDialog(false);
      await loadIndexedFiles();

      // Show success message
      console.log(`Successfully synced ${result.count} files`);
    } catch (err) {
      console.error("Failed to sync directory:", err);
    }
  }, [syncDirectory, directoryPath, loadIndexedFiles]);

  const handleDeleteFile = useCallback(
    async (filePathToDelete: string) => {
      try {
        await deleteFile(filePathToDelete);
        await loadIndexedFiles();
      } catch (err) {
        console.error("Failed to delete file:", err);
      }
    },
    [deleteFile, loadIndexedFiles],
  );

  const handleInitialize = useCallback(async () => {
    try {
      await initializeDb(collectionName);
      setShowInitDialog(false);
    } catch (err) {
      console.error("Failed to initialize database:", err);
    }
  }, [initializeDb, collectionName]);

  const handleClearDatabase = useCallback(async () => {
    try {
      await clearDatabase();
      setShowClearDialog(false);
      setIndexedFiles([]);
      setSearchResults([]);
    } catch (err) {
      console.error("Failed to clear database:", err);
    }
  }, [clearDatabase]);

  const formatFileSize = (bytes?: number) => {
    if (!bytes) return "Unknown";
    const sizes = ["Bytes", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round((bytes / Math.pow(1024, i)) * 100) / 100 + " " + sizes[i];
  };

  const formatDate = (timestamp?: number) => {
    if (!timestamp) return "Unknown";
    return new Date(timestamp).toLocaleString();
  };

  const renderSearchTab = () => (
    <Box sx={{ p: 2 }}>
      <Paper sx={{ p: 2, mb: 2 }}>
        <Box sx={{ display: "flex", gap: 2, mb: 2 }}>
          <TextField
            fullWidth
            label="Search Query"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Enter search terms..."
            onKeyPress={(e) => e.key === "Enter" && handleSearch()}
          />
          <Button
            variant="contained"
            onClick={handleSearch}
            disabled={!isInitialized || isLoading || !searchQuery.trim()}
            startIcon={<SearchIcon />}
          >
            Search
          </Button>
        </Box>

        <Box sx={{ display: "flex", alignItems: "center", gap: 2, mb: 2 }}>
          <FormControlLabel
            control={
              <Checkbox
                checked={showAdvancedSearch}
                onChange={(e) => setShowAdvancedSearch(e.target.checked)}
              />
            }
            label="Advanced Search"
          />
          {searchResults.length > 0 && (
            <Chip
              label={`${searchResults.length} results`}
              color="primary"
              size="small"
            />
          )}
        </Box>

        <Collapse in={showAdvancedSearch}>
          <Box sx={{ p: 2, bgcolor: "grey.50", borderRadius: 1, mb: 2 }}>
            <Grid container spacing={2}>
              <Grid item xs={6}>
                <Typography gutterBottom>
                  Max Results: {searchOptions.topK}
                </Typography>
                <Slider
                  value={searchOptions.topK || 10}
                  onChange={(_, value) =>
                    setSearchOptions((prev) => ({
                      ...prev,
                      topK: value as number,
                    }))
                  }
                  min={1}
                  max={50}
                  valueLabelDisplay="auto"
                />
              </Grid>
              <Grid item xs={6}>
                <Typography gutterBottom>
                  Min Score: {searchOptions.scoreThreshold}
                </Typography>
                <Slider
                  value={searchOptions.scoreThreshold || 0.5}
                  onChange={(_, value) =>
                    setSearchOptions((prev) => ({
                      ...prev,
                      scoreThreshold: value as number,
                    }))
                  }
                  min={0}
                  max={1}
                  step={0.1}
                  valueLabelDisplay="auto"
                />
              </Grid>
            </Grid>
          </Box>
        </Collapse>
      </Paper>

      {searchResults.length > 0 && (
        <Paper sx={{ p: 2 }}>
          <Typography variant="h6" gutterBottom>
            Search Results
          </Typography>
          <List>
            {searchResults.map((result, index) => (
              <ListItem key={`${result.id}-${index}`} divider>
                <ListItemText
                  primary={
                    <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
                      <Typography variant="subtitle1">
                        {result.payload.file_name || result.payload.file_path}
                      </Typography>
                      <Chip
                        label={`Score: ${result.score.toFixed(3)}`}
                        size="small"
                        color={
                          result.score > 0.8
                            ? "success"
                            : result.score > 0.6
                              ? "warning"
                              : "default"
                        }
                      />
                      {result.payload.extension && (
                        <Chip
                          label={result.payload.extension}
                          size="small"
                          variant="outlined"
                        />
                      )}
                    </Box>
                  }
                  secondary={
                    <Box>
                      <Typography variant="body2" color="text.secondary">
                        {result.payload.file_path}
                      </Typography>
                      {result.payload.content_preview && (
                        <Typography
                          variant="body2"
                          sx={{ mt: 1, fontStyle: "italic" }}
                        >
                          "{result.payload.content_preview}..."
                        </Typography>
                      )}
                      <Box sx={{ mt: 1, display: "flex", gap: 2 }}>
                        {result.payload.file_size && (
                          <Typography variant="caption">
                            Size: {formatFileSize(result.payload.file_size)}
                          </Typography>
                        )}
                        {result.payload.modified && (
                          <Typography variant="caption">
                            Modified: {formatDate(result.payload.modified)}
                          </Typography>
                        )}
                      </Box>
                    </Box>
                  }
                />
                <ListItemSecondaryAction>
                  {onFileSelect && (
                    <IconButton
                      edge="end"
                      onClick={() => onFileSelect(result.payload.file_path)}
                    >
                      <FolderIcon />
                    </IconButton>
                  )}
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>
        </Paper>
      )}
    </Box>
  );

  const renderManageTab = () => (
    <Box sx={{ p: 2 }}>
      <Grid container spacing={2}>
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Add Files
              </Typography>
              <Box sx={{ display: "flex", gap: 1, mb: 2 }}>
                <Button
                  variant="outlined"
                  startIcon={<AddIcon />}
                  onClick={() => setShowFileDialog(true)}
                  disabled={!isInitialized}
                >
                  Add Single File
                </Button>
                <Button
                  variant="outlined"
                  startIcon={<SyncIcon />}
                  onClick={() => setShowDirDialog(true)}
                  disabled={!isInitialized}
                >
                  Sync Directory
                </Button>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Database Actions
              </Typography>
              <Box sx={{ display: "flex", gap: 1, mb: 2, flexWrap: "wrap" }}>
                <Button
                  variant="outlined"
                  startIcon={<RefreshIcon />}
                  onClick={refreshStats}
                  disabled={!isInitialized}
                >
                  Refresh
                </Button>
                <Button
                  variant="outlined"
                  startIcon={<ClearIcon />}
                  onClick={() => setShowClearDialog(true)}
                  disabled={!isInitialized}
                  color="warning"
                >
                  Clear All
                </Button>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Box
                sx={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  mb: 2,
                }}
              >
                <Typography variant="h6">
                  Indexed Files ({indexedFiles.length})
                </Typography>
                <Button
                  size="small"
                  onClick={loadIndexedFiles}
                  disabled={!isInitialized}
                  startIcon={<RefreshIcon />}
                >
                  Refresh List
                </Button>
              </Box>
              {indexedFiles.length > 0 ? (
                <List sx={{ maxHeight: 300, overflow: "auto" }}>
                  {indexedFiles.map((file, index) => (
                    <ListItem key={index} divider>
                      <ListItemText
                        primary={file.split("/").pop()}
                        secondary={file}
                      />
                      <ListItemSecondaryAction>
                        <IconButton
                          edge="end"
                          onClick={() => handleDeleteFile(file)}
                          color="error"
                          size="small"
                        >
                          <DeleteIcon />
                        </IconButton>
                      </ListItemSecondaryAction>
                    </ListItem>
                  ))}
                </List>
              ) : (
                <Typography variant="body2" color="text.secondary">
                  No files indexed yet
                </Typography>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );

  const renderStatsTab = () => (
    <Box sx={{ p: 2 }}>
      <Grid container spacing={2}>
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Database Statistics
              </Typography>
              {stats ? (
                <Box>
                  <Typography variant="body1" gutterBottom>
                    <strong>Point Count:</strong>{" "}
                    {stats.point_count.toLocaleString()}
                  </Typography>
                  <Typography variant="body1" gutterBottom>
                    <strong>Mapped Files:</strong>{" "}
                    {stats.mapped_files.toLocaleString()}
                  </Typography>
                  <Typography variant="body1" gutterBottom>
                    <strong>Embedding Model:</strong> {stats.embedding_model}
                  </Typography>
                  <Divider sx={{ my: 2 }} />
                  <Typography variant="h6" gutterBottom>
                    Collection Info
                  </Typography>
                  <pre style={{ fontSize: "12px", overflow: "auto" }}>
                    {JSON.stringify(stats.collection_info, null, 2)}
                  </pre>
                </Box>
              ) : (
                <Typography variant="body2" color="text.secondary">
                  No statistics available
                </Typography>
              )}
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Connection Status
              </Typography>
              <Box
                sx={{ display: "flex", alignItems: "center", gap: 2, mb: 2 }}
              >
                <Chip
                  label={isInitialized ? "Connected" : "Not Connected"}
                  color={isInitialized ? "success" : "error"}
                  variant="filled"
                />
                <Button
                  size="small"
                  onClick={async () => {
                    const connected = await testConnection();
                    console.log("Connection test result:", connected);
                  }}
                  disabled={isLoading}
                >
                  Test Connection
                </Button>
              </Box>
              {!isInitialized && (
                <Button
                  variant="contained"
                  startIcon={<StorageIcon />}
                  onClick={() => setShowInitDialog(true)}
                  disabled={isLoading}
                >
                  Initialize Database
                </Button>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );

  return (
    <Box sx={{ width: "100%", height: "100%" }}>
      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={clearError}>
          {error}
        </Alert>
      )}

      <Paper sx={{ width: "100%", mb: 2 }}>
        <Tabs
          value={currentTab}
          onChange={(_, newValue) => setCurrentTab(newValue)}
        >
          <Tab icon={<SearchIcon />} label="Search" />
          <Tab icon={<SettingsIcon />} label="Manage" />
          <Tab icon={<StatsIcon />} label="Statistics" />
        </Tabs>
      </Paper>

      {isLoading && (
        <Box sx={{ display: "flex", justifyContent: "center", p: 2 }}>
          <CircularProgress />
        </Box>
      )}

      {currentTab === 0 && renderSearchTab()}
      {currentTab === 1 && renderManageTab()}
      {currentTab === 2 && renderStatsTab()}

      {/* Initialize Dialog */}
      <Dialog open={showInitDialog} onClose={() => setShowInitDialog(false)}>
        <DialogTitle>Initialize Vector Database</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Collection Name"
            fullWidth
            variant="outlined"
            value={collectionName}
            onChange={(e) => setCollectionName(e.target.value)}
            sx={{ mt: 2 }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowInitDialog(false)}>Cancel</Button>
          <Button onClick={handleInitialize} variant="contained">
            Initialize
          </Button>
        </DialogActions>
      </Dialog>

      {/* Clear Database Dialog */}
      <Dialog open={showClearDialog} onClose={() => setShowClearDialog(false)}>
        <DialogTitle>Clear Vector Database</DialogTitle>
        <DialogContent>
          <Typography>
            Are you sure you want to clear all embeddings from the database?
            This action cannot be undone.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowClearDialog(false)}>Cancel</Button>
          <Button
            onClick={handleClearDatabase}
            color="error"
            variant="contained"
          >
            Clear All
          </Button>
        </DialogActions>
      </Dialog>

      {/* Add File Dialog */}
      <Dialog open={showFileDialog} onClose={() => setShowFileDialog(false)}>
        <DialogTitle>Add File to Vector Database</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="File Path"
            fullWidth
            variant="outlined"
            value={filePath}
            onChange={(e) => setFilePath(e.target.value)}
            placeholder="/path/to/your/file.txt"
            sx={{ mt: 2 }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowFileDialog(false)}>Cancel</Button>
          <Button onClick={handleAddFile} variant="contained">
            Add File
          </Button>
        </DialogActions>
      </Dialog>

      {/* Sync Directory Dialog */}
      <Dialog open={showDirDialog} onClose={() => setShowDirDialog(false)}>
        <DialogTitle>Sync Directory to Vector Database</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Directory Path"
            fullWidth
            variant="outlined"
            value={directoryPath}
            onChange={(e) => setDirectoryPath(e.target.value)}
            placeholder="/path/to/your/directory"
            sx={{ mt: 2 }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowDirDialog(false)}>Cancel</Button>
          <Button onClick={handleSyncDirectory} variant="contained">
            Sync Directory
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default VectorDbManager;
