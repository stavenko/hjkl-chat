# RUST-COMMON-SPEC.md

We will keep there Rust-only project specification.

## 1. Module Organization

### 1.1 Never Use Inline Modules

```rust
// BAD - inline module
mod my_module {
    pub fn foo() {}
}
```

### 1.2 Never Use mod.rs Files

```rust
// BAD - old style
src/
  my_module/
    mod.rs        // Don't do this
    sub_module.rs
```

### 1.3 Correct Pattern: module_name.rs + module_name/ Directory

```rust
// GOOD
src/
  lib.rs
  my_module.rs           // Contains only mod declarations
  my_module/
    sub_module.rs        // Actual implementation
    other_module.rs      // Actual implementation
```

**my_module.rs:**
```rust
mod sub_module;
mod other_module;

pub use sub_module::SomeType;
pub use other_module::OtherType;
```

### 1.4 Parent Modules Must Contain Only Declarations and Re‑exports

When a module has nested modules, it must contain **only** module declarations (`mod …`) and re‑exports (`pub use …`). No structs, enums, functions, or other definitions belong in the parent module.

```rust
// BAD - parent module contains definitions alongside submodules
// my_module.rs
mod sub_module;
mod other_module;

pub use sub_module::SomeType;

// Don't define types in parent module!
pub struct ParentType {
    // This should be in its own submodule
}

pub fn helper_function() {
    // This should be in its own submodule
}
```

```rust
// GOOD - parent module only declares submodules and re‑exports
// my_module.rs
mod sub_module;
mod other_module;
mod parent_type;  // Move ParentType to its own file
mod helpers;      // Move helper functions to their own file

pub use sub_module::SomeType;
pub use other_module::OtherType;
pub use parent_type::ParentType;
pub use helpers::helper_function;
```


**my_module/sub_module.rs:**
```rust
pub struct SomeType {
    // implementation
}
```

**my_module/other_module.rs:**
```rust
pub struct OtherType {
    // implementation
}
```

### 1.5 Split Modules When Appropriate

A module should be split into submodules when any of the following conditions apply:

1. **Multiple major implementations** – the file contains several parallel implementations (e.g., multiple LLM providers, several API handlers).
2. **File size** – the file exceeds roughly **200‑300 lines**.
3. **Self‑documentation** – the file’s structure does not clearly convey its contents; the name alone should describe what is inside.

**Goal:** File names should document what they contain.

```text
// BAD - what is inside llm_types.rs? You must open it to discover
src/
  llm_types.rs    // 890 lines, 4 providers hidden inside

// GOOD - structure is self‑explanatory
src/
  llm_types.rs    // Only mod declarations + re‑exports
  llm_types/
    types.rs      // Common types
    qwen.rs       // Qwen provider
    openai.rs     // OpenAI provider
    deepseek.rs   // DeepSeek provider
    gpt_oss.rs   // GptOss provider
```

### 1.6 No Stub Modules

Never create stub modules merely to make the crate compile. Declare modules in parent files (e.g., `lib.rs`) **only when they are fully implemented**.

```rust
// BAD – declaring modules before they exist
// lib.rs
pub mod types;      // implemented
pub mod prompts;    // stub – empty struct
pub mod nodes;      // stub – empty struct
pub mod pipeline;   // stub – empty file
```

The above forces placeholder files that add confusion:
```rust
// prompts/classify.rs
pub struct ClassifyPrompt; // only a stub
```

```rust
// GOOD – declare only what is implemented
// lib.rs (Phase 2)
pub mod types;
pub mod context;

// lib.rs (Phase 3 – after prompts are implemented)
pub mod types;
pub mod context;
pub mod prompts;  // now fully implemented
```

**Why:** Stub modules create ambiguity – the code compiles but the functionality is missing. When the codebase is later compacted or reviewed, it is unclear what is a placeholder versus a real implementation.

**Rule:** Each development phase must result in a compilable crate containing **only fully implemented modules**.