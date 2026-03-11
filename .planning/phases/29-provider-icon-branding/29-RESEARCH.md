# Phase 29: Provider Icon Branding - Research

**Researched:** 2026-03-11
**Domain:** React SVG icon components, UI branding
**Confidence:** HIGH

## Summary

This phase replaces text initials with inline SVG icons for 5 AI providers (OpenAI, Anthropic, Google Gemini, xAI, OpenRouter) in two locations: the onboarding provider selection step and the settings provider dropdown. The SVG paths are already available in the project's own `showcase/index.html` (lines 463-467), so no external icon library or asset download is needed.

The implementation is straightforward: create React TSX icon components from existing SVG paths, then integrate them into `StepProviderSelect.tsx` and `AccountTab.tsx`. The xAI icon uses a non-standard viewBox (`3 9 908 1007`) that requires normalization for consistent sizing. All other icons use standard 24x24 viewBoxes.

**Primary recommendation:** Create a single `ProviderIcon` component that maps provider IDs to SVG paths, accepting `size` and `className` props consistent with lucide-react's API pattern.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Monochrome white icons -- all icons rendered as white/opacity variants, consistent with the existing dark glass UI theme
- Icons sit inside circular background containers (bg-white/10) -- same pattern as current text initials
- No icon appearance change on selection -- the existing row highlight (bg-white/15) is sufficient for active state
- Use the exact same SVG paths as the showcase site (`showcase/index.html` lines 463-467)
- Onboarding: 16x16 icon inside 32x32 circle (matches lucide-react sizing used elsewhere)
- Settings trigger button: 24x24 circle with proportionally-sized icon, layout is Circle | Name | ChevronDown
- Settings dropdown items: 24x24 circle with icon, consistent with trigger button
- React TSX components -- each provider icon as a component accepting size/color props (like lucide-react icons)
- Location: `src/components/icons/` directory -- dedicated icons directory
- SVG paths sourced from showcase site's provider cards section
- Dropdown trigger: 24x24 icon circle on left, provider name center, ChevronDown on right
- Dropdown items: 24x24 icon circle on left, provider name center, green checkmark on right (for providers with stored keys)

### Claude's Discretion
- Exact icon component API design (prop names, defaults)
- Whether to use a single ProviderIcon component with a provider ID prop or separate components per provider
- Internal viewBox normalization if showcase SVGs have different viewBox sizes
- Tailwind spacing/gap adjustments to accommodate the new icons

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| ICON-01 | Onboarding provider selection shows inline SVG icons (OpenAI, Anthropic, Gemini, xAI, OpenRouter) matching showcase site provider cards | SVG paths extracted from showcase/index.html lines 463-467; StepProviderSelect.tsx integration point identified at line 57-59 |
| ICON-02 | Settings provider selector shows same SVG icons next to provider names | AccountTab.tsx integration points: trigger button (line 172-179) and dropdown items (line 182-199) |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | 19.1.0 | Component framework | Already in project |
| TypeScript | 5.8.3 | Type safety | Already in project |
| Tailwind CSS | 4.2.0 | Styling | Already in project, all styling uses Tailwind |
| lucide-react | 0.575.0 | Existing icon library | Reference for icon component API pattern |

### Supporting
No new libraries needed. All SVG paths come from existing project assets.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom SVG components | react-icons or simple-icons | Unnecessary dependency for 5 icons already in the codebase |
| Single ProviderIcon component | Separate component per provider | Single component is cleaner; maps provider ID to path data |

## Architecture Patterns

### Recommended Project Structure
```
src/
  components/
    icons/
      ProviderIcon.tsx    # Single component mapping provider ID to SVG
      index.ts            # Re-export
    Onboarding/
      StepProviderSelect.tsx  # Modified to use ProviderIcon
    Settings/
      AccountTab.tsx          # Modified to use ProviderIcon
```

### Pattern 1: Unified Provider Icon Component
**What:** A single `ProviderIcon` component that accepts a provider ID string and renders the corresponding SVG.
**When to use:** When you have a fixed set of icons mapped by an identifier (the provider ID from the PROVIDERS array).
**Example:**
```typescript
// src/components/icons/ProviderIcon.tsx
import { type SVGProps } from "react";

interface ProviderIconProps extends SVGProps<SVGSVGElement> {
  provider: string;
  size?: number;
}

// Each entry: [viewBox, pathData]
const PROVIDER_PATHS: Record<string, [string, string]> = {
  openai: ["0 0 24 24", "M22.282 9.821a5.985..."],
  anthropic: ["0 0 24 24", "M17.3041 3.541h-3.6718..."],
  gemini: ["0 0 24 24", "M11.04 19.32Q12..."],
  xai: ["0 0 24 24", "M827.76 200.32..."],  // normalized viewBox
  openrouter: ["0 0 24 24", "M16.778 1.844v1.919..."],
};

export function ProviderIcon({ provider, size = 16, ...rest }: ProviderIconProps) {
  const entry = PROVIDER_PATHS[provider];
  if (!entry) return null;
  const [viewBox, d] = entry;
  return (
    <svg width={size} height={size} viewBox={viewBox} fill="currentColor" {...rest}>
      <path d={d} />
    </svg>
  );
}
```

