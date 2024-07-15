use std::str::Split;

pub trait DataType {
    #[must_use]
    fn mapping_ok(header: &str) -> bool;
    fn load(value: &str) -> Self;
    fn id(&self) -> u64;
}

fn cell_str(cells: &mut Split<char>, nth: usize) -> String {
    cells.nth(nth).unwrap().to_string()
}

fn cell_id(cells: &mut Split<char>, nth: usize, prefix: &str) -> u64 {
    cells
        .nth(nth)
        .unwrap()
        .trim_start_matches(prefix)
        .parse()
        .expect("valid ID")
}

#[derive(Debug)]
pub struct Show {
    pub id: u64,
    pub title_type: String,
    pub title: String,
    pub original_title: String,
    pub start_year: String,
    pub genres: String,
}

impl DataType for Show {
    fn mapping_ok(header: &str) -> bool {
        header
            == "tconst	titleType	primaryTitle	originalTitle	isAdult	startYear	endYear	runtimeMinutes	genres"
    }

    fn load(value: &str) -> Self {
        let mut cells = value.split('\t');
        Self {
            id: cell_id(&mut cells, 0, "tt"),
            title_type: cell_str(&mut cells, 0),
            title: cell_str(&mut cells, 0),
            original_title: cell_str(&mut cells, 0),
            start_year: cell_str(&mut cells, 1),
            genres: cell_str(&mut cells, 2),
        }
    }

    fn id(&self) -> u64 {
        self.id.clone()
    }
}

#[derive(Debug)]
pub struct Episode {
    pub id: u64,
    pub show_id: u64,
}

impl DataType for Episode {
    fn mapping_ok(header: &str) -> bool {
        header == "tconst	parentTconst	seasonNumber	episodeNumber"
    }

    fn load(value: &str) -> Self {
        let mut cells = value.split('\t');
        Self {
            id: cell_id(&mut cells, 0, "tt"),
            show_id: cell_id(&mut cells, 0, "tt"),
        }
    }

    fn id(&self) -> u64 {
        self.id.clone()
    }
}

#[derive(Debug)]
pub struct Principal {
    pub show_id: u64,
    pub peep_id: u64,
    pub category: String,
    pub job: String,
}

impl DataType for Principal {
    fn mapping_ok(header: &str) -> bool {
        header == "tconst	ordering	nconst	category	job	characters"
    }

    fn load(value: &str) -> Self {
        let mut cells = value.split('\t');
        Self {
            show_id: cell_id(&mut cells, 0, "tt"),
            peep_id: cell_id(&mut cells, 1, "nm"),
            category: cell_str(&mut cells, 0),
            job: cell_str(&mut cells, 0),
        }
    }

    fn id(&self) -> u64 {
        self.peep_id.clone()
    }
}

#[derive(Debug)]
pub struct Rating {
    pub show_id: u64,
    pub rating: String,
}

impl DataType for Rating {
    fn mapping_ok(header: &str) -> bool {
        header == "tconst	averageRating	numVotes"
    }

    fn load(value: &str) -> Self {
        let mut cells = value.split('\t');
        Self {
            show_id: cell_id(&mut cells, 0, "tt"),
            rating: cell_str(&mut cells, 0),
        }
    }

    fn id(&self) -> u64 {
        self.show_id.clone()
    }
}

#[derive(Debug)]
pub struct Peep {
    pub id: u64,
    pub name: String,
    pub born: String,
}

impl DataType for Peep {
    fn mapping_ok(header: &str) -> bool {
        header == "nconst	primaryName	birthYear	deathYear	primaryProfession	knownForTitles"
    }

    fn load(value: &str) -> Self {
        let mut cells = value.split('\t');
        Self {
            id: cell_id(&mut cells, 0, "nm"),
            name: cell_str(&mut cells, 0),
            born: cell_str(&mut cells, 0),
        }
    }

    fn id(&self) -> u64 {
        self.id.clone()
    }
}
