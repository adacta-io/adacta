import { Component } from '@angular/core';
import {InboxService} from './services/inbox.service';
import {Observable} from 'rxjs';
import {map} from 'rxjs/operators';
import {AuthService} from './services/auth.service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.less']
})
export class AppComponent {
  constructor(private inbox: InboxService,
              public auth: AuthService) {
  }

  public get inboxCounter() {
    const count = this.inbox.count;
    if (count === undefined || count <= 0) {
      return null;
    } else if (count > 99) {
      return '99+';
    } else {
      return `${count}`;
    }
  }
}
