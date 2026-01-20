use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::ops::Index;

/// Represents the extracted data of an ebuild file.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EbuildData {
    variables: HashMap<String, String>,
}

impl EbuildData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a variable (name is stored in lowercase).
    pub fn insert(&mut self, name: String, value: String) {
        self.variables.insert(name.to_lowercase(), value);
    }

    /// Retrieves the value of a variable by name (case-insensitive).
    pub fn get(&self, name: &str) -> Option<&String> {
        self.variables.get(&name.to_lowercase())
    }

    /// Returns all variables.
    pub fn variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Scans an ebuild file and extracts variable assignments.
    pub fn scan<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(Self::parse(&content))
    }

    /// Parses the content of an ebuild file.
    /// It is not a bash-syntax parser, but rather a simple variable assignment extractor.
    pub fn parse(content: &str) -> Self {
        let mut data = Self::new();
        let mut lines = content.lines().peekable();
        
        while let Some(line) = lines.next() {
            let trimmed = line.trim();
            
            // Ignore comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Ignore shell functions: blafasel() { ... }
            if (trimmed.contains("()") && (trimmed.contains('{') || lines.peek().map_or(false, |l| l.trim().starts_with('{')))) ||
               (trimmed.starts_with("function ") && (trimmed.contains('{') || lines.peek().map_or(false, |l| l.trim().starts_with('{')))) {
                // Simple skipping of functions (until the closing brace)
                let mut brace_count = 0;
                let mut current_line_content = trimmed.to_string();
                
                loop {
                    brace_count += current_line_content.chars().filter(|&c| c == '{').count();
                    brace_count -= current_line_content.chars().filter(|&c| c == '}').count();
                    
                    if brace_count <= 0 && current_line_content.contains('}') {
                        break;
                    }
                    if let Some(next) = lines.next() {
                        current_line_content = next.trim().to_string();
                    } else {
                        break;
                    }
                }
                continue;
            }

            // Detect variable assignments: NAME=VALUE or NAME=( VALUE )
            if let Some(eq_idx) = trimmed.find('=') {
                let name = trimmed[..eq_idx].trim();
                
                // Validate variable name (must not contain spaces and should start with letter/underscore)
                if !name.chars().all(|c| c.is_alphanumeric() || c == '_') || name.is_empty() {
                    continue;
                }

                let mut value_part = trimmed[eq_idx + 1..].trim();
                
                // Safety check for empty value_part length when accessing chars (though trim() handles empty)
                if value_part.is_empty() && !lines.peek().map_or(false, |l| l.trim().starts_with('(')) {
                    data.insert(name.to_string(), String::new());
                    continue;
                }
                
                // Remove comments at the end of the line (if not within quotes)
                // We search for '#' outside of quotes.
                if let Some(hash_idx) = value_part.find('#') {
                    let prefix = &value_part[..hash_idx];
                    let quote_count = prefix.chars().filter(|&c| c == '"' || c == '\'').count();
                    if quote_count % 2 == 0 {
                        value_part = prefix.trim();
                    }
                }

                let raw_value;

                if value_part.starts_with('(') || (value_part.is_empty() && lines.peek().map_or(false, |l| l.trim().starts_with('('))) {
                    // Array assignment
                    let mut array_content = String::new();
                    let mut current_part = value_part.to_string();
                    
                    if current_part.is_empty() {
                        if let Some(next) = lines.next() {
                            current_part = next.trim().to_string();
                        } else {
                            break;
                        }
                    }
                    
                    if current_part.contains(')') {
                        let start_idx = current_part.find('(').unwrap_or(0);
                        if let Some(end_idx) = current_part.rfind(')') {
                            if current_part.contains('(') {
                                array_content.push_str(&current_part[start_idx + 1..end_idx]);
                            } else {
                                array_content.push_str(&current_part[..end_idx]);
                            }
                        }
                    } else {
                        if current_part.starts_with('(') {
                            array_content.push_str(&current_part[1..]);
                        } else {
                            array_content.push_str(&current_part);
                        }
                        
                        while let Some(next_line) = lines.next() {
                            let next_trimmed = next_line.trim();
                            if let Some(end_idx) = next_trimmed.find(')') {
                                array_content.push(' ');
                                array_content.push_str(&next_trimmed[..end_idx]);
                                break;
                            } else {
                                array_content.push(' ');
                                array_content.push_str(next_trimmed);
                            }
                        }
                    }
                    let raw_val = array_content.replace('\t', " ").trim().to_string();
                    raw_value = raw_val.split_whitespace().collect::<Vec<_>>().join(" ");
                } else if !value_part.is_empty() && ((value_part.starts_with('"') && !value_part[1..].contains('"')) || (value_part.starts_with('\'') && !value_part[1..].contains('\''))) {
                    // Multi-line assignment with quotes
                    let quote = value_part.chars().next().unwrap();
                    let mut quoted_content = value_part[1..].to_string();
                    
                    while let Some(next_line) = lines.next() {
                        quoted_content.push(' ');
                        let next_trimmed = next_line.trim();
                        if let Some(end_idx) = next_trimmed.find(quote) {
                            quoted_content.push_str(&next_trimmed[..end_idx]);
                            break;
                        } else {
                            quoted_content.push_str(next_trimmed);
                        }
                    }
                    let raw_val = quoted_content.replace('\t', " ").trim().to_string();
                    raw_value = raw_val.split_whitespace().collect::<Vec<_>>().join(" ");
                } else {
                    // Simple assignment
                    if value_part.len() >= 2 && ((value_part.starts_with('"') && value_part.ends_with('"')) || 
                       (value_part.starts_with('\'') && value_part.ends_with('\''))) {
                        raw_value = value_part[1..value_part.len() - 1].to_string();
                    } else {
                        raw_value = value_part.to_string();
                    }
                }

                // Immediate resolution of self-references to support extensions
                let mut final_value = raw_value;
                if final_value.contains(&format!("${{{}}}", name.to_uppercase())) || final_value.contains(&format!("${}", name.to_uppercase())) {
                    if let Some(old_val) = data.get(name) {
                        final_value = final_value.replace(&format!("${{{}}}", name.to_uppercase()), old_val);
                        final_value = final_value.replace(&format!("${}", name.to_uppercase()), old_val);
                    }
                }
                if final_value.contains(&format!("${{{}}}", name.to_lowercase())) || final_value.contains(&format!("${}", name.to_lowercase())) {
                    if let Some(old_val) = data.get(name) {
                        final_value = final_value.replace(&format!("${{{}}}", name.to_lowercase()), old_val);
                        final_value = final_value.replace(&format!("${}", name.to_lowercase()), old_val);
                    }
                }

                data.insert(name.to_string(), final_value);
                continue;
            }
        }

        // Two-step process for resolving variable references
        data.resolve_variables();

        data
    }

    pub fn resolve_variables(&mut self) {
        let keys: Vec<String> = self.variables.keys().cloned().collect();
        
        // We do this in two passes to resolve simple dependencies
        for _ in 0..2 {
            let mut updates = Vec::new();
            for key in &keys {
                if let Some(value) = self.variables.get(key) {
                    if value.contains('$') {
                        let mut new_value = value.clone();
                        let mut changed = false;
                        
                        for (vname, vval) in &self.variables {
                            // Look for ${VAR} or $VAR
                            let patterns = vec![format!("${{{}}}", vname.to_uppercase()), format!("${}", vname.to_uppercase())];
                            for pattern in patterns {
                                if new_value.contains(&pattern) {
                                    new_value = new_value.replace(&pattern, vval);
                                    changed = true;
                                }
                            }
                            
                            // Also support lowercase if needed, ebuilds mostly use uppercase
                            let patterns_lc = vec![format!("${{{}}}", vname.to_lowercase()), format!("${}", vname.to_lowercase())];
                            for pattern in patterns_lc {
                                if new_value.contains(&pattern) {
                                    new_value = new_value.replace(&pattern, vval);
                                    changed = true;
                                }
                            }
                        }
                        
                        if changed {
                            updates.push((key.clone(), new_value));
                        }
                    }
                }
            }
            
            for (key, val) in updates {
                self.variables.insert(key, val);
            }
        }
    }
}

