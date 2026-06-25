import { useState, useEffect, useRef } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft, Send, Globe, ChevronDown, ChevronUp, Plus, ExternalLink, Rocket } from 'lucide-react'

interface Message {
  id: string
  role: 'user' | 'assistant'
  content: string
  timestamp: number
}

interface Inspiration {
  url: string
  title: string | null
  colors: string[]
  fonts: string[]
  layout_type: string | null
}

export function Editor() {
  const { projectName } = useParams<{ projectName: string }>()
  const navigate = useNavigate()
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [sending, setSending] = useState(false)
  const [previewHtml, setPreviewHtml] = useState('<html><body><h1>Your site will appear here</h1></body></html>')
  const [inspirations, setInspirations] = useState<Inspiration[]>([])
  const [showInspirations, setShowInspirations] = useState(true)
  const [inspirationUrl, setInspirationUrl] = useState('')
  const [scraping, setScraping] = useState(false)
  const [publishing, setPublishing] = useState(false)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    // Load existing conversation from project metadata
    loadProject()
    // Add initial greeting
    if (messages.length === 0) {
      setMessages([{
        id: '1',
        role: 'assistant',
        content: "Hi! I'm here to help you build your website. Let's start with the basics — what's this website for? A business, a personal page, a portfolio, or something else?",
        timestamp: Date.now(),
      }])
    }
  }, [])

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  async function loadProject() {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const html = await invoke('get_site_preview', { projectName }) as string
      setPreviewHtml(html)
    } catch {
      // Dev mode
    }
  }

  async function sendMessage() {
    if (!input.trim() || sending) return

    const userMessage: Message = {
      id: Date.now().toString(),
      role: 'user',
      content: input.trim(),
      timestamp: Date.now(),
    }

    setMessages(prev => [...prev, userMessage])
    setInput('')
    setSending(true)

    try {
      const { invoke } = await import('@tauri-apps/api/core')

      // Build context from inspirations
      const context = JSON.stringify({
        inspirations: inspirations,
        currentHtml: previewHtml,
      })

      const allMessages = [...messages, userMessage].map(m => ({
        role: m.role,
        content: m.content,
      }))

      const response = await invoke('chat_with_llm', {
        request: {
          provider: 'chatgpt', // TODO: use configured provider
          messages: allMessages,
          projectContext: context,
        }
      }) as { content: string }

      const assistantMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: response.content,
        timestamp: Date.now(),
      }

      setMessages(prev => [...prev, assistantMessage])

      // Extract HTML from response if present
      const htmlMatch = response.content.match(/```html\n([\s\S]*?)```/)
      if (htmlMatch) {
        setPreviewHtml(htmlMatch[1])
      }
    } catch (e: any) {
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: `Sorry, I had trouble connecting. Error: ${e.toString()}. Make sure your AI is connected in Settings.`,
        timestamp: Date.now(),
      }
      setMessages(prev => [...prev, errorMessage])
    }

    setSending(false)
  }

  async function addInspiration() {
    if (!inspirationUrl.trim() || scraping) return
    setScraping(true)

    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke('scrape_site', { url: inspirationUrl.trim() }) as Inspiration
      setInspirations(prev => [...prev, result])
      setInspirationUrl('')

      // Tell the AI about this inspiration
      const infoMessage: Message = {
        id: Date.now().toString(),
        role: 'user',
        content: `I like this website: ${result.url}${result.title ? ` (${result.title})` : ''}`,
        timestamp: Date.now(),
      }
      setMessages(prev => [...prev, infoMessage])

      // Get AI response about the inspiration
      setSending(true)
      const context = JSON.stringify({ inspirations: [...inspirations, result] })
      const allMessages = [...messages, infoMessage].map(m => ({ role: m.role, content: m.content }))

      const response = await invoke('chat_with_llm', {
        request: {
          provider: 'chatgpt',
          messages: allMessages,
          projectContext: context,
        }
      }) as { content: string }

      setMessages(prev => [...prev, {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: response.content,
        timestamp: Date.now(),
      }])
      setSending(false)
    } catch (e: any) {
      alert(`Couldn't load that site: ${e.toString()}`)
    }

    setScraping(false)
  }

  async function publishSite() {
    setPublishing(true)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke('publish_site', {
        request: { projectName, commitMessage: null }
      }) as { github_url: string; live_url: string | null; status: string }

      const publishMessage: Message = {
        id: Date.now().toString(),
        role: 'assistant',
        content: `🚀 ${result.status}\n\n${result.live_url ? `Your site is live at: ${result.live_url}` : `GitHub: ${result.github_url}`}`,
        timestamp: Date.now(),
      }
      setMessages(prev => [...prev, publishMessage])
    } catch (e: any) {
      alert(`Publish failed: ${e.toString()}`)
    }
    setPublishing(false)
  }

  return (
    <div className="h-screen flex flex-col">
      {/* Header */}
      <header className="bg-white border-b border-gray-200 px-4 py-3 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-3">
          <button
            onClick={() => navigate('/')}
            className="p-1.5 text-gray-500 hover:text-gray-700 rounded-lg hover:bg-gray-100"
          >
            <ArrowLeft size={20} />
          </button>
          <h1 className="font-semibold">{decodeURIComponent(projectName || '')}</h1>
        </div>
        <button
          onClick={publishSite}
          disabled={publishing}
          className="bg-green-600 text-white px-4 py-2 rounded-lg font-medium hover:bg-green-700 disabled:opacity-50 transition-colors inline-flex items-center gap-2 text-sm"
        >
          <Rocket size={16} />
          {publishing ? 'Publishing...' : 'Publish'}
        </button>
      </header>

      {/* Main Split View */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left: Chat */}
        <div className="w-1/2 flex flex-col border-r border-gray-200">
          {/* Inspirations Panel */}
          <div className="border-b border-gray-200 bg-gray-50">
            <button
              onClick={() => setShowInspirations(!showInspirations)}
              className="w-full px-4 py-2.5 flex items-center justify-between text-sm font-medium text-gray-700 hover:bg-gray-100"
            >
              <span className="flex items-center gap-2">
                <Globe size={16} />
                Websites I Like ({inspirations.length})
              </span>
              {showInspirations ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
            </button>
            {showInspirations && (
              <div className="px-4 pb-3 space-y-2">
                {inspirations.map((insp, i) => (
                  <div key={i} className="flex items-center gap-2 text-xs bg-white rounded-lg px-3 py-2 border border-gray-200">
                    <div className="flex gap-1">
                      {insp.colors.slice(0, 3).map((color, ci) => (
                        <div key={ci} className="w-3 h-3 rounded-full border border-gray-200" style={{ backgroundColor: color }} />
                      ))}
                    </div>
                    <span className="truncate flex-1">{insp.title || insp.url}</span>
                    <a href={insp.url} target="_blank" className="text-gray-400 hover:text-gray-600">
                      <ExternalLink size={12} />
                    </a>
                  </div>
                ))}
                <div className="flex gap-2">
                  <input
                    type="url"
                    value={inspirationUrl}
                    onChange={e => setInspirationUrl(e.target.value)}
                    onKeyDown={e => e.key === 'Enter' && addInspiration()}
                    placeholder="Paste a website URL you like..."
                    className="flex-1 px-3 py-1.5 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-1 focus:ring-indigo-500"
                  />
                  <button
                    onClick={addInspiration}
                    disabled={!inspirationUrl.trim() || scraping}
                    className="px-3 py-1.5 bg-indigo-600 text-white rounded-lg text-sm disabled:opacity-50 hover:bg-indigo-700"
                  >
                    {scraping ? '...' : <Plus size={14} />}
                  </button>
                </div>
              </div>
            )}
          </div>

          {/* Messages */}
          <div className="flex-1 overflow-y-auto p-4 space-y-4">
            {messages.map(msg => (
              <div key={msg.id} className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
                <div className={`max-w-[85%] rounded-2xl px-4 py-3 text-sm ${
                  msg.role === 'user'
                    ? 'bg-indigo-600 text-white'
                    : 'bg-gray-100 text-gray-800'
                }`}>
                  <p className="whitespace-pre-wrap">{msg.content}</p>
                </div>
              </div>
            ))}
            {sending && (
              <div className="flex justify-start">
                <div className="bg-gray-100 rounded-2xl px-4 py-3">
                  <div className="flex gap-1">
                    <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" />
                    <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce [animation-delay:0.1s]" />
                    <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce [animation-delay:0.2s]" />
                  </div>
                </div>
              </div>
            )}
            <div ref={messagesEndRef} />
          </div>

          {/* Input */}
          <div className="border-t border-gray-200 p-4">
            <div className="flex gap-2">
              <input
                type="text"
                value={input}
                onChange={e => setInput(e.target.value)}
                onKeyDown={e => e.key === 'Enter' && !e.shiftKey && sendMessage()}
                placeholder="Describe what you want..."
                className="flex-1 px-4 py-3 border border-gray-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-indigo-500"
                disabled={sending}
              />
              <button
                onClick={sendMessage}
                disabled={!input.trim() || sending}
                className="bg-indigo-600 text-white px-4 py-3 rounded-xl hover:bg-indigo-700 disabled:opacity-50 transition-colors"
              >
                <Send size={18} />
              </button>
            </div>
          </div>
        </div>

        {/* Right: Preview */}
        <div className="w-1/2 bg-white">
          <div className="h-full flex flex-col">
            <div className="px-4 py-2 bg-gray-50 border-b border-gray-200 text-xs text-gray-500 font-medium">
              Live Preview
            </div>
            <div className="flex-1">
              <iframe
                srcDoc={previewHtml}
                className="w-full h-full border-0"
                title="Site Preview"
                sandbox="allow-scripts"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
