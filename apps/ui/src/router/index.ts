import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'dashboard',
    component: () => import('@/views/Dashboard.vue'),
  },
  {
    path: '/project/:id',
    name: 'project',
    component: () => import('@/views/ProjectView.vue'),
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('@/views/Settings.vue'),
  },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
});
