# You AI Website Builder

**Describe it. We'll build it.**

Create beautiful websites through conversation. No design skills needed — just tell the AI what you want, show it sites you like, and publish with one click.

## Download & Install

### Mac
1. Download `You-AI-Website-Builder.dmg` from the [Releases page]
2. Open the DMG file
3. Drag the app to your Applications folder
4. Open the app from Applications (first time: right-click → Open)

### Windows
1. Download `You-AI-Website-Builder-Setup.exe` from the [Releases page]
2. Run the installer
3. Follow the installation wizard
4. Launch from the Start menu

---

## First-Time Setup

When you first open the app, a setup wizard walks you through connecting three services. Each one takes about 2 minutes.

### Step 1: Connect an AI

The AI is what designs your website through conversation. You have several options:

#### Option A: ChatGPT (recommended for beginners)
1. In the app, click **ChatGPT**
2. A browser window opens — sign into your OpenAI account
3. Click "Allow" to authorize
4. Your browser will show a URL in the address bar — **copy the entire URL**
5. Go back to the app and paste it into the text field
6. Click **Save Connection**

#### Option B: Claude
1. Go to [console.anthropic.com](https://console.anthropic.com)
2. Create an account or sign in
3. Click **API Keys** in the left menu
4. Click **Create Key** → copy the key
5. In the app, click **Claude** and paste your key

#### Option C: Gemini
1. In the app, click **Gemini**
2. A browser window opens — sign into your Google account
3. Click "Allow" to authorize
4. Copy the URL from your browser and paste it back into the app

#### Option D: Custom / Local AI
For advanced users who want to use their own model (Ollama, etc.):
1. Click **Custom / Local AI**
2. Enter your API endpoint and key

---

### Step 2: Connect GitHub

GitHub stores your website's files securely. Think of it as a cloud filing cabinet that also lets you undo changes.

**Don't have a GitHub account? Here's how to get one:**
1. Go to [github.com](https://github.com) and click **Sign Up**
2. Create a free account (no credit card needed)
3. Verify your email address

**Once you have an account:**
1. In the app, click **Connect GitHub**
2. A browser window opens — sign into GitHub
3. Click "Authorize" to give the app permission to create repositories for your sites
4. Copy the URL from your browser address bar
5. Paste it into the app and click **Save Connection**

**What permissions does it need?**
- Create repositories (where your site files are stored)
- Push code (to save your site updates)
- That's it — it cannot access your other repositories

---

### Step 3: Connect Cloudflare

Cloudflare is what makes your website visible to the world. It's fast, secure, and free for basic websites.

**Don't have a Cloudflare account? Here's how to get one:**
1. Go to [dash.cloudflare.com](https://dash.cloudflare.com)
2. Click **Sign Up** and create a free account
3. You don't need to add a domain right away — you'll get a free `.pages.dev` address

**Get your API token:**
1. Click your **profile icon** (top right) → **My Profile**
2. Click **API Tokens** in the left menu
3. Click **Create Token**
4. Find "Edit Cloudflare Pages" and click **Use template**
5. Under "Account Resources," select your account
6. Click **Continue to summary** → **Create Token**
7. **Copy the token** (you won't see it again!)
8. In the app, paste the token and click **Connect Cloudflare**

**What does this token allow?**
- Create and manage Pages projects (your websites)
- Deploy updates to your sites
- Nothing else — it can't access your DNS, email, or other Cloudflare settings

---

## Using the App

### Creating a Website

1. Click **New Website** from the dashboard
2. Give your project a name (this is just for you, not your site's title)
3. Start chatting with the AI!

The AI will ask you questions like:
- "What's this website for?"
- "Who will visit it?"
- "What feeling should visitors get?"

You don't need to know design terms — just answer naturally.

### Sharing Sites You Like

The **"Websites I Like"** panel lets you show the AI examples. Paste any URL and the app will analyze its colors, fonts, and style.

The AI will then ask what specifically you like about it — the colors? The layout? The overall vibe? This helps it understand your taste without you needing design vocabulary.

### Editing Your Site

Just keep chatting:
- "Make the header bigger"
- "I don't like that blue — can we try something warmer?"
- "Add a page for our team"

The preview updates in real-time on the right side of the screen.

### Publishing

When you're happy with how it looks:
1. Click the green **Publish** button
2. Your site goes live in seconds
3. You'll get a link like `your-project.pages.dev`

Want a custom domain (like `yourbusiness.com`)? You can add one later in your Cloudflare dashboard.

---

## Custom Domains

After publishing, you can connect your own domain:

1. Buy a domain from any registrar (Namecheap, Google Domains, Cloudflare Registrar, etc.)
2. In your [Cloudflare dashboard](https://dash.cloudflare.com), go to **Pages** → your project → **Custom domains**
3. Click **Set up a custom domain** and follow the instructions
4. If your domain isn't already on Cloudflare, you'll need to update your nameservers (Cloudflare walks you through this)

---

## Troubleshooting

**"Connection failed" when connecting a service**
- Make sure you copied the *entire* URL from your browser
- Try the connection again — authorization codes expire after a few minutes

**Preview isn't updating**
- The AI might still be generating — wait for the typing indicator to finish
- Try asking for a simpler change to test

**Publish failed**
- Check that both GitHub and Cloudflare are connected (Settings page)
- Your Cloudflare API token may have expired — reconnect it

**Site looks different than the preview**
- Give it 30 seconds — Cloudflare needs a moment to update
- Try clearing your browser cache or opening in a private window

---

## Privacy & Security

- **Your credentials** are stored in your operating system's secure keychain (macOS Keychain or Windows Credential Manager) — never in plain text files
- **Your site files** live in your GitHub account — you own them completely
- **The app** doesn't phone home, track usage, or share your data
- **You can disconnect** any service at any time from the Settings page

---

## Future Add-ons

Coming soon — plugins that add dynamic features to your site:
- **Contact Form** — visitors can send you messages
- **Blog** — write and publish posts
- **Analytics** — see who's visiting your site
- **E-commerce** — sell products or services

These use Cloudflare's database and serverless features, keeping everything fast and in your control.
