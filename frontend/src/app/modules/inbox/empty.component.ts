import {Component, Injectable} from '@angular/core';
import {ActivatedRoute, ActivatedRouteSnapshot, CanActivate, Router, RouterStateSnapshot, UrlTree} from '@angular/router';
import {combineLatest, Observable} from 'rxjs';
import {map} from 'rxjs/operators';
import {InboxService} from '../../shared/services/inbox.service';

@Injectable()
export class CanActivateEmpty implements CanActivate {
  private first$: Observable<string | null>;

  constructor(private inbox: InboxService,
              private route: ActivatedRoute,
              private router: Router) {
    // this.route.paramMap.subscribe(params => {
    //     console.log(params.get('id'), this.inbox.docs.length);
    //     if (params.get('id') == null && this.inbox.docs.length > 0) {
    //       this.first = this.inbox.docs[0];
    //     } else {
    //       this.first = null;
    //     }
    //   });
  }

  canActivate(route: ActivatedRouteSnapshot,
              state: RouterStateSnapshot): Observable<boolean | UrlTree> | Promise<boolean | UrlTree> | boolean | UrlTree {
    return true;
    return this.first$.pipe(map(value => {
      if (value !== null) {
        return this.router.parseUrl(`inbox/${value}`);
      } else {
        return true;
      }
    }));
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
