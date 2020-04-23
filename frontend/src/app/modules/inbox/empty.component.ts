import {Component, Injectable} from '@angular/core';
import {InboxService} from '../../services/inbox.service';
import {ActivatedRoute, ActivatedRouteSnapshot, CanActivate, Router, RouterStateSnapshot, UrlTree} from '@angular/router';
import {combineLatest, Observable} from 'rxjs';

@Injectable()
export class CanActivateEmpty implements CanActivate {
  private first: string | null;

  constructor(private inbox: InboxService,
              private route: ActivatedRoute,
              private router: Router) {
    this.route.paramMap.subscribe(params => {
        console.log(params.get('id'), this.inbox.docs.length);
        if (params.get('id') == null && this.inbox.docs.length > 0) {
          this.first = this.inbox.docs[0];
        } else {
          this.first = null;
        }
      });
  }

  canActivate(route: ActivatedRouteSnapshot,
              state: RouterStateSnapshot): Observable<boolean | UrlTree> | Promise<boolean | UrlTree> | boolean | UrlTree {
    if (this.first != null) {
      return this.router.parseUrl(`inbox/${this.first}`);
    } else {
      return true;
    }
  }
}

@Component({
  selector: 'app-empty',
  templateUrl: './empty.component.html',
  styleUrls: ['./empty.component.less']
})
export class EmptyComponent {

  constructor() {
  }
}
