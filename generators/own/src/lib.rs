//! uxui-gen: the smallest own generator. It reads an `ess-ui/1` document, its fixtures and the
//! ESS model it names, and writes one page as static HTML and plain JS that plays the fixture
//! live scripts. No third-party crates: the YAML subset reader and the JSON writer are this
//! repository's own (crate `uxui-doc`).

pub mod binding;
pub mod emit;
pub mod fixtures;
pub mod model;
pub mod resolve;

pub use uxui_doc::{json, yaml};

use std::path::Path;

/// What one run wrote and found.
#[derive(Debug)]
pub struct Report {
    pub files: Vec<String>,
    pub warnings: Vec<String>,
}

/// Generates `page` of the document at `doc_path`, bound to the model in `model_dir`, into `out`.
/// `themes`, a uxui theme's files, are copied to `theme/`: a stylesheet is loaded after the page's own,
/// a script (the light and dark mode toggle) in the head, before the body is drawn.
pub fn generate(doc_path: &Path, model_dir: &Path, page: &str, out: &Path, themes: &[&Path]) -> Result<Report, String> {
    let text = std::fs::read_to_string(doc_path).map_err(|e| format!("{}: {e}", doc_path.display()))?;
    let doc = yaml::parse(&text).map_err(|e| format!("{}: {e}", doc_path.display()))?;
    if doc.get_str("format") != Some("ess-ui/1") {
        return Err(format!("{}: not an ess-ui/1 document", doc_path.display()));
    }
    let model = model::load(model_dir)?;
    let mut resolved = resolve::page(&doc, page)?;
    typed(&mut resolved, &model);
    let findings = binding::check(&resolved, &doc, &model);
    if !findings.errors.is_empty() {
        return Err(findings.errors.join("\n"));
    }
    let doc_dir = doc_path.parent().unwrap_or(Path::new("."));
    let fixtures = fixtures::load(&doc, doc_dir)?;
    let channels = resolve::channels(&doc);
    std::fs::create_dir_all(out.join("theme")).map_err(|e| format!("{}: {e}", out.display()))?;
    let mut hrefs = Vec::new();
    for theme in themes {
        let name = theme.file_name().and_then(|n| n.to_str()).ok_or(format!("{}: no file name", theme.display()))?;
        std::fs::copy(theme, out.join("theme").join(name)).map_err(|e| format!("{}: {e}", theme.display()))?;
        hrefs.push(format!("theme/{name}"));
    }
    let files = [
        ("index.html", emit::index_html(&doc, &resolved, &hrefs)),
        ("page.js", emit::page_js(&resolved, &channels, &fixtures)),
        ("runtime.js", emit::RUNTIME.to_string()),
        ("style.css", format!("{}{}", emit::tokens_css(&doc), emit::STYLE)),
    ];
    let mut written = Vec::new();
    for (name, body) in files {
        let path = out.join(name);
        std::fs::write(&path, body).map_err(|e| format!("{}: {e}", path.display()))?;
        written.push(name.to_string());
    }
    written.extend(hrefs);
    Ok(Report { files: written, warnings: findings.warnings })
}

/// Each field of a section that reads a view gets the view field's model type (`type`), so the
/// runtime can draw a value by what it is, such as a list of KPI readings (change C3).
fn typed(page: &mut yaml::Value, model: &model::Model) {
    use yaml::Value;
    let Value::Map(entries) = page else { return };
    let Some((_, Value::List(sections))) = entries.iter_mut().find(|(k, _)| k == "sections") else { return };
    for section in sections {
        let view = section.get("reads").and_then(|r| r.get_str("view")).and_then(|v| model.views.get(v));
        let (Some(view), Value::Map(fields)) = (view, section) else { continue };
        let Some((_, Value::List(fields))) = fields.iter_mut().find(|(k, _)| k == "fields") else { continue };
        for field in fields {
            let ty = field.get_str("field").and_then(|f| view.field(f)).map(|f| f.ty.clone());
            if let (Some(ty), Value::Map(entries)) = (ty, field) {
                entries.push(("type".to_string(), Value::Str(ty)));
            }
        }
    }
}
