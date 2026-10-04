import { mount } from 'svelte'
import '@fontsource-variable/noto-serif-sc'
import 'lxgw-wenkai-webfont/lxgwwenkai-regular.css'
import './app.css'
import './lib/theme.svelte'
import App from './App.svelte'

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
