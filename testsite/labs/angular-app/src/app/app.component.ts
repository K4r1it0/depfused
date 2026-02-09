import { Component, OnInit } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { ApiService } from './services/api.service';

// Import from @xq9zk7823/design-system (uses CJS with React, import the init/VERSION)
import * as DesignSystem from '@xq9zk7823/design-system';

// Import company-internal-utils (ESM)
import * as InternalUtils from 'company-internal-utils';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet],
  template: `
    <h1>Angular Dependency Confusion Test App</h1>
    <p>Design System Version: {{ dsVersion }}</p>
    <p>API Status: {{ apiStatus }}</p>
    <router-outlet></router-outlet>
  `
})
export class AppComponent implements OnInit {
  title = 'angular-app';
  dsVersion = '';
  apiStatus = 'loading...';

  constructor(private apiService: ApiService) {
    // Use @xq9zk7823/design-system
    const dsResult = DesignSystem.init({ theme: 'dark' });
    this.dsVersion = DesignSystem.VERSION;
    console.log('Design system ready:', dsResult.ready);

    // Use company-internal-utils
    const formatted = InternalUtils.format('  hello from internal utils  ');
    InternalUtils.log('App initialized with: ' + formatted);
  }

  ngOnInit() {
    this.apiService.getStatus().then(status => {
      this.apiStatus = status;
    });
  }
}
