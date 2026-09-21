//! Template helper functions and utilities.
//!
//! This module provides utility functions used by templates for common tasks.

/// Format for date-time inputs used by templates (YYYY-MM-DDTHH:MM).
pub(crate) const DATE_FORMAT: &str = "%Y-%m-%dT%H:%M";

/// The date format used in templates (YYYY-MM-DD).
pub(crate) const DATE_FORMAT_2: &str = "%Y-%m-%d";

/// Builds an absolute URL from a configured base URL and path.
pub(crate) fn absolute_url(base_url: &str, path: &str) -> String {
    let base_url = base_url.strip_suffix('/').unwrap_or(base_url);
    if path.starts_with('/') {
        format!("{base_url}{path}")
    } else {
        format!("{base_url}/{path}")
    }
}

/// Builds the public URL for a configured Open Graph image.
pub(crate) fn open_graph_image_url(base_url: &str, image_url: &str) -> String {
    // Route uploaded images through the public Open Graph image endpoint
    if let Some(file_name) = image_url.strip_prefix("/images/") {
        return absolute_url(base_url, &format!("/images/og/{file_name}"));
    }

    // Convert other local paths into absolute URLs
    if image_url.starts_with('/') {
        return absolute_url(base_url, image_url);
    }

    // Keep already-absolute external URLs unchanged
    image_url.to_string()
}

/// Returns the first image URL that social crawlers can use for a link preview.
pub(crate) fn first_open_graph_image_url<'a>(
    candidates: impl IntoIterator<Item = Option<&'a str>>,
) -> Option<&'a str> {
    candidates
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|image_url| is_open_graph_compatible_image_url(image_url))
}

/// Returns whether a URL is usable as an Open Graph image.
///
/// Social crawlers (WhatsApp, Facebook, Slack, LinkedIn) reject SVG previews
/// and then fall back to the site favicon.
pub(crate) fn is_open_graph_compatible_image_url(image_url: &str) -> bool {
    let image_url = image_url.trim();
    if image_url.is_empty() {
        return false;
    }

    let path = image_url.split(['?', '#']).next().unwrap_or(image_url);
    let extension = path.rsplit('.').next().unwrap_or("");
    !extension.eq_ignore_ascii_case("svg")
}

/// Generates initials from a name and username.
pub(crate) fn user_initials(name: Option<&str>, username: &str) -> String {
    // Helper to split a string into words based on whitespace and non-alphabetic chars
    fn split_words(value: &str) -> Vec<&str> {
        value
            .trim()
            .split(|c: char| c.is_whitespace() || !c.is_alphabetic())
            .filter(|word| !word.is_empty())
            .collect()
    }

    // Helper to get the nth alphabetic character from a string, uppercased
    let get_nth_alpha = |s: &str, n: usize| {
        s.chars()
            .filter(|c| c.is_alphabetic())
            .nth(n)
            .map(|c| c.to_ascii_uppercase())
    };

    // Prefer name if available, otherwise derive from username
    let words = name
        .and_then(|name| {
            let words = split_words(name);
            if words.is_empty() { None } else { Some(words) }
        })
        .unwrap_or_else(|| split_words(username));

    // Generate initials based on number of words
    match words.len() {
        0 => "?".to_string(),
        1 => {
            // Single word: first two chars (one if only one letter)
            let first_word = words[0];
            match (get_nth_alpha(first_word, 0), get_nth_alpha(first_word, 1)) {
                (Some(first_char), Some(second_char)) => format!("{first_char}{second_char}"),
                (Some(first_char), None) => format!("{first_char}"),
                _ => "?".to_string(),
            }
        }
        _ => {
            // Two or more words: first char of first and last words
            let (first_word, last_word) = (words[0], words[words.len() - 1]);
            match (get_nth_alpha(first_word, 0), get_nth_alpha(last_word, 0)) {
                (Some(first_char), Some(second_char)) => format!("{first_char}{second_char}"),
                (Some(first_char), None) => format!("{first_char}"),
                (None, Some(second_char)) => format!("{second_char}"),
                _ => "?".to_string(),
            }
        }
    }
}

// Tests.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_initials() {
        // Name present, two words -> first and last initials
        assert_eq!(user_initials(Some("John Doe"), ""), "JD");

        // Single-word name -> first two letters
        assert_eq!(user_initials(Some("Alice"), ""), "AL");

        // Single-letter name -> keep one letter
        assert_eq!(user_initials(Some("A"), ""), "A");

        // Leading and trailing spaces -> trimmed before processing
        assert_eq!(user_initials(Some("  Bob Johnson  "), ""), "BJ");

        // Multiple middle names -> still first and last
        assert_eq!(
            user_initials(Some("Alexander Graham Bell Hamilton"), ""),
            "AH"
        );

        // Hyphenated names -> treat as separate words
        assert_eq!(user_initials(Some("Mary-Jane Watson-Parker"), ""), "MP");

        // Name with no alphabetic characters in first name
        assert_eq!(user_initials(Some("123 Doe"), ""), "DO");

        // Name missing -> derive from username
        assert_eq!(user_initials(None, "jdoe"), "JD");

        // Name with no alphabetic characters -> fallback to username
        assert_eq!(user_initials(Some("123"), "alpha"), "AL");

        // Username with separators -> split into words
        assert_eq!(user_initials(None, "john_doe"), "JD");

        // Username without letters -> fallback placeholder
        assert_eq!(user_initials(None, "1234"), "?");
    }

    #[test]
    fn test_is_open_graph_compatible_image_url() {
        assert!(is_open_graph_compatible_image_url("/images/event-logo.png"));
        assert!(is_open_graph_compatible_image_url(
            "https://cdn.example/event.jpg?v=1"
        ));
        assert!(!is_open_graph_compatible_image_url("/images/banner.svg"));
        assert!(!is_open_graph_compatible_image_url(
            "https://cdn.example/banner.SVG#preview"
        ));
        assert!(!is_open_graph_compatible_image_url(""));
        assert!(!is_open_graph_compatible_image_url("   "));
    }

    #[test]
    fn test_first_open_graph_image_url_skips_svg_and_empty() {
        assert_eq!(
            first_open_graph_image_url([
                Some("/images/banner.svg"),
                Some(""),
                Some("/images/event-logo.png"),
            ]),
            Some("/images/event-logo.png")
        );
        assert_eq!(
            first_open_graph_image_url([Some("/images/banner.svg"), Some("   ")]),
            None
        );
    }
}
