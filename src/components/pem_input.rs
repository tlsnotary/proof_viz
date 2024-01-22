use elliptic_curve::{pkcs8::DecodePublicKey, PublicKey};

#[allow(unused_imports)]
use gloo::console::log;
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

// from https://notary.pse.dev/info
pub const NOTARY_PSE_PEM: &str = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAExpX/4R4z40gI6C/j9zAM39u58LJu
3Cx5tXTuqhhu/tirnBi5GniMmspOTEsps4ANnPLpMmMSfhJ+IFHbc3qVOA==
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
                    callback.emit(public_key);
                }
                Err(err) => {
                    input_value.set(value.clone());
                    invalid_input.set(Some(err.to_string()));
                    // do not emit a false pem here
                }
            }
        })
    };

    let notary_pse_dev = {
        let input_value = input_value.clone();
        let callback = pem_callback.clone();
        let invalid_input = invalid_input.clone();

        Callback::from(move |_| {
            let public_key = p256::PublicKey::from_public_key_pem(NOTARY_PSE_PEM)
                .expect("should be a valid public key");
            input_value.set(NOTARY_PSE_PEM.into());
            invalid_input.set(None);
            callback.emit(public_key);
        })
    };

    let default = {
        let input_value = input_value.clone();
        let callback = pem_callback.clone();
        let invalid_input = invalid_input.clone();

        Callback::from(move |_| {
            let public_key = p256::PublicKey::from_public_key_pem(DEFAULT_PEM)
                .expect("should be a valid public key");
            input_value.set(DEFAULT_PEM.into());
            invalid_input.set(None);
            callback.emit(public_key);
        })
    };

    // Toggling styles based on the presence of an error
    let style = if invalid_input.is_none() {
        "text-sm text-white border-gray-600 focus:ring-blue-500 focus:border-blue-500"
    } else {
        "text-sm text-red-500 border-red-500 focus:border-red-500 focus:ring-red-500"
    };

    html! {
        <div class="container flex mx-auto p-4">
            <div class="w-full">
                <details class="w-full" open={false}>
                    <summary class="cursor-pointer px-8 py-2"><b>{"Change Notary Public Key:" }</b>{if invalid_input.as_ref().is_some() {" ❌"} else {""}}</summary>
                    <div class="px-8">
                        <textarea class={style.to_string() + " block p-2.5 w-full bg-zinc-700 mt-2 border rounded"}
                            id="pem-input"
                            rows="4"
                            value={input_value.to_string()}
                            oninput={oninput} >
                        </textarea>
                        if let Some(error_message) = invalid_input.as_ref() {
                            <p class="mt-2 text-red-500">{error_message}</p>
                        }
                        <div class="h-fit min-h-full flex justify-end">
                          <button class="float-right px-4 py-2 hover:bg-black hover:text-white rounded border-black border"
                           onclick={notary_pse_dev}>{ "notary.pse.dev" }
                           </button>
                           <button class="float-right px-4 py-2 hover:bg-black hover:text-white rounded border-black border"
                           onclick={default}>{ "default" }
                           </button>
                        </div>
                    </div>
                </details>
            </div>
        </div>
    }
}
