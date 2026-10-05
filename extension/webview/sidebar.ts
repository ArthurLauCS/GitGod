import { mount } from 'svelte'
import '../../src/app.css'
import '../../src/lib/theme.svelte'
import GraphView from './GraphView.svelte'

mount(GraphView, { target: document.getElementById('app')! })
