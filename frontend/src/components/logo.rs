use leptos::*;

#[component]
pub fn Logo() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 157 111" fill="none" class="logo">
            // H rect (green)
            <rect rx="12" ry="12" x="0" y="13" width="38" height="45" fill="#64ffbd"/>
            <rect rx="9.5" ry="9.5" x="2.5" y="15.5" width="33" height="40" fill="none" stroke="#1c773b" stroke-width="5"/>
            // J rect (peach)
            <rect rx="12" ry="12" x="40" y="0" width="38" height="45" fill="#f1d19f"/>
            <rect rx="9.5" ry="9.5" x="42.5" y="2.5" width="33" height="40" fill="none" stroke="#d28715" stroke-width="5"/>
            // K rect (purple)
            <rect rx="12" ry="12" x="80" y="0" width="37" height="45" fill="#e9b2ff"/>
            <rect rx="9.5" ry="9.5" x="82.5" y="2.5" width="32" height="40" fill="none" stroke="#8915b7" stroke-width="5"/>
            // L rect (teal)
            <rect rx="12" ry="12" x="119" y="13" width="38" height="45" fill="#bbedeb"/>
            <rect rx="9.5" ry="9.5" x="121.5" y="15.5" width="33" height="40" fill="none" stroke="#37a39d" stroke-width="5"/>
            // Letters
            <text x="7" y="54" dominant-baseline="ideographic" font-family="Inter" font-weight="700" font-size="32" fill="#1c773b">"H"</text>
            <text x="49" y="41" dominant-baseline="ideographic" font-family="Inter" font-weight="700" font-size="32" fill="#d28715">"J"</text>
            <text x="87" y="41" dominant-baseline="ideographic" font-family="Inter" font-weight="700" font-size="32" fill="#8915b7">"K"</text>
            <text x="129" y="55" dominant-baseline="ideographic" font-family="Inter" font-weight="700" font-size="32" fill="#37a39d">"L"</text>
            // Speech bubble tails
            <path d="M18,58 C18,67 26,71 35,71 C37,71 52,71 55,71 C63,71 70,76 70,85" fill="none" stroke="#1c773b" stroke-width="6"/>
            <path d="M59,45 C59,61 76,52 76,85" fill="none" stroke="#d28715" stroke-width="6"/>
            <path d="M99,45 C99,61 80,48 80,85" fill="none" stroke="#8915b7" stroke-width="6"/>
            <path d="M138,58 C138,66 131,71 126,71 C114.91,71 105,71 99,71 C87,71 86,77 86,85" fill="none" stroke="#37a39d" stroke-width="6"/>
            // White bar
            <rect rx="12" ry="12" x="19" y="85" width="110" height="25" fill="#ffffff"/>
            // {CHAT} text
            <text x="22.09" y="109" dominant-baseline="ideographic" font-family="'JetBrains Mono'" font-weight="800" font-size="18" letter-spacing="8" fill="#000000">"{CHAT}"</text>
        </svg>
    }
}
