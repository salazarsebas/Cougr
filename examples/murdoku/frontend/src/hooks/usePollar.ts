import { usePollarAuth } from '../pollar';

export function usePollar() {
  const p = usePollarAuth();
  return {
    isAuthenticated: p?.isAuthenticated ?? false,
    address: p?.address ?? null,
    login: p?.login ?? (() => Promise.resolve()),
    logout: p?.logout ?? (() => Promise.resolve()),
  };
}
