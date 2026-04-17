pub(super) const WSDD_MARKER_START: &str = "# WSDD Developer Area Docker";
pub(super) const WSDD_MARKER_END: &str = "# WSDD End of Area";

const DEFAULT_DOMAINS: [&str; 2] = ["pma.wsdd.dock", "mysql.wsdd.dock"];

pub(super) fn default_domains(extra_domains: Option<&[&str]>) -> Vec<String> {
    let mut domains: Vec<String> = DEFAULT_DOMAINS.iter().map(|s| s.to_string()).collect();
    if let Some(extra) = extra_domains {
        for domain in extra {
            let domain = domain.to_string();
            if !domains.contains(&domain) {
                domains.push(domain);
            }
        }
    }
    domains
}

pub(super) fn upsert_wsdd_block(content: &str, domains: &mut Vec<String>) -> String {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let start_idx = lines
        .iter()
        .position(|line| line.trim() == WSDD_MARKER_START);

    if let Some(start) = start_idx {
        let end = lines[start + 1..]
            .iter()
            .position(|line| line.trim() == WSDD_MARKER_END)
            .map(|idx| start + 1 + idx)
            .unwrap_or(lines.len());

        for line in lines.iter().take(end).skip(start + 1) {
            if let Some(domain) = line.split_whitespace().last() {
                let domain = domain.to_string();
                if !domains.contains(&domain) {
                    domains.push(domain);
                }
            }
        }

        lines.drain((start + 1)..end);
        let entries: Vec<String> = domains
            .iter()
            .map(|domain| format!("127.0.0.1 {domain}"))
            .collect();
        for (offset, entry) in entries.into_iter().enumerate() {
            lines.insert(start + 1 + offset, entry);
        }
    } else {
        if lines
            .last()
            .map(|line| !line.trim().is_empty())
            .unwrap_or(false)
        {
            lines.push(String::new());
        }
        lines.push(WSDD_MARKER_START.to_string());
        for domain in domains {
            lines.push(format!("127.0.0.1 {domain}"));
        }
        lines.push(WSDD_MARKER_END.to_string());
    }

    normalize_crlf(lines.join("\r\n"))
}

pub(super) fn remove_wsdd_block(content: &str) -> String {
    let mut result = String::new();
    let mut inside_block = false;
    for line in content.lines() {
        if line.contains(WSDD_MARKER_START) {
            inside_block = true;
            continue;
        }
        if line.contains(WSDD_MARKER_END) {
            inside_block = false;
            continue;
        }
        if !inside_block {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

pub(super) fn remove_domains_from_block(content: &str, domains_to_remove: &[&str]) -> String {
    let targets: Vec<String> = domains_to_remove
        .iter()
        .map(|domain| domain.trim().to_string())
        .collect();
    if targets.is_empty() {
        return content.to_string();
    }

    let lines: Vec<&str> = content.lines().collect();
    let Some(start) = lines
        .iter()
        .position(|line| line.trim() == WSDD_MARKER_START)
    else {
        return content.to_string();
    };
    let end = lines[start + 1..]
        .iter()
        .position(|line| line.trim() == WSDD_MARKER_END)
        .map(|idx| start + 1 + idx)
        .unwrap_or(lines.len());

    let remaining_entries: Vec<&str> = lines[start + 1..end]
        .iter()
        .copied()
        .filter(|line| {
            line.split_whitespace()
                .last()
                .map(|domain| {
                    !targets
                        .iter()
                        .any(|target| target.eq_ignore_ascii_case(domain))
                })
                .unwrap_or(true)
        })
        .collect();

    let mut rewritten = Vec::new();
    rewritten.extend_from_slice(&lines[..start]);
    if !remaining_entries.is_empty() {
        rewritten.push(WSDD_MARKER_START);
        rewritten.extend(remaining_entries);
        if end < lines.len() && lines[end].trim() == WSDD_MARKER_END {
            rewritten.push(WSDD_MARKER_END);
            rewritten.extend_from_slice(&lines[end + 1..]);
        } else {
            rewritten.extend_from_slice(&lines[end..]);
        }
    } else {
        let tail_start = if end < lines.len() && lines[end].trim() == WSDD_MARKER_END {
            end + 1
        } else {
            end
        };
        rewritten.extend_from_slice(&lines[tail_start..]);
    }

    rewritten.join("\n")
}

pub(super) fn normalize_crlf(content: String) -> String {
    let mut normalized = content.replace("\r\n", "\n").replace('\n', "\r\n");
    if !normalized.ends_with("\r\n") {
        normalized.push_str("\r\n");
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_domains_from_block_keeps_other_wsdd_entries() {
        let content = "\
127.0.0.1 localhost
# WSDD Developer Area Docker
127.0.0.1 pma.wsdd.dock
127.0.0.1 mysql.wsdd.dock
127.0.0.1 alpha.dock
127.0.0.1 beta.dock
# WSDD End of Area
";
        let updated = remove_domains_from_block(content, &["alpha.dock"]);

        assert!(updated.contains("pma.wsdd.dock"));
        assert!(updated.contains("mysql.wsdd.dock"));
        assert!(updated.contains("beta.dock"));
        assert!(!updated.contains("alpha.dock"));
        assert!(updated.contains(WSDD_MARKER_START));
        assert!(updated.contains(WSDD_MARKER_END));
    }

    #[test]
    fn remove_domains_from_block_removes_empty_wsdd_block() {
        let content = "\
# WSDD Developer Area Docker
127.0.0.1 alpha.dock
# WSDD End of Area
127.0.0.1 localhost
";
        let updated = remove_domains_from_block(content, &["alpha.dock"]);

        assert!(!updated.contains(WSDD_MARKER_START));
        assert!(!updated.contains(WSDD_MARKER_END));
        assert!(updated.contains("127.0.0.1 localhost"));
    }
}
