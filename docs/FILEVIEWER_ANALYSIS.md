# FileViewer Component & Data Structures Analysis

## Overview
This document provides a comprehensive analysis of the FileViewer component and the data flow from Rust backend to Svelte frontend in the JSONL Viewer application.

---

## Architecture Summary

### Technology Stack
- **Frontend**: Svelte 5 (with SvelteKit), TypeScript, Tailwind CSS
- **Backend**: Rust (Tauri v2), Tokio async runtime
- **Data Transfer**: Tauri Channels (streaming for large files)
- **Virtualization**: Custom VirtualList component for performance

---

## Key Files and Responsibilities

### Frontend (Svelte)

#### 1. `src/lib/components/FileViewer.svelte` (263 lines)
**Primary component for displaying JSONL data**

**Key Features:**
- Dynamic column rendering based on detected schema
- Two-row grouped header system (group name + sub-keys)
- Grid-based layout with CSS Grid
- Smart formatting for complex types (arrays, objects)
- Highlighting of selected rows
- Integration with VirtualList for performance
- Synced horizontal scrolling between header and body

**Data Flow:**
1. Derives `displayItems` from either `$fileStore.lines` (normal mode) or `$searchStore.results` (search mode)
2. Derives `columns` from `$fileStore.columns`
3. Calculates `gridCols` based on column count
4. Renders headers with grouping logic (e.g., "user" group spans "user_name", "user_id")
5. Passes items to VirtualList for virtualized rendering

**Helper Functions:**
- `getValue(parsed, colPath)`: Retrieves nested values using underscore-separated paths
- `smartFormat(value)`: Intelligently formats arrays and objects for display

---

#### 2. `src/lib/stores/fileStore.ts` (134 lines)
**Central state management for file data**

**State Structure:**
```typescript
interface FileState {
  filePath: string | null;
  lines: JsonLine[];
  totalLines: number;
  fileSize: number;
  format: 'JsonL' | 'JsonArray';
  isLoading: boolean;
  error: string | null;
  metadata: FileMetadata | null;
  selectedLineId: number | null;
  columns: string[];
}
```

**Key Operations:**
- `addLines(lines)`: Appends new lines and auto-detects columns
- `setSelectedLine(id)`: Updates the currently selected row
- `setMetadata(metadata)`: Updates file information

**Column Detection Logic (lines 56-102):**
- Samples first 50 lines to build schema
- Recursively collects nested keys (max depth: 2)
- Flattens keys with underscore notation (e.g., `user_name` from `{user: {name: "..."}}`)
- Prioritizes common keys: `id`, `timestamp`, `level`, `message`, etc.
- Sorts by priority first, then by frequency
- Caps at 100 columns

---

#### 3. `src/lib/components/VirtualList.svelte` (120 lines)
**Generic virtualized list component for performance**

**Props:**
- `items: T[]`: Array of items to render
- `itemHeight: number`: Fixed height per item (default: 60px)
- `overscan: number`: Extra items to render beyond viewport (default: 5)
- `minWidth: string | number`: Minimum width for horizontal scrolling
- `children`: Snippet for rendering each item

**Key Mechanics:**
- Calculates visible range based on scroll position
- Uses `transform: translateY()` to position visible items
- Implements overscan to prevent blank areas during scroll
- ResizeObserver tracks container height changes
- Exports `scrollToItem(index)` for programmatic scrolling

**Performance Optimization:**
- Only renders visible items + overscan buffer
- Fixed item height for predictable positioning
- Debounced viewport updates via ResizeObserver

---

#### 4. `src/lib/stores/searchStore.ts` (58 lines)
**State management for search functionality**

**State Structure:**
```typescript
interface SearchState {
  query: SearchQuery;  // { text?, json_path?, case_sensitive, regex }
  results: SearchResult[];
  stats: SearchStats | null;  // { total_matches, lines_searched }
  isSearching: boolean;
  error: string | null;
}
```

**Search Modes:**
1. **Text-only search**: Searches full line content
2. **JSONPath search**: Queries specific fields using JSONPath syntax
3. **Combined**: JSONPath + text filter on results

---

