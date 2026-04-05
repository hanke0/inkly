use axum::body::Body;
use axum::http::HeaderValue;
use axum::http::header::ACCEPT_LANGUAGE;
use axum::middleware::Next;
use axum::response::Response;

/// Resolved UI locale from [`ACCEPT_LANGUAGE`](axum::http::header::ACCEPT_LANGUAGE).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Locale {
    #[default]
    En,
    ZhHans,
}

impl Locale {
    /// Stable tag for API clients (e.g. `GET /v1/session`).
    pub fn as_api_tag(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::ZhHans => "zh-Hans",
        }
    }

    /// Pick the best supported locale from an `Accept-Language` header value.
    pub fn from_accept_language(header: Option<&HeaderValue>) -> Self {
        let Some(raw) = header.and_then(|h| h.to_str().ok()) else {
            return Self::default();
        };
        parse_accept_language(raw).unwrap_or_default()
    }
}

/// Inserts [`Locale`] into request extensions from `Accept-Language`.
pub async fn locale_middleware(mut req: axum::http::Request<Body>, next: Next) -> Response {
    let loc = Locale::from_accept_language(req.headers().get(ACCEPT_LANGUAGE));
    req.extensions_mut().insert(loc);
    next.run(req).await
}

#[derive(Clone, Copy, Debug)]
struct LangRange<'a> {
    tag: &'a str,
    quality: f32,
}

#[allow(clippy::collapsible_if)]
fn parse_accept_language(raw: &str) -> Option<Locale> {
    let mut ranges: Vec<LangRange<'_>> = Vec::new();
    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut tag = part;
        let mut quality = 1.0f32;
        if let Some((t, params)) = part.split_once(';') {
            tag = t.trim();
            for p in params.split(';') {
                let p = p.trim();
                let Some((k, v)) = p.split_once('=') else {
                    continue;
                };
                if k.trim().eq_ignore_ascii_case("q") {
                    if let Ok(q) = v.trim().parse::<f32>() {
                        quality = q.clamp(0.0, 1.0);
                    }
                }
            }
        }
        if tag.is_empty() {
            continue;
        }
        ranges.push(LangRange { tag, quality });
    }
    ranges.sort_by(|a, b| {
        b.quality
            .partial_cmp(&a.quality)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for r in ranges {
        if let Some(loc) = match_primary_tag(r.tag) {
            return Some(loc);
        }
    }
    None
}

fn match_primary_tag(tag: &str) -> Option<Locale> {
    let t = tag.trim().to_ascii_lowercase();
    let primary = t.split('-').next().unwrap_or("");
    match primary {
        "zh" => Some(Locale::ZhHans),
        "en" => Some(Locale::En),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_str(s: &str) -> Locale {
        HeaderValue::from_str(s)
            .map(|h| Locale::from_accept_language(Some(&h)))
            .unwrap_or_default()
    }

    #[test]
    fn empty_header_defaults_en() {
        assert_eq!(Locale::from_accept_language(None), Locale::En);
    }

    #[test]
    fn prefers_higher_quality() {
        assert_eq!(from_str("en;q=0.5, zh-CN;q=0.9"), Locale::ZhHans);
    }

    #[test]
    fn zh_cn_maps() {
        assert_eq!(from_str("zh-CN"), Locale::ZhHans);
        assert_eq!(from_str("zh"), Locale::ZhHans);
    }

    #[test]
    fn en_us_maps() {
        assert_eq!(from_str("en-US"), Locale::En);
    }
}
