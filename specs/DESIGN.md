# Design System

## Design Source

All designs live in a Penpot project.

**Penpot instance:** `https://penpot.hjkl.pro`
**Design file ID:** `742d722a-06ca-817e-8007-a42f6283e7ed`

### Pages

| Page | ID | Purpose |
| ---- | -- | ------- |
| Main(outdated) | `988fdbaf-c8f8-808f-8007-a55ba615f576` | Legacy frames, do not use |
| Chat Interface | `988fdbaf-c8f8-808f-8007-a955201d53f9` | Chat UI screens |
| Login | `b950d130-c95a-8005-8007-ad3c0ebbb6de` | Authentication screens |
| Logo | `b950d130-c95a-8005-8007-ad45b1fe8358` | Logo master design |

### Reusable Components

| Component | Description |
| --------- | ----------- |
| hjkl-chat | Logo component |
| AuthenticationInput | Text input for auth forms (label + input container + error text) |
| AuthenticationButton | Submit button for auth forms |
| Primary | Active button variant |
| Disabled | Disabled button variant |
| Text | Empty input with placeholder |
| Filled | Input with entered value |
| Error | Input with validation error |
| Strength | Password strength bar |

## Design Tokens

Design tokens are defined in `design-tokens.json` at the project root. The file follows the Tokens Studio format compatible with Penpot import.

### Token Sets

| Set | Role | Description |
| --- | ---- | ----------- |
| `core` | Primitives | Colors, spacing, sizing, typography, border radius, opacity. Not themed. |
| `mode/light` | Theme | Semantic colors, shadows, component colors for light mode. References `core` tokens. |
| `mode/dark` | Theme | Same structure as light, with values appropriate for dark backgrounds. |

### Themes

Two themes defined in `$themes`:
- **mode/light** — enables `core` + `mode/light` sets
- **mode/dark** — enables `core` + `mode/dark` sets

### Token Categories

**Core primitives (`core`):**
- `color.palette.*` — raw color values (gray-50..900, red, green, amber, indigo, white, black)
- `color.brand.*` — brand primary and hover colors
- `logo.*` — logo-specific colors (h, j, k, l letter and background colors), font-weight, bubble border
- `spacing.*` — xs(4), sm(8), md(16), lg(24), xl(32), 2xl(48)
- `sizing.*` — input-height, button-height, form-max-width, icon sizes, strength-bar-height
- `borderRadius.*` — sm(4), md(8), lg(12), full(9999)
- `strokeWidth.*` — default(1), thick(2)
- `fontFamilies.*` — sans (Inter), mono (JetBrains Mono)
- `fontSizes.*` — xs(12), sm(14), md(16), lg(20), xl(24), 2xl(32)
- `fontWeights.*` — normal(400), medium(500), semibold(600), bold(700)
- `letterSpacing.*` — normal(0), tight(-0.5)
- `opacity.*` — disabled(0.5), placeholder(0.6)

**Themed tokens (`mode/light`, `mode/dark`):**
- `color.semantic.*` — error, success, warning + background tints
- `color.text.*` — primary, secondary, muted, inverse
- `color.bg.*` — page, surface, surface-alt
- `color.button.*` — primary-bg, primary-text, primary-hover, disabled-bg, disabled-text
- `color.input.*` — bg, border, border-focus, border-error, placeholder
- `color.link.*` — default, hover
- `color.strength.*` — weak, medium, strong
- `color.file-browser.*` — row-odd, row-even, project-active, project-inactive
- `shadow.*` — card, card-hover, input-focus

### Importing Tokens into Penpot

In Penpot UI: **Tokens panel** (left sidebar) → **Import** → select `design-tokens.json`.

Penpot uses Tokens Studio type names. Supported `$type` values:
- `color`, `spacing`, `sizing`, `borderRadius`, `fontFamilies`, `fontSizes`, `fontWeights`, `letterSpacing`, `opacity`, `shadow`, `borderWidth`
- For stroke widths, use `$type: "dimension"` (Penpot does not support `strokeWidth` type)

### Retrieving Tokens via API

Authentication header required for all requests:
```
Authorization: Token <PENPOT_ACCESS_TOKEN>
```

The access token is stored in the project `.env` file as `PENPOT_ACCESS_TOKEN`.

**Get the full design file (includes all tokens, pages, shapes):**
```
GET https://penpot.hjkl.pro/api/rpc/command/get-file?id=742d722a-06ca-817e-8007-a42f6283e7ed&components-v2=true
```

Returns transit+json. The response contains a `data` object with:
- `pages` — ordered list of page IDs
- `pages-index` — map of page ID to page object (contains `objects` tree with all shapes)
- `tokens-lib` — design tokens library (all sets, themes, and token values)

**Get frame thumbnails (map of frame IDs to thumbnail asset UUIDs):**
```
GET https://penpot.hjkl.pro/api/rpc/command/get-file-object-thumbnails?file-id=742d722a-06ca-817e-8007-a42f6283e7ed
```

Returns a map where keys are `<file-id>/<page-id>/<frame-id>/frame` and values are asset UUIDs.

**Download a thumbnail PNG:**
```
GET https://penpot.hjkl.pro/assets/by-id/<thumbnail-uuid>
```

**How to get a frame image:**
1. Call `get-file-object-thumbnails` with the file ID
2. Find the entry matching `742d722a-06ca-817e-8007-a42f6283e7ed/<page-id>/<frame-id>/frame`
3. Download from `/assets/by-id/<uuid>`

Frame IDs are extracted from Penpot view URLs:
```
https://penpot.hjkl.pro/view/<file-id>?page-id=<page-id>&frame-id=<frame-id>
```

## Login Page Design

**Page:** Login (`b950d130-c95a-8005-8007-ad3c0ebbb6de`)

The login page is the first thing an unauthorized user sees. It is logo-centric with authentication form.

**Layout:** 393x852 mobile-first frame, centered flex column with:
- Logo (hjkl-chat component)
- Tagline: "Authenticate to see your projects"
- Form card (white surface, rounded corners, shadow, 24px padding)
- Navigation links with 44px+ touch targets

### Frames

| Frame | ID | Description |
| ----- | -- | ----------- |
| Login - Empty | `82852407-cb8b-809b-8007-ad6962f86950` | Both fields empty, button disabled |
| Login - Filled | `82852407-cb8b-809b-8007-ad6c1156d750` | Both fields filled, button active (blue) |
| Login - Error | `82852407-cb8b-809b-8007-ad6ed7fd51a7` | Fields filled, password shows red error border with "Invalid email or password", button disabled |

**View URLs:**
- [Login - Empty](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=b950d130-c95a-8005-8007-ad3c0ebbb6de&frame-id=82852407-cb8b-809b-8007-ad6962f86950)
- [Login - Filled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=b950d130-c95a-8005-8007-ad3c0ebbb6de&frame-id=82852407-cb8b-809b-8007-ad6c1156d750)
- [Login - Error](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=b950d130-c95a-8005-8007-ad3c0ebbb6de&frame-id=82852407-cb8b-809b-8007-ad6ed7fd51a7)

### Error State

When the server returns invalid credentials, the password input switches to error state:
- Input container border changes to red (`#dc2626`), 2px width
- Error text "Invalid email or password" appears below the input in red
- Button becomes disabled (gray)
