# Roadmap

## Trackers

- Merge the various trackers into a single state management system.  
- Once merged, figure out how to properly modularize them.  
- Current trackers to merge:
  - `entity_tracker.rs`
  - `id_tracker.rs`
  - `party_tracker.rs`
  - `status_tracker.rs`

## Configuration
- Replace hardcoded URLs with environment variables using `dotenv`.

## Packet Recorder (DEV)

- Simple module to record all packets to file for development/debugging.
- Features to consider:
-- Rotate files on zone change or other events

## Backend Processing
- Evaluate party/raid stats (e.g., percentages) on the backend instead of the web frontend.
- Currently, the web frontend processes data by scanning entities and aggregating results.
- Moving computation to the backend will:
- Reduce frontend complexity (improve performance?)

## Misc
- Merge `Entity` & `EntityType` to enum-based polymorphism i.e tagged union 
- Merge `Entity` & `EncounterEntity` hashmaps
- Save boss hp log to db immediately
- Migrate DB to DuckDB