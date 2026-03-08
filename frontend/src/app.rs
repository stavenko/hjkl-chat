use leptos::*;
use wasm_bindgen::JsCast;

use crate::components::*;

#[component]
pub fn App() -> impl IntoView {
    let (is_dark, set_is_dark) = create_signal(false);

    let toggle_theme = move |_| {
        let next = !is_dark.get_untracked();
        set_is_dark.set(next);

        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let html = document
            .document_element()
            .expect("no <html>")
            .unchecked_into::<web_sys::HtmlElement>();

        let theme = if next { "dark" } else { "light" };
        html.dataset().set("theme", theme).expect("failed to set data-theme");
    };

    // -- Text Input signals --
    let normal_value = create_rw_signal(String::new());
    let error_value = create_rw_signal("bad-email".to_string());
    let normal_error: Signal<Option<String>> = Signal::derive(|| None);
    let error_error: Signal<Option<String>> =
        Signal::derive(|| Some("Invalid email address".to_string()));

    // -- Password Input signals --
    let pw_none_val = create_rw_signal(String::new());
    let pw_weak_val = create_rw_signal("abc".to_string());
    let pw_med_val = create_rw_signal("abc123!".to_string());
    let pw_strong_val = create_rw_signal("Str0ng!Pa$$w0rd".to_string());

    let pw_none_str: Signal<PasswordStrength> = Signal::derive(|| PasswordStrength::None);
    let pw_weak_str: Signal<PasswordStrength> = Signal::derive(|| PasswordStrength::Weak);
    let pw_med_str: Signal<PasswordStrength> = Signal::derive(|| PasswordStrength::Medium);
    let pw_strong_str: Signal<PasswordStrength> = Signal::derive(|| PasswordStrength::Strong);

    let no_error: Signal<Option<String>> = Signal::derive(|| None);
    let no_error2: Signal<Option<String>> = Signal::derive(|| None);
    let no_error3: Signal<Option<String>> = Signal::derive(|| None);
    let no_error4: Signal<Option<String>> = Signal::derive(|| None);

    view! {
        <div class="showcase">
            <header class="showcase__header">
                <h1 class="showcase__title">"Component Showcase"</h1>
                <button class="theme-toggle" on:click=toggle_theme>
                    <span class="theme-toggle__icon">
                        {move || if is_dark.get() { "\u{2600}\u{FE0E}" } else { "\u{263E}" }}
                    </span>
                    {move || if is_dark.get() { "Light" } else { "Dark" }}
                </button>
            </header>

            // -- Buttons --
            <section class="showcase__section">
                <h3 class="showcase__section-title">"Buttons"</h3>
                <div class="showcase__row">
                    <Button label="Primary Button" on_click=|| {} />
                    <Button label="Disabled Button" disabled=true on_click=|| {} />
                </div>
            </section>

            // -- Text Inputs --
            <section class="showcase__section">
                <h3 class="showcase__section-title">"Text Inputs"</h3>
                <Surface>
                    <FormHeader text="Sign In" />
                    <FormDescription text="Enter your credentials to continue." />
                    <TextInput
                        label="Email"
                        placeholder="you@example.com"
                        value=normal_value
                        error=normal_error
                    />
                    <TextInput
                        label="Email (with error)"
                        placeholder="you@example.com"
                        value=error_value
                        error=error_error
                    />
                </Surface>
            </section>

            // -- Password Inputs --
            <section class="showcase__section">
                <h3 class="showcase__section-title">"Password Inputs"</h3>
                <Surface>
                    <FormHeader text="Password Strength States" />
                    <PasswordInput
                        label="None"
                        placeholder="Type a password..."
                        value=pw_none_val
                        error=no_error
                        strength=pw_none_str
                    />
                    <PasswordInput
                        label="Weak"
                        placeholder="Type a password..."
                        value=pw_weak_val
                        error=no_error2
                        strength=pw_weak_str
                    />
                    <PasswordInput
                        label="Medium"
                        placeholder="Type a password..."
                        value=pw_med_val
                        error=no_error3
                        strength=pw_med_str
                    />
                    <PasswordInput
                        label="Strong"
                        placeholder="Type a password..."
                        value=pw_strong_val
                        error=no_error4
                        strength=pw_strong_str
                    />
                </Surface>
            </section>

            // -- Countdown Button --
            <section class="showcase__section">
                <h3 class="showcase__section-title">"Countdown Button"</h3>
                <div class="showcase__row">
                    <CountdownButton
                        label="Send Code"
                        countdown_secs=5u32
                        on_click=|| {}
                    />
                </div>
            </section>

            // -- Auth Links --
            <section class="showcase__section">
                <h3 class="showcase__section-title">"Auth Links"</h3>
                <div class="showcase__column">
                    <AuthLink text="Forgot password?" href="#" />
                    <AuthLink text="Already have an account?" href="#" />
                    <AuthLink text="Create an account" href="#" />
                </div>
            </section>
        </div>
    }
}
