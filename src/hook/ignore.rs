//! Gitignore-style patterns, for the two places the hooks need them: D10's
//! check that the log location is ignored by the enclosing repository, and
//! D8's `.worktreeinclude` copy, which Claude documents as the same pattern
//! syntax.
//!
//! This is a reader of pattern files, not a reimplementation of Git: it
//! supports comments, negation, anchoring, directory-only patterns and the
//! `*`, `?` and `**` wildcards, which is the whole of what a `.gitignore`
//! line or a `.worktreeinclude` line uses in practice. Nothing here decides
//! a GWZ question, and nothing here writes.

use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
struct Rule {
    negated: bool,
    directory_only: bool,
    anchored: bool,
    pattern: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Patterns {
    rules: Vec<Rule>,
}

impl Patterns {
    pub(crate) fn parse(text: &str) -> Self {
        let mut rules = Vec::new();
        for raw in text.lines() {
            let line = raw.trim_end();
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let (negated, rest) = match line.strip_prefix('!') {
                Some(rest) => (true, rest),
                None => (false, line),
            };
            let (directory_only, rest) = match rest.strip_suffix('/') {
                Some(rest) => (true, rest),
                None => (false, rest),
            };
            let anchored = rest.contains('/');
            let pattern = rest.strip_prefix('/').unwrap_or(rest).to_owned();
            if pattern.is_empty() {
                continue;
            }
            rules.push(Rule {
                negated,
                directory_only,
                anchored,
                pattern,
            });
        }
        Self { rules }
    }

    pub(crate) fn read(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .map(|text| Self::parse(&text))
            .unwrap_or_default()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Whether `relative` matches, with the last matching rule winning, as
    /// Git resolves a pattern list.
    pub(crate) fn matches(&self, relative: &str, is_directory: bool) -> bool {
        let mut verdict = false;
        for rule in &self.rules {
            if rule.matches_path(relative, is_directory) {
                verdict = !rule.negated;
            }
        }
        verdict
    }
}

impl Rule {
    /// A rule covers a path when it names the path itself -- a directory-only
    /// rule only when the path is a directory -- or when it names one of the
    /// path's ancestor directories, which is how `/.gwz/` covers
    /// `.gwz/claude-hooks.log`.
    fn matches_path(&self, relative: &str, is_directory: bool) -> bool {
        if (!self.directory_only || is_directory) && self.matches(relative) {
            return true;
        }
        self.matches_ancestor(relative)
    }

    fn matches(&self, relative: &str) -> bool {
        if self.anchored {
            return glob_match(&self.pattern, relative);
        }
        // An unanchored pattern matches at any depth.
        let mut suffix = Some(relative);
        while let Some(candidate) = suffix {
            if glob_match(&self.pattern, candidate) {
                return true;
            }
            suffix = candidate.split_once('/').map(|(_, rest)| rest);
        }
        false
    }

    /// A directory-only rule also covers everything below the directory it
    /// names, which is how `/.gwz/` ignores `.gwz/claude-hooks.log`.
    fn matches_ancestor(&self, relative: &str) -> bool {
        let mut current = relative;
        while let Some((head, _)) = current.rsplit_once('/') {
            if self.matches(head) {
                return true;
            }
            current = head;
        }
        false
    }
}

/// `*` and `?` never cross `/`; `**` does.
fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    matches_at(&pattern, 0, &text, 0)
}

fn matches_at(pattern: &[char], mut pi: usize, text: &[char], mut ti: usize) -> bool {
    while pi < pattern.len() {
        match pattern[pi] {
            '*' => {
                let double = pattern.get(pi + 1) == Some(&'*');
                let next = if double { pi + 2 } else { pi + 1 };
                // A trailing wildcard swallows the rest, within its limit.
                for skip in ti..=text.len() {
                    if !double && text[ti..skip].contains(&'/') {
                        break;
                    }
                    if matches_at(pattern, next, text, skip) {
                        return true;
                    }
                }
                return false;
            }
            '?' => {
                if ti >= text.len() || text[ti] == '/' {
                    return false;
                }
                pi += 1;
                ti += 1;
            }
            expected => {
                if ti >= text.len() || text[ti] != expected {
                    return false;
                }
                pi += 1;
                ti += 1;
            }
        }
    }
    ti == text.len()
}

/// The repository that encloses `path`, found by walking up looking for a
/// `.git` entry rather than by asking git: on the operator's Mac an empty
/// `~/.git` file makes `git rev-parse` fail for every directory under
/// `$HOME` that is not in a repository (D2, O4).
pub(crate) fn enclosing_repository(path: &Path) -> Option<PathBuf> {
    let mut current = Some(path);
    while let Some(directory) = current {
        if is_repository(directory) {
            return Some(directory.to_path_buf());
        }
        current = directory.parent();
    }
    None
}

/// A `.git` that is a directory holding a `HEAD`, or a gitfile naming a
/// directory. Anything else -- notably the stray empty `~/.git` file on the
/// operator's Mac (O4) -- is not a repository, which is the same verdict
/// git's own "invalid gitfile format" error carries for this purpose (D2).
fn is_repository(directory: &Path) -> bool {
    let marker = directory.join(".git");
    if marker.is_dir() {
        return marker.join("HEAD").is_file();
    }
    std::fs::read_to_string(&marker)
        .map(|text| text.trim_start().starts_with("gitdir:"))
        .unwrap_or(false)
}

/// D10: would a file at `path` show in `git status` of the repository that
/// encloses it? A location that would is not used for the log.
pub(crate) fn path_is_ignored(path: &Path) -> bool {
    let Some(repository) = enclosing_repository(path) else {
        // Outside every repository nothing can show in a status.
        return true;
    };
    let Ok(relative) = path.strip_prefix(&repository) else {
        return true;
    };
    let relative = relative.to_string_lossy().replace('\\', "/");
    let mut patterns = Vec::new();
    patterns.push(Patterns::read(&repository.join(".git/info/exclude")));
    patterns.push(Patterns::read(&repository.join(".gitignore")));
    let components: Vec<&str> = relative.split('/').collect();
    let mut prefix = PathBuf::new();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        prefix.push(component);
        patterns.push(Patterns::read(&repository.join(&prefix).join(".gitignore")));
    }
    patterns
        .iter()
        .filter(|set| !set.is_empty())
        .any(|set| set.matches(&relative, false))
}
