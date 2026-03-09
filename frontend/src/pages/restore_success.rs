use leptos::*;

use crate::components::*;

#[component]
pub fn RestoreSuccessPage() -> impl IntoView {
    view! {
        <div class="auth-page">
            <Logo/>
            <p class="auth-page__tagline">"Restore access to your account"</p>
            <Surface>
                <FormHeader text="Password Changed"/>
                <p class="form-description">
                    "Your password has been changed successfully. You can now "
                    <a href="/login" class="auth-link auth-link--inline">"sign in"</a>
                    " with your new password."
                </p>
            </Surface>
        </div>
    }
}
