import React from 'react';
import { PollarProvider as _PollarProvider, usePollar as _usePollar } from '@pollar/react';

const POLLAR_API_KEY = import.meta.env.VITE_POLLAR_API_KEY;
const STELLAR_NETWORK = import.meta.env.VITE_STELLAR_NETWORK;
const CONTRACT_ID = import.meta.env.VITE_CONTRACT_ID;

export function PollarProvider({ children }: { children: React.ReactNode }) {
  return (
    <_PollarProvider
      apiKey={POLLAR_API_KEY}
      network={STELLAR_NETWORK}
      options={{ contractId: CONTRACT_ID }}
    >
      {children}
    </_PollarProvider>
  );
}

export const usePollarAuth = _usePollar;
