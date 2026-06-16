---
name: Keyboard Locker
colors:
  surface: '#f8f9ff'
  surface-dim: '#cbdbf5'
  surface-bright: '#f8f9ff'
  surface-container-lowest: '#ffffff'
  surface-container-low: '#eff4ff'
  surface-container: '#e5eeff'
  surface-container-high: '#dce9ff'
  surface-container-highest: '#d3e4fe'
  on-surface: '#0b1c30'
  on-surface-variant: '#434655'
  inverse-surface: '#213145'
  inverse-on-surface: '#eaf1ff'
  outline: '#737686'
  outline-variant: '#c3c6d7'
  surface-tint: '#0053db'
  primary: '#004ac6'
  on-primary: '#ffffff'
  primary-container: '#2563eb'
  on-primary-container: '#eeefff'
  inverse-primary: '#b4c5ff'
  secondary: '#855300'
  on-secondary: '#ffffff'
  secondary-container: '#fea619'
  on-secondary-container: '#684000'
  tertiary: '#943700'
  on-tertiary: '#ffffff'
  tertiary-container: '#bc4800'
  on-tertiary-container: '#ffede6'
  error: '#ba1a1a'
  on-error: '#ffffff'
  error-container: '#ffdad6'
  on-error-container: '#93000a'
  primary-fixed: '#dbe1ff'
  primary-fixed-dim: '#b4c5ff'
  on-primary-fixed: '#00174b'
  on-primary-fixed-variant: '#003ea8'
  secondary-fixed: '#ffddb8'
  secondary-fixed-dim: '#ffb95f'
  on-secondary-fixed: '#2a1700'
  on-secondary-fixed-variant: '#653e00'
  tertiary-fixed: '#ffdbcd'
  tertiary-fixed-dim: '#ffb596'
  on-tertiary-fixed: '#360f00'
  on-tertiary-fixed-variant: '#7d2d00'
  background: '#f8f9ff'
  on-background: '#0b1c30'
  surface-variant: '#d3e4fe'
typography:
  display:
    fontFamily: Inter
    fontSize: 32px
    fontWeight: '700'
    lineHeight: 40px
    letterSpacing: -0.02em
  headline-lg:
    fontFamily: Inter
    fontSize: 24px
    fontWeight: '600'
    lineHeight: 32px
    letterSpacing: -0.01em
  headline-md:
    fontFamily: Inter
    fontSize: 20px
    fontWeight: '600'
    lineHeight: 28px
  body-lg:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  body-md:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  label-md:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: '500'
    lineHeight: 16px
  label-sm:
    fontFamily: JetBrains Mono
    fontSize: 11px
    fontWeight: '500'
    lineHeight: 14px
    letterSpacing: 0.05em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  base: 4px
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 32px
  container-padding: 24px
  gutter: 16px
---

## Brand & Style
The design system is engineered for a utility-first desktop experience that prioritizes reliability and precision. The brand personality is **Protective, Logical, and Unobtrusive**. It serves professionals who need a tool that feels like a native extension of their operating system rather than a distracting third-party application.

The design style follows a **Modern Corporate** aesthetic with a focus on **Systematic Functionalism**. It utilizes clean lines, generous negative space for focus, and a clear hierarchy of information. The UI avoids unnecessary ornamentation to ensure the user feels in total control of their hardware security.

## Colors
The palette is rooted in safety and clarity. 
- **Safety Blue (#2563EB)** is the primary driver for interaction, signaling trust and system-level actions. 
- **Lock Orange (#F59E0B)** is used sparingly for high-alert active states, such as when a specific hardware lockout is engaged.
- **Functional Status:** We use a semantic color model where **Green (#10B981)** signifies an "Unlocked/Safe" state and **Red (#EF4444)** indicates a "Locked/Secured" state.
- **Neutrals:** A range of Cool Grays provides the scaffolding for the interface, maintaining a professional "utility" feel that blends with modern desktop environments.

## Typography
The typography system prioritizes legibility and technical precision. **Inter** is the workhorse font for the entire interface, providing a neutral and modern sans-serif feel that scales perfectly from small labels to large status headers.

For technical data—such as key codes, hardware IDs, or shortcuts—this design system introduces **JetBrains Mono**. This monospaced font provides the "technical" anchor needed for a hardware utility tool, ensuring that character-specific data is unmistakable. Use all-caps with increased tracking for `label-sm` to denote secondary system metadata.

## Layout & Spacing
The layout follows a **Fixed Grid** philosophy suitable for a desktop utility window. The default application width is optimized for a 800px to 1020px range, utilizing a 12-column grid within the main content area.

Spacing is built on a **4px baseline**. Most components use 16px (`md`) for internal padding and 24px (`lg`) for section margins. This creates a structured, "airy" but dense enough environment to feel like a high-productivity tool. Layouts should be grouped into logical panels with clear vertical alignment to guide the eye through configuration steps.

## Elevation & Depth
This design system uses **Tonal Layers** and **Low-Contrast Outlines** to communicate hierarchy. 
- **Level 0 (Background):** The main application canvas uses a subtle off-white (#F8FAFC).
- **Level 1 (Panels):** Content containers use a pure white background with a 1px border (#E2E8F0). No shadows are used for standard panels to maintain a "flat" professional look.
- **Level 2 (Interactive/Floating):** Modals or active dropdowns utilize a very soft, high-diffusion shadow (0px 4px 20px rgba(0,0,0,0.05)) to suggest they are temporarily above the workspace.

Depth is primarily signaled through color shifts (darkening background on hover) rather than heavy shadows, ensuring the app feels "lightweight" and integrated into the OS.

## Shapes
The shape language is **Soft and Precise**. A 0.25rem (4px) base radius is applied to most UI elements (buttons, inputs, panels). This small radius softens the "brutalist" edge of a utility tool while maintaining a disciplined, professional appearance. 

Status indicators and "Lock" toggles may use full pill-rounding (rounded-xl) to distinguish them as high-priority interactive components distinct from the structural grid.

## Components
- **Action Buttons:** Primary buttons use the Safety Blue background with white text. High-alert "Lock Now" buttons should use a heavy weight and larger horizontal padding.
- **Toggle Switches:** Used for global states (e.g., "Block Win Key"). Toggles should use a distinct "Safe/Locked" color transition from Neutral to Primary or Lock Orange.
- **Status Indicators:** Use a combination of an icon (Lock/Unlock) and a colored pill. Never rely on color alone; include text labels within the indicator for accessibility.
- **Data Tables:** For shortcut lists or blocked key logs, use a stripped-back table style with horizontal separators only. Headers use `label-sm` with a light gray background.
- **Input Fields:** Precise 1px borders. When focused, the border should transition to Safety Blue with a 2px outer "halo" of 10% opacity blue.
- **Cards:** Use for grouping hardware settings. They should have a subtle border and no shadow, appearing "embedded" in the interface.