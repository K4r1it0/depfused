import { Component, OnInit } from '@angular/core';

@Component({
  selector: 'app-auth',
  standalone: true,
  template: `
    <h2>Authentication Module</h2>
    <p>Auth SDK Status: {{ authStatus }}</p>
  `
})
export class AuthComponent implements OnInit {
  authStatus = 'initializing...';

  async ngOnInit() {
    // Dynamic import of @xq9zk7823/auth-sdk for lazy loading
    const authSdk = await import('@xq9zk7823/auth-sdk');
    const authResult = authSdk.init({
      provider: 'oauth2',
      clientId: 'acme-angular-app',
      redirectUri: '/auth/callback'
    });
    this.authStatus = authResult.ready ? 'ready' : 'failed';
    console.log('Auth SDK v' + authSdk.VERSION + ' loaded lazily');
  }
}
