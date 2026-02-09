<template>
  <div id="acme-app">
    <AppHeader />
    <nav class="nav-bar">
      <router-link to="/">Home</router-link>
      <router-link to="/dashboard">Dashboard</router-link>
      <router-link to="/settings">Settings</router-link>
    </nav>
    <main class="content">
      <router-view />
    </main>
    <AppFooter />
  </div>
</template>

<script setup>
import AppHeader from './components/AppHeader.vue'
import AppFooter from './components/AppFooter.vue'
</script>

<style>
#acme-app {
  font-family: Arial, sans-serif;
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}
.nav-bar {
  display: flex;
  gap: 16px;
  padding: 12px 0;
  border-bottom: 1px solid #e0e0e0;
  margin-bottom: 24px;
}
.nav-bar a {
  text-decoration: none;
  color: #4a90d9;
  font-weight: 600;
}
.content {
  min-height: 400px;
}
</style>
