extern crate base64;
use elliptic_curve::pkcs8::DecodePublicKey;
use gloo::file::callbacks::FileReader;
use gloo::file::File;
use std::collections::HashMap;
use std::str;
use web_time::Duration;

use web_sys::{DragEvent, Event, FileList, HtmlInputElement};
use yew::html::TargetCast;
use yew::{html, Callback, Component, Context, Html};
use yew_icons::{Icon, IconId};

use tlsn_core::proof::{SessionProof, TlsProof};
use tlsn_core::NotarizedSession;

const REDACTED_CHAR: &str = "█";

struct FileDetails {
    name: String,
    file_type: String,
    data: Vec<u8>,
}

pub enum Msg {
    Loaded(String, String, Vec<u8>),
    Files(Vec<File>),
}

pub struct App {
    readers: HashMap<String, FileReader>,
    files: Vec<FileDetails>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
            files: Vec::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(file_name, file_type, data) => {
                self.files.push(FileDetails {
                    data,
                    file_type,
                    name: file_name.clone(),
                });
                self.readers.remove(&file_name);
                true
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let file_type = file.raw_mime_type();

                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();

                        gloo::file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::Loaded(
                                file_name,
                                file_type,
                                res.expect("failed to read file"),
                            ))
                        })
                    };
                    self.readers.insert(file_name, task);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="w-4/5 m-auto">
                <p class="text-2xl text-center">{ "Upload Your TLSNotary Proof" }</p>

                <label for="file-upload" class="cursor-pointer">
                    <div class="p-16 flex flex-col justify-center items-center bg-zinc-700 border border-white border-dashed rounded-2xl"
                        id="drop-container"
                        ondrop={ctx.link().callback(|event: DragEvent| {
                            event.prevent_default();
                            let files = event.data_transfer().unwrap().files();
                            Self::upload_files(files)
                        })}
                        ondragover={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                        ondragenter={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                    >
                        <svg class="w-16 h-16 text-white-50" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 16">
                            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 13h3a3 3 0 0 0 0-6h-.025A5.56 5.56 0 0 0 16 6.5 5.5 5.5 0 0 0 5.207 5.021C5.137 5.017 5.071 5 5 5a4 4 0 0 0 0 8h2.167M10 15V6m0 0L8 8m2-2 2 2"/>
                        </svg>
                        <p class="text-base text-white-50"><span class="font-semibold">{"Drop your \""}<span class="font-mono">{"proof.json"}</span>{"\" file here"}</span>{" or click to select"}</p>
                        <p class="text-sm text-gray-500">{"This is the JSON output of your prover"}</p>
                    </div>
                </label>
                <input
                    id="file-upload"
                    class="invisible"
                    type="file"
                    accept="application/json"
                    multiple={true}
                    onchange={ctx.link().callback(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Self::upload_files(input.files())
                    })}
                />

                <div>
                    { for self.files.iter().rev().map(Self::view_file) }
                </div>
            </div>
        }
    }
}

impl App {
    fn view_file(file: &FileDetails) -> Html {
        fn parse_notarized_session(json_str: &str) -> Html {
            let notarized_session: Result<NotarizedSession, serde_json::Error> =
                serde_json::from_str(json_str);

            match notarized_session {
                Ok(notarized_session) => {
                    let header = notarized_session.header();
                    let time = chrono::DateTime::UNIX_EPOCH
                        + Duration::from_secs(header.handshake_summary().time());

                    return html! {
                        <>
                        <li>
                        <b>{"domain: " }</b>{notarized_session.data().server_name().as_str().to_string()}
                        </li>
                        <li>
                        <b>{"Notarization time: " }</b>{time}
                        </li>
                        </>
                    };
                }
                Err(e) => html! {
                     <div>{format!("Parsing failed {}", e.to_string())}</div>
                },
            }
        }

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
                .or_else(|err| return Err(err.to_string()))
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
                                <li>
                                    <p style="color:red">
                                        <b>{"Invalid Proof: " }</b>{ "❌ " }{proof_verification.unwrap_err().to_string()}{ " ❌" }
                                    </p>
                                </li>
                            </>
                        };
                    }

                    let proof_verification_feedback =
                        "✅ Proof successfully verified ✅".to_string();

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
                    //
                    // This returns the redacted transcripts
                    let (mut sent, mut recv) = substrings.verify(&header).unwrap();

                    // Replace the bytes which the Prover chose not to disclose with 'X'
                    sent.set_redacted(b'\0');
                    recv.set_redacted(b'\0');

                    let bytes_send = String::from_utf8(sent.data().to_vec())
                        .unwrap()
                        .replace("\0", REDACTED_CHAR);

                    let bytes_received = String::from_utf8(recv.data().to_vec())
                        .unwrap()
                        .replace("\0", REDACTED_CHAR);

                    return html! {
                        <div class="flex flex-col">
                            <div class="w-full">
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
                            <div class="w-full">
                                <b>{"Bytes send: " }</b>
                                <div class="bg-black text-white p-4 rounded-md">
                                    <pre>{format!("{}", bytes_send)}</pre>
                                </div>
                            </div>
                            <div class="w-full">
                                <b>{"Bytes received: " }</b>
                                <div class="bg-black text-white p-4 rounded-md">
                                    <pre>{format!("{}", bytes_received)}</pre>
                                </div>
                            </div>
                        </div>
                    };
                }
                Err(e) => html! {
                     <div>{format!("Parsing failed {}", e.to_string())}</div>
                },
            }
        }

        let json_str = str::from_utf8(&file.data).unwrap();
        html! {
            <div class="p-4 flex flex-col justify-center items-center bg-zinc-700 border border-white border-dashed rounded-2xl">
                <p class="text-center">{ format!("{}", file.name) }</p>

                <div class="flex-1 flex flex-col justify-center p-4">
                    <div class="container mx-auto px-4">
                    if file.file_type.contains("application/json") {
                        <div>
                            <ul>
                                {parse_tls_proof(json_str)}
                            </ul>
                        </div>
                        // <div>
                        //     {"Notarized session:"}
                        //     <ul>
                        //         {parse_notarized_session(json_str)}
                        //     </ul>
                        // </div>

                        // <div>
                        //     {"Raw json:"}
                        //     <div class="bg-black text-white p-4 rounded-md">
                        //         <pre id="logContent" class="whitespace-pre-wrap font-mono">{json_str}</pre>
                        //     </div>
                        // </div>
                    }
                    </div>
                </div>
            </div>
        }
    }

    fn upload_files(files: Option<FileList>) -> Msg {
        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from)
                .collect();
            Msg::Files(files)
        } else {
            Msg::Files(Vec::with_capacity(0))
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<App>::new().render();
}
