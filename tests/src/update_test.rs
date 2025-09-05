// TODO we will move to sqllogictests soon
#[cfg(test)]
mod tests {
    use simpledb::{DbResult, SimpleDB};
    use tempfile::TempDir;

    #[test]
    fn test_update() -> DbResult<()> {
        let temp_dir = TempDir::new().unwrap();
        println!("Test database directory: {:?}", temp_dir.path());

        let db = SimpleDB::new(temp_dir.path())?;
        let planner = db.planner();

        {
            let tx = db.new_tx()?;
            planner.execute_update(
                "CREATE TABLE users(id int, name VARCHAR(10), age int)",
                tx.clone(),
            )?;
            tx.commit()?;
        }

        {
            let tx = db.new_tx()?;
            planner.execute_update(
                "INSERT INTO users(id, name, age) VALUES(1, 'Alice', 25)",
                tx.clone(),
            )?;
            planner.execute_update(
                "INSERT INTO users(id, name, age) VALUES(2, 'Bob', 30)",
                tx.clone(),
            )?;
            planner.execute_update(
                "INSERT INTO users(id, name, age) VALUES(3, 'Charlie', 35)",
                tx.clone(),
            )?;
            tx.commit()?;
        }

        {
            let tx = db.new_tx()?;
            let affected_rows =
                planner.execute_update("UPDATE users SET age = 26 WHERE id = 1", tx.clone())?;
            assert_eq!(affected_rows, 1, "Should update exactly 1 row");
            tx.commit()?;
        }

        {
            let tx = db.new_tx()?;
            let plan = planner
                .create_query_plan("SELECT id, name, age FROM users WHERE id = 1", tx.clone())?;
            let mut scan = plan.open(tx.clone());

            scan.before_first()?;
            assert!(scan.next()?, "Should find the updated record");

            let id = scan.get_int("id")?;
            let name = scan.get_string("name")?;
            let age = scan.get_int("age")?;

            assert_eq!(id, 1, "ID should be 1");
            assert_eq!(name, "Alice", "Name should still be 'Alice'");
            assert_eq!(age, 26, "Age should be updated to 26");

            tx.commit()?;
        }

        {
            let tx = db.new_tx()?;
            let affected_rows = planner.execute_update("UPDATE users SET age = 40", tx.clone())?;
            assert_eq!(affected_rows, 3, "Should update all 3 rows");
            tx.commit()?;
        }

        {
            let tx = db.new_tx()?;
            let plan = planner.create_query_plan("SELECT id, name, age FROM users", tx.clone())?;
            let mut scan = plan.open(tx.clone());

            scan.before_first()?;

            let mut records = Vec::new();
            while scan.next()? {
                let id = scan.get_int("id")?;
                let name = scan.get_string("name")?;
                let age = scan.get_int("age")?;
                records.push((id, name, age));
            }

            assert_eq!(records.len(), 3, "Should have exactly 3 records");

            for (_id, _name, age) in &records {
                assert_eq!(*age, 40, "All ages should be 40 after bulk update");
            }

            tx.commit()?;
        }

        Ok(())
    }

    #[test]
    fn test_update_by_index() -> DbResult<()> {
        let temp_dir = TempDir::new().unwrap();
        println!("Test database directory: {:?}", temp_dir.path());

        let db = SimpleDB::new(temp_dir.path())?;
        let planner = db.planner();

        {
            let tx = db.new_tx()?;
            planner.execute_update(
                "CREATE TABLE users(id int, name VARCHAR(10), age int)",
                tx.clone(),
            )?;
            planner.execute_update(
                "CREATE INDEX id_idx ON users (id)",
                tx.clone(),
            )?;
            tx.commit()?;
        }

        {
            let tx = db.new_tx()?;
            for id in 0..10 {
                let age = 20 + (id % 5);
                planner.execute_update(
                   & format!("INSERT INTO users(id, name, age) VALUES({id}, 'User{id}', {age})"),
                    tx.clone(),
                )?;
            }
            tx.commit()?;
        }

        {
            let tx = db.new_tx()?;
            let cnt = planner.execute_update(
                & format!("UPDATE users SET age = 30 WHERE id = 2"),
                tx.clone(),
            )?;
            assert_eq!(cnt, 1);
            tx.commit()?;
        }

        {
            let tx = db.new_tx()?;
            let plan = planner.create_query_plan("SELECT id, name, age FROM users WHERE id = 2", tx.clone())?;
            let mut scan = plan.open(tx.clone());
            scan.before_first()?;
            assert_eq!(true, scan.next()?);
            assert_eq!(30, scan.get_int("age")?);
            tx.commit()?;
        }

        Ok(())
    }
}