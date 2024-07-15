use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    hash::BuildHasherDefault,
    io::{self, BufRead, BufReader, Lines},
    iter::Flatten,
    path::Path,
    time::{Duration, Instant},
};

use crate::loading::{DataType, Episode, Peep, Principal, Rating, Show};
use nohash_hasher::{BuildNoHashHasher, NoHashHasher};
use rusqlite::{params, Connection, Statement};

type FastMap<T> = HashMap<u64, T, BuildHasherDefault<NoHashHasher<u64>>>;

fn read_lines<D: DataType>(filename: &str) -> Flatten<Lines<BufReader<File>>> {
    let file = File::open(filename).expect("File should exist");
    let mut lines = io::BufReader::new(file).lines().flatten();

    if !D::mapping_ok(&lines.next().expect("File should contain header")) {
        panic!("File {filename} does not match expected header");
    }

    lines
}

fn index_data<D: DataType>(
    db: &mut Connection,
    filename: &str,
    query: &str,
    inserter: impl Fn(&mut Statement, D),
) -> HashSet<u64> {
    let query_start = Instant::now();
    println!("Loading {filename}");

    let mut ids = HashSet::new();

    let tx = db.transaction().unwrap();
    {
        let mut statement = tx.prepare(query).unwrap();
        read_lines::<D>(filename).for_each(|line| {
            let d = D::load(&line);
            ids.insert(d.id());
            inserter(&mut statement, d);
        });
    }
    tx.commit().unwrap();

    let query_end = Instant::now().duration_since(query_start);
    println!("Loaded {filename} in {query_end:#?}");

    ids
}

pub fn create() {
    let db_path = Path::new("caterer.db");
    _ = fs::remove_file(db_path);
    while db_path.exists() {
        std::thread::sleep(Duration::from_millis(50));
    }

    let mut db = Connection::open(db_path).expect("can create db");
    db.pragma_update(None, "foreign_keys", "ON").unwrap();

    db.execute_batch(
        "BEGIN;
        CREATE TABLE shows (
            id             INTEGER PRIMARY KEY,
            title          TEXT,
            title_type     TEXT,
            start_year     TEXT,
            genres         TEXT,
            rating         TEXT,
            parent_show_id INTEGER,
            FOREIGN KEY (parent_show_id) REFERENCES shows(id)
        );
        CREATE TABLE peeps (
            id     INTEGER PRIMARY KEY,
            name   TEXT,
            born   TEXT
        );
        CREATE TABLE principals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            peep_id     INTEGER NOT NULL,
            show_id     INTEGER NOT NULL,
            category    TEXT,
            job         TEXT,
            FOREIGN KEY (peep_id) REFERENCES peeps(id),
            FOREIGN KEY (show_id) REFERENCES shows(id)
        );
        CREATE INDEX idx_parent_show_id ON shows(parent_show_id);
        CREATE INDEX idx_principals_show_id ON principals(show_id);
        CREATE INDEX idx_principals_peep_id ON principals(peep_id);
        COMMIT;",
    )
    .expect("sqlite tables should create ok");

    println!("db created ok");

    let show_ids = index_data::<Show>(
        &mut db,
        "title.basics.tsv",
        "INSERT INTO shows (id, title, title_type, start_year, genres) VALUES (?1, ?2, ?3, ?4, ?5)",
        |statement, show| {
            statement
                .execute(params![
                    &show.id,
                    &show.title,
                    &show.title_type,
                    &show.start_year,
                    &show.genres,
                ])
                .expect("sqlite should be healthy");
        },
    );

    index_data::<Episode>(
        &mut db,
        "title.episode.tsv",
        "UPDATE shows SET parent_show_id = ?1 WHERE id = ?2",
        |statement, episode| {
            if show_ids.contains(&episode.show_id) && show_ids.contains(&episode.id) {
                statement
                    .execute(params![&episode.show_id, &episode.id,])
                    .expect("sqlite should be healthy");
            }
        },
    );

    index_data::<Rating>(
        &mut db,
        "title.ratings.tsv",
        "UPDATE shows SET rating = ?1 WHERE id = ?2",
        |statement, rating| {
            if !show_ids.contains(&rating.show_id) {
                return;
            };

            statement
                .execute(params![&rating.rating, &rating.show_id])
                .expect("sqlite should be healthy");
        },
    );

    let peep_ids = index_data::<Peep>(
        &mut db,
        "name.basics.tsv",
        "INSERT INTO peeps (id, name, born) VALUES (?1, ?2, ?3)",
        |statement, peep| {
            statement
                .execute(params![&peep.id, &peep.name, &peep.born,])
                .expect("sqlite should be healthy");
        },
    );

    index_data::<Principal>(
        &mut db,
        "title.principals.tsv",
        "INSERT INTO principals (peep_id, show_id, category, job) VALUES (?1, ?2, ?3, ?4)",
        |statement, principal| {
            if !show_ids.contains(&principal.show_id) || !peep_ids.contains(&principal.peep_id) {
                return;
            };

            statement
                .execute(params![
                    &principal.peep_id,
                    &principal.show_id,
                    &principal.category,
                    &principal.job,
                ])
                .expect("sqlite should be healthy");
        },
    );

    println!("Loaded the files!");

    // println!("{:?}", basics.iter().next());
    // println!("{:?}", episodes.iter().next());
    // println!("{:?}", principals.iter().next());
    // println!("{:?}", ratings.iter().next());

    // let query_start = Instant::now();

    // let castmem: u64 = 4423632;
    // let shows = principals.get(&castmem);

    // let query_end = Instant::now().duration_since(query_start);

    // println!("shows: {shows:#?}");
    // println!("'queried' in {query_end:#?}");
}