### Pattern 2: Icon in Circular Container (Onboarding)
**What:** The icon wrapped in a circular div, replacing the text initial.
**Example:**
```tsx
{/* Replace the text initial with ProviderIcon */}
<div className="w-8 h-8 rounded-full bg-white/10 flex items-center justify-center">
  <ProviderIcon provider={provider.id} size={16} className="text-white/70" />
</div>
```

### Pattern 3: Icon in Circular Container (Settings - smaller)
**What:** 24x24 circle for settings trigger and dropdown items.
**Example:**
```tsx
<div className="w-6 h-6 rounded-full bg-white/10 flex items-center justify-center shrink-0">
  <ProviderIcon provider={providerId} size={12} className="text-white/70" />
</div>
```

### Anti-Patterns to Avoid
- **Separate files per icon:** 5 separate component files for 5 simple SVG paths is over-engineering. A single component with a path map is cleaner.
- **Importing SVG as assets:** Don't use Vite's SVG import; inline SVG via JSX gives full control over size, color via `currentColor`, and no extra HTTP requests.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| viewBox normalization | Manual coordinate math | Normalize xAI viewBox to 0 0 24 24 once | Only one icon (xAI) has non-standard viewBox; normalize the path data at component definition time |

**Key insight:** The xAI icon has viewBox `3 9 908 1007`. Rather than handling different viewBoxes at runtime, normalize it to `0 0 24 24` by keeping the original viewBox string in the path map and letting the SVG element handle scaling. The SVG `viewBox` attribute handles this automatically -- just pass the original viewBox and set width/height to desired pixel size.

## Common Pitfalls

### Pitfall 1: xAI ViewBox Scaling
**What goes wrong:** The xAI icon uses viewBox `3 9 908 1007` (not square). At small sizes (16x16, 12x12), the icon may look distorted or off-center because the viewBox is not 1:1 aspect ratio.
**Why it happens:** The source SVG has a 908:1007 aspect ratio (roughly 0.9:1).
**How to avoid:** Keep the original viewBox and let SVG scaling handle it. The ~10% aspect ratio difference is negligible at 16px. Alternatively, use `preserveAspectRatio="xMidYMid meet"` (the default) to center the icon within the square bounds.
**Warning signs:** Icon looks squished or has unexpected padding on one axis.

### Pitfall 2: Color Inheritance
**What goes wrong:** Icons don't change color when container text color changes.
**Why it happens:** Using hardcoded `fill="#fff"` instead of `fill="currentColor"`.
**How to avoid:** Always use `fill="currentColor"` so the icon inherits `text-*` color from its parent/className.

### Pitfall 3: Shrink in Flex Containers
**What goes wrong:** Icon circle containers shrink when provider names are long.
**Why it happens:** Flex items shrink by default.
**How to avoid:** Add `shrink-0` class to the circle container div.

### Pitfall 4: Multi-path SVGs
**What goes wrong:** Some provider SVGs have multiple `<path>` elements but only one is rendered.
**Why it happens:** The component only renders a single `<path>`.
**How to avoid:** Check each SVG from the showcase. The Anthropic icon has 2 paths (the "A" body and the separate stroke). Store path data as an array of strings and map them.

## Code Examples

### Exact SVG Path Data from Showcase (lines 463-467)

