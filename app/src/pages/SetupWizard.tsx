import { useState } from 'react'
import { Globe, GitBranch, Cloud, Bot, ChevronRight, Check, ExternalLink, ClipboardPaste } from 'lucide-react'

interface Props {
  onComplete: () => void
}

type Step = 'welcome' | 'llm' | 'github' | 'cloudflare' | 'done'

export function SetupWizard({ onComplete }: Props) {
  const [step, setStep] = useState<Step>('welcome')
  const [llmProvider, setLlmProvider] = useState<string>('')
  const [oauthUrl, setOauthUrl] = useState<string>('')
  const [redirectUrl, setRedirectUrl] = useState<string>('')
  const [apiKey, setApiKey] = useState<string>('')
  const [connecting, setConnecting] = useState(false)
  const [error, setError] = useState<string>('')
  const [, setConnectedServices] = useState({
    llm: false,
    github: false,
    cloudflare: false,
  })

  const steps: { id: Step; label: string; icon: any }[] = [
    { id: 'welcome', label: 'Welcome', icon: Globe },
    { id: 'llm', label: 'Connect AI', icon: Bot },
    { id: 'github', label: 'Connect GitHub', icon: GitBranch },
    { id: 'cloudflare', label: 'Connect Cloudflare', icon: Cloud },
    { id: 'done', label: 'All Set!', icon: Check },
  ]

  async function handleOAuthConnect(provider: string) {
    setError('')
    setConnecting(true)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke('generate_oauth_url', {
        provider,
        clientId: 'YOUR_CLIENT_ID', // Will be configured per-user
      }) as any

      setOauthUrl(result.url)

      // Open in system browser
      const { open } = await import('@tauri-apps/plugin-shell')
      await open(result.url)
    } catch (e: any) {
      setError(e.toString())
    }
    setConnecting(false)
  }

  async function handlePasteRedirectUrl(provider: string) {
    setError('')
    setConnecting(true)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('exchange_oauth_code', {
        request: {
          provider,
          redirectUrl: redirectUrl,
          clientId: 'YOUR_CLIENT_ID',
          clientSecret: 'YOUR_CLIENT_SECRET',
        }
      })

      setConnectedServices(prev => ({ ...prev, [provider === 'chatgpt' ? 'llm' : provider]: true }))
      setRedirectUrl('')
      setOauthUrl('')

      // Auto advance
      if (step === 'llm') setStep('github')
      else if (step === 'github') setStep('cloudflare')
      else if (step === 'cloudflare') setStep('done')
    } catch (e: any) {
      setError(e.toString())
    }
    setConnecting(false)
  }

  async function handleApiKeyConnect(provider: string) {
    setError('')
    setConnecting(true)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('save_credential', {
        request: {
          provider,
          token: apiKey,
          tokenType: 'api_key',
        }
      })

      setConnectedServices(prev => ({ ...prev, llm: true }))
      setApiKey('')
      setStep('github')
    } catch (e: any) {
      setError(e.toString())
    }
    setConnecting(false)
  }

  return (
    <div className="min-h-screen flex">
      {/* Sidebar - Step Progress */}
      <div className="w-64 bg-white border-r border-gray-200 p-6">
        <h2 className="text-lg font-semibold mb-6 text-indigo-600">Setup</h2>
        <nav className="space-y-2">
          {steps.map((s, i) => {
            const Icon = s.icon
            const isActive = s.id === step
            const isPast = steps.findIndex(x => x.id === step) > i
            return (
              <div
                key={s.id}
                className={`flex items-center gap-3 px-3 py-2 rounded-lg text-sm ${
                  isActive ? 'bg-indigo-50 text-indigo-700 font-medium' :
                  isPast ? 'text-green-600' : 'text-gray-400'
                }`}
              >
                <Icon size={18} />
                <span>{s.label}</span>
                {isPast && <Check size={14} className="ml-auto" />}
              </div>
            )
          })}
        </nav>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex items-center justify-center p-12">
        <div className="max-w-lg w-full">

          {/* Welcome */}
          {step === 'welcome' && (
            <div className="text-center">
              <div className="w-16 h-16 bg-indigo-100 rounded-2xl flex items-center justify-center mx-auto mb-6">
                <Globe size={32} className="text-indigo-600" />
              </div>
              <h1 className="text-3xl font-bold mb-4">Welcome to You AI Website Builder</h1>
              <p className="text-gray-600 text-lg mb-8">
                We'll help you create a beautiful website just by having a conversation.
                First, let's connect a few services.
              </p>
              <p className="text-gray-500 text-sm mb-8">
                This takes about 5 minutes. We'll walk you through each step.
              </p>
              <button
                onClick={() => setStep('llm')}
                className="bg-indigo-600 text-white px-8 py-3 rounded-xl text-lg font-medium hover:bg-indigo-700 transition-colors inline-flex items-center gap-2"
              >
                Let's get started <ChevronRight size={20} />
              </button>
            </div>
          )}

          {/* Connect LLM */}
          {step === 'llm' && (
            <div>
              <h1 className="text-2xl font-bold mb-2">Connect an AI</h1>
              <p className="text-gray-600 mb-6">
                This is the brain that will help design your website. Pick one to start — you can always add more later.
              </p>

              {error && (
                <div className="bg-red-50 border border-red-200 rounded-lg p-3 mb-4 text-red-700 text-sm">
                  {error}
                </div>
              )}

              {!oauthUrl && !llmProvider && (
                <div className="space-y-3">
                  <button
                    onClick={() => { setLlmProvider('chatgpt'); handleOAuthConnect('chatgpt') }}
                    className="w-full flex items-center gap-4 p-4 border border-gray-200 rounded-xl hover:border-indigo-300 hover:bg-indigo-50 transition-colors text-left"
                  >
                    <div className="w-10 h-10 bg-green-100 rounded-lg flex items-center justify-center">
                      <Bot size={20} className="text-green-600" />
                    </div>
                    <div>
                      <div className="font-medium">ChatGPT</div>
                      <div className="text-sm text-gray-500">Great for conversational website building</div>
                    </div>
                    <ChevronRight size={18} className="ml-auto text-gray-400" />
                  </button>

                  <button
                    onClick={() => setLlmProvider('claude')}
                    className="w-full flex items-center gap-4 p-4 border border-gray-200 rounded-xl hover:border-indigo-300 hover:bg-indigo-50 transition-colors text-left"
                  >
                    <div className="w-10 h-10 bg-orange-100 rounded-lg flex items-center justify-center">
                      <Bot size={20} className="text-orange-600" />
                    </div>
                    <div>
                      <div className="font-medium">Claude</div>
                      <div className="text-sm text-gray-500">Excellent at following design instructions</div>
                    </div>
                    <ChevronRight size={18} className="ml-auto text-gray-400" />
                  </button>

                  <button
                    onClick={() => { setLlmProvider('gemini'); handleOAuthConnect('gemini') }}
                    className="w-full flex items-center gap-4 p-4 border border-gray-200 rounded-xl hover:border-indigo-300 hover:bg-indigo-50 transition-colors text-left"
                  >
                    <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                      <Bot size={20} className="text-blue-600" />
                    </div>
                    <div>
                      <div className="font-medium">Gemini</div>
                      <div className="text-sm text-gray-500">Strong at understanding visual references</div>
                    </div>
                    <ChevronRight size={18} className="ml-auto text-gray-400" />
                  </button>

                  <button
                    onClick={() => setLlmProvider('openai_compatible')}
                    className="w-full flex items-center gap-4 p-4 border border-gray-200 rounded-xl hover:border-indigo-300 hover:bg-indigo-50 transition-colors text-left"
                  >
                    <div className="w-10 h-10 bg-gray-100 rounded-lg flex items-center justify-center">
                      <Bot size={20} className="text-gray-600" />
                    </div>
                    <div>
                      <div className="font-medium">Custom / Local AI</div>
                      <div className="text-sm text-gray-500">Use any OpenAI-compatible API or local model</div>
                    </div>
                    <ChevronRight size={18} className="ml-auto text-gray-400" />
                  </button>
                </div>
              )}

              {/* API Key flow (Claude, custom) */}
              {llmProvider && (llmProvider === 'claude' || llmProvider === 'openai_compatible') && (
                <div className="space-y-4">
                  <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 text-sm">
                    <p className="font-medium text-blue-800 mb-2">How to get your API key:</p>
                    {llmProvider === 'claude' && (
                      <ol className="list-decimal list-inside space-y-1 text-blue-700">
                        <li>Go to <span className="font-mono">console.anthropic.com</span></li>
                        <li>Sign in or create an account</li>
                        <li>Click "API Keys" in the left menu</li>
                        <li>Click "Create Key" and copy it</li>
                      </ol>
                    )}
                    {llmProvider === 'openai_compatible' && (
                      <ol className="list-decimal list-inside space-y-1 text-blue-700">
                        <li>Get the API key from your model provider</li>
                        <li>You'll also need the API endpoint URL</li>
                      </ol>
                    )}
                  </div>
                  <input
                    type="password"
                    value={apiKey}
                    onChange={e => setApiKey(e.target.value)}
                    placeholder="Paste your API key here"
                    className="w-full px-4 py-3 border border-gray-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-indigo-500"
                  />
                  <div className="flex gap-3">
                    <button
                      onClick={() => { setLlmProvider(''); setApiKey('') }}
                      className="px-4 py-2 text-gray-600 hover:text-gray-800"
                    >
                      Back
                    </button>
                    <button
                      onClick={() => handleApiKeyConnect(llmProvider)}
                      disabled={!apiKey || connecting}
                      className="flex-1 bg-indigo-600 text-white px-6 py-3 rounded-xl font-medium hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    >
                      {connecting ? 'Connecting...' : 'Connect'}
                    </button>
                  </div>
                </div>
              )}

              {/* OAuth redirect paste flow (ChatGPT, Gemini, GitHub) */}
              {oauthUrl && (
                <div className="space-y-4">
                  <div className="bg-green-50 border border-green-200 rounded-lg p-4 text-sm">
                    <div className="flex items-center gap-2 text-green-800 font-medium mb-2">
                      <ExternalLink size={16} />
                      A browser window opened for you to sign in
                    </div>
                    <p className="text-green-700">
                      After you authorize, your browser will show a URL in the address bar.
                      Copy that entire URL and paste it below.
                    </p>
                  </div>
                  <div className="relative">
                    <ClipboardPaste size={18} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
                    <input
                      type="text"
                      value={redirectUrl}
                      onChange={e => setRedirectUrl(e.target.value)}
                      placeholder="Paste the URL from your browser here"
                      className="w-full pl-10 pr-4 py-3 border border-gray-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    />
                  </div>
                  <div className="flex gap-3">
                    <button
                      onClick={() => { setOauthUrl(''); setLlmProvider(''); setRedirectUrl('') }}
                      className="px-4 py-2 text-gray-600 hover:text-gray-800"
                    >
                      Back
                    </button>
                    <button
                      onClick={() => handlePasteRedirectUrl(llmProvider || 'chatgpt')}
                      disabled={!redirectUrl || connecting}
                      className="flex-1 bg-indigo-600 text-white px-6 py-3 rounded-xl font-medium hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    >
                      {connecting ? 'Connecting...' : 'Save Connection'}
                    </button>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Connect GitHub */}
          {step === 'github' && (
            <div>
              <h1 className="text-2xl font-bold mb-2">Connect GitHub</h1>
              <p className="text-gray-600 mb-6">
                GitHub is where your website's files are stored — like a secure filing cabinet in the cloud. It also lets you undo changes if you ever need to.
              </p>

              {error && (
                <div className="bg-red-50 border border-red-200 rounded-lg p-3 mb-4 text-red-700 text-sm">
                  {error}
                </div>
              )}

              {!oauthUrl ? (
                <div className="space-y-4">
                  <div className="bg-gray-50 border border-gray-200 rounded-lg p-4 text-sm">
                    <p className="font-medium text-gray-800 mb-2">Don't have a GitHub account?</p>
                    <ol className="list-decimal list-inside space-y-1 text-gray-600">
                      <li>Go to <span className="font-mono">github.com</span> and click "Sign Up"</li>
                      <li>Create a free account (no credit card needed)</li>
                      <li>Come back here and click "Connect GitHub"</li>
                    </ol>
                  </div>
                  <button
                    onClick={() => handleOAuthConnect('github')}
                    disabled={connecting}
                    className="w-full bg-gray-900 text-white px-6 py-3 rounded-xl font-medium hover:bg-gray-800 transition-colors inline-flex items-center justify-center gap-2"
                  >
                    <GitBranch size={20} />
                    {connecting ? 'Opening browser...' : 'Connect GitHub'}
                  </button>
                </div>
              ) : (
                <div className="space-y-4">
                  <div className="bg-green-50 border border-green-200 rounded-lg p-4 text-sm">
                    <div className="flex items-center gap-2 text-green-800 font-medium mb-2">
                      <ExternalLink size={16} />
                      A browser window opened for you to sign in
                    </div>
                    <p className="text-green-700">
                      After you authorize, copy the URL from your browser and paste it below.
                    </p>
                  </div>
                  <div className="relative">
                    <ClipboardPaste size={18} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
                    <input
                      type="text"
                      value={redirectUrl}
                      onChange={e => setRedirectUrl(e.target.value)}
                      placeholder="Paste the URL from your browser here"
                      className="w-full pl-10 pr-4 py-3 border border-gray-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    />
                  </div>
                  <button
                    onClick={() => handlePasteRedirectUrl('github')}
                    disabled={!redirectUrl || connecting}
                    className="w-full bg-indigo-600 text-white px-6 py-3 rounded-xl font-medium hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                  >
                    {connecting ? 'Connecting...' : 'Save Connection'}
                  </button>
                </div>
              )}

              <button
                onClick={() => setStep('cloudflare')}
                className="mt-4 text-sm text-gray-500 hover:text-gray-700"
              >
                Skip for now →
              </button>
            </div>
          )}

          {/* Connect Cloudflare */}
          {step === 'cloudflare' && (
            <div>
              <h1 className="text-2xl font-bold mb-2">Connect Cloudflare</h1>
              <p className="text-gray-600 mb-6">
                Cloudflare is what makes your website visible on the internet. It's fast, free for basic sites, and keeps your site secure.
              </p>

              {error && (
                <div className="bg-red-50 border border-red-200 rounded-lg p-3 mb-4 text-red-700 text-sm">
                  {error}
                </div>
              )}

              <div className="space-y-4">
                <div className="bg-orange-50 border border-orange-200 rounded-lg p-4 text-sm">
                  <p className="font-medium text-orange-800 mb-2">How to get your Cloudflare API token:</p>
                  <ol className="list-decimal list-inside space-y-1 text-orange-700">
                    <li>Go to <span className="font-mono">dash.cloudflare.com</span> and sign in (or create a free account)</li>
                    <li>Click your profile icon → "My Profile"</li>
                    <li>Click "API Tokens" in the left menu</li>
                    <li>Click "Create Token"</li>
                    <li>Use the "Edit Cloudflare Pages" template</li>
                    <li>Click "Continue to summary" → "Create Token"</li>
                    <li>Copy the token and paste it below</li>
                  </ol>
                </div>
                <input
                  type="password"
                  value={apiKey}
                  onChange={e => setApiKey(e.target.value)}
                  placeholder="Paste your Cloudflare API token here"
                  className="w-full px-4 py-3 border border-gray-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-indigo-500"
                />
                <button
                  onClick={async () => {
                    setError('')
                    setConnecting(true)
                    try {
                      const { invoke } = await import('@tauri-apps/api/core')
                      await invoke('save_credential', {
                        request: {
                          provider: 'cloudflare',
                          token: apiKey,
                          tokenType: 'api_key',
                        }
                      })
                      setConnectedServices(prev => ({ ...prev, cloudflare: true }))
                      setApiKey('')
                      setStep('done')
                    } catch (e: any) {
                      setError(e.toString())
                    }
                    setConnecting(false)
                  }}
                  disabled={!apiKey || connecting}
                  className="w-full bg-orange-500 text-white px-6 py-3 rounded-xl font-medium hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors inline-flex items-center justify-center gap-2"
                >
                  <Cloud size={20} />
                  {connecting ? 'Connecting...' : 'Connect Cloudflare'}
                </button>
              </div>

              <button
                onClick={() => setStep('done')}
                className="mt-4 text-sm text-gray-500 hover:text-gray-700"
              >
                Skip for now →
              </button>
            </div>
          )}

          {/* Done */}
          {step === 'done' && (
            <div className="text-center">
              <div className="w-16 h-16 bg-green-100 rounded-2xl flex items-center justify-center mx-auto mb-6">
                <Check size={32} className="text-green-600" />
              </div>
              <h1 className="text-3xl font-bold mb-4">You're all set!</h1>
              <p className="text-gray-600 text-lg mb-8">
                Everything is connected. Let's build your first website.
              </p>
              <button
                onClick={onComplete}
                className="bg-indigo-600 text-white px-8 py-3 rounded-xl text-lg font-medium hover:bg-indigo-700 transition-colors"
              >
                Start Building
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
