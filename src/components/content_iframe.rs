use std::fmt;

use gloo::console::log;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub bytes: String,
}

fn render(content: String) -> Html {
    html! {
        <iframe class="w-full h-64" srcdoc={content} src="demo_iframe_srcdoc.htm">
            <p>{">Your browser does not support iframes."}</p>
        </iframe>
    }
}

#[function_component]
pub fn ContentIFrame(props: &Props) -> Html {
    let content = format!("{}", &props.bytes);

    // Content-Type: text/html
    let start_html = content.find("<html");
    let end_html = content.find("/html>");

    let frame = if start_html.is_some() && end_html.is_some() {
        let html_content = content[start_html.unwrap()..end_html.unwrap() + 5].to_string();
        log!("html: {}", html_content.clone());
        render(html_content)
    } else {
        // Content-Type: application/json

        let start_html = content.find("<html");
        let end_html = content.find("/html>");

        render(content)
    };

    html! {
        <details class="p-4 w-5/6" open={true}>
            {frame}
        </details>
    }
}
