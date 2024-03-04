// use gloo::console::log;
// use std::fmt;

use spansy::http::parse_response;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub bytes: Vec<u8>,
}

fn render_json(content: String) -> String {
    let json = serde_json::from_str::<serde_json::Value>(content.as_str());
    if let Ok(json) = json {
        serde_json::to_string_pretty(&json).unwrap()
    } else {
        content
    }
}

#[derive(Debug)]
enum ContentType {
    Html,
    Json,
    Other,
}
fn get_content_type(bytes: &[u8]) -> (ContentType, String) {
    match parse_response(bytes) {
        Ok(x) => {
            // log!(format!("Test {:?}", x.headers));

            let content_type = x
                .headers
                .iter()
                .find(|h| h.name.as_str().to_lowercase() == "content-type")
                .map_or(ContentType::Other, |header| {
                    let type_string = String::from_utf8_lossy(header.value.as_bytes());
                    match type_string {
                        s if s.contains("text/html") => ContentType::Html,
                        s if s.contains("application/json") => ContentType::Json,
                        _ => ContentType::Other,
                    }
                });

            let body = x.body.map_or(String::new(), |body| {
                String::from_utf8_lossy(body.as_bytes()).to_string()
            });

            // log!(format!("Test {:?}", content_type));

            (content_type, body)
        }
        Err(e) => (ContentType::Other, e.to_string()),
    }
}

#[function_component]
pub fn ContentIFrame(props: &Props) -> Html {
    // JavaScript function to trigger Prism highlighting
    use_effect(highlight_code);

    match get_content_type(&props.bytes) {
        (ContentType::Html, content_html) => html! {
            <details class="p-4 w-5/6" open={true}>
                <summary><b>{"Received HTML content:"}</b></summary>
                <iframe class="w-full h-64" srcdoc={content_html} src="demo_iframe_srcdoc.htm">
                    <p>{">Your browser does not support iframes."}</p>
                </iframe>
            </details>
        },
        (ContentType::Json, content_json) => html! {
            <details class="p-4 w-5/6" open={true}>
                <summary><b>{"Received JSON content:"}</b></summary>
                <div class="bg-black text-white p-4 rounded-md overflow-x-auto">
                    <pre>
                        <code class="lang-json">
                            {render_json(content_json)}
                        </code>
                    </pre>
                </div>
            </details>
        },
        _ => html! {},
    }
}

#[wasm_bindgen(inline_js = "export function highlight_code() { Prism.highlightAll(); }")]
extern "C" {
    fn highlight_code();
}
