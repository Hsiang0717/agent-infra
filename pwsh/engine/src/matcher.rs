use nucleo_matcher::{Config, Matcher, Utf32Str};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScoredCandidate<T> {
    pub item: T,
    pub score: u32,
}

pub struct FuzzyEngine {
    matcher: Matcher,
}

impl FuzzyEngine {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    pub fn score(&mut self, text: &str, query: &str) -> Option<u32> {
        if query.is_empty() {
            return Some(100);
        }

        let mut text_buf = Vec::new();
        let mut query_buf = Vec::new();

        let text_u32 = Utf32Str::new(text, &mut text_buf);
        let query_u32 = Utf32Str::new(query, &mut query_buf);

        self.matcher.fuzzy_match(text_u32, query_u32).map(|s| s as u32)
    }

    #[allow(dead_code)]
    pub fn filter_and_rank<T: Clone>(
        &mut self,
        items: Vec<T>,
        get_text: impl Fn(&T) -> String,
        query: &str,
        base_boost: impl Fn(&T) -> u32,
    ) -> Vec<ScoredCandidate<T>> {
        let mut scored = Vec::new();

        for item in items {
            let text = get_text(&item);
            let boost = base_boost(&item);
            if query.is_empty() {
                scored.push(ScoredCandidate {
                    item,
                    score: boost + 100,
                });
            } else if let Some(match_score) = self.score(&text, query) {
                // Combine fuzzy match score with contextual boost
                let total_score = match_score + boost;
                scored.push(ScoredCandidate {
                    item,
                    score: total_score,
                });
            }
        }

        // Sort descending by score
        scored.sort_by(|a, b| b.score.cmp(&a.score));
        scored
    }
}
