import './app.css';
import '@xyflow/svelte/dist/style.css';
import './styles/material-symbols-outlined-subset.css';
import { mount } from 'svelte';
import App from './App.svelte';

const app = mount(App, {
  target: document.getElementById('app')!
});

export default app;
