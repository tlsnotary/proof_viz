extern crate base64;
use elliptic_curve::pkcs8::DecodePublicKey;
use std::collections::HashMap;
use std::time::Duration;

use gloo::file::callbacks::FileReader;
use gloo::file::File;
use std::str;
use tlsn_core::NotarizedSession;
use web_sys::{DragEvent, Event, FileList, HtmlInputElement};
use yew::html::TargetCast;
use yew::{html, Callback, Component, Context, Html};

use tlsn_core::proof::{SessionProof, TlsProof};

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
            <div id="wrapper">
                <p id="title">{ "Upload Your TLSNotary Proof" }</p>
                <label for="file-upload">
                    <div
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
                        <i class="fa fa-cloud-upload"></i>
                        <p>{"Drop your proof.json file here or click to select"}</p>
                    </div>
                </label>
                <input
                    id="file-upload"
                    type="file"
                    accept="application/json"
                    multiple={true}
                    onchange={ctx.link().callback(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Self::upload_files(input.files())
                    })}
                />
                <div id="preview-area">
                    { for self.files.iter().map(Self::view_file) }
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
                        <b>{"domain:" }</b>{notarized_session.data().server_name().as_str().to_string()}
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
        fn verify_proof(session: &SessionProof) -> String {
            // FIXME: this does not work yet
            return String::from("TODO");

            // This verifies the identity of the server using a default certificate verifier which trusts
            // the root certificates from the `webpki-roots` crate.

            // let v = session.verify_with_default_cert_verifier(notary_pubkey());
            // match v {
            //     Ok(_) => return "Proof successfully verified ✅".to_string(),
            //     Err(error) => return error.to_string(),
            // };
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

                    let proof_verification_feedback = verify_proof(&session);

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
                    sent.set_redacted(b'X');
                    recv.set_redacted(b'X');

                    return html! {
                        <>
                            <li>
                                <b>{"domain:" }</b>{server_name.as_str().to_string()}
                            </li>
                            <li>
                                <b>{"Notarization time: " }</b>{time}
                            </li>
                            <li>
                                <b>{"Proof: " }</b>{proof_verification_feedback}
                            </li>
                            <li>
                                <b>{"Bytes send: " }</b>
                                <pre>{format!("{}", String::from_utf8(sent.data().to_vec()).unwrap())}</pre>
                            </li>
                            <li>
                                <b>{"Bytes received: " }</b>
                                <pre>{format!("{}", String::from_utf8(recv.data().to_vec()).unwrap())}</pre>
                            </li>
                        </>
                    };
                }
                Err(e) => html! {
                     <div>{format!("Parsing failed {}", e.to_string())}</div>
                },
            }
        }

        let json_str = str::from_utf8(&file.data).unwrap();
        html! {
            <div class="preview-tile">
                <p class="preview-name">{ format!("{}", file.name) }</p>
                <div class="preview-media">
                    if file.file_type.contains("application/json") {
                        <div>
                            {"Proof:"}
                            <ul>
                                {parse_tls_proof(json_str)}
                            </ul>
                        </div>
                        <div>
                            {"Notarized session:"}
                            <ul>
                                {parse_notarized_session(json_str)}
                            </ul>
                        </div>
                        <div>
                            {"Raw json:"}
                            <pre>
                            {json_str}
                            </pre>
                        </div>
                    }
                </div>
            </div>
        }
    }

    fn upload_files(files: Option<FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        Msg::Files(result)
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<App>::new().render();
}