#### 5. `src/lib/components/SearchBar.svelte` (163 lines)
**Search input UI with advanced options**

**Features:**
- Debounced text input (300ms delay)
- JSONPath field for field-specific queries
- Case sensitivity toggle
- Regex support (UI disabled, backend ready)
- Clear button to reset search
- Keyboard shortcut (Cmd+F to focus)

**Interaction:**
- Updates `searchStore.query` on changes
- Dispatches 'search' event to trigger backend search
- Syncs with store for bidirectional binding

---

#### 6. Supporting Components

##### `DetailPanel.svelte` (112 lines)
- Right-side panel showing selected line details
- Resizable with min/max width constraints
- Displays formatted JSON tree + raw content
- Copy raw content button

##### `JsonTree.svelte` / `JsonTreeNode.svelte` (197 lines)
- Recursive JSON visualization
- Expandable/collapsible nodes
- Color-coded syntax highlighting
- **Context menu on primitives**: Right-click to filter by value
  - Generates JSONPath and sets search query

##### `StatsDialog.svelte` (202 lines)
- Statistical analysis of dataset
- Analyzes all lines (or search results)
- Shows: count, unique, min/max/avg (numeric columns)
- Sorted by most populated columns first

---

### Backend (Rust)

#### 7. `src-tauri/src/commands/file_parser.rs` (213 lines)
**File parsing and streaming logic**

**Key Structures:**
```rust
pub struct JsonLine {
  pub id: usize,
  pub content: String,
  pub parsed: serde_json::Value,
  pub byte_offset: u64,
}

pub struct FileMetadata {
  pub path: String,
  pub total_lines: usize,
  pub file_size: u64,
  pub format: FileFormat,  // JsonL | JsonArray
}
```

**Command: `parse_file_streaming`**
- Handles both JSONL (newline-delimited) and JSON Array formats
- Streams data via Tauri Channel in chunks (2000 lines)
- Auto-detects format based on extension and content
- Supports:
  - Strict JSONL (.jsonl, .ndjson files)
  - Single-line JSON arrays (compact .json)
  - Multi-line/pretty-printed JSON arrays

**Data Flow:**
1. Opens file and reads metadata
2. Determines format (JsonL vs JsonArray)
3. For JsonL: Parses line-by-line
4. For JsonArray: Parses entire file and explodes array
5. Sends chunks to frontend via Channel
6. Returns FileMetadata on completion

---

#### 8. `src-tauri/src/commands/search.rs` (241 lines)
**File search functionality**

**Key Structures:**
```rust
pub struct SearchQuery {
  pub text: Option<String>,
  pub json_path: Option<String>,
  pub case_sensitive: bool,
  pub regex: bool,
}

pub struct SearchResult {
  pub line_id: usize,
  pub matches: Vec<String>,
  pub context: String,
}
```

**Command: `search_in_file`**
- Streaming search via Channel (100 results per chunk)
- Handles both JsonL and JsonArray formats
- Three search scenarios:
  1. Text-only: Searches entire line
  2. JSONPath-only: Queries specific field(s)
  3. Combined: JSONPath results filtered by text

**JSONPath Integration:**
- Uses `jsonpath_rust` crate for querying
- Supports standard JSONPath syntax (`$.users[*].name`)
- Results converted to string array for matching

---

#### 9. `src-tauri/src/commands/export.rs` (318 lines)
**Export functionality (CSV/Excel)**

**Key Features:**
- Scans first 1000 lines to collect all headers
- Flattens nested structures (same logic as frontend)
- Groups headers in Excel export (merged cells for groups)
- Sorts headers alphabetically

**Functions:**
- `export_to_csv`: Uses `csv` crate for valid CSV output
- `export_to_excel`: Uses `rust_xlsxwriter` with formatting

**Shared Utilities:**
- `collect_headers`: Recursively builds flat header list
- `get_flat_value`: Retrieves nested values via path

---

#### 10. `src-tauri/src/main.rs` (109 lines)
**Application entry point**

**Registered Commands:**
```rust
.invoke_handler(tauri::generate_handler![
  file_parser::parse_file_streaming,
  search::search_in_file,
  export::export_to_csv,
  export::export_to_excel,
  network::download_url_to_temp
])
```

