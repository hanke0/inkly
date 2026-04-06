import React from 'react';
import { createRoot } from 'react-dom/client';

import App from './App';
import { ApiErrorDialog } from './components/ApiErrorDialog';
import { I18nProvider } from './i18n/context';
import 'katex/dist/katex.min.css';
import './index.css';

createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <I18nProvider>
      <ApiErrorDialog />
      <App />
    </I18nProvider>
  </React.StrictMode>,
);
