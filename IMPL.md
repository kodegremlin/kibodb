## 1 Encoding Decoding && Disk Manager

1. ~~encode-decode methods for leaf and internal nodes.~~
2. ~~file open, read/write methods.~~

## 2 Buffer Pool & Cache Management

1. ~~simple lru in-memory cache to avoid syscalls.~~
2. ~~reading/writing in page-sized buffers.~~
3. add a 75% 25% flusher that runs in the background stealing pages and writing
   them to the disk but keeping them in ram for queries but marking them clean/ready
   to be evicted on notice.

## 3 Write-Ahead Log & Recovery

1. ~~Provide the 'D' in acid, simplified.~~
2. ~~Reloading according to the wal.~~

## 4 B+ Tree implementation

1. Core data manipulation methods: insert, search, delete(tombstone).
2. use rust cursor pattern thing.
3. In-page defragmentation.

## 5 Relation Service

1. Sqlite-like row_id aliasing to be used as PRIMARY_KEY and using that row_id for point lookups.
2. Lexical analysis
3. Ast definition and parsing

## 6 Language semantics
1. `Where` clause evaluation
2. volcano iterator model
3. block nested loop join
4. pipelines breakers like count, avg, orderby, limit, offset
5. insert, update, delete.

## 7 Secondary Indexes and ACID guarantees

1. Secondary indexes using Primary indexes only to return data, but on other fields.
3. 75/25 background flusher.
4. Transactional rollbacks accounting for secondary indexes.

## 8 Final pieces
1. `repl` using rustyline
2. handle sessions
3. pages that are completely empty should be reused when asked for new pages - will need some 
   bookeeping as well.

(To be continued...)
