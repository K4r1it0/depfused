import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: 'auth',
    loadComponent: () => import('./lazy/auth.component').then(m => m.AuthComponent)
  }
];
