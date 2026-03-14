# Phase 29: Provider Icon Branding - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace text initials and plain text with recognizable SVG icons for the 5 AI providers (OpenAI, Anthropic, Google Gemini, xAI, OpenRouter) in the onboarding provider selection step and the settings provider dropdown/selector. Icons must match the showcase site's provider cards.

</domain>

<decisions>
## Implementation Decisions

### Icon style & color
- Monochrome white icons — all icons rendered as white/opacity variants, consistent with the existing dark glass UI theme
- Icons sit inside circular background containers (bg-white/10) — same pattern as current text initials
- No icon appearance change on selection — the existing row highlight (bg-white/15) is sufficient for active state
- Use the exact same SVG paths as the showcase site (`showcase/index.html` lines 463-467)

### Icon placement & sizing
- Onboarding: 16x16 icon inside 32x32 circle (matches lucide-react sizing used elsewhere)
- Settings trigger button: 24x24 circle with proportionally-sized icon, layout is Circle | Name | ChevronDown
- Settings dropdown items: 24x24 circle with icon, consistent with trigger button

### Icon source & format
- React TSX components — each provider icon as a component accepting size/color props (like lucide-react icons)
- Location: `src/components/icons/` directory — dedicated icons directory
- SVG paths sourced from showcase site's provider cards section

### Settings dropdown restyle
- Dropdown trigger: 24x24 icon circle on left, provider name center, ChevronDown on right
- Dropdown items: 24x24 icon circle on left, provider name center, green checkmark on right (for providers with stored keys)
- Layout: icon circle | name | indicator (checkmark or selected highlight) — consistent across trigger and items

### Claude's Discretion
- Exact icon component API design (prop names, defaults)
- Whether to use a single ProviderIcon component with a provider ID prop or separate components per provider
- Internal viewBox normalization if showcase SVGs have different viewBox sizes
- Tailwind spacing/gap adjustments to accommodate the new icons

</decisions>

<specifics>
## Specific Ideas

- Icons must match the showcase site's provider cards exactly — SVG paths from `showcase/index.html` lines 463-467
- The xAI icon has a non-standard viewBox (3 9 908 1007) — needs normalization to work at 16x16 and the proportional size inside 24x24 circles
- OpenRouter icon reference: same as showcase website

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `PROVIDERS` array (`src/store/index.ts:4-10`): Has `{id, name}` — icons can be mapped by provider ID
- `PROVIDER_INITIALS` (`StepProviderSelect.tsx:9-15`): Will be replaced by icon components
- `lucide-react` icons: Already used for utility icons (Eye, Check, ChevronDown, Loader2) — provider icons should follow similar component API

### Established Patterns
- Tailwind CSS for all styling — white/opacity theme (`bg-white/10`, `text-white/70`, etc.)
- Inline className arrays joined with `.join(" ")` for conditional styling
- Components import from `@/store` for PROVIDERS list

### Integration Points
- `StepProviderSelect.tsx`: Replace `PROVIDER_INITIALS[provider.id]` text with icon component inside the existing 32x32 circle div
- `AccountTab.tsx` lines 171-202: Provider dropdown trigger and items — add icon circle before provider name text
- `PROVIDERS` array in store: May optionally be extended with icon component reference, or icons can be looked up by ID in a separate map

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 29-provider-icon-branding*
*Context gathered: 2026-03-11*
