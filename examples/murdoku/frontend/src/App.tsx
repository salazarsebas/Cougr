import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { NavBar } from './components/NavBar';
import { HomePage } from './pages/HomePage';
import { PlayPage } from './pages/PlayPage';
import { CreatePage } from './pages/CreatePage';
import { usePollar } from './hooks/usePollar';

function App() {
  const { isAuthenticated, address, login, logout } = usePollar();

  return (
    <BrowserRouter>
      <a href="#main-content" className="skip-link">
        Skip to main content
      </a>

      <NavBar wallet={{ connected: isAuthenticated, address }} onConnect={login} onDisconnect={logout} />

      <Routes>
        <Route path="/" element={<HomePage />} />
        <Route
          path="/play/:puzzleId"
          element={isAuthenticated ? <PlayPage /> : <Navigate to="/" replace />}
        />
        <Route
          path="/create"
          element={isAuthenticated ? <CreatePage /> : <Navigate to="/" replace />}
        />
        <Route
          path="*"
          element={
            <main id="main-content" style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', padding: '2rem', textAlign: 'center' }}>
              <div>
                <h1 style={{ fontFamily: 'var(--font-serif)', fontSize: '2rem', color: 'var(--accent-red)', marginBottom: '0.75rem' }}>404</h1>
                <p style={{ fontFamily: 'var(--font-serif)', fontStyle: 'italic', color: 'var(--noir-muted)' }}>
                  This page vanished - like the alibi of a guilty man.
                </p>
              </div>
            </main>
          }
        />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
