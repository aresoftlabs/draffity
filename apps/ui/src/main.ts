// Draffity — aplicación de escritura desktop multi-formato.
// Copyright (C) 2026 Aresoft SpA
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version. This program is distributed WITHOUT ANY WARRANTY; without even the
// implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See
// the GNU General Public License for more details. You should have received a
// copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';
import ConfirmationService from 'primevue/confirmationservice';

import App from './App.vue';
import { router } from './router';
import { i18n } from './locales';
import { applyInitialTheme } from './styles/theme';
import { DraffityPreset } from './styles/preset';

import 'primeicons/primeicons.css';
import './styles/fonts';
import './styles/main.css';

applyInitialTheme();

const app = createApp(App);

app.use(createPinia());
app.use(router);
app.use(i18n);
app.use(PrimeVue, {
  theme: {
    preset: DraffityPreset,
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
