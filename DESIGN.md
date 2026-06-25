# You AI Website Builder

## Overview

A desktop app (Windows + Mac) that lets non-technical users create and deploy websites through conversation with an AI. No design knowledge required — the AI acts as the designer by learning what the user likes.

## Core Principles

1. **No jargon.** The user never sees "deploy," "commit," or "repository." They see "publish," "save," and "my site."
2. **Conversation over configuration.** Instead of forms and templates, the AI asks questions and builds iteratively.
3. **Own your stuff.** The site lives on the user's GitHub, deploys to their Cloudflare. They own everything.
4. **Bring your own AI.** Connect ChatGPT, Claude, Gemini, local models, or work with us to hook up something custom.

## Target Audience

Non-designers who are comfortable installing apps but not running terminals. They might not know what "good design" means in technical terms, but they know what they like when they see it.

## User Flow

### First Launch — Setup Wizard

Step-by-step, one thing at a time. Each step has plain-language instructions with screenshots.

1. **Connect an AI** — Pick from supported LLMs (ChatGPT, Claude, Gemini, etc.)
   - OAuth flow: click a button → browser opens → authorize → copy the redirect URL → paste it back
   - API key flow: for providers that use keys, guided instructions to get one
   - "Need help?" option to contact us for custom LLM hookup

2. **Connect GitHub** — Where your site files live
   - OAuth flow (same copy-paste-URL pattern)
   - Instructions: "GitHub is like a filing cabinet for your website. Here's how to get a free account..."

3. **Connect Cloudflare** — Where your site goes live
   - API token flow with step-by-step instructions
   - "Cloudflare is what makes your website visible on the internet..."
   - Guide covers: create account, get API token, (optional) connect a domain

### Creating a Website

1. **New Project** — User clicks "Create a new website"
2. **Conversation starts** — AI introduces itself:
   > "Hi! I'm going to help you build a website. Let's start simple — what's the website for? A business? A personal page? Something else?"
3. **"Websites I Like" panel** — Collapsible side section
   - User can paste URLs at any time during the conversation
   - App scrapes the site (screenshot, color palette, layout analysis, font detection)
   - AI references these: "I see you like [site] — is it the clean layout or the color scheme that appeals to you?"
4. **Iterative building** — AI generates HTML/CSS, shows live preview in-app
   - User can say things like "make it more blue" or "I don't like that font"
   - Preview updates in real-time
5. **Content entry** — AI asks for their actual content (text, images, logo)
   - Can work with placeholder content first, fill in real stuff later
6. **Publish** — One button. Site goes live.
   - Behind the scenes: commit to GitHub → Cloudflare Pages auto-deploys

### Editing an Existing Site

- Open project → conversation continues where they left off
- "I want to add a new page" / "Change the header image" / etc.
- Same conversational flow, same one-click publish

## Technical Architecture

### Stack

- **Desktop framework:** Tauri (small install, native feel, cross-platform)
- **Frontend:** React + Tailwind (inside Tauri webview)
- **Backend (Rust side):** File management, Git operations, API calls to Cloudflare
- **Site output:** Pure HTML + CSS (no build step, directly deployable to Cloudflare Pages)

### LLM Integration Layer

Abstract interface that supports multiple providers:

```
┌────────────────────────────────┐
│        LLM Adapter Layer       │
├────────────────────────────────┤
│ ChatGPT (OAuth)                │
│ Claude (API key or OAuth)      │
│ Gemini (API key or OAuth)      │
│ OpenAI-compatible (custom URL) │
│ Local models (Ollama, etc.)    │
└────────────────────────────────┘
```

Each adapter handles:
- Authentication (OAuth redirect-paste flow or API key)
- Message formatting (each provider has its own format)
- Streaming responses (for live preview updates)
- System prompts (the "designer persona" that knows how to ask the right questions)

### OAuth Flow (Copy-Paste Pattern)

1. App generates auth URL with state parameter
2. Opens system browser to that URL
3. User authorizes in browser
4. Browser redirects to a URL (could be localhost or a custom scheme)
5. User copies the full redirect URL from the browser address bar
6. Pastes into the app's input field
7. App extracts the auth code, exchanges for tokens, stores securely in OS keychain

### Site Inspiration / "Websites I Like"

When a user pastes a URL:
1. App fetches the page (via Tauri's HTTP client)
2. Takes a screenshot (headless browser or screenshot API)
3. Extracts: dominant colors, font families, layout structure (header/hero/grid/footer pattern), spacing density
4. Stores this as a "mood" reference the LLM can use in its system context
5. LLM asks targeted follow-up questions about what specifically appeals to them

### File Structure (per project)

```
~/You AI Sites/my-business-site/
├── index.html
├── about.html
├── contact.html
├── css/
│   └── style.css
├── images/
│   └── (user uploads)
├── .you-ai/
│   ├── project.json      (metadata, LLM conversation history)
│   ├── inspirations.json (scraped site data)
│   └── settings.json    (project-specific overrides)
└── .github/
    └── (auto-configured for Pages deploy)
```

### Cloudflare Pages Integration

- Auto-creates a Pages project linked to the GitHub repo
- Configures custom domain if user has one
- No build step needed (static HTML/CSS served directly)
- Future: Workers/D1 plugins for dynamic features

### Plugin / Add-on System (Future)

Extensible architecture for Cloudflare capabilities:

- **Contact form** — Cloudflare Worker + D1 to store submissions
- **Blog** — D1-backed posts with a simple admin interface
- **Analytics** — Cloudflare Web Analytics integration
- **E-commerce** — Product catalog in D1, Stripe checkout via Worker
- **Authentication** — Cloudflare Access for gated content

Plugins generate the necessary Worker code, D1 schemas, and wrangler config alongside the HTML/CSS.

## App Screens

1. **Welcome / Setup Wizard** — Clean, one-step-at-a-time
2. **Dashboard** — List of user's site projects, status, last edited
3. **Editor** — Split view: chat on left, live preview on right
4. **Inspirations panel** — Collapsible, shows scraped sites with thumbnails
5. **Settings** — Manage connections (LLMs, GitHub, Cloudflare), preferences
6. **Publish confirmation** — Simple "Your site is live at [url]" with link

## Naming and Branding

- **App name:** You AI Website Builder
- **Tone:** Friendly, encouraging, non-technical
- **Tagline idea:** "Describe it. We'll build it."

## Installation

- **Mac:** DMG download, drag to Applications
- **Windows:** MSI/NSIS installer, standard install wizard
- Size target: under 20MB (Tauri advantage)
- No prerequisites (no Node.js, no Git CLI needed — all bundled or handled via APIs)

## Security

- All credentials stored in OS keychain (macOS Keychain, Windows Credential Manager)
- OAuth tokens refreshed automatically
- No credentials in plaintext files
- Git operations via GitHub API (no local Git needed)
- All network calls from the app, no browser extensions or local servers needed
