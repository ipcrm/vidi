use crate::error::AppResult;
use crate::markdown::{render, RenderInput};
use crate::model::{LinkResolution, RenderOptions, RenderedDoc, ResolvedBase, Source};
use crate::sources::{github, local, remote};
use crate::AppState;

#[tauri::command(rename_all = "camelCase")]
pub async fn render_markdown(
    state: tauri::State<'_, AppState>,
    source: Source,
    options: Option<RenderOptions>,
) -> AppResult<RenderedDoc> {
    let options = options.unwrap_or_default();

    let (text, base) = match source {
        Source::LocalFile { path } => {
            let text = local::read_markdown_file(&path)?;
            let root = path.parent().unwrap_or(&path).to_path_buf();
            (text, ResolvedBase::Folder { file: path, root })
        }
        Source::LocalFolder { .. } => {
            return Err(crate::error::AppError::InvalidArgument(
                "cannot render a folder; open a file inside it".into(),
            ));
        }
        Source::Remote { url } => {
            let normalized = github::normalize(&url);
            let fetched = remote::fetch(&state.http, &normalized.fetch_url).await?;
            (
                fetched.text,
                ResolvedBase::Remote {
                    base_url: normalized.base_url,
                },
            )
        }
    };

    Ok(render(RenderInput {
        text: &text,
        base,
        options,
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn render_markdown_inline(
    text: String,
    base_url: Option<String>,
) -> AppResult<RenderedDoc> {
    let base = match base_url {
        Some(url) => ResolvedBase::Remote { base_url: url },
        None => ResolvedBase::Inline,
    };
    Ok(render(RenderInput {
        text: &text,
        base,
        options: RenderOptions::default(),
    }))
}

#[tauri::command(rename_all = "camelCase")]
pub async fn resolve_link(href: String, base: ResolvedBase) -> AppResult<LinkResolution> {
    Ok(crate::markdown::links::resolve(&base, &href))
}
