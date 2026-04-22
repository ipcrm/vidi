import { mount } from 'svelte';
import App from './App.svelte';
import './lib/styles/fonts.css';
import './lib/styles/tokens.css';
import './lib/styles/typography.css';
import './lib/styles/prose.css';
import './lib/styles/alerts.css';
import './lib/styles/dark.css';
import './lib/styles/chrome.css';
import './lib/styles/print.css';

const target = document.getElementById('app');
if (!target) {
  throw new Error('missing #app mount point');
}

mount(App, { target });
