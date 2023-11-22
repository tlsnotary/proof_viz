use std::{fmt, ops::Range};

use gloo::console::log;
use yew::prelude::*;

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
    pub redacted_char: char,
    pub bytes: Vec<u8>,
    pub redacted_ranges: Vec<Range<usize>>,
}

fn get_redacted_string(redacted_char: &char, size: usize) -> String {
    if true || size < 10 {
        redacted_char.to_string().repeat(size)
    } else {
        format! {"{}...{}", redacted_char.to_string().repeat(3), redacted_char.to_string().repeat(3)}
    }
}

fn redactions_in_red(
    bytes: &Vec<u8>,
    redacted_ranges: &Vec<Range<usize>>,
    redacted_char: &char,
) -> Html {
    if redacted_ranges.is_empty() {
        return Html::from(String::from_utf8(bytes.to_vec()).unwrap());
    }

    // create ranges for non redacted parts and store last redacted position
    let (non_redacted_ranges, last_redacted_position) = redacted_ranges.iter().fold(
        (Vec::new(), 0), // (Accumulator vector, last redacted position)
        |(mut acc, last_end), range| {
            acc.push(Range {
                start: last_end,
                end: range.start,
            });
            (acc, range.end)
        },
    );

    // interweave the redacted and non-redacted ranges
    let all_ranges = non_redacted_ranges
        .into_iter()
        .zip(redacted_ranges.iter())
        .flat_map(|(non_redacted, redacted)| {
            vec![
                (non_redacted.start, non_redacted.end, false),
                (redacted.start, redacted.end, true),
            ]
        })
        .chain(std::iter::once((
            last_redacted_position,
            bytes.len(),
            false,
        ))); // Handle remaining non-redacted part

    let html_nodes = all_ranges
        .map(|(start, end, is_redacted)| {
            if is_redacted {
                Html::from_html_unchecked(AttrValue::from(format!(
                    "<span style=\"color:red;\">{}</span>",
                    get_redacted_string(redacted_char, end - start)
                )))
            } else {
                Html::from(String::from_utf8_lossy(&bytes[start..end]))
            }
        })
        .collect::<Vec<_>>();

    html! {
        <>
            { for html_nodes }
        </>
    }
}

#[function_component]
pub fn RedactedBytesComponent(props: &Props) -> Html {
    let Props {
        direction,
        redacted_char,
        bytes,
        redacted_ranges,
    } = props;

    html! {
        <details class="p-4 w-5/6" open={true}>
            <summary><b>{"Bytes "}{direction}{": " }</b></summary>
            <div class="bg-black text-white p-4 rounded-md overflow-x-auto">
                <pre>{redactions_in_red(bytes, redacted_ranges, redacted_char)}</pre>
            </div>
        </details>
    }
}
