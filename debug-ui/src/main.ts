import { createApp } from 'vue';
import './style.css';
import App from './App.vue';

import {
  // create naive ui
  create,
  // component
  NProgress
} from 'naive-ui';

const naive = create({
    components: [
        NProgress
    ]
});

createApp(App)
    .use(naive)
    .mount('#app');
