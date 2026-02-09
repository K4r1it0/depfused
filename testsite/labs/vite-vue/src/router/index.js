import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/dashboard',
      name: 'dashboard',
      // Lazy-loaded route - dynamic import
      component: () => import('../views/DashboardView.vue')
    },
    {
      path: '/settings',
      name: 'settings',
      // Another lazy-loaded route
      component: () => import('../views/SettingsView.vue')
    }
  ]
})

export default router