**OpenAI** (viewBox: `0 0 24 24`, single path):
```
M22.282 9.821a5.985 5.985 0 0 0-.516-4.91 6.046 6.046 0 0 0-6.51-2.9A6.065 6.065 0 0 0 4.981 4.18a5.985 5.985 0 0 0-3.998 2.9 6.046 6.046 0 0 0 .743 7.097 5.98 5.98 0 0 0 .51 4.911 6.051 6.051 0 0 0 6.515 2.9A5.985 5.985 0 0 0 13.26 24a6.056 6.056 0 0 0 5.772-4.206 5.99 5.99 0 0 0 3.997-2.9 6.056 6.056 0 0 0-.747-7.073zM13.26 22.43a4.476 4.476 0 0 1-2.876-1.04l.141-.081 4.779-2.758a.795.795 0 0 0 .392-.681v-6.737l2.02 1.168a.071.071 0 0 1 .038.052v5.583a4.504 4.504 0 0 1-4.494 4.494zM3.6 18.304a4.47 4.47 0 0 1-.535-3.014l.142.085 4.783 2.759a.771.771 0 0 0 .78 0l5.843-3.369v2.332a.08.08 0 0 1-.033.062L9.74 19.95a4.5 4.5 0 0 1-6.14-1.646zM2.34 7.896a4.485 4.485 0 0 1 2.366-1.973V11.6a.766.766 0 0 0 .388.676l5.815 3.355-2.02 1.168a.076.076 0 0 1-.071 0l-4.83-2.786A4.504 4.504 0 0 1 2.34 7.872zm16.597 3.855l-5.833-3.387L15.119 7.2a.076.076 0 0 1 .071 0l4.83 2.791a4.494 4.494 0 0 1-.676 8.105v-5.678a.79.79 0 0 0-.407-.667zm2.01-3.023l-.141-.085-4.774-2.782a.776.776 0 0 0-.785 0L9.409 9.23V6.897a.066.066 0 0 1 .028-.061l4.83-2.787a4.5 4.5 0 0 1 6.68 4.66zm-12.64 4.135l-2.02-1.164a.08.08 0 0 1-.038-.057V6.075a4.5 4.5 0 0 1 7.375-3.453l-.142.08L8.704 5.46a.795.795 0 0 0-.393.681zm1.097-2.365l2.602-1.5 2.607 1.5v2.999l-2.597 1.5-2.607-1.5z
```

**Anthropic** (viewBox: `0 0 24 24`, single path with two subpaths):
```
M17.3041 3.541h-3.6718l6.696 16.918H24Zm-10.6082 0L0 20.459h3.7442l1.3693-3.5527h7.0052l1.3693 3.5528h3.7442L10.5363 3.5409Zm-.3712 10.2232 2.2914-5.9456 2.2914 5.9456Z
```

**Google Gemini** (viewBox: `0 0 24 24`, single path):
```
M11.04 19.32Q12 21.51 12 24q0-2.49.93-4.68.96-2.19 2.58-3.81t3.81-2.55Q21.51 12 24 12q-2.49 0-4.68-.93a12.3 12.3 0 0 1-3.81-2.58 12.3 12.3 0 0 1-2.58-3.81Q12 2.49 12 0q0 2.49-.96 4.68-.93 2.19-2.55 3.81a12.3 12.3 0 0 1-3.81 2.58Q2.49 12 0 12q2.49 0 4.68.96 2.19.93 3.81 2.55t2.55 3.81
```

**xAI** (viewBox: `3 9 908 1007`, multiple paths):
```
Path 1: M827.76 200.32L745.02 318.5l-.01 348.75L745 1016h166.002l-.251-466.93-.251-466.93-82.74 118.18
Path 2: M3.167 365.816c.183.449 102.641 146.926 227.684 325.505l227.35 324.689 100.486-.255 100.485-.255-227.675-325.25L203.822 365H103.328c-55.272 0-100.345.367-100.161.816
Path 3: M801 8.787l-93.5.286-174 248.569c-95.7 136.713-174.388 249.381-174.863 250.374-.686 1.436 9.177 16.156 48.345 72.144 27.065 38.687 49.728 70.88 50.363 71.54 1.033 1.073 37.65-50.44 128.994-181.471 2.112-3.029 54.285-77.557 115.941-165.618C763.937 216.55 815.619 142.7 817.13 140.5c1.51-2.2 22.768-32.575 47.238-67.5L908.86 9.5l-7.18-.5c-3.949-.275-49.255-.371-100.68-.213
Path 4: M103.273 872.277L3.047 1015.5l100.726.21 100.727.21 45.206-64.71c24.864-35.591 47.462-67.909 50.219-71.819l5.013-7.109-49.972-71.391c-27.484-39.265-50.308-71.491-50.719-71.614-.411-.122-45.849 64.228-100.974 143
```

**OpenRouter** (viewBox: `0 0 24 24`, single path):
```
M16.778 1.844v1.919q-.569-.026-1.138-.032-.708-.008-1.415.037c-1.93.126-4.023.728-6.149 2.237-2.911 2.066-2.731 1.95-4.14 2.75-.396.223-1.342.574-2.185.798-.841.225-1.753.333-1.751.333v4.229s.768.108 1.61.333c.842.224 1.789.575 2.185.799 1.41.798 1.228.683 4.14 2.75 2.126 1.509 4.22 2.11 6.148 2.236.88.058 1.716.041 2.555.005v1.918l7.222-4.168-7.222-4.17v2.176c-.86.038-1.611.065-2.278.021-1.364-.09-2.417-.357-3.979-1.465-2.244-1.593-2.866-2.027-3.68-2.508.889-.518 1.449-.906 3.822-2.59 1.56-1.109 2.614-1.377 3.978-1.466.667-.044 1.418-.017 2.278.02v2.176L24 6.014Z
```

