extern crate base64;
use elliptic_curve::pkcs8::DecodePublicKey;
use gloo::file::callbacks::FileReader;
use gloo::file::File;
use std::collections::HashMap;
use std::str;
use web_time::Duration;

use web_sys::{DragEvent, Event, FileList, HtmlInputElement};
use yew::html::TargetCast;
use yew::{html, AttrValue, Callback, Component, Context, Html};
use yew_icons::{Icon, IconId};

use tlsn_core::proof::{SessionProof, TlsProof};
use tlsn_core::NotarizedSession;

const REDACTED_CHAR: &str = "█";

mod components;
use crate::components::redactedBytesComponent::Direction;
use crate::components::redactedBytesComponent::RedactedBytesComponent;

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
        let link_classes =
            "block px-4 py-2 hover:bg-black hover:text-white rounded border-black border";
        let links = [
            ("About TLSNotary", "https://tlsnotary.org"),
            // ("PSE", "https://pse.dev"),
        ];
        html! {
        <div class="flex flex-col h-screen">
            <nav class="bg-zinc-700 h-16 px-8 py-2">
                <div class="container flex mx-auto gap-6 items-center h-full">
                    <svg class="w-8 h-8" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <g>
                            <path fill="#ffffff" d="m15 22-.693 1.04A1.25 1.25 0 0 0 16.25 22H15Zm-3-2 .693-1.04a1.25 1.25 0 0 0-1.386 0L12 20Zm-3 2H7.75a1.25 1.25 0 0 0 1.943 1.04L9 22ZM8.75 3.537l-.1 1.246.1-1.246Zm1.685-.697-.952-.81.952.81ZM6.532 5.686l-1.246.1 1.246-.1Zm2.154-2.154.1-1.246-.1 1.246ZM5.84 7.435l.81.952-.81-.952Zm.697-1.684 1.246-.1-1.246.1Zm-.747 4.772-.81.952.81-.952Zm0-3.046-.81-.952.81.952Zm.747 4.772-1.246-.1 1.246.1Zm-.697-1.684.81-.952-.81.952Zm2.846 3.903.1 1.246-.1-1.246Zm-2.154-2.154 1.246.1-1.246-.1Zm3.903 2.846.952-.81-.952.81Zm-1.684-.697-.1-1.246.1 1.246Zm4.772.747.952.81-.952-.81Zm-3.046 0-.952.81.952-.81Zm4.772-.747.1-1.246-.1 1.246Zm-1.684.697-.952-.81.952.81Zm3.903-2.846 1.246-.1-1.246.1Zm-2.154 2.154-.1 1.246.1-1.246Zm2.846-3.903.81.952-.81-.952Zm-.697 1.684-1.246.1 1.246-.1Zm.747-4.772.81-.952-.81.952Zm0 3.046-.81-.952.81.952Zm-.747-4.772-1.246-.1 1.246.1Zm.697 1.684-.81.952.81-.952Zm-2.846-3.903-.1-1.246.1 1.246Zm2.154 2.154 1.246.1-1.246-.1ZM13.565 2.84l.952-.81-.952.81Zm1.684.697.1 1.246-.1-1.246Zm-1.726-.747-.952.81.952-.81Zm-3.046 0 .952.81-.952-.81ZM9 14.458l.055-1.248L9 14.458Zm6.693 6.502-3-2-1.386 2.08 3 2 1.386-2.08Zm-4.386-2-3 2 1.386 2.08 3-2-1.386-2.08ZM12.57 3.6l.042.05 1.904-1.62-.042-.05L12.57 3.6Zm2.779 1.183.064-.005-.2-2.492-.065.005.2 2.492Zm.872.803-.005.064 2.492.201.005-.065-2.492-.2Zm1.128 2.8.05.043 1.62-1.904-.05-.042-1.62 1.904Zm.05 1.185-.05.042 1.62 1.904.05-.042-1.62-1.904Zm-1.183 2.779.005.064 2.492-.2-.005-.065-2.492.2Zm-.803.872-.064-.005-.201 2.492.065.005.2-2.492Zm-2.8 1.128-.043.05 1.904 1.62.042-.05-1.904-1.62Zm-1.185.05-.042-.05-1.904 1.62.042.05 1.904-1.62ZM8.65 13.217l-.064.005.2 2.492.065-.005-.2-2.492Zm-.872-.803.005-.064-2.492-.201-.005.065 2.492.2Zm-1.128-2.8L6.6 9.57l-1.62 1.904.05.042 1.62-1.904ZM6.6 8.428l.05-.042-1.62-1.904-.05.042L6.6 8.429ZM7.783 5.65l-.005-.064-2.492.2.005.065 2.492-.2Zm.803-.872.064.005.201-2.492-.065-.005-.2 2.492Zm2.8-1.128.043-.05-1.904-1.62-.042.05 1.904 1.62ZM8.65 4.783a3.25 3.25 0 0 0 2.737-1.133L9.483 2.03a.75.75 0 0 1-.632.261l-.2 2.492Zm-.872.803a.75.75 0 0 1 .808-.808l.2-2.492a3.25 3.25 0 0 0-3.5 3.5l2.492-.2Zm-1.128 2.8A3.25 3.25 0 0 0 7.783 5.65l-2.492.201a.75.75 0 0 1-.261.632l1.62 1.904ZM6.6 9.572a.75.75 0 0 1 0-1.142L4.98 6.525a3.25 3.25 0 0 0 0 4.95L6.6 9.571Zm1.183 2.779A3.25 3.25 0 0 0 6.65 9.613l-1.62 1.904a.75.75 0 0 1 .261.632l2.492.2Zm.803.872a.75.75 0 0 1-.808-.808l-2.492-.2a3.25 3.25 0 0 0 3.5 3.5l-.2-2.492ZM12.57 14.4a.75.75 0 0 1-1.142 0l-1.904 1.62a3.25 3.25 0 0 0 4.95 0L12.57 14.4Zm3.651-1.986a.75.75 0 0 1-.808.808l-.2 2.492a3.25 3.25 0 0 0 3.5-3.5l-2.492.2Zm1.128-2.8a3.25 3.25 0 0 0-1.133 2.736l2.492-.201a.75.75 0 0 1 .261-.632l-1.62-1.904Zm.05-1.185a.75.75 0 0 1 0 1.142l1.62 1.904a3.25 3.25 0 0 0 0-4.95L17.4 8.429ZM16.217 5.65a3.25 3.25 0 0 0 1.133 2.737l1.62-1.904a.75.75 0 0 1-.261-.632l-2.492-.2Zm-.803-.872a.75.75 0 0 1 .808.808l2.492.2a3.25 3.25 0 0 0-3.5-3.5l.2 2.492Zm-2.8-1.128a3.25 3.25 0 0 0 2.736 1.133l-.201-2.492a.75.75 0 0 1-.632-.261l-1.904 1.62Zm1.861-1.67a3.25 3.25 0 0 0-4.95 0l1.904 1.62a.75.75 0 0 1 1.142 0l1.904-1.62Zm-3.088 12.37a3.25 3.25 0 0 0-2.332-1.14l-.11 2.497a.75.75 0 0 1 .538.263l1.904-1.62Zm-2.332-1.14a3.26 3.26 0 0 0-.405.007l.201 2.492a.732.732 0 0 1 .094-.002l.11-2.497ZM10.25 22v-7.542h-2.5V22h2.5Zm5.1-8.783a3.26 3.26 0 0 0-.405-.007l.11 2.497a.99.99 0 0 1 .094.002l.2-2.492Zm-.405-.007a3.25 3.25 0 0 0-2.332 1.14l1.904 1.62a.75.75 0 0 1 .538-.263l-.11-2.497Zm-1.195 1.248V22h2.5v-7.542h-2.5Z"/>
                            <path stroke="#ffffff" stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="m14 8-3 3-1-1"/>
                    </g>
                    </svg>
                    <h1 class="font-bold text-2xl text-white">{"Check TLSNotary proofs"}</h1>
                    <div class="flex-1"></div>
                    {for links.iter().map(|(label, href)| html! {
                        <a class={link_classes} href={*href}>{label}</a>
                    })}
                </div>
            </nav>
            <div class="w-4/5 m-auto">
                // <p class="text-2xl text-center">{ "Upload Your TLSNotary Proof" }</p>

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
                        <br/>
                        <p class="text-sm text-gray-400 text-center">{"🕵️ Your proof is "}<strong>{"checked locally in the browser"}</strong>{" 🕵️"}<br />{"Your file will not be uploaded"}</p>
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

                    // This returns the redacted transcripts
                    let (mut sent, mut recv) = substrings.verify(&header).unwrap();

                    // Replace the bytes which the Prover chose not to disclose with '\0'
                    sent.set_redacted(b'\0');
                    recv.set_redacted(b'\0');

                    let bytes_send = String::from_utf8(sent.data().to_vec()).unwrap();

                    let bytes_received = String::from_utf8(recv.data().to_vec()).unwrap();

                    let direction = Direction::Send;
                    return html! {
                        <div class="p-4 flex flex-col justify-center items-center w-full">
                            // {test}
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
