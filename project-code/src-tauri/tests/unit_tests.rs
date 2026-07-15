/// Unit tests for the WhatsApp Archive Viewer Rust backend.
/// Tests are for pure/helper functions that don't require Tauri app state.

// ============================================================================
// NOTE: The functions under test (sanitize_filename, validate_chat_id, etc.)
// are private to lib.rs. We test them via a dedicated pub(crate) test module
// defined inside lib.rs, OR we expose them for testing with #[cfg(test)].
//
// Since modifying lib.rs for test visibility is the cleaner approach,
// the tests below use a re-export module added to lib.rs under #[cfg(test)].
// See the `tests` module at the bottom of lib.rs for the actual test logic.
// ============================================================================

// Integration-style test: build a real SQLite DB in a temp dir and exercise
// the database schema initialization and basic CRUD.
#[cfg(test)]
mod db_integration {
    use rusqlite::Connection;
    use tempfile::TempDir;

    /// Create an in-memory SQLite DB with the full schema applied.
    fn open_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open_in_memory failed");
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;",
        )
        .unwrap();

        // ---- chats table ----
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chats (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                original_name TEXT,
                last_message TEXT,
                timestamp TEXT,
                is_group INTEGER NOT NULL DEFAULT 0,
                zip_path TEXT,
                last_message_epoch INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();

        // ---- messages table ----
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                sender TEXT NOT NULL,
                msg_type TEXT NOT NULL DEFAULT 'text',
                content TEXT NOT NULL,
                media TEXT,
                duration TEXT,
                tag_ext TEXT,
                display_name TEXT,
                is_favorite INTEGER DEFAULT 0,
                FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id)",
            [],
        )
        .unwrap();

        // ---- profiles table ----
        conn.execute(
            "CREATE TABLE IF NOT EXISTS profiles (
                chat_id TEXT PRIMARY KEY,
                name TEXT,
                notes TEXT,
                photo_path TEXT,
                phone_number TEXT,
                profile_modified INTEGER DEFAULT 0,
                background_path TEXT,
                FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE
            )",
            [],
        )
        .unwrap();

        conn
    }

    #[test]
    fn test_schema_creates_tables() {
        let conn = open_test_db();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        // chats, messages, profiles
        assert!(count >= 3, "Expected at least 3 tables, got {}", count);
    }

    #[test]
    fn test_insert_and_retrieve_chat() {
        let conn = open_test_db();
        conn.execute(
            "INSERT INTO chats (id, name, original_name, last_message, timestamp, is_group, last_message_epoch) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                "chat-uuid-1",
                "Alice",
                "Alice",
                "Hello",
                "01/01/2024 10:00",
                0i32,
                1_704_067_200i64,
            ],
        )
        .unwrap();

        let name: String = conn
            .query_row(
                "SELECT name FROM chats WHERE id = ?1",
                rusqlite::params!["chat-uuid-1"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(name, "Alice");
    }

    #[test]
    fn test_insert_and_retrieve_message() {
        let conn = open_test_db();
        conn.execute(
            "INSERT INTO chats (id, name, original_name, last_message, timestamp, is_group, last_message_epoch) VALUES ('c1','Bob','Bob','Hi','01/01/2024 09:00',0,0)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content) VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params!["c1", "01/01/2024 09:05", "Alice", "text", "Hello Bob"],
        )
        .unwrap();

        let content: String = conn
            .query_row(
                "SELECT content FROM messages WHERE chat_id = ?1",
                rusqlite::params!["c1"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(content, "Hello Bob");
    }

    #[test]
    fn test_cascade_delete_removes_messages() {
        let conn = open_test_db();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        conn.execute(
            "INSERT INTO chats (id, name, original_name, last_message, timestamp, is_group, last_message_epoch) VALUES ('c2','Carol','Carol','Hey','02/01/2024 12:00',0,0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content) VALUES ('c2','02/01/2024 12:01','Dave','text','World')",
            [],
        )
        .unwrap();

        conn.execute("DELETE FROM chats WHERE id = 'c2'", []).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE chat_id = 'c2'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "Cascade delete should remove child messages");
    }

    #[test]
    fn test_is_favorite_toggle_via_sql() {
        let conn = open_test_db();
        conn.execute(
            "INSERT INTO chats (id, name, original_name, last_message, timestamp, is_group, last_message_epoch) VALUES ('c3','Eve','Eve','Yo','03/01/2024 08:00',0,0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content, is_favorite) VALUES ('c3','03/01/2024 08:01','Frank','text','Nice',0)",
            [],
        )
        .unwrap();

        let id: i64 = conn
            .query_row("SELECT id FROM messages WHERE chat_id = 'c3'", [], |r| r.get(0))
            .unwrap();

        // Toggle favorite ON
        conn.execute(
            "UPDATE messages SET is_favorite = 1 WHERE id = ?1",
            rusqlite::params![id],
        )
        .unwrap();

        let fav: i64 = conn
            .query_row(
                "SELECT is_favorite FROM messages WHERE id = ?1",
                rusqlite::params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fav, 1);

        // Toggle favorite OFF
        conn.execute(
            "UPDATE messages SET is_favorite = 0 WHERE id = ?1",
            rusqlite::params![id],
        )
        .unwrap();

        let fav2: i64 = conn
            .query_row(
                "SELECT is_favorite FROM messages WHERE id = ?1",
                rusqlite::params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fav2, 0);
    }

    #[test]
    fn test_profile_insert_and_retrieve() {
        let conn = open_test_db();
        conn.execute(
            "INSERT INTO chats (id, name, original_name, last_message, timestamp, is_group, last_message_epoch) VALUES ('c4','Grace','Grace','',''  ,0,0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO profiles (chat_id, name, phone_number) VALUES ('c4', 'Grace Smith', '+31612345678')",
            [],
        )
        .unwrap();

        let phone: String = conn
            .query_row(
                "SELECT phone_number FROM profiles WHERE chat_id = 'c4'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(phone, "+31612345678");
    }

    #[test]
    fn test_messages_full_text_search_via_like() {
        let conn = open_test_db();
        conn.execute(
            "INSERT INTO chats (id, name, original_name, last_message, timestamp, is_group, last_message_epoch) VALUES ('c5','Henry','Henry','','' ,0,0)",
            [],
        )
        .unwrap();
        for (i, text) in ["hello world", "foo bar baz", "hello there"].iter().enumerate() {
            conn.execute(
                "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content) VALUES ('c5',?1,'Ivan','text',?2)",
                rusqlite::params![format!("01/01/2024 10:0{}", i), text],
            )
            .unwrap();
        }

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE chat_id = 'c5' AND content LIKE '%hello%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_msg_type_migration_sticker() {
        let conn = open_test_db();
        conn.execute(
            "INSERT INTO chats (id, name, original_name, last_message, timestamp, is_group, last_message_epoch) VALUES ('c6','Jack','Jack','','' ,0,0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content, media) VALUES ('c6','01/01/2024 11:00','Jack','image','','STK-20240101-WA0001.webp')",
            [],
        )
        .unwrap();

        // Apply the sticker migration logic
        conn.execute(
            "UPDATE messages SET msg_type = 'sticker' WHERE msg_type = 'image' AND (media LIKE 'STK-%' OR media LIKE '\u{200e}STK-%')",
            [],
        )
        .unwrap();

        let msg_type: String = conn
            .query_row(
                "SELECT msg_type FROM messages WHERE chat_id = 'c6'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(msg_type, "sticker");
    }

    #[test]
    fn test_tempfile_usage() {
        // Verify tempfile crate works (smoke test for dev dependency)
        let dir = TempDir::new().expect("create temp dir");
        let path = dir.path().join("test.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute("CREATE TABLE t (x INT)", []).unwrap();
        drop(conn);
        assert!(path.exists());
    }
}
