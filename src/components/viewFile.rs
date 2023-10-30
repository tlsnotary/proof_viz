extern crate base64;
use elliptic_curve::pkcs8::DecodePublicKey;
use std::str;
use web_time::Duration;

use yew::{function_component, html, Html, Properties};

use tlsn_core::proof::{SessionProof, TlsProof};

use crate::components::redactedBytesComponent::Direction;
use crate::components::redactedBytesComponent::RedactedBytesComponent;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub name: String,
    pub file_type: String,
    pub data: Vec<u8>,
}

#[function_component]
pub fn ViewFile(props: &Props) -> Html {
    fn notary_pubkey() -> p256::PublicKey {
        // from https://github.com/tlsnotary/notary-server/tree/main/src/fixture/notary/notary.key
        // converted with `openssl ec -in notary.key -pubout -outform PEM`

        let pem = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEBv36FI4ZFszJa0DQFJ3wWCXvVLFr
cRzMG5kaTeHGoSzDu6cFqx3uEWYpFGo6C0EOUgf+mEgbktLrXocv5yHzKg==
-----END PUBLIC KEY-----";

        p256::PublicKey::from_public_key_pem(pem).unwrap()
    }

    // Verify the session proof against the Notary's public key
    fn verify_proof(session: &SessionProof) -> Result<(), String> {
        // This verifies the identity of the server using a default certificate verifier which trusts
        // the root certificates from the `webpki-roots` crate.

        session
            .verify_with_default_cert_verifier(notary_pubkey())
            .map_err(|err| err.to_string())
    }

    fn parse_tls_proof(json_str: &str) -> Html {
        let tls_proof: Result<TlsProof, serde_json::Error> = serde_json::from_str(json_str);

        // info!("Parsing");

        match tls_proof {
            Ok(tls_proof) => {
                let TlsProof {
                    // The session proof establishes the identity of the server and the commitments
                    // to the TLS transcript.
                    session,
                    // The substrings proof proves select portions of the transcript, while redacting
                    // anything the Prover chose not to disclose.
                    substrings,
                } = tls_proof;

                let proof_verification = verify_proof(&session);

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
                    server_name,
                    ..
                } = session;

                // The time at which the session was recorded
                let time = chrono::DateTime::UNIX_EPOCH + Duration::from_secs(header.time());

                // Verify the substrings proof against the session header.

                // This returns the redacted transcripts
                let (mut sent, mut recv) = substrings.verify(&header).unwrap();

                // Replace the bytes which the Prover chose not to disclose with '\0'
                sent.set_redacted(b'\0');
                recv.set_redacted(b'\0');

                let bytes_send = String::from_utf8(sent.data().to_vec()).unwrap();

                let bytes_received = String::from_utf8(recv.data().to_vec()).unwrap();

                html! {
                    <div class="p-4 flex flex-col justify-center items-center w-full">
                        <div class="p-4 w-5/6">
                            <b>{"Server domain:" }</b>
                            <div class="bg-black text-white p-4 rounded-md">
                                <pre>{server_name.as_str().to_string()}</pre>
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

                        <RedactedBytesComponent direction={Direction::Send} bytes={bytes_send} />

                        <RedactedBytesComponent direction={Direction::Received} bytes={bytes_received} />
                    </div>
                }
            }
            Err(e) => html! {
                 <div>{format!("Parsing failed {}", e.to_string())}</div>
            },
        }
    }

    let json_str = str::from_utf8(&props.data).unwrap();
    html! {
            <div class="p-4 flex flex-col justify-center items-center bg-zinc-700 border border-white border-dashed rounded-2xl">
                <p class="text-center">{ format!("{}", &props.name) }</p>

                <div class="flex-1 flex flex-col justify-center p-4">
                    <div class="container mx-auto px-4">
                    if props.file_type.contains("application/json") {
                        {parse_tls_proof(json_str)}
                    }
                    </div>
                </div>
            </div>
    }
}
