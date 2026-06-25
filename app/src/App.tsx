import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { SetupWizard } from './pages/SetupWizard'
import { Dashboard } from './pages/Dashboard'
import { Editor } from './pages/Editor'
import { useEffect, useState } from 'react'
import './index.css'

function App() {
  const [setupComplete, setSetupComplete] = useState<boolean | null>(null)

  useEffect(() => {
    checkSetup()
  }, [])

  async function checkSetup() {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const status = await invoke('get_setup_status') as any
      setSetupComplete(status.llm_connected && status.github_connected && status.cloudflare_connected)
    } catch {
      // In dev mode without Tauri, default to showing setup
      setSetupComplete(false)
    }
  }

  if (setupComplete === null) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin mx-auto mb-4" />
          <p className="text-gray-500">Loading...</p>
        </div>
      </div>
    )
  }

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={
          setupComplete ? <Dashboard /> : <SetupWizard onComplete={() => setSetupComplete(true)} />
        } />
        <Route path="/editor/:projectName" element={<Editor />} />
        <Route path="/setup" element={<SetupWizard onComplete={() => setSetupComplete(true)} />} />
      </Routes>
    </BrowserRouter>
  )
}

export default App