### Component Implementation Pattern (recommended)

```typescript
// src/components/icons/ProviderIcon.tsx
import { type SVGProps } from "react";

interface ProviderIconProps extends SVGProps<SVGSVGElement> {
  provider: string;
  size?: number;
}

// [viewBox, pathData[]]
const ICON_DATA: Record<string, { viewBox: string; paths: string[] }> = {
  openai: {
    viewBox: "0 0 24 24",
    paths: ["M22.282 9.821a5.985..."],  // full path from above
  },
  anthropic: {
    viewBox: "0 0 24 24",
    paths: ["M17.3041 3.541h-3.6718..."],  // single path with subpaths (M...Z moves)
  },
  gemini: {
    viewBox: "0 0 24 24",
    paths: ["M11.04 19.32Q12..."],
  },
  xai: {
    viewBox: "3 9 908 1007",  // keep original viewBox; SVG handles scaling
    paths: [
      "M827.76 200.32L745.02...",
      "M3.167 365.816c.183...",
      "M801 8.787l-93.5...",
      "M103.273 872.277L3.047...",
    ],
  },
  openrouter: {
    viewBox: "0 0 24 24",
    paths: ["M16.778 1.844v1.919..."],
  },
};

export function ProviderIcon({ provider, size = 16, className, ...rest }: ProviderIconProps) {
  const data = ICON_DATA[provider];
  if (!data) return null;
  return (
    <svg
      width={size}
      height={size}
      viewBox={data.viewBox}
      fill="currentColor"
      className={className}
      {...rest}
    >
      {data.paths.map((d, i) => (
        <path key={i} d={d} />
      ))}
    </svg>
  );
}
```

### Integration in StepProviderSelect.tsx

```tsx
// Replace line 57-59:
// Before:
<div className="w-8 h-8 rounded-full bg-white/10 flex items-center justify-center text-xs font-medium text-white/70">
  {PROVIDER_INITIALS[provider.id] ?? "?"}
</div>

// After:
<div className="w-8 h-8 rounded-full bg-white/10 flex items-center justify-center">
  <ProviderIcon provider={provider.id} size={16} className="text-white/70" />
</div>
```

### Integration in AccountTab.tsx - Trigger Button

```tsx
// Replace line 175-178:
// Before:
<button ... className="w-full flex items-center justify-between ...">
  <span>{currentProviderName}</span>
  <ChevronDown size={14} className="text-white/40" />
</button>

// After:
<button ... className="w-full flex items-center gap-2 ...">
  <div className="w-6 h-6 rounded-full bg-white/10 flex items-center justify-center shrink-0">
    <ProviderIcon provider={selectedProvider} size={12} className="text-white/70" />
  </div>
  <span className="flex-1 text-left">{currentProviderName}</span>
  <ChevronDown size={14} className="text-white/40" />
</button>
```

### Integration in AccountTab.tsx - Dropdown Items

```tsx
// Replace lines 183-198:
<button ... className="w-full flex items-center gap-2 px-3 py-2 ...">
  <div className="w-6 h-6 rounded-full bg-white/10 flex items-center justify-center shrink-0">
    <ProviderIcon provider={p.id} size={12} className="text-white/70" />
  </div>
  <span className="flex-1 text-left">{p.name}</span>
  {providerHasKey[p.id] && (
    <Check size={14} className="text-green-400" />
  )}
</button>
```

## State of the Art

No changes in the ecosystem affect this phase. This is a pure UI task using standard inline SVG in React -- a stable, well-understood pattern.

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Text initials in circles | SVG icons in circles | This phase | Visual polish, brand recognition |

## Open Questions

1. **xAI icon at very small sizes (12px)**
   - What we know: The viewBox is non-square (908x1007), so the rendered icon will be slightly narrower than tall within the square bounds
   - What's unclear: Whether this is visually noticeable at 12px inside a 24px circle
   - Recommendation: Implement with original viewBox first, verify visually, adjust only if needed

## Sources

### Primary (HIGH confidence)
- `showcase/index.html` lines 463-467 -- exact SVG paths for all 5 provider icons
- `src/components/Onboarding/StepProviderSelect.tsx` -- current implementation with text initials
- `src/components/Settings/AccountTab.tsx` -- current settings dropdown implementation
- `src/store/index.ts` -- PROVIDERS array with provider IDs

### Secondary (MEDIUM confidence)
- lucide-react icon API pattern -- referenced for component prop design consistency

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new libraries, all existing project dependencies
- Architecture: HIGH -- straightforward SVG-to-component conversion with clear integration points
- Pitfalls: HIGH -- well-understood domain (inline SVG in React), only xAI viewBox is notable

**Research date:** 2026-03-11
**Valid until:** Indefinite -- SVG icon components are a stable pattern
