mod apply;
mod files;
mod status;
use apply::import_batch;
pub(crate) use apply::{ImportFailure, ImportOutcome};
pub use status::ImportStatus;
#[cfg(test)]
mod status_tests;

use crate::error::EngramError;
use crate::portability::markdown::{paths::validate_export_dir, MarkdownContext};

pub(crate) struct ImportRequest<'a> {
    pub input_dir: &'a str,
    pub workspace: Option<&'a str>,
    pub force_version: bool,
    pub confirm: bool,
}

pub(crate) fn import(
    ctx: &MarkdownContext<'_>,
    request: ImportRequest<'_>,
) -> Result<ImportOutcome, ImportFailure> {
    let root = validate_export_dir(request.input_dir).map_err(EngramError::InvalidInput)?;
    let sources = files::read_markdown(&root, ctx.storage)
        .map_err(|error| EngramError::InvalidInput(format!("Markdown input refused: {error}")))?;
    import_batch(
        ctx,
        sources,
        request.workspace,
        request.force_version,
        request.confirm,
    )
}
