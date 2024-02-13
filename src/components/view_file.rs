extern crate base64;
use std::ops::Range;
use std::str;
use web_time::Duration;

use yew::{function_component, html, Html, Properties};

use tlsn_core::proof::{SessionProof, TlsProof};

use crate::components::content_iframe::ContentIFrame;
use crate::components::redacted_bytes_component::Direction;
use crate::components::redacted_bytes_component::RedactedBytesComponent;

const REDACTED_CHAR: char = 'X'; // '█' '🙈' 'X'

#[derive(Properties, PartialEq)]
pub struct Props {
    pub name: String,
    pub file_type: String,
    pub data: Vec<u8>,
    pub pem: p256::PublicKey,
}

#[function_component]
pub fn ViewFile(props: &Props) -> Html {
    // Verify the session proof against the Notary's public key
    fn verify_proof(session: &SessionProof, pem: p256::PublicKey) -> Result<(), String> {
        session
            .verify_with_default_cert_verifier(pem)
            .map_err(|err| err.to_string())
    }

    fn parse_tls_proof(json_str: &str, pem: p256::PublicKey) -> Html {
        let tls_proof: Result<TlsProof, serde_json::Error> = serde_json::from_str(json_str);

        match tls_proof {
            Err(e) => html! {
                 <div>{format!("Parsing failed {}", e.to_string())}</div>
            },
            Ok(tls_proof) => {
                let TlsProof {
                    // The session proof establishes the identity of the server and the commitments
                    // to the TLS transcript.
                    session,
                    // The substrings proof proves select portions of the transcript, while redacting
                    // anything the Prover chose not to disclose.
                    substrings,
                } = tls_proof;

                let proof_verification = verify_proof(&session, pem);

                if proof_verification.is_err() {
                    return html! {
                        <>
                            <div role="alert">
                                <div class="bg-red-500 text-white font-bold rounded-t px-4 py-2">
                                    {"Invalid Proof"}
                                </div>
                                <div class="border border-t-0 border-red-400 rounded-b bg-red-100 px-4 py-3 text-red-700">
                                    { "❌ " }{proof_verification.unwrap_err().to_string()}
                                </div>
                            </div>
                        </>
                    };
                }

                let proof_verification_feedback = "✅ Proof successfully verified ✅".to_string();

                let SessionProof {
                    // The session header that was signed by the Notary is a succinct commitment to the TLS transcript.
                    header,
                    // This is the server name, checked against the certificate chain shared in the TLS handshake.
                    session_info,
                    ..
                } = session;

                // The time at which the session was recorded
                let time = chrono::DateTime::UNIX_EPOCH + Duration::from_secs(header.time());

                // Verify the substrings proof against the session header.
                let substring_verification_result = substrings.verify(&header);

                if substring_verification_result.is_err() {
                    return html! {
                        <>
                            <div role="alert">
                                <div class="bg-red-500 text-white font-bold rounded-t px-4 py-2">
                                    {"Invalid Proof"}
                                </div>
                                <div class="border border-t-0 border-red-400 rounded-b bg-red-100 px-4 py-3 text-red-700">
                                    { "❌ " }{substring_verification_result.unwrap_err().to_string()}
                                </div>
                            </div>
                        </>
                    };
                }

                // This returns the redacted transcripts
                let (mut sent, mut recv) = substring_verification_result.unwrap();

                // Replace the bytes which the Prover chose not to disclose with 'X'
                sent.set_redacted(b'X');
                recv.set_redacted(b'X');

                let redacted_ranges_send: Vec<Range<usize>> =
                    sent.redacted().clone().iter_ranges().collect();
                let redacted_ranges_recv: Vec<Range<usize>> =
                    recv.redacted().clone().iter_ranges().collect();

                html! {
                    <div class="p-4 flex flex-col justify-center items-center w-full">
                        <div class="p-4 w-5/6">
                            <b>{"Server domain:" }</b>
                            <div class="bg-black text-white p-4 rounded-md">
                                <pre>{session_info.server_name.as_str().to_string()}</pre>
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

                        <RedactedBytesComponent direction={Direction::Send} redacted_char={REDACTED_CHAR} bytes={sent.data().to_vec()} redacted_ranges={redacted_ranges_send} />

                        <ContentIFrame bytes={recv.data().to_vec()} />

                        <RedactedBytesComponent direction={Direction::Received} redacted_char={REDACTED_CHAR} bytes={recv.data().to_vec()} redacted_ranges={redacted_ranges_recv} />

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
