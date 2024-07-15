use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::compare::{best_job, job_buff};
use rusqlite::{types::Value, Connection};

#[derive(Debug)]
pub struct ShowPeepRecord {
    pub id: i64,
    pub name: String,
    pub jobs: Vec<String>,
    pub root_credit: bool,
    pub episode_count: usize,
    pub stylistic: bool,
    pub score: f32,
}

#[derive(Debug)]
pub struct ShowRecord {
    pub id: i64,
    pub title: String,
    pub start_year: String,
    pub title_type: String,
    pub genres: String,
    pub rating: Option<String>,
    pub peeps: HashMap<i64, ShowPeepRecord>,
    pub episodes: Vec<i64>,
}

impl ShowRecord {
    pub fn stylistic_peeps(&self) -> Vec<i64> {
        self.peeps
            .iter()
            .filter_map(|(id, p)| if p.stylistic { Some(*id) } else { None })
            .collect()
    }

    pub fn ignored_jobs(&self) -> HashSet<&String> {
        self.peeps
            .values()
            .flat_map(|p| p.jobs.iter().filter(|j| job_buff(j) <= 1.0))
            .collect()
    }
}

impl ShowRecord {
    fn hydrate_episodes(&mut self, db: &Connection) {
        let mut episode_q = db
            .prepare("SELECT id, parent_show_id FROM shows WHERE parent_show_id IN (?1);")
            .unwrap();
        self.episodes = episode_q
            .query_map([&self.id], |row| Ok(row.get::<_, i64>(0)?))
            .expect("search succeeds")
            .filter_map(Result::ok)
            .collect();
    }

    fn hydrate_direct_peeps(&mut self, db: &Connection) {
        let mut principal_q = db
            .prepare(
                "SELECT
                    principals.peep_id, principals.show_id, principals.category, principals.job, peeps.name
                FROM principals
                JOIN peeps ON principals.peep_id = peeps.id
                WHERE principals.show_id
                IN (?1);")
            .unwrap();
        self.peeps = principal_q
            .query_map([&self.id], |row| {
                let id = row.get(0)?;
                let mut jobs = vec![row.get(2)?, row.get(3)?];
                jobs.retain(|j| j != "\\N");
                Ok((
                    id,
                    ShowPeepRecord {
                        id,
                        jobs,
                        name: row.get(4)?,
                        root_credit: true,
                        episode_count: 0,
                        stylistic: false,
                        score: 0.0,
                    },
                ))
            })
            .expect("search succeeds")
            .filter_map(Result::ok)
            .collect();
    }

    fn hydrate_episode_peeps(&mut self, db: &Connection) {
        struct EpPeep {
            peep_id: i64,
            category: String,
            job: String,
            name: String,
        }

        let episode_ids: Rc<Vec<Value>> =
            Rc::new(self.episodes.iter().cloned().map(Into::into).collect());
        let mut ep_principal_q = db
                .prepare(
                    "SELECT
                        principals.peep_id, principals.show_id, principals.category, principals.job, peeps.name
                    FROM principals
                    JOIN peeps ON principals.peep_id = peeps.id
                    WHERE principals.show_id
                    IN rarray(?1);")
                .unwrap();
        let ep_principals: Vec<_> = ep_principal_q
            .query_map([&episode_ids], |row| {
                Ok(EpPeep {
                    peep_id: row.get(0)?,
                    category: row.get(2)?,
                    job: row.get(3)?,
                    name: row.get(4)?,
                })
            })
            .expect("search succeeds")
            .filter_map(Result::ok)
            .collect();

        for ep in ep_principals {
            let peep_record = self
                .peeps
                .entry(ep.peep_id)
                .or_insert_with(|| ShowPeepRecord {
                    id: ep.peep_id,
                    name: ep.name,
                    jobs: vec![],
                    root_credit: false,
                    episode_count: 0,
                    stylistic: false,
                    score: 0.0,
                });

            peep_record.episode_count += 1;
            if ep.category != "\\N" && !peep_record.jobs.contains(&ep.category) {
                peep_record.jobs.push(ep.category);
            }
            if ep.job != "\\N" && !peep_record.jobs.contains(&ep.job) {
                peep_record.jobs.push(ep.job);
            }
        }
    }
}

pub fn fetch_show_record(db: &Connection, show_id: Value) -> ShowRecord {
    let mut show = db
        .query_row(
            "SELECT id, title, start_year, title_type, genres, rating FROM shows WHERE id=(?1);",
            [&show_id],
            |row| {
                Ok(ShowRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    start_year: row.get(2)?,
                    title_type: row.get(3)?,
                    genres: row.get(4)?,
                    rating: row.get(5)?,
                    peeps: HashMap::new(),
                    episodes: vec![],
                })
            },
        )
        .expect("Show ID should exist but does not");

    show.hydrate_episodes(db);
    show.hydrate_direct_peeps(db);
    show.hydrate_episode_peeps(db);

    for show_peep in show.peeps.values_mut() {
        show_peep.score = job_buff(best_job(&show_peep.jobs));

        if show_peep.score > 1.0 {
            show_peep.stylistic = true;
        }

        if !show.episodes.is_empty() && show_peep.episode_count > 0 {
            let proportion =
                ((show_peep.episode_count as f32 / show.episodes.len() as f32) * 2.0).min(1.0);
            show_peep.score *= proportion;
        }
    }

    show
}
