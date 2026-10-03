use std::sync::LazyLock;

use hypertext::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct ViteAssetValue {
    file: String,
}

#[derive(Deserialize)]
struct RawAssetsConfig {
    #[serde(rename = "src/main.ts")]
    main_js: ViteAssetValue,

    #[serde(rename = "style.css")]
    main_css: ViteAssetValue,
}

struct AssetsConfig {
    js_entrypoint: String,
    css_entrypoint: String,
}

static ASSET_CONFIG: LazyLock<Option<AssetsConfig>> = LazyLock::new(|| {
    let contents = std::fs::read_to_string("./dist/.vite/manifest.json")
        .inspect_err(|err| tracing::warn!("Failed to read vite manifest.json: {err}"))
        .ok()?;

    serde_json::from_str::<RawAssetsConfig>(&contents)
        .map(|raw| AssetsConfig {
            js_entrypoint: format!("/assets/{}", raw.main_js.file),
            css_entrypoint: format!("/assets/{}", raw.main_css.file),
        })
        .inspect_err(|err| tracing::warn!("Failed to deserialize vite manifest.json: {err}"))
        .ok()
});

pub fn layout<R: Renderable>(body: &R) -> impl Renderable {
    rsx! {
        <!DOCTYPE html>
        <head>
            <meta charset="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <link rel="icon" href="data:image/png;base64,iVBORw0KGgo=">
            @if let Some(assets_config) = &*ASSET_CONFIG {
                <link href=(assets_config.css_entrypoint) rel="stylesheet">
                <script src=(assets_config.js_entrypoint)></script>
            }
        </head>
        <body>
            (body)
        </body>
    }
}
