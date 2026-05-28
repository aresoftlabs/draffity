import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import Aura from '@primevue/themes/aura';
import ToastService from 'primevue/toastservice';
import ConfirmationService from 'primevue/confirmationservice';

import App from './App.vue';
import { router } from './router';
import { i18n } from './locales';
import { applyInitialTheme } from './styles/theme';

import 'primeicons/primeicons.css';
import './styles/main.css';

applyInitialTheme();

const app = createApp(App);

app.use(createPinia());
app.use(router);
app.use(i18n);
app.use(PrimeVue, {
  theme: {
    preset: Aura,
    options: {
      darkModeSelector: '.app-dark',
      cssLayer: {
        name: 'primevue',
        order: 'tailwind-base, primevue, tailwind-utilities',
      },
    },
  },
  ripple: false,
});
app.use(ToastService);
app.use(ConfirmationService);

app.mount('#app');
