extern crate base64;
use k256::{pkcs8::DecodePublicKey, PublicKey};
use std::ops::Range;
use std::str;
use tlsn_core::presentation::PresentationOutput;
use tlsn_core::{presentation::Presentation, CryptoProvider};
use web_time::Duration;

use yew::{function_component, html, Html, Properties};

use crate::components::content_iframe::ContentIFrame;
use crate::components::redacted_bytes_component::Direction;
use crate::components::redacted_bytes_component::RedactedBytesComponent;

const REDACTED_CHAR: char = 'X'; // '█' '🙈' 'X'

#[derive(Properties, PartialEq)]
pub struct Props {
    pub name: String,
    pub file_type: String,
    pub data: Vec<u8>,
    pub pem: k256::PublicKey,
}

#[function_component]
pub fn ViewFile(props: &Props) -> Html {
    // Verify the session proof against the Notary's public key
    fn verify_proof(
        presentation: Presentation,
        pem: &PublicKey,
    ) -> Result<PresentationOutput, String> {
        let provider = CryptoProvider::default();

        let presentation_output = presentation
            .verify(&provider)
            .map_err(|e| format!("Verification failed: {}", e))?;

        let verifying_key = presentation_output
            .attestation
            .body
            .verifying_key()
            .data
            .clone();

        let bar = verifying_key == *pem.to_sec1_bytes();

        Ok(presentation_output)
    }

    fn parse_tls_proof(json_str: &str, pem: k256::PublicKey) -> Html {
        let presentation: Result<Presentation, serde_json::Error> = serde_json::from_str(json_str);

        match presentation {
            Err(e) => html! {
                 <div>{format!("Parsing failed {}", e.to_string())}</div>
            },
            Ok(presentation) => {
                let presentation_output = verify_proof(presentation, &pem);

                if presentation_output.is_err() {
                    return html! {
                        <>
                            <div role="alert">
                                <div class="bg-red-500 text-white font-bold rounded-t px-4 py-2">
                                    {"Invalid Proof"}
                                </div>
                                <div class="border border-t-0 border-red-400 rounded-b bg-red-100 px-4 py-3 text-red-700">
                                    { "❌ " }{presentation_output.unwrap_err().to_string()}
                                </div>
                            </div>
                        </>
                    };
                }

                let presentation_output = presentation_output.unwrap();

                let proof_verification_feedback = "✅ Proof successfully verified ✅".to_string();

                let connection_info = presentation_output.connection_info;

                // The time at which the session was recorded
                let time = chrono::DateTime::UNIX_EPOCH + Duration::from_secs(connection_info.time);

                let mut partial_transcript = presentation_output.transcript.unwrap();
                partial_transcript.set_unauthed(b'X');

                let sent = partial_transcript.sent_unsafe();
                let recv = partial_transcript.received_unsafe();

                let redacted_ranges_sent: Vec<Range<usize>> =
                    partial_transcript.sent_unauthed().iter_ranges().collect();
                let redacted_ranges_recv: Vec<Range<usize>> = partial_transcript
                    .received_unauthed()
                    .iter_ranges()
                    .collect();

                let server_name = presentation_output.server_name;
                let server_name = server_name.unwrap().clone();

                html! {
                    <div class="p-4 flex flex-col justify-center items-center w-full">
                        <div class="p-4 w-5/6">
                            <b>{"Server domain:" }</b>
                            <div class="bg-black text-white p-4 rounded-md">
                                <pre>{server_name.as_str()}</pre>
                            </div>
                            <b>{"Notarization time:" }</b>
                            <div class="bg-black text-white p-4 rounded-md">
                                <pre>{time.to_string()}</pre>
                            </div>
                            <b>{"Proof:" }</b>
                            <div class="bg-black text-white p-4 rounded-md">
                                <pre>{proof_verification_feedback}</pre>
                            </div>
                        </div>

                        <RedactedBytesComponent direction={Direction::Sent} redacted_char={REDACTED_CHAR} bytes={sent.to_vec()} redacted_ranges={redacted_ranges_sent} />

                        <ContentIFrame bytes={recv.to_vec()} />

                        <RedactedBytesComponent direction={Direction::Received} redacted_char={REDACTED_CHAR} bytes={recv.to_vec()} redacted_ranges={redacted_ranges_recv} />

                    </div>
                }
            }
        }
    }

    let json_str = str::from_utf8(&props.data).unwrap();
    html! {
            <div class="p-4 flex flex-col justify-center items-center bg-zinc-700 border border-white border-dashed rounded-2xl">
                <p class="text-center">{ format!("{}", &props.name) }</p>

                <div class="flex-1 flex flex-col justify-center p-4">
                    <div class="container mx-auto px-4">
                    if props.file_type.contains("application/json") {
                        {parse_tls_proof(json_str, props.pem)}
                    }
                    </div>
                </div>
            </div>
    }
}
