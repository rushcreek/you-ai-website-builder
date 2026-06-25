import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { Plus, Globe, Settings, Clock } from 'lucide-react'

interface Project {
  name: string
  path: string
  github_repo: string | null
  cloudflare_project: string | null
  live_url: string | null
  created_at: number
}

export function Dashboard() {
  const navigate = useNavigate()
  const [projects, setProjects] = useState<Project[]>([])
  const [showNewProject, setShowNewProject] = useState(false)
  const [newProjectName, setNewProjectName] = useState('')
  const [creating, setCreating] = useState(false)

  useEffect(() => {
    loadProjects()
  }, [])

  async function loadProjects() {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke('list_projects') as Project[]
      setProjects(result)
    } catch {
      // Dev mode fallback
      setProjects([])
    }
  }

  async function createProject() {
    if (!newProjectName.trim()) return
    setCreating(true)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('create_project', {
        request: { name: newProjectName.trim(), description: null }
      })
      setShowNewProject(false)
      setNewProjectName('')
      navigate(`/editor/${encodeURIComponent(newProjectName.trim())}`)
    } catch (e: any) {
      alert(e.toString())
    }
    setCreating(false)
  }

  function formatDate(timestamp: number) {
    if (!timestamp) return 'Just now'
    return new Date(timestamp * 1000).toLocaleDateString(undefined, {
      month: 'short', day: 'numeric', year: 'numeric'
    })
  }

  return (
    <div className="min-h-screen">
      {/* Header */}
      <header className="bg-white border-b border-gray-200 px-8 py-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 bg-indigo-100 rounded-lg flex items-center justify-center">
            <Globe size={18} className="text-indigo-600" />
          </div>
          <h1 className="text-xl font-semibold">You AI Website Builder</h1>
        </div>
        <button
          onClick={() => navigate('/setup')}
          className="p-2 text-gray-500 hover:text-gray-700 rounded-lg hover:bg-gray-100"
        >
          <Settings size={20} />
        </button>
      </header>

      {/* Main Content */}
      <main className="max-w-4xl mx-auto px-8 py-12">
        <div className="flex items-center justify-between mb-8">
          <h2 className="text-2xl font-bold">My Websites</h2>
          <button
            onClick={() => setShowNewProject(true)}
            className="bg-indigo-600 text-white px-5 py-2.5 rounded-xl font-medium hover:bg-indigo-700 transition-colors inline-flex items-center gap-2"
          >
            <Plus size={18} />
            New Website
          </button>
        </div>

        {/* New Project Modal */}
        {showNewProject && (
          <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-white rounded-2xl p-8 max-w-md w-full mx-4 shadow-xl">
              <h3 className="text-xl font-bold mb-4">Create a New Website</h3>
              <p className="text-gray-600 mb-6">
                Give your project a name. Don't worry — this won't be your website's title, just what you call it.
              </p>
              <input
                type="text"
                value={newProjectName}
                onChange={e => setNewProjectName(e.target.value)}
                onKeyDown={e => e.key === 'Enter' && createProject()}
                placeholder="e.g. My Business Site, Portfolio, etc."
                className="w-full px-4 py-3 border border-gray-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-indigo-500 mb-4"
                autoFocus
              />
              <div className="flex gap-3">
                <button
                  onClick={() => { setShowNewProject(false); setNewProjectName('') }}
                  className="px-4 py-2 text-gray-600 hover:text-gray-800"
                >
                  Cancel
                </button>
                <button
                  onClick={createProject}
                  disabled={!newProjectName.trim() || creating}
                  className="flex-1 bg-indigo-600 text-white px-6 py-3 rounded-xl font-medium hover:bg-indigo-700 disabled:opacity-50 transition-colors"
                >
                  {creating ? 'Creating...' : 'Create & Start Building'}
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Project Grid */}
        {projects.length === 0 && !showNewProject ? (
          <div className="text-center py-20">
            <div className="w-16 h-16 bg-gray-100 rounded-2xl flex items-center justify-center mx-auto mb-6">
              <Globe size={32} className="text-gray-400" />
            </div>
            <h3 className="text-xl font-medium mb-2">No websites yet</h3>
            <p className="text-gray-500 mb-8">
              Create your first website — just tell the AI what you want and it'll build it for you.
            </p>
            <button
              onClick={() => setShowNewProject(true)}
              className="bg-indigo-600 text-white px-6 py-3 rounded-xl font-medium hover:bg-indigo-700 transition-colors inline-flex items-center gap-2"
            >
              <Plus size={18} />
              Create My First Website
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {projects.map(project => (
              <button
                key={project.name}
                onClick={() => navigate(`/editor/${encodeURIComponent(project.name)}`)}
                className="text-left p-6 bg-white rounded-xl border border-gray-200 hover:border-indigo-300 hover:shadow-md transition-all"
              >
                <h3 className="font-semibold text-lg mb-1">{project.name}</h3>
                {project.live_url && (
                  <p className="text-indigo-600 text-sm mb-2 truncate">{project.live_url}</p>
                )}
                <div className="flex items-center gap-1 text-sm text-gray-400">
                  <Clock size={14} />
                  <span>{formatDate(project.created_at)}</span>
                </div>
              </button>
            ))}
          </div>
        )}
      </main>
    </div>
  )
}
