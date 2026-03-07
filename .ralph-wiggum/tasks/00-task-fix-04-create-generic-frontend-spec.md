# Task 0-Fix-04: Create GENERIC-FRONTEND.md Spec File

## Summary

Create the missing `specs/GENERIC-FRONTEND.md` file that is referenced by `specs/FRONTEND.md` for component, page, service, form, routing, and state management patterns.

## User Story

@user-stories/00-bootstrap-and-testing.md

## Issue Details

**Problem:** `specs/FRONTEND.md` references `specs/GENERIC-FRONTEND.md` for architectural patterns, but the file does not exist.

**Reference from FRONTEND.md (line 8):**
```
## Architecture
See @specs/GENERIC-FRONTEND.md for component, page, service, form, routing, and state management patterns.
```

## Required Changes

### Create specs/GENERIC-FRONTEND.md

Create a comprehensive spec file covering:

1. **Component Pattern**
   - How to structure Leptos components
   - Props pattern and typing
   - Component file organization
   - Reusable vs page-specific components

2. **Page Pattern**
   - How to structure page components
   - Layout composition
   - Page routing integration
   - Page-specific state management

3. **Service Pattern**
   - How to structure API service functions
   - Error handling patterns
   - Request/response typing
   - Service file organization

4. **Form Pattern**
   - Form state management with Leptos signals
   - Validation patterns
   - Submission handling
   - Error display

5. **Routing Pattern**
   - How to set up Leptos routing
   - Route configuration
   - Nested routes
   - Route guards (auth, etc.)

6. **State Management Pattern**
   - Signal usage guidelines
   - Resource usage for async data
   - Context API for shared state
   - localStorage integration (for auth tokens)

## Acceptance Criteria

- [ ] `specs/GENERIC-FRONTEND.md` file created
- [ ] All six sections documented (component, page, service, form, routing, state management)
- [ ] Patterns are consistent with GENERIC-BACKEND.md style
- [ ] Examples use Leptos 0.6 syntax
- [ ] Patterns align with FRONTEND.md requirements (e.g., localStorage for auth tokens)
- [ ] File is well-formatted and easy to read

## Verification

- [ ] File exists at `specs/GENERIC-FRONTEND.md`
- [ ] All required sections are present
- [ ] Content is comprehensive and actionable

## Related Files

- @.ralph-wiggum/reports/00-review-bootstrap.md
- @specs/FRONTEND.md
- @specs/GENERIC-BACKEND.md