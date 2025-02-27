use thiserror::Error;

#[derive(Error, Debug)]
pub enum SvgearError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("SVG error: {0}")]
    SvgError(#[from] resvg::usvg::Error),
    // Add more error types as needed
}
