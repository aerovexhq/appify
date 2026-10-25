import React from 'react';
import ReactDOM from 'react-dom/client';
import { StrataThemeProvider } from '@aerovex/strata-ui';
import { App } from './App';
import './index.css';

window.addEventListener('error', (e) => {
  console.error('Appify Runtime Error:', e.error || e.message);
});

window.addEventListener('unhandledrejection', (e) => {
  console.error('Unhandled Promise Rejection:', e.reason);
});

const rootEl = document.getElementById('root');
if (rootEl) {
  ReactDOM.createRoot(rootEl).render(
    <React.StrictMode>
      <StrataThemeProvider theme="aerovex" preset="launcher">
        <App />
      </StrataThemeProvider>
    </React.StrictMode>
  );
}
