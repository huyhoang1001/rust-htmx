use html_node::{html, Node};
pub mod home;

fn layout(content: Node) -> Node {
    html! {
        <html>
            <head>
                <title> "Twitter clone in htmx" </title>
                <link
                    href="https://cdn.jsdelivr.net/npm/bootstrap@5.0.0-beta2/dist/css/bootstrap.min.css"
                    rel="stylesheet"
                    integrity="sha384-BmbxuPwQa2lc/FVzBcNJ7UAyJxM6wuqIj61tLrc4wSX0szH/Ev+nYRRuWlolflfl"
                    crossorigin="anonymous"
                />
                <script src="https://unpkg.com/htmx.org@1.9.12"></script>
                <script src="/lib/idiomorph-ext.min.js"></script>
                <script src="https://unpkg.com/htmx.org@1.9.12/dist/ext/sse.js"></script>
                <script src="https://unpkg.com/hyperscript.org@0.0.5"> </script>
            </head>
            {content}
        </html>
    }
}
