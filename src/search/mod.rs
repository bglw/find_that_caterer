use std::{collections::HashSet, path::Path, rc::Rc, time::Instant};

use compare::{score_show_affinity, ShowAffinity};
use console::style;
use rusqlite::{types::Value, Connection};
use show_tree::fetch_show_record;

mod compare;
mod show_tree;

#[derive(Debug, Clone)]
struct ShowEntity {
    id: i64,
    title: String,
    start_year: String,
    parent_show_id: Option<i64>,
}

#[derive(Debug)]
struct PrincipalEntity {
    peep_id: i64,
    show_id: i64,
    category: String,
    job: String,
    name: String,
}

pub fn search(titles: Vec<String>) {
    let db_path = Path::new("caterer.db");
    let db = Connection::open(db_path).expect("can create db");
    db.pragma_update(None, "foreign_keys", "ON").unwrap();
    rusqlite::vtab::array::load_module(&db).expect("vtab shount load");

    let show_ids_ints: Vec<i64> = titles
        .into_iter()
        .map(|title| title.trim_start_matches("tt").parse::<i64>().unwrap())
        .collect();

    println!("----> Starting search for root shows");
    let start_q = Instant::now();

    let shows: Vec<_> = show_ids_ints
        .iter()
        .map(|show_id| fetch_show_record(&db, (*show_id).into()))
        .collect();

    let end_q = Instant::now().duration_since(start_q);
    let per_q = end_q / shows.len() as u32;

    for show in &shows {
        println!(
            "  • Found record for root show {} ({}) in {}ms",
            show.title,
            show.start_year,
            per_q.as_millis()
        );
    }

    let all_staff: HashSet<i64> = shows.iter().flat_map(|s| s.stylistic_peeps()).collect();
    let all_staff: Rc<Vec<Value>> = Rc::new(all_staff.into_iter().map(Into::into).collect());

    println!(
        "----> Found {} staff, starting search for linked shows",
        all_staff.len()
    );

    let mut all_principals_q = db
        .prepare(
            "SELECT
                    principals.show_id, principals.peep_id
                FROM principals
                JOIN peeps ON principals.peep_id = peeps.id
                WHERE principals.peep_id
                IN rarray(?1);",
        )
        .unwrap();
    let all_show_and_episode_ids: Vec<_> = all_principals_q
        .query_map([&all_staff], |row| Ok(row.get::<_, i64>(0)?))
        .expect("search succeeds")
        .filter_map(Result::ok)
        .collect();

    let all_show_ids: HashSet<_> =
        all_show_and_episode_ids
            .into_iter()
            .map(|show_or_episode_id| {
                let mut ep_q = db
                    .prepare("SELECT id, parent_show_id FROM shows WHERE id=(?1);")
                    .unwrap();
                let show_id: i64 = ep_q
                    .query_row([&show_or_episode_id], |r| {
                        Ok((r.get::<_, Option<i64>>(1)?)
                            .unwrap_or_else(|| r.get::<_, i64>(0).unwrap()))
                    })
                    .unwrap();

                show_id
            })
            .collect();

    let est = per_q * all_show_ids.len() as u32;
    println!(
        "----> Found {} linked shows, fetching full records. (estimated {} seconds)",
        all_show_ids.len(),
        est.as_secs()
    );

    let start_q = Instant::now();

    let candidate_shows: Vec<_> = all_show_ids
        .into_iter()
        .filter(|l| !show_ids_ints.contains(l))
        .map(|show_id| fetch_show_record(&db, show_id.into()))
        .collect();

    let end_q = Instant::now().duration_since(start_q);

    println!("Loaded candidate shows in {}s", end_q.as_secs());

    let ignored_jobs: HashSet<&String> = shows
        .iter()
        .chain(candidate_shows.iter())
        .flat_map(|s| s.ignored_jobs())
        .collect();
    println!("Ignoring the following jobs as non-stylistic:");
    for ij in ignored_jobs {
        println!("  • {ij}");
    }

    println!("----> Scoring shows");

    let start_q = Instant::now();

    let mut show_affinities: Vec<ShowAffinity> = candidate_shows
        .into_iter()
        .map(|cs| score_show_affinity(&shows[..], cs))
        .collect();

    show_affinities.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    let end_q = Instant::now().duration_since(start_q);
    println!("Scored shows in {}s", end_q.as_secs());

    println!("----> Top 100 shows:");

    for affinity in show_affinities.into_iter().take(100) {
        println!(
            "\n\n### {} ({})\nRating: {}\n{}: {}",
            style(affinity.show.title).bold(),
            affinity.show.start_year,
            affinity
                .show
                .rating
                .unwrap_or_else(|| "unknown".to_string()),
            affinity.show.title_type,
            affinity.show.genres,
        );
        for (d, bar) in affinity.descriptions {
            println!("{} {}", bar, d);
        }
    }
}
