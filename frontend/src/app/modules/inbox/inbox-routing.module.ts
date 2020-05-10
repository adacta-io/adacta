import {NgModule} from '@angular/core';
import {RouterModule, Routes} from '@angular/router';
import {ReviewComponent} from './review.component';
import {InboxComponent} from './inbox.component';
import {CanActivateEmpty, EmptyComponent} from './empty.component';

const routes: Routes = [
  {
    path: 'inbox',
    component: InboxComponent,
    children: [
      {
        path: ':id',
        component: ReviewComponent,
        pathMatch: 'full',
      },
      {
        path: '',
        component: EmptyComponent,
        pathMatch: 'full',
        canActivate: [CanActivateEmpty],
      },
    ],
  },
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule],
  providers: [CanActivateEmpty],
})
export class InboxRoutingModule {
}
