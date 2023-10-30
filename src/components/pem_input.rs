use elliptic_curve::pkcs8::DecodePublicKey;
// use gloo::console::log;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub pem_callback: Callback<p256::PublicKey>,
}

// from https://github.com/tlsnotary/notary-server/tree/main/src/fixture/notary/notary.key
// converted with `openssl ec -in notary.key -pubout -outform PEM`
pub const DEFAULT_PEM: &str = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEBv36FI4ZFszJa0DQFJ3wWCXvVLFr
cRzMG5kaTeHGoSzDu6cFqx3uEWYpFGo6C0EOUgf+mEgbktLrXocv5yHzKg==
-----END PUBLIC KEY-----";

#[function_component(PemInputComponent)]
pub fn pem_input_component(Props { pem_callback }: &Props) -> Html {
    let input_value = use_state(|| DEFAULT_PEM.to_string());
    let invalid_input = use_state(|| None);

    let oninput = {
        let input_value = input_value.clone();
        let callback = pem_callback.clone();
        let invalid_input = invalid_input.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value().trim().to_string();

            let result = p256::PublicKey::from_public_key_pem(value.as_str());
            match result {
                Ok(public_key) => {
                    input_value.set(value.clone());
                    invalid_input.set(None);
                    callback.emit(public_key.clone());
                }
                Err(err) => {
                    input_value.set(value.clone());
                    invalid_input.set(Some(err.to_string()));
                    // do not emit a false pem here
                }
            }
        })
    };

    let style = if invalid_input.is_none() {
        "text-sm text-white border-gray-600 focus:ring-blue-500 focus:border-blue-500"
    } else {
        "text-sm text-pink-600 border-pink-500 focus:border-pink-500 focus:ring-pink-500"
    };

    html! {
        <div class={style}>
            <details class="p-4 w-5/6" open={false}>
                <summary><b>{"Change Notary Public Key:" }</b>{if invalid_input.as_ref().is_some() {" ❌"} else {""}}</summary>
                <textarea class="block p-2.5 w-full bg-gray-700"
                    id="pem-input"
                    rows="4"
                    value={input_value.to_string()}
                    oninput={oninput} >
                </textarea>
                if let Some(x) = invalid_input.as_ref() {
                    <p>{x}</p>
                }
            </details>
        </div>
    }
}