**Menu System:**
- Custom application menu (About, Services, Hide, Quit)
- File menu (Open File, Open URL, Export, Close)
- Edit menu (Undo, Redo, Cut, Copy, Paste, Select All)
- View menu (Fullscreen)
- Window menu (Minimize, Maximize, Close)

---

## Data Flow: From Rust to Svelte

### File Loading Flow

```
User Action (Open File/Drag-Drop)
    ↓
+page.svelte → invoke("parse_file_streaming", path, channel)
    ↓
Rust: file_parser.rs → parse_file_streaming()
    ↓
Stream: [JsonLine] chunks (2000 lines each)
    ↓
Frontend: Channel.onmessage → buffer → fileStore.addLines()
    ↓
fileStore: updates lines[] → triggers reactivity
    ↓
FileViewer.svelte: $derived displayItems = $fileStore.lines
    ↓
VirtualList renders visible items
```

### Search Flow

```
User types in SearchBar (debounced 300ms)
    ↓
searchStore.setQuery() → dispatch('search')
    ↓
+page.svelte → handleSearch() → invoke("search_in_file", path, query, channel)
    ↓
Rust: search.rs → search_in_file()
    ↓
Stream: [SearchResult] chunks (100 results each)
    ↓
Frontend: Channel.onmessage → searchStore.addResults()
    ↓
FileViewer.svelte: $derived displayItems = search results (mapped to JsonLine)
    ↓
VirtualList renders filtered results
```

---

## Current Column/Header Rendering Approach

### Schema Detection
1. **Frontend-only**: No backend schema inference
2. **Sampling**: First 50 lines analyzed
3. **Flattening**: Nested objects flattened with underscore notation
4. **Max depth**: 2 levels deep

### Column Prioritization
```typescript
const priorityKeys = ['id', 'timestamp', 'time', 'date', 'level', 
                     'severity', 'message', 'msg', 'name', 'type', 
                     'status', 'user', 'meta'];
```
- Priority keys appear first
- Remaining keys sorted by frequency
- Max 100 columns

### Header Grouping Logic
Two-row system:

**Row 1 (Groups):**
- Groups related columns by first path segment
- Example: `user_id`, `user_name` → Group: "user" (span: 2)
- Uses `grid-column: span {n}` for merging

**Row 2 (Sub-keys):**
- Shows individual field names
- Strips group prefix from display name
- Example: `user_id` → displays "id"

**Grid Layout:**
```css
grid-template-columns: 60px repeat({n}, minmax(100px, 1fr));
```
- Fixed 60px for ID column
- Dynamic columns with min-width 100px
- Horizontal scroll when content overflows

---

## Existing Sort/Filter Patterns

### ✅ Filtering (Implemented)

#### 1. Search-Based Filtering
- **Location**: `SearchBar.svelte` + `search.rs`
- **Method**: Stream-based search via backend
- **Types**:
  - Text search (full line or specific text)
  - JSONPath search (field-specific queries)
  - Combined (JSONPath + text filter)
- **UX**: Debounced input, real-time results, clear button

#### 2. Context Menu Filtering
- **Location**: `JsonTreeNode.svelte`
- **Method**: Right-click on any primitive value in detail panel
- **Action**: Generates JSONPath and sets search query
- **UX**: Confirmation dialog, auto-executes search

#### 3. Export Filtering
- **Location**: `export.rs`
- **Type**: ExportFilter interface (line_ids, search_query)
- **Status**: Interface exists, but filter parameter not currently used in exports

### ❌ Sorting (NOT Implemented)

**Current State:**
- No sorting functionality anywhere in the codebase
- Lines displayed in file order (line_id ascending)
- No sort controls in UI
- No sort state in stores

**Opportunities:**
1. Add sort state to `fileStore` or separate `sortStore`
2. Add sort indicators to headers (up/down arrows)
3. Implement client-side sorting for in-memory data
4. Consider backend sorting for large datasets

---

## Data Structure Summary

### TypeScript Types (`src/lib/types.ts`)

