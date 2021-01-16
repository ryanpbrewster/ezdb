#[cfg(test)]
mod test {
    use rusqlite::{Connection, NO_PARAMS};

    #[test]
    fn smoke_test() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS foo (
                my_int INTEGER NOT NULL,
                my_string TEXT NOT NULL,
                my_float REAL
            )
        "#,
            NO_PARAMS,
        )
        .unwrap();
        let stmt = conn
            .prepare("SELECT my_int, my_string, my_float FROM foo")
            .unwrap();
        assert_eq!(
            stmt.column_names(),
            vec!["my_int", "my_string", "my_float",]
        );
    }
}
