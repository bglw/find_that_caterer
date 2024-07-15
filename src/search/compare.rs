use console::{style, Color};

use super::show_tree::ShowRecord;

pub struct ShowAffinity {
    pub show: ShowRecord,
    pub score: f32,
    pub descriptions: Vec<(String, String)>,
}

fn calc_overlap_bar(
    l_color: Color,
    r_color: Color,
    mut l_eps: usize,
    mut l_peep_eps: usize,
    mut r_eps: usize,
    mut r_peep_eps: usize,
) -> String {
    if l_eps == 0 {
        l_eps = 10;
    }
    if r_eps == 0 {
        r_eps = 10;
    }

    if l_peep_eps == 0 {
        l_peep_eps = 10;
    }
    if r_peep_eps == 0 {
        r_peep_eps = 10;
    }

    let l_peep_chars = (((l_peep_eps as f32 / l_eps as f32) * 10.0).ceil() as usize).min(10);
    let r_peep_chars = (((r_peep_eps as f32 / r_eps as f32) * 10.0).ceil() as usize).min(10);

    format!(
        "{}{} / {}{}",
        style(vec!["─"; 10 - l_peep_chars].join("")).red().dim(),
        style(vec!["▓"; l_peep_chars].join("")).fg(l_color),
        style(vec!["▓"; r_peep_chars].join("")).fg(r_color),
        style(vec!["─"; 10 - r_peep_chars].join("")).red().dim(),
    )
}

pub fn score_show_affinity(root_shows: &[ShowRecord], candidate_show: ShowRecord) -> ShowAffinity {
    let mut score = 0.0;
    let mut descriptions = vec![];
    for root_show in root_shows {
        for root_peep in root_show.peeps.values() {
            if let Some(candidate_peep) = candidate_show.peeps.get(&root_peep.id) {
                score += root_peep.score * candidate_peep.score;
                let name = &root_peep.name;
                let root_jobs = style(
                    root_peep
                        .jobs
                        .iter()
                        .map(|j| style(j).fg(job_color(j)).to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                )
                .cyan();
                let candidate_jobs = style(
                    candidate_peep
                        .jobs
                        .iter()
                        .map(|j| style(j).fg(job_color(j)).to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                )
                .cyan();

                let title = &root_show.title;

                let root_peep_eps = root_peep.episode_count;
                let root_eps = root_show.episodes.len();
                let root_cred = if root_peep_eps > 0 {
                    format!("{name}: {root_peep_eps}/{root_eps} {title} ({root_jobs})")
                } else {
                    format!("{name}: {title} ({root_jobs})")
                };

                let candidate_peep_eps = candidate_peep.episode_count;
                let candidate_eps = candidate_show.episodes.len();
                let candidate_cred = if candidate_peep_eps > 0 {
                    format!("{candidate_peep_eps}/{candidate_eps} ({candidate_jobs})")
                } else {
                    format!("({candidate_jobs})")
                };

                let l_color = job_color(best_job(&root_peep.jobs));
                let r_color = job_color(best_job(&candidate_peep.jobs));

                let bar = calc_overlap_bar(
                    l_color,
                    r_color,
                    root_eps,
                    root_peep_eps,
                    candidate_eps,
                    candidate_peep_eps,
                );

                descriptions.push((format!("{root_cred} → {candidate_cred}"), format!("{bar}")));
            }
        }
    }

    ShowAffinity {
        show: candidate_show,
        score,
        descriptions,
    }
}

pub fn best_job(roles: &Vec<String>) -> &String {
    roles
        .iter()
        .reduce(|r1, r2| if job_buff(r1) > job_buff(r2) { r1 } else { r2 })
        .expect("At least one role should exist")
}

pub fn normalize_job(role: &str) -> &str {
    if role.contains("written")
        || role.contains("script")
        || role.contains("writer")
        || role.contains("developed")
        || role.contains("created")
        || role.contains("story")
        || role.contains("screenplay")
        || role.contains("writing")
        || role.contains("adapted")
        || role.contains("devise")
        || role.contains("idea")
    {
        "written by"
    } else if role.contains("casting") {
        "casting_director"
    } else if role.contains("designer") {
        "production_designer"
    } else if role.contains("editor") {
        "editor"
    } else if role.contains("composer") {
        "composer"
    } else if role.contains("cinematographer") || role.contains("photograph") {
        "cinematographer"
    } else if role.contains("producer") {
        "producer"
    } else if role.contains("based") || role.contains("original") || role.contains("novel") {
        "based on"
    } else if role.contains("director") || role.contains("showrunner") {
        "director"
    } else {
        role
    }
}

pub fn job_buff(role: &str) -> f32 {
    (match normalize_job(role) {
        "cinematographer" | "director of photography" => 60,
        "composer" => 60,
        "director" => 50,
        "writer" | "written by" | "original idea" | "creator" => 40,
        "production_designer" => 30,
        "editor" => 20,
        "based on" => 20,
        "producer" => 20,
        "casting_director" => 10,
        _ => 1,
    }) as f32
}

pub fn job_color(role: &str) -> console::Color {
    use console::Color::*;
    match normalize_job(role) {
        "cinematographer" | "director of photography" => Magenta,
        "composer" => Green,
        "director" => Cyan,
        "writer" | "written by" | "original idea" | "creator" => Yellow,
        "based on" => Yellow,
        "production_designer" => Blue,
        "editor" => Blue,
        "producer" => Blue,
        "casting_director" => Blue,
        _ => Red,
    }
}
