import { hydrate } from 'solid-js/web';
import App from './App';

// Hydrate the server-rendered content
hydrate(() => App(), document.getElementById('app')!);