impl Index<&str> for EbuildData {
    type Output = String;

    fn index(&self, index: &str) -> &Self::Output {
        self.variables.get(&index.to_lowercase()).unwrap_or(&EMPTY_STRING)
    }
}

static EMPTY_STRING: String = String::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_assignment() {
        let content = "EAPI=8\nKEYWORDS=\"~amd64 x86\"";
        let data = EbuildData::parse(content);
        assert_eq!(data["eapi"], "8");
        assert_eq!(data["keywords"], "~amd64 x86");
    }

    #[test]
    fn test_parse_array_assignment() {
        let content = "IUSE=( foo bar )";
        let data = EbuildData::parse(content);
        assert_eq!(data["iuse"], "foo bar");
    }

    #[test]
    fn test_ignore_functions() {
        let content = "VAR1=val1\nsrc_compile() {\n  emake\n}\nVAR2=val2";
        let data = EbuildData::parse(content);
        assert_eq!(data["var1"], "val1");
        assert_eq!(data["var2"], "val2");
    }

    #[test]
    fn test_resolve_variables() {
        let content = "RDEPEND=\"dev-libs/libxml2\"\nDEPEND=\"${RDEPEND}\"";
        let data = EbuildData::parse(content);
        assert_eq!(data["rdepend"], "dev-libs/libxml2");
        assert_eq!(data["depend"], "dev-libs/libxml2");
    }

    #[test]
    fn test_parse_malformed_ebuild() {
        // These should not panic
        let _ = EbuildData::parse("VAR=(\n");
        let _ = EbuildData::parse("VAR=\"\n");
        let _ = EbuildData::parse("VAR='");
        let _ = EbuildData::parse("VAR=");
        let _ = EbuildData::parse("VAR=()");
        let _ = EbuildData::parse("function test() {");
    }

    #[test]
    fn test_scan_all_example_files() {
        // 1. nginx-1.29.3.ebuild
        let data = EbuildData::scan("testdata/ebuild/nginx-1.29.3.ebuild").unwrap();
        assert_eq!(data["eapi"], "8");
        assert!(data["keywords"].contains("~amd64"));
        assert!(data["nginx_subsystems"].contains("+http"));
        assert_eq!(data["nginx_update_stream"], "mainline");
        assert_eq!(data["nginx_tests_commit"], "06a36245e134eac985cdfc5fac982cb149f61412");
        assert!(data["nginx_misc_files"].contains("nginx-{r2.logrotate"));

        // 2. perl-5.40.2.ebuild
        let data = EbuildData::scan("testdata/ebuild/perl-5.40.2.ebuild").unwrap();
        assert_eq!(data["eapi"], "8");
        assert_eq!(data["dist_author"], "SHAY");
        assert_eq!(data["license"], "|| ( Artistic GPL-1+ )");
        assert_eq!(data["homepage"], "https://www.perl.org/");

        // 3. php-8.4.14.ebuild
        let data = EbuildData::scan("testdata/ebuild/php-8.4.14.ebuild").unwrap();
        assert_eq!(data["eapi"], "8");
        assert_eq!(data["sapis"], "embed cli cgi fpm apache2 phpdbg");
        assert!(data["iuse"].contains("bcmath"));
        assert!(data["iuse"].contains("threads")); // From the first IUSE assignment
        assert_eq!(data["description"], "The PHP language runtime engine");
        assert!(data["license"].contains("PHP-3.01"));
        assert!(data["license"].contains("Zend-2.0"));
        assert!(data["keywords"].contains("~amd64"));
        assert!(data["common_depend"].contains("dev-libs/libpcre2"));
        assert!(data["common_depend"].contains("app-crypt/argon2:="));

        // 4. postfix-3.10.4.ebuild
        let data = EbuildData::scan("testdata/ebuild/postfix-3.10.4.ebuild").unwrap();
        assert_eq!(data["eapi"], "8");
        assert_eq!(data["description"], "A fast and secure drop-in replacement for sendmail");
        assert_eq!(data["homepage"], "https://www.postfix.org/");
        assert_eq!(data["license"], "|| ( IBM EPL-2.0 )");
        assert_eq!(data["slot"], "0");
        assert!(data["keywords"].contains("amd64"));
        assert!(data["iuse"].contains("+berkdb"));
        assert!(data["iuse"].contains("ldap-bind"));
        assert!(data["depend"].contains("acct-group/postfix"));
        assert!(data["depend"].contains("ssl? ( >=dev-libs/openssl-1.1.1:0= )"));

        // 5. rust-bin-1.89.0.ebuild
        let data = EbuildData::scan("testdata/ebuild/rust-bin-1.89.0.ebuild").unwrap();
        assert_eq!(data["eapi"], "8");
        assert_eq!(data["description"], "Systems programming language from Mozilla");
        assert_eq!(data["llvm_optional"], "yes");
        assert_eq!(data["homepage"], "https://www.rust-lang.org/");
        assert_eq!(data["license"], "|| ( MIT Apache-2.0 ) BSD BSD-1 BSD-2 BSD-4");
        assert!(data["keywords"].contains("amd64"));
        assert!(data["iuse"].contains("rust-analyzer"));
        assert!(data["rdepend"].contains("net-misc/curl"));
        // SLOT="${PV%%_*}" resolves to "${PV%%_*}"
        assert!(data["qa_prebuilt"].contains("opt/rust-bin-${PV%%_*}/bin/.*"));
    }
}
