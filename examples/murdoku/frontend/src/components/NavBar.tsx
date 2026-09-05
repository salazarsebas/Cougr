import { useState } from 'react';
import { Wallet, LogOut, LogIn, Menu, X } from 'lucide-react';
import type { WalletState } from '../types';

interface NavBarProps {
  wallet: WalletState;
  onConnect: () => void;
  onDisconnect: () => void;
}

function shortenAddress(addr: string): string {
  if (addr.length <= 12) return addr;
  return `${addr.slice(0, 6)}…${addr.slice(-4)}`;
}

export function NavBar({ wallet, onConnect, onDisconnect }: NavBarProps) {
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <header
      role="banner"
      style={{
        background: 'var(--noir-bg)',
        borderBottom: '1px solid var(--noir-border)',
        boxShadow: '0 2px 16px rgba(0,0,0,0.6)',
        position: 'sticky',
        top: 0,
        zIndex: 50,
      }}
    >
      <nav
        aria-label="Primary navigation"
        style={{
          maxWidth: 1280,
          margin: '0 auto',
          padding: '0 1.25rem',
          height: 56,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: '1rem',
        }}
      >
        {/* Logo */}
        <a
          href="/"
          id="nav-logo"
          style={{
            fontFamily: 'var(--font-serif)',
            fontSize: '1.375rem',
            fontWeight: 700,
            color: 'var(--noir-text)',
            textDecoration: 'none',
            letterSpacing: '0.02em',
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            flexShrink: 0,
          }}
          aria-label="Murdoku - home"
        >
          <span style={{ color: 'var(--accent-gold)' }}>✦</span>
          Murdoku
        </a>

        {/* Desktop wallet area */}
        <div
          className="desktop-wallet"
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.75rem',
          }}
        >
          {wallet.connected && wallet.address ? (
            <>
              {/* Connection indicator */}
              <span
                aria-label="Wallet connected"
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '0.4rem',
                  fontFamily: 'var(--font-mono)',
                  fontSize: '0.8125rem',
                  color: 'var(--noir-muted)',
                }}
              >
                <span
                  style={{
                    width: 7,
                    height: 7,
                    borderRadius: '50%',
                    background: 'var(--accent-green)',
                    display: 'inline-block',
                    boxShadow: '0 0 6px var(--accent-green)',
                    flexShrink: 0,
                  }}
                  aria-hidden="true"
                />
                {/* Full address on desktop, icon on narrow */}
                <span className="addr-full" aria-label={`Wallet: ${wallet.address}`}>
                  {shortenAddress(wallet.address)}
                </span>
                <Wallet
                  size={14}
                  className="addr-icon"
                  aria-hidden="true"
                  style={{ display: 'none' }}
                />
              </span>

              <button
                id="btn-disconnect"
                className="btn-outline"
                onClick={onDisconnect}
                aria-label="Disconnect wallet"
              >
                <LogOut size={14} aria-hidden="true" />
                <span className="btn-label">Disconnect</span>
              </button>
            </>
          ) : (
            <button
              id="btn-connect"
              className="btn-outline"
              onClick={onConnect}
              aria-label="Connect wallet"
              style={{ borderColor: 'var(--accent-gold)', color: 'var(--accent-gold)' }}
            >
              <LogIn size={14} aria-hidden="true" />
              Connect Wallet
            </button>
          )}
        </div>

        {/* Mobile hamburger */}
        <button
          id="btn-mobile-menu"
          className="mobile-menu-btn"
          onClick={() => setMobileOpen((o) => !o)}
          aria-label={mobileOpen ? 'Close menu' : 'Open menu'}
          aria-expanded={mobileOpen}
          style={{
            display: 'none',
            background: 'none',
            border: 'none',
            color: 'var(--noir-text)',
            cursor: 'pointer',
            padding: '0.25rem',
          }}
        >
          {mobileOpen ? <X size={20} /> : <Menu size={20} />}
        </button>
      </nav>

      {/* Mobile wallet drawer */}
      {mobileOpen && (
        <div
          id="mobile-drawer"
          style={{
            borderTop: '1px solid var(--noir-border)',
            padding: '1rem 1.25rem',
            display: 'flex',
            flexDirection: 'column',
            gap: '0.75rem',
          }}
        >
          {wallet.connected && wallet.address ? (
            <>
              <span
                style={{
                  fontFamily: 'var(--font-mono)',
                  fontSize: '0.8125rem',
                  color: 'var(--noir-muted)',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '0.5rem',
                }}
              >
                <span
                  style={{
                    width: 7, height: 7, borderRadius: '50%',
                    background: 'var(--accent-green)',
                    boxShadow: '0 0 6px var(--accent-green)',
                    flexShrink: 0,
                  }}
                />
                {wallet.address}
              </span>
              <button
                id="btn-disconnect-mobile"
                className="btn-outline"
                style={{ alignSelf: 'flex-start' }}
                onClick={() => { onDisconnect(); setMobileOpen(false); }}
              >
                <LogOut size={14} aria-hidden="true" /> Disconnect
              </button>
            </>
          ) : (
            <button
              id="btn-connect-mobile"
              className="btn-outline"
              style={{ alignSelf: 'flex-start', borderColor: 'var(--accent-gold)', color: 'var(--accent-gold)' }}
              onClick={() => { onConnect(); setMobileOpen(false); }}
            >
              <LogIn size={14} aria-hidden="true" /> Connect Wallet
            </button>
          )}
        </div>
      )}

      <style>{`
        @media (max-width: 639px) {
          .desktop-wallet { display: none !important; }
          .mobile-menu-btn { display: flex !important; }
        }
        @media (max-width: 767px) {
          .addr-full { display: none; }
          .addr-icon { display: inline-block !important; }
          .btn-label { display: none; }
        }
      `}</style>
    </header>
  );
}
