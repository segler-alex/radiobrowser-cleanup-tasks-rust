extern crate mysql;

use std::env;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::cmp;

#[derive(Debug)]
struct ItemCounts {
    all: u32,
    working: u32,
}

fn get_column(pool: &mysql::Pool, column: &str, max_keylen: usize) -> HashMap<String, ItemCounts> {
    let mut items: HashMap<String, ItemCounts> = HashMap::new();

    let query = format!("SELECT {}, LastCheckOK FROM Station", column);
    let results = pool.prep_exec(query, ());
    for result in results {
        for row_ in result {
            let mut row = row_.unwrap();

            let item_str: String = row.take_opt(column)
                .unwrap_or(Ok(String::from("")))
                .unwrap_or(String::from("")).to_lowercase();
            let station_ok: bool = row.take_opt("LastCheckOK")
                .unwrap_or(Ok(false))
                .unwrap_or(false);

            let x = item_str.split(',');
            for item in x {
                let item_1 = item.trim();
                let item_2 = &item_1[0..cmp::min(max_keylen,item_1.len())];
                let item_trimmed = String::from(item_2);
                if item_trimmed != "" {
                    let entry = items
                        .entry(item_trimmed)
                        .or_insert(ItemCounts { all: 0, working: 0 });
                    entry.all += 1;
                    if station_ok {
                        entry.working += 1;
                    }
                }
            }
        }
    }
    items
}

fn save_cache(
    pool: &mysql::Pool,
    table_name: &str,
    column_name: &str,
    cache_new: HashMap<String, ItemCounts>,
) {
    let mut cache_old: HashMap<String, ItemCounts> = HashMap::new();
    let query = format!(
        "SELECT {},StationCount,StationCountWorking FROM {}",
        column_name, table_name
    );
    let results = pool.prep_exec(query, ());
    for result in results {
        for row_ in result {
            let mut row = row_.unwrap();
            let item_name: String = row.take_opt(column_name)
                .unwrap_or(Ok(String::from("")))
                .unwrap_or(String::from(""));
            let count: u32 = row.take_opt("StationCount").unwrap_or(Ok(0)).unwrap_or(0);
            let count_working: u32 = row.take_opt("StationCountWorking")
                .unwrap_or(Ok(0))
                .unwrap_or(0);
            cache_old.insert(
                item_name,
                ItemCounts {
                    all: count,
                    working: count_working,
                },
            );
        }
    }
    for i in cache_new.iter() {
        if cache_old.contains_key(i.0) {
            save_cache_single_update(pool, table_name, column_name, i.0, i.1);
        } else {
            save_cache_single_insert(pool, table_name, column_name, i.0, i.1);
        }
    }

    for i in cache_old.iter() {
        if !cache_new.contains_key(i.0) {
            save_cache_single_delete(pool, table_name, column_name, i.0);
        }
    }
}

fn save_cache_single_insert(
    pool: &mysql::Pool,
    table_name: &str,
    column_name: &str,
    name: &String,
    counts: &ItemCounts,
) {
    let query = format!(
        "INSERT INTO {}({},StationCount,StationCountWorking) VALUES(?,?,?)",
        table_name, column_name
    );
    println!("+ {}", name);
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute((name, counts.all, counts.working));
    match result {
        Ok(_) => {}
        Err(err) => {
            println!("INSERT {}", err);
        }
    }
}

fn save_cache_single_update(
    pool: &mysql::Pool,
    table_name: &str,
    column_name: &str,
    name: &String,
    counts: &ItemCounts,
) {
    let query = format!(
        "UPDATE {} SET StationCount=?,StationCountWorking=? WHERE {}=?",
        table_name, column_name
    );
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute((counts.all, counts.working, name));
    match result {
        Ok(_) => {}
        Err(err) => {
            println!("UPDATE {}", err);
        }
    }
}

fn save_cache_single_delete(
    pool: &mysql::Pool,
    table_name: &str,
    column_name: &str,
    name: &String,
) {
    let query = format!("DELETE FROM {} WHERE {}=?", table_name, column_name);
    println!("- {}", name);
    let mut my_stmt = pool.prepare(query).unwrap();
    let result = my_stmt.execute((name,));
    match result {
        Ok(_) => {}
        Err(err) => {
            println!("DELETE {}", err);
        }
    }
}

fn main() {
    let pause_seconds: u64 = env::var("PAUSE_SECONDS")
        .unwrap_or(String::from("10"))
        .parse()
        .expect("PAUSE_SECONDS is not u64");
    let do_loop: bool = env::var("LOOP")
        .unwrap_or(String::from("false"))
        .parse()
        .expect("LOOP is not bool");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");

    println!("DATABASE_URL  : {}", database_url);
    println!("LOOP          : {}", do_loop);
    println!("PAUSE_SECONDS : {}", pause_seconds);

    loop {
        let pool = mysql::Pool::new(database_url.clone());
        match pool {
            Ok(pool) => {
                let list = get_column(&pool, "Language", 100);
                println!("Languages: {}", list.len());
                save_cache(&pool, "LanguageCache", "LanguageName", list);

                let list = get_column(&pool, "Tags", 100);
                println!("Tags: {}", list.len());
                save_cache(&pool, "TagCache", "TagName", list);
            }
            Err(e) => {
                println!("Connection error {}", e);
            }
        }

        if !do_loop {
            break;
        }
        thread::sleep(Duration::from_secs(pause_seconds));
    }
}
