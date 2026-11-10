import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { PollarProvider } from './pollar'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <PollarProvider>
      <App />
    </PollarProvider>
  </StrictMode>,
)
