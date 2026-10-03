use std::sync::LazyLock;

use hypertext::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ViteAssetValue {
    pub file: String,
}

#[derive(Debug, Deserialize)]
struct RawAssetsConfig {
    #[serde(rename = "src/main.ts")]
    pub main_js: ViteAssetValue,

    #[serde(rename = "style.css")]
    pub main_css: ViteAssetValue,
}

#[derive(Debug)]
struct AssetsConfig {
    pub js_entrypoint: String,
    pub css_entrypoint: String,
}

static ASSET_CONFIG: LazyLock<Option<AssetsConfig>> = LazyLock::new(|| {
    const RAW_JSON: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../dist/.vite/manifest.json"
    ));

    serde_json::from_str::<RawAssetsConfig>(RAW_JSON)
        .map(|raw| AssetsConfig {
            js_entrypoint: format!("/assets/{}", raw.main_js.file),
            css_entrypoint: format!("/assets/{}", raw.main_css.file),
        })
        .inspect_err(|err| tracing::warn!("Failed to deserialize vite mainifest.json: {err}"))
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
