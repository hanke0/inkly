//! User-visible API strings keyed by [`Msg`].
//!
//! Variants are grouped by module in the enum definition (`crate::auth`, `crate::routes`, `crate::error`).
//!
//! Use [`t`] for static copy and [`search_query_parse_detail`] for the search-parse template with a detail fragment.

use crate::locale::Locale;

/// Catalog entry; use with [`t`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Msg {
    // crate::auth
    SignInRequired,
    BasicScheme,
    CredentialsDecode,
    CredentialsMismatch,

    // crate::routes
    InvalidFolderPath,

    TagNonempty,
    TagNoControl,
    TagAlphanumeric,

    TitleRequired,

    MultipartReadFailed,
    MultipartMultipleFile,

    DocNeedsFilePart,
    UploadedFileUtf8,

    SearchTooManyTags,
    SearchNeedsCriteria,

    DocIdPositive,

    /// Summarization is disabled on the server (`SUMMARIZE_ENABLED` not set).
    SummarizeDisabled,
    /// Document summary is already scheduled; no need to request again.
    SummaryAlreadyQueued,
    /// Document summary has been queued for generation.
    SummaryQueued,

    SearchQueryParseEmpty,

    // crate::error
    NotFoundResource,
    InternalServer,
    InvalidRequestGeneric,
}

