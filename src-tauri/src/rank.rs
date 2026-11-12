#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitKind {
    App,
    Folder,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub name: String,
    pub path: String,
    pub subtitle: String,
    pub kind: HitKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedHit {
    pub hit: Hit,
    pub score: i32,
    pub strong: bool,
    pub exact: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Quality {
    None = 0,
    Subsequence = 1,
    Substring = 2,
    Prefix = 3,
    Exact = 4,
}

pub fn rank_hits(query: &str, hits: &[Hit], limit: usize) -> Vec<RankedHit> {
    let mut ranked = Vec::new();
    for hit in hits {
        let Some((score, strong, exact)) = judge(query, &hit.name) else {
            continue;
        };
        let score = score + if hit.kind == HitKind::App { 120 } else { 0 };
        ranked.push(RankedHit {
            hit: hit.clone(),
            score,
            strong,
            exact,
        });
    }
    ranked.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| kind_order(a.hit.kind).cmp(&kind_order(b.hit.kind)))
            .then_with(|| a.hit.name.to_lowercase().cmp(&b.hit.name.to_lowercase()))
    });
    if ranked.len() > limit {
        ranked.truncate(limit);
    }
    dedupe_paths(ranked)
}

fn dedupe_paths(ranked: Vec<RankedHit>) -> Vec<RankedHit> {
    let mut seen = std::collections::HashSet::new();
    ranked
        .into_iter()
        .filter(|item| seen.insert(path_key(&item.hit.path)))
        .collect()
}

pub fn path_key(path: &str) -> String {
    if cfg!(windows) {
        path.to_lowercase()
    } else {
        path.to_string()
    }
}

fn kind_order(kind: HitKind) -> u8 {
    match kind {
        HitKind::App => 0,
        HitKind::Folder => 1,
        HitKind::File => 2,
    }
}

/// Score used while walking the filesystem. `None` means "do not show".
pub fn score_name(query: &str, name: &str) -> Option<(i32, bool)> {
    judge(query, name).map(|(score, strong, _)| (score, strong))
}

fn judge(query: &str, name: &str) -> Option<(i32, bool, bool)> {
    let query = query.trim();
    if query.is_empty() || name.is_empty() {
        return None;
    }
    let (score, quality) = best_quality(query, name);
    if quality == Quality::None || score <= 0 {
        return None;
    }
    let len = query.chars().count();
    let keep = matches!(quality, Quality::Exact | Quality::Prefix)
        || (len >= 3 && quality != Quality::None);
    if !keep {
        return None;
    }
    let strong = match quality {
        Quality::Exact | Quality::Prefix => true,
        Quality::Substring => len >= 3,
        _ => false,
    };
    Some((score, strong, quality == Quality::Exact))
}

fn best_quality(query: &str, name: &str) -> (i32, Quality) {
    let mut best = score_one(query, name);
    if let Some(stem) = file_stem(name) {
        let stem_score = score_one(query, stem);
        if stem_score.1 > best.1 || (stem_score.1 == best.1 && stem_score.0 > best.0) {
            best = stem_score;
        }
    }
    best
}

fn file_stem(name: &str) -> Option<&str> {
    if name.starts_with('.') {
        return None;
    }
    let (stem, ext) = name.rsplit_once('.')?;
    if stem.is_empty()
        || ext.is_empty()
        || ext.len() > 5
        || !ext.chars().all(|ch| ch.is_ascii_alphanumeric())
    {
        return None;
    }
    Some(stem)
}

fn score_one(query: &str, target: &str) -> (i32, Quality) {
    let query = query.to_lowercase();
    let target = target.to_lowercase();
    if query.is_empty() || target.is_empty() {
        return (0, Quality::None);
    }
    if target == query {
        return (1000, Quality::Exact);
    }
    if target.starts_with(&query) {
        let penalty = (target.len() - query.len()).min(40) as i32;
        return (820 - penalty, Quality::Prefix);
    }
    if let Some(index) = target.find(&query) {
        return (640 - index.min(40) as i32, Quality::Substring);
    }
    if let Some(score) = subsequence_score(&query, &target) {
        return (score, Quality::Subsequence);
    }
    (0, Quality::None)
}

fn subsequence_score(query: &str, target: &str) -> Option<i32> {
    let chars: Vec<char> = target.chars().collect();
    let mut index = 0usize;
    let mut score = 160i32;
    let mut streak = 0i32;
    let mut previous: Option<usize> = None;
    for needle in query.chars() {
        let mut found = false;
        while index < chars.len() {
            if chars[index] == needle {
                if previous == Some(index.wrapping_sub(1)) {
                    streak += 1;
                    score += 12 + streak * 4;
                } else {
                    streak = 0;
                    score += 6;
                }
                if index == 0 || !chars[index - 1].is_alphanumeric() {
                    score += 18;
                }
                previous = Some(index);
                index += 1;
                found = true;
                break;
            }
            index += 1;
        }
        if !found {
            return None;
        }
    }
    Some(score.min(500))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(name: &str, kind: HitKind) -> Hit {
        Hit {
            name: name.into(),
            path: format!("/{name}"),
            subtitle: String::new(),
            kind,
        }
    }

    #[test]
    fn exact_beats_prefix_and_apps_beat_files() {
        let hits = vec![
            hit("Chrome Beta", HitKind::App),
            hit("Chrome", HitKind::File),
            hit("Chrome", HitKind::App),
        ];
        let ranked = rank_hits("chrome", &hits, 5);
        assert_eq!(ranked[0].hit.kind, HitKind::App);
        assert_eq!(ranked[0].hit.name, "Chrome");
        assert!(ranked[0].exact);
        assert!(ranked[0].score > ranked[1].score);
    }

    #[test]
    fn prefix_beats_subsequence() {
        let hits = vec![hit("Search Archive", HitKind::File), hit("Chrome", HitKind::App)];
        let ranked = rank_hits("chr", &hits, 5);
        assert_eq!(ranked[0].hit.name, "Chrome");
        assert!(ranked[0].strong);
        if ranked.len() > 1 {
            assert!(!ranked.iter().any(|item| item.hit.name == "Search Archive" && item.strong));
        }
    }

    #[test]
    fn short_queries_do_not_substring_match() {
        let hits = vec![hit("Chrome", HitKind::App), hit("Notepad", HitKind::App)];
        let ranked = rank_hits("h", &hits, 10);
        assert!(ranked.iter().all(|item| item.hit.name.to_lowercase().starts_with('h')));
    }

    #[test]
    fn stem_exact_matches_files() {
        let hits = vec![hit("report.pdf", HitKind::File)];
        let ranked = rank_hits("report", &hits, 5);
        assert!(ranked[0].exact);
        assert!(ranked[0].strong);
    }

    #[test]
    fn empty_and_missing_queries_score_nothing() {
        assert!(score_name("", "Chrome").is_none());
        assert!(score_name("zzzqq", "Chrome").is_none());
        assert!(rank_hits("zzzqq", &[hit("Chrome", HitKind::App)], 5).is_empty());
    }
}
