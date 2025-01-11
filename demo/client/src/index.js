import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

if (!window.__created) {
  const root = ReactDOM.createRoot(document.getElementById('root'));

  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );

  window.__created = true;
}