```typescript
// Core data structure
interface JsonLine {
  id: number;
  content: string;
  parsed: Record<string, unknown>;
  byte_offset: number;
}

// File metadata
interface FileMetadata {
  path: string;
  total_lines: number;
  file_size: number;
  format: 'JsonL' | 'JsonArray';
}

// Search query
interface SearchQuery {
  text?: string;
  json_path?: string;
  case_sensitive: boolean;
  regex: boolean;
}

// Search result
interface SearchResult {
  line_id: number;
  matches: string[];
  context: string;
}

// Search statistics
interface SearchStats {
  total_matches: number;
  lines_searched: number;
}

// Export filter
interface ExportFilter {
  line_ids?: number[];
  search_query?: SearchQuery;
}
```

### Rust Structures (mirrored)

```rust
pub struct JsonLine {
    pub id: usize,
    pub content: String,
    pub parsed: serde_json::Value,
    pub byte_offset: u64,
}

pub struct FileMetadata {
    pub path: String,
    pub total_lines: usize,
    pub file_size: u64,
    pub format: FileFormat,
}

pub enum FileFormat {
    JsonL,
    JsonArray,
}

pub struct SearchQuery {
    pub text: Option<String>,
    pub json_path: Option<String>,
    pub case_sensitive: bool,
    pub regex: bool,
}

pub struct SearchResult {
    pub line_id: usize,
    pub matches: Vec<String>,
    pub context: String,
}

pub struct SearchStats {
    pub total_matches: usize,
    pub lines_searched: usize,
}
```

---

## Performance Characteristics

### File Loading
- **Streaming**: Yes, via Tauri Channel
- **Chunk size**: 2000 lines per chunk
- **Buffer size**: 5000 lines frontend buffer
- **Flush interval**: 100ms or buffer full

### Search
- **Streaming**: Yes, results sent in chunks
- **Chunk size**: 100 results per chunk
- **Latency**: Debounced 300ms input

### Rendering
- **Virtualization**: Yes, VirtualList component
- **Overscan**: 5 items beyond viewport
- **Item height**: Fixed 36px (FileViewer rows)
- **Column cap**: 100 columns

### Known Limitations
- Column detection samples only first 50 lines
- Schema changes mid-file not reflected
- No client-side sorting
- No column reordering or hiding
- Statistics computed on all lines (can be slow for >100k)

---

## Extension Points

### Adding Column Sorting
1. Create `sortStore.ts` with sort column + direction state
2. Add sort indicators to FileViewer headers
3. Implement `sort()` function in fileStore or derived
4. Consider memory limits for large datasets

### Adding Column Filtering
1. Extend search to filter specific column values
2. Add filter dropdowns to headers
3. Implement distinct value aggregation (like StatsDialog)
4. Support multiple column filters simultaneously

### Enhancing Schema Detection
1. Move to backend for full-file scanning
2. Support schema evolution detection
3. Add column type inference
4. Provide schema visualization

---

## Key Design Decisions

1. **Streaming over batch**: Channels used for large file operations
2. **Frontend schema detection**: Faster, but limited to sample
3. **Flattened keys**: Simplifies rendering and querying
4. **Virtualization**: Essential for performance with large datasets
5. **No sorting initially**: Trade-off for simplicity, can be added later
6. **Search in backend**: Better for large files, supports complex queries
7. **Two-row headers**: Balances information density with readability
8. **Fixed row height**: Simplifies virtualization at cost of variable content

---

## Dependencies

### Frontend
- `@tauri-apps/api`: Tauri JS API
- `@tauri-apps/plugin-dialog`: File dialogs
- `@tauri-apps/plugin-fs`: File system access
- `bits-ui`: UI components
- `lucide-svelte`: Icons
- `tailwindcss`: Styling
- `svelte`: Component framework

### Backend
- `tauri`: Application framework
- `serde`: Serialization/deserialization
- `tokio`: Async runtime
- `serde_json`: JSON parsing
- `csv`: CSV writing
- `rust_xlsxwriter`: Excel writing
- `regex`: Regular expression support
- `jsonpath-rust`: JSONPath queries
- `reqwest`: HTTP client for URL downloads

