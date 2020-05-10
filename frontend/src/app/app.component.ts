import {Component} from '@angular/core';
import {Observable} from 'rxjs';
import {map} from 'rxjs/operators';
import {AuthService} from './shared/services/auth.service';
import {InboxService} from './shared/services/inbox.service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.less']
})
export class AppComponent {
  constructor(private inbox: InboxService,
              public auth: AuthService) {
  }

  public get inboxCounter(): Observable<string> {
    return this.inbox.count.pipe(map(count => {
      if (count === undefined || count <= 0) {
        return null;
      } else if (count > 99) {
        return '99+';
      } else {
        return `${count}`;
      }
    }));
  }
}
