use std::sync::atomic::Ordering;

use simpledb::{server::Config, DbResult, SimpleDB};
use tempfile::TempDir;

fn insert_some_rows(db: &SimpleDB) -> DbResult<()> {

    let tx = db.new_tx()?;
    
    for id in 0..50 {
        let age = id + 20;
        db.planner().execute_update(
            &format!("insert into test_table(id, name, age) values({id}, 'Person{id}', {age})"),
            tx.clone(),
        )?;
    }
    tx.commit()?;
    Ok(())
}

fn main() -> DbResult<()> {
    let temp_dir = TempDir::new().unwrap();
    println!("Test database directory: {:?}", temp_dir.path());
    
    let mut cfg = Config::file(temp_dir.path());
    cfg = cfg.buffer_capacity(128);
    let db = SimpleDB::with_config(cfg)?;
    let planner = db.planner();

    {
        let tx = db.new_tx()?;
        planner.execute_update(
            "create table test_table(id int, name VARCHAR(20), age int)",
            tx.clone(),
        )?;
        tx.commit()?;
    }

/*     {
        let tx = db.new_tx()?;
        db.metadata_mgr()
            .create_index("id_index", "test_table", "id", tx.clone())?;
        println!("Created index on id field");
        tx.commit()?;
    } */

    insert_some_rows(&db).unwrap();

    println!(
        "Before. Pages read: {}, written: {}", 
        db.stats().storage_mgr_stats.pages_read.get(), 
        db.stats().storage_mgr_stats.pages_written.get()
    );

    let start_time = std::time::Instant::now();
    {
        let tx = db.new_write_tx()?;
        db.planner().execute_update(
            "insert into test_table(id, name, age) values(1, 'Alice', 25)",
            tx.clone(),
        )?;
        tx.commit()?;
    }
    {
        let tx = db.new_write_tx()?;
        db.planner().execute_update(
            "insert into test_table(id, name, age) values(2, 'Bob', 30)",
            tx.clone(),
        )?;
        tx.commit()?;
    }
    {
        let tx = db.new_write_tx()?;
        db.planner().execute_update(
            "insert into test_table(id, name, age) values(3, 'John', 40)",
            tx.clone(),
        )?;
        tx.commit()?;
    }
    let duration = start_time.elapsed();
    println!("Insert took {} us", duration.as_micros());
    println!(
        "Before. Pages read: {}, written: {}", 
        db.stats().storage_mgr_stats.pages_read.get(), 
        db.stats().storage_mgr_stats.pages_written.get()
    );

    Ok(())
}
