use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::models::{HostEntry, HostsLine};

fn make_id(ip: &str, domain: &str, index: usize) -> String {
    let mut hasher = DefaultHasher::new();
    ip.hash(&mut hasher);
    domain.hash(&mut hasher);
    index.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn parse_mapping(line: &str) -> Option<(String, Vec<String>)> {
    let mut parts = line.split_whitespace();
    let ip = parts.next()?.to_string();
    let domains: Vec<String> = parts.map(str::to_string).collect();
    if domains.is_empty() {
        return None;
    }
    Some((ip, domains))
}

pub fn parse_hosts(content: &str) -> Vec<HostsLine> {
    let mut lines = Vec::new();

    for (index, raw) in content.lines().enumerate() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            lines.push(HostsLine::Raw(raw.to_string()));
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix('#') {
            let body = rest.trim_start();
            if let Some((ip, domains)) = parse_mapping(body) {
                for domain in domains {
                    lines.push(HostsLine::Managed(HostEntry {
                        id: make_id(&ip, &domain, index),
                        ip: ip.clone(),
                        domain,
                        enabled: false,
                    }));
                }
            } else {
                lines.push(HostsLine::Raw(raw.to_string()));
            }
            continue;
        }

        if let Some((ip, domains)) = parse_mapping(trimmed) {
            for domain in domains {
                lines.push(HostsLine::Managed(HostEntry {
                    id: make_id(&ip, &domain, index),
                    ip: ip.clone(),
                    domain,
                    enabled: true,
                }));
            }
            continue;
        }

        lines.push(HostsLine::Raw(raw.to_string()));
    }

    lines
}

pub fn render_hosts(lines: &[HostsLine]) -> String {
    let mut out = Vec::new();
    for line in lines {
        match line {
            HostsLine::Managed(entry) => {
                if entry.enabled {
                    out.push(format!("{} {}", entry.ip, entry.domain));
                } else {
                    out.push(format!("# {} {}", entry.ip, entry.domain));
                }
            }
            HostsLine::Raw(raw) => out.push(raw.clone()),
        }
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{parse_hosts, render_hosts};
    use crate::models::HostsLine;

    #[test]
    fn parse_and_render_toggle_roundtrip() {
        let content = "127.0.0.1 example.local\n# 127.0.0.1 disabled.local\n";
        let mut lines = parse_hosts(content);
        let parsed = lines
            .iter()
            .filter_map(|line| match line {
                HostsLine::Managed(e) => Some((e.domain.clone(), e.enabled)),
                HostsLine::Raw(_) => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            parsed,
            vec![
                ("example.local".into(), true),
                ("disabled.local".into(), false)
            ]
        );

        for line in &mut lines {
            if let HostsLine::Managed(entry) = line {
                if entry.domain == "example.local" {
                    entry.enabled = false;
                }
            }
        }

        let rendered = render_hosts(&lines);
        assert!(rendered.contains("# 127.0.0.1 example.local"));
    }
}
