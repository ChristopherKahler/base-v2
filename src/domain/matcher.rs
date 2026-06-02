use crate::domain::session::{rules_hash, SessionState};
use crate::domain::DomainDef;

/// Why a domain matched. Only tracked when DEVMODE is on.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchReason {
    Always,
    Keyword,
    Filepath,
    KeywordAndFilepath,
}

impl std::fmt::Display for MatchReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::Keyword => write!(f, "keyword"),
            Self::Filepath => write!(f, "filepath"),
            Self::KeywordAndFilepath => write!(f, "keyword + filepath"),
        }
    }
}

/// A matched domain paired with why it matched.
#[derive(Debug)]
pub struct DomainMatch<'a> {
    pub domain: &'a DomainDef,
    pub reason: MatchReason,
}

/// Match domains against prompt text and active file paths.
/// Returns domains that should be injected (not yet injected, or rules changed),
/// along with the reason each matched.
pub fn match_domains<'a>(
    prompt: &str,
    domains: &'a [DomainDef],
    session: &SessionState,
    active_paths: &[String],
) -> Vec<DomainMatch<'a>> {
    let prompt_lower = prompt.to_lowercase();

    domains
        .iter()
        .filter_map(|d| {
            // Check if domain should be matched
            let reason = is_matched(d, &prompt_lower, active_paths)?;

            // Check dedup: skip if already injected with same rules hash
            let hash = rules_hash(&d.rules);
            if session.is_injected(&d.name, hash) {
                return None;
            }

            Some(DomainMatch { domain: d, reason })
        })
        .collect()
}

/// Determine if a domain matches the current context.
/// Returns Some(reason) on match, None on no match.
fn is_matched(domain: &DomainDef, prompt_lower: &str, active_paths: &[String]) -> Option<MatchReason> {
    // Always-on domains always match
    if domain.is_always() {
        return Some(MatchReason::Always);
    }

    // Check exclude patterns first — any match vetoes the domain
    for pattern in &domain.exclude {
        if prompt_lower.contains(&pattern.to_lowercase()) {
            return None;
        }
    }

    // Keyword match: any prompt_keyword substring in prompt text
    let keyword_hit = domain
        .prompt_keywords
        .iter()
        .any(|kw| prompt_lower.contains(&kw.to_lowercase()));

    // Path match: any active file path starts with any domain path trigger
    let path_hit = domain.paths.iter().any(|dp| {
        active_paths
            .iter()
            .any(|ap| ap.starts_with(dp) || ap.contains(dp))
    });

    match (keyword_hit, path_hit) {
        (true, true) => Some(MatchReason::KeywordAndFilepath),
        (true, false) => Some(MatchReason::Keyword),
        (false, true) => Some(MatchReason::Filepath),
        (false, false) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_domain(name: &str, mode: &str, keywords: &[&str], rules: &[&str]) -> DomainDef {
        DomainDef {
            name: name.into(),
            mode: mode.into(),
            prompt_keywords: keywords.iter().map(|s| s.to_string()).collect(),
            file_keywords: Vec::new(),
            paths: Vec::new(),
            exclude: Vec::new(),
            sticky: false,
            rules: rules.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn always_on_always_matches() {
        let domains = vec![make_domain("global", "always", &[], &["Rule 1"])];
        let session = SessionState::default();
        let matched = match_domains("anything", &domains, &session, &[]);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].domain.name, "global");
        assert_eq!(matched[0].reason, MatchReason::Always);
    }

    #[test]
    fn keyword_match() {
        let domains = vec![make_domain("dev", "triggered", &["fix bug"], &["Dev rule"])];
        let session = SessionState::default();

        let matched = match_domains("please fix bug in auth", &domains, &session, &[]);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].reason, MatchReason::Keyword);

        let matched = match_domains("check my calendar", &domains, &session, &[]);
        assert!(matched.is_empty());
    }

    #[test]
    fn keyword_case_insensitive() {
        let domains = vec![make_domain("dev", "triggered", &["Fix Bug"], &["Rule"])];
        let session = SessionState::default();
        let matched = match_domains("FIX BUG please", &domains, &session, &[]);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn path_match() {
        let mut domain = make_domain("dev", "triggered", &[], &["Rule"]);
        domain.paths = vec!["src/".into()];
        let domains = vec![domain];
        let session = SessionState::default();

        let matched = match_domains(
            "hello",
            &domains,
            &session,
            &["src/main.rs".into()],
        );
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].reason, MatchReason::Filepath);
    }

    #[test]
    fn both_keyword_and_path() {
        let mut domain = make_domain("dev", "triggered", &["code"], &["Rule"]);
        domain.paths = vec!["src/".into()];
        let domains = vec![domain];
        let session = SessionState::default();

        let matched = match_domains(
            "write code",
            &domains,
            &session,
            &["src/main.rs".into()],
        );
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].reason, MatchReason::KeywordAndFilepath);
    }

    #[test]
    fn exclude_vetoes_match() {
        let mut domain = make_domain("dev", "triggered", &["code"], &["Rule"]);
        domain.exclude = vec!["review only".into()];
        let domains = vec![domain];
        let session = SessionState::default();

        let matched = match_domains("write code for this", &domains, &session, &[]);
        assert_eq!(matched.len(), 1);

        let matched = match_domains("review only the code", &domains, &session, &[]);
        assert!(matched.is_empty());
    }

    #[test]
    fn dedup_skips_already_injected() {
        let domains = vec![make_domain("global", "always", &[], &["Rule 1"])];
        let hash = rules_hash(&["Rule 1".into()]);
        let mut session = SessionState::default();
        session.mark_injected("global", hash);

        let matched = match_domains("anything", &domains, &session, &[]);
        assert!(matched.is_empty(), "Should be deduped");
    }

    #[test]
    fn dedup_reinjects_on_rule_change() {
        let domains = vec![make_domain("global", "always", &[], &["Rule 1 UPDATED"])];
        let old_hash = rules_hash(&["Rule 1 ORIGINAL".into()]);
        let mut session = SessionState::default();
        session.mark_injected("global", old_hash);

        let matched = match_domains("anything", &domains, &session, &[]);
        assert_eq!(matched.len(), 1, "Should re-inject because hash changed");
    }

    #[test]
    fn no_domains_no_match() {
        let session = SessionState::default();
        let matched = match_domains("anything", &[], &session, &[]);
        assert!(matched.is_empty());
    }

    #[test]
    fn no_rules_domain_still_matched_but_empty() {
        let domains = vec![make_domain("empty", "always", &[], &[])];
        let session = SessionState::default();
        let matched = match_domains("anything", &domains, &session, &[]);
        assert_eq!(matched.len(), 1);
        assert!(matched[0].domain.rules.is_empty());
    }
}
