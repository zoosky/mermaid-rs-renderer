use std::path::{Path, PathBuf};

use mermaid_rs_renderer::layout::validate_layout_invariants;
use mermaid_rs_renderer::{LayoutConfig, Theme, compute_layout, parse_mermaid, render_svg};

fn collect_fixtures(root: &Path) -> Vec<PathBuf> {
    let mut fixtures = Vec::new();
    collect_fixtures_recursive(root, &mut fixtures);
    fixtures.sort();
    fixtures
}

fn collect_fixtures_recursive(dir: &Path, fixtures: &mut Vec<PathBuf>) {
    for entry in
        std::fs::read_dir(dir).unwrap_or_else(|err| panic!("read_dir {}: {err}", dir.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("read_dir entry {}: {err}", dir.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_fixtures_recursive(&path, fixtures);
        } else if path.extension().is_some_and(|ext| ext == "mmd") {
            fixtures.push(path);
        }
    }
}

#[test]
fn all_repository_fixtures_satisfy_layout_invariants() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut fixtures = collect_fixtures(&manifest.join("tests/fixtures"));
    fixtures.extend(collect_fixtures(&manifest.join("benches/fixtures")));
    fixtures.sort();

    let theme = Theme::modern();
    let config = LayoutConfig::default();
    let mut failures = Vec::new();

    for path in fixtures {
        let rel = path
            .strip_prefix(manifest)
            .unwrap_or(&path)
            .display()
            .to_string();
        let input = match std::fs::read_to_string(&path) {
            Ok(input) => input,
            Err(err) => {
                failures.push(format!("{rel}: read failed: {err}"));
                continue;
            }
        };
        let parsed = match parse_mermaid(&input) {
            Ok(parsed) => parsed,
            Err(err) => {
                failures.push(format!("{rel}: parse failed: {err}"));
                continue;
            }
        };
        let layout = compute_layout(&parsed.graph, &theme, &config);
        if let Err(errors) = validate_layout_invariants(&layout) {
            failures.push(format!(
                "{rel}: layout invariant violations:\n{}",
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
            continue;
        }
        let svg = render_svg(&layout, &theme, &config);
        if !svg.contains("<svg")
            || !svg.contains("</svg>")
            || svg.contains("NaN")
            || svg.contains("inf")
        {
            failures.push(format!("{rel}: invalid SVG output"));
        }
    }

    assert!(
        failures.is_empty(),
        "fixture invariant failures ({}):\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}
