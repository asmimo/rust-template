use hypertext::prelude::*;

pub fn layout<R: Renderable>(body: &R) -> impl Renderable {
    rsx! {
        <!DOCTYPE html>
        <head>
            <meta charset="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <link rel="icon" href="data:image/png;base64,iVBORw0KGgo=">
            <link href="/assets/output.css" rel="stylesheet">
            <script src="/assets/main.js"></script>
        </head>
        <body>
            (body)
        </body>
    }
}
