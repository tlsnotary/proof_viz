use std::fmt;

use yew::prelude::*;

pub const REDACTED_CHAR: char = '█';
// pub const REDACTED_CHAR: char = '🙈';

#[derive(Clone, PartialEq)]
pub enum Direction {
    Send,
    Received,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::Send => write!(f, "send"),
            Direction::Received => write!(f, "received"),
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub direction: Direction,
    pub bytes: String,
}

fn redactions_in_red(text: String) -> Html {
    //split the text in regular text parts and redacted parts
    let (mut parts, last_part) = text.chars().fold(
        (Vec::new(), String::new()),
        |(mut acc, mut current_part), c| {
            let previous_c = current_part.chars().last().unwrap_or(REDACTED_CHAR);
            if (c == REDACTED_CHAR) == (previous_c == REDACTED_CHAR) {
                current_part.push(c);
            } else {
                acc.push(current_part.clone());
                current_part.clear();
                current_part.push(c);
            }
            (acc, current_part)
        },
    );
    parts.push(last_part);

    //color redacted parts in red
    let html: Html = parts
        .iter()
        .map(|part| match part {
            x if x.starts_with(REDACTED_CHAR) => Html::from_html_unchecked(AttrValue::from(
                format!("<span style=\"color:red;\">{}</span>", x),
            )),
            _ => Html::from(part),
        })
        .collect();

    html
}

#[function_component]
pub fn RedactedBytesComponent(props: &Props) -> Html {
    let Props { direction, bytes } = props;

    let redacted_transcript = bytes.replace("\0", REDACTED_CHAR.to_string().as_str());

    html! {
        <details class="p-4 w-5/6" open={true}>
            <summary><b>{"Bytes "}{direction}{": " }</b></summary>
            <div class="bg-black text-white p-4 rounded-md overflow-x-auto">
                <pre>{redactions_in_red(redacted_transcript)}</pre>
            </div>
        </details>
    }
}
