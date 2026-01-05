use crate::Language;
use rand::seq::SliceRandom;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Poem {
    pub title: String,
    pub author: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PoemLibrary {
    en: Vec<Poem>,
    zh: Vec<Poem>,
    fr: Vec<Poem>,
    ja: Vec<Poem>,
    es: Vec<Poem>,
}

impl PoemLibrary {
    pub fn for_language(&self, lang: Language) -> &[Poem] {
        match lang {
            Language::English => &self.en,
            Language::Chinese => &self.zh,
            Language::French => &self.fr,
            Language::Japanese => &self.ja,
            Language::Spanish => &self.es,
        }
    }

    pub fn random_poem(&self, lang: Language) -> Option<Poem> {
        let mut rng = rand::thread_rng();
        self.for_language(lang).choose(&mut rng).cloned()
    }

    fn push(&mut self, lang: Language, poem: Poem) {
        match lang {
            Language::English => self.en.push(poem),
            Language::Chinese => self.zh.push(poem),
            Language::French => self.fr.push(poem),
            Language::Japanese => self.ja.push(poem),
            Language::Spanish => self.es.push(poem),
        }
    }
}

fn lang_dir(lang: Language) -> &'static str {
    match lang {
        Language::English => "en",
        Language::Chinese => "zh",
        Language::French => "fr",
        Language::Japanese => "ja",
        Language::Spanish => "es",
    }
}

fn parse_poem_text(text: &str) -> Option<Poem> {
    // File format:
    // Line 1: title
    // Line 2: author
    // Optional line 3: --- (separator)
    // Remaining lines: poem body (blank lines preserved)
    let mut lines_iter = text.lines();
    let title = lines_iter.next()?.trim().to_string();
    let author = lines_iter.next().unwrap_or("").trim().to_string();

    let mut body: Vec<String> = Vec::new();
    let mut started = false;
    for (i, l) in lines_iter.enumerate() {
        let line = l.trim_end_matches('\r');
        if !started {
            // Skip an optional separator line early in the file.
            if i == 0 && line.trim() == "---" {
                started = true;
                continue;
            }
            started = true;
        }
        body.push(line.to_string());
    }

    // Trim trailing empty lines
    while body.last().is_some_and(|s| s.trim().is_empty()) {
        body.pop();
    }

    if title.is_empty() || body.is_empty() {
        return None;
    }

    Some(Poem {
        title,
        author,
        lines: body,
    })
}

fn load_poems_from_dir(base_dir: &Path) -> PoemLibrary {
    let mut lib = PoemLibrary::default();

    for lang in [
        Language::English,
        Language::Chinese,
        Language::French,
        Language::Japanese,
        Language::Spanish,
    ] {
        let mut dir = PathBuf::from(base_dir);
        dir.push(lang_dir(lang));

        let Ok(read_dir) = fs::read_dir(&dir) else { continue };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("txt") {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else { continue };
            if let Some(poem) = parse_poem_text(&text) {
                lib.push(lang, poem);
            }
        }
    }

    lib
}

fn default_poems() -> PoemLibrary {
    let mut lib = PoemLibrary::default();

    // Keep defaults in-repo but embedded in the binary, so the app still works
    // even when run from a directory without `./poems`.
    let defaults: &[(Language, &str)] = &[
        (Language::English, include_str!("../poems/en/the_moon_stevenson.txt")),
        (
            Language::English,
            include_str!("../poems/en/to_the_moon_shelley_excerpt.txt"),
        ),
        (
            Language::English,
            include_str!("../poems/en/the_moon_dickinson_1896.txt"),
        ),
        (Language::Chinese, include_str!("../poems/zh/jing_ye_si_li_bai.txt")),
        (
            Language::Chinese,
            include_str!("../poems/zh/wang_yue_huai_yuan_zhang_jiu_ling.txt"),
        ),
        (
            Language::Chinese,
            include_str!("../poems/zh/shi_wu_ye_wang_yue_wang_jian.txt"),
        ),
        (
            Language::French,
            include_str!("../poems/fr/clair_de_lune_verlaine_excerpt.txt"),
        ),
        (
            Language::French,
            include_str!("../poems/fr/au_clair_de_la_lune_traditionnel.txt"),
        ),
        (
            Language::French,
            include_str!("../poems/fr/la_lune_blanche_verlaine.txt"),
        ),
        (Language::Japanese, include_str!("../poems/ja/meigetsu_ya_basho.txt")),
        (Language::Japanese, include_str!("../poems/ja/meigetsu_wo_issa.txt")),
        (
            Language::Japanese,
            include_str!("../poems/ja/tsuki_tenshin_buson.txt"),
        ),
        (
            Language::Spanish,
            include_str!("../poems/es/romance_de_la_luna_lorca_excerpt.txt"),
        ),
        (
            Language::Spanish,
            include_str!("../poems/es/luna_lunera_tradicional.txt"),
        ),
    ];

    for (lang, text) in defaults {
        if let Some(poem) = parse_poem_text(text) {
            lib.push(*lang, poem);
        }
    }

    lib
}

/// Load poems from the filesystem (for customization) and merge with built-in defaults.
///
/// - If `poems_dir` is `None`, we try `./poems` (current working directory).
/// - If a language has at least one poem in the filesystem dir, we use those poems for that language.
///   Otherwise, we fall back to built-in poems for that language.
pub fn load_poems(poems_dir: Option<&Path>) -> PoemLibrary {
    let defaults = default_poems();

    let dir = poems_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("poems"));

    let fs_lib = load_poems_from_dir(&dir);

    let mut merged = PoemLibrary::default();
    for lang in [
        Language::English,
        Language::Chinese,
        Language::French,
        Language::Japanese,
        Language::Spanish,
    ] {
        let fs_poems = fs_lib.for_language(lang);
        if !fs_poems.is_empty() {
            for p in fs_poems {
                merged.push(lang, p.clone());
            }
        } else {
            for p in defaults.for_language(lang) {
                merged.push(lang, p.clone());
            }
        }
    }

    merged
}