/// Localized static string for `msg`.
pub fn t(locale: Locale, msg: Msg) -> &'static str {
    match msg {
        Msg::SignInRequired => match locale {
            Locale::En => {
                "Sign-in required. Send an `Authorization: Basic …` header or open the app and sign in again."
            }
            Locale::ZhHans => {
                "需要登录。请携带 `Authorization: Basic …` 请求头，或在应用中重新登录。"
            }
        },
        Msg::BasicScheme => match locale {
            Locale::En => {
                "Authorization must use Basic authentication (`Authorization: Basic <base64>`). Check the header format."
            }
            Locale::ZhHans => {
                "Authorization 须使用 Basic 认证（`Authorization: Basic <base64>`）。请检查请求头格式。"
            }
        },
        Msg::CredentialsDecode => match locale {
            Locale::En => {
                "Credentials could not be decoded. Re-enter your username and password and try again."
            }
            Locale::ZhHans => "无法解析凭据。请重新输入用户名和密码后重试。",
        },
        Msg::CredentialsMismatch => match locale {
            Locale::En => {
                "Username or password did not match the server configuration. Check your credentials and try again."
            }
            Locale::ZhHans => "用户名或密码与服务器配置不一致。请核对凭据后重试。",
        },

        Msg::InvalidFolderPath => match locale {
            Locale::En => {
                "The path must be a workspace folder path: use `/` for the root, or `/notes/` or `/team/docs/` for subfolders (always start with `/`, separate segments with `/`, end non-root paths with `/`). Do not use `..` or try to escape the workspace. Fix the path and try again."
            }
            Locale::ZhHans => {
                "路径须为工作区内的文件夹路径：根目录用 `/`；子文件夹用 `/笔记/`、`/项目/文档/` 等形式（必须以 `/` 开头，各段用 `/` 分隔，非根路径须以 `/` 结尾）。不要使用 `..` 或试图跳出工作区。请修正路径后重试。"
            }
        },

        Msg::TagNonempty => match locale {
            Locale::En => "Each tag must be non-empty after trimming spaces.",
            Locale::ZhHans => "每个标签在去掉首尾空格后不能为空。",
        },
        Msg::TagNoControl => match locale {
            Locale::En => {
                "Tags cannot contain control characters. Use letters, numbers, and underscores."
            }
            Locale::ZhHans => "标签不能包含控制字符。请使用字母、数字和下划线。",
        },
        Msg::TagAlphanumeric => match locale {
            Locale::En => "Tags may only contain letters, numbers, and underscores.",
            Locale::ZhHans => "标签只能包含字母、数字和下划线。",
        },

        Msg::TitleRequired => match locale {
            Locale::En => "Title is required. Enter a non-empty title and try again.",
            Locale::ZhHans => "标题为必填项。请输入非空标题后重试。",
        },

        Msg::MultipartReadFailed => match locale {
            Locale::En => {
                "Could not read the multipart form. Retry the upload and ensure the request is multipart/form-data."
            }
            Locale::ZhHans => {
                "无法读取 multipart 表单。请重试上传，并确保请求为 multipart/form-data。"
            }
        },
        Msg::MultipartMultipleFile => match locale {
            Locale::En => "The form contains more than one `file` field. Send a single file part.",
            Locale::ZhHans => "表单中包含多个 `file` 字段。请只发送一个文件部分。",
        },
        Msg::DocNeedsFilePart => match locale {
            Locale::En => {
                "This request needs a `file` part in the multipart body. Add the file field and try again."
            }
            Locale::ZhHans => {
                "该请求需要在 multipart 正文中提供 `file` 部分。请添加文件字段后重试。"
            }
        },
        Msg::UploadedFileUtf8 => match locale {
            Locale::En => {
                "The uploaded file must be valid UTF-8 text or HTML. Convert the encoding and try again."
            }
            Locale::ZhHans => "上传的文件须为有效的 UTF-8 文本或 HTML。请转换编码后重试。",
        },

        Msg::SearchTooManyTags => match locale {
            Locale::En => {
                "Too many tag filters in this search. Remove some tag filters (comma-separated) and try again."
            }
            Locale::ZhHans => "本次搜索的标签过滤过多。请减少部分标签（逗号分隔）后重试。",
        },
        Msg::SearchNeedsCriteria => match locale {
            Locale::En => {
                "Enter search text and/or pick a folder path or tag filter. At least one of these is required."
            }
            Locale::ZhHans => "请输入搜索词，和/或选择文件夹路径或标签过滤。至少需要其中一项。",
        },

        Msg::DocIdPositive => match locale {
            Locale::En => {
                "Document ID must be a positive number. Use the ID shown in search or the catalog."
            }
            Locale::ZhHans => "文档 ID 必须为正整数。请使用搜索或目录中显示的 ID。",
        },

        Msg::SummarizeDisabled => match locale {
            Locale::En => {
                "Automatic summarization is disabled on this server. Ask an administrator to enable `SUMMARIZE_ENABLED` if you need summaries."
            }
            Locale::ZhHans => {
                "此服务器未开启自动摘要。如需摘要，请联系管理员启用 `SUMMARIZE_ENABLED`。"
            }
        },
        Msg::SummaryAlreadyQueued => match locale {
            Locale::En => {
                "A summary for this document is already in the processing queue. Please wait; there is no need to request it again."
            }
            Locale::ZhHans => "该文档的摘要已在处理队列中，请稍候，无需重复触发。",
        },
        Msg::SummaryQueued => match locale {
            Locale::En => {
                "Summary generation has been queued. It will appear on the document when processing finishes."
            }
            Locale::ZhHans => "已加入摘要生成队列，处理完成后将显示在文档中。",
        },

        Msg::SearchQueryParseEmpty => match locale {
            Locale::En => "Search query could not be parsed. Simplify the query and try again.",
            Locale::ZhHans => "无法解析搜索查询。请简化查询后重试。",
        },

        Msg::NotFoundResource => match locale {
            Locale::En => {
                "The requested resource was not found. Check the document ID or path and try again."
            }
            Locale::ZhHans => "未找到请求的资源。请核对文档 ID 或路径后重试。",
        },
        Msg::InternalServer => match locale {
            Locale::En => {
                "Something went wrong on the server. Please try again in a moment; if the problem continues, contact support."
            }
            Locale::ZhHans => "服务器出现问题。请稍后重试；若问题持续，请联系支持。",
        },
        Msg::InvalidRequestGeneric => match locale {
            Locale::En => "The request could not be processed. Check your input and try again.",
            Locale::ZhHans => "无法处理该请求。请检查输入后重试。",
        },
    }
}

/// Search query parse error with engine detail text (not a single [`Msg`] variant for the templated line).
pub fn search_query_parse_detail(locale: Locale, detail: &str) -> String {
    let d = detail.trim();
    if d.is_empty() {
        return t(locale, Msg::SearchQueryParseEmpty).to_string();
    }
    match locale {
        Locale::En => {
            format!("Search query could not be parsed ({d}). Adjust the syntax and try again.")
        }
        Locale::ZhHans => format!("无法解析搜索查询（{d}）。请调整语法后重试。"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_non_empty_for_both_locales() {
        for msg in [
            Msg::SignInRequired,
            Msg::InternalServer,
            Msg::SearchQueryParseEmpty,
            Msg::SummarizeDisabled,
            Msg::SummaryAlreadyQueued,
            Msg::SummaryQueued,
        ] {
            assert!(!t(Locale::En, msg).is_empty());
            assert!(!t(Locale::ZhHans, msg).is_empty());
        }
    }
}
