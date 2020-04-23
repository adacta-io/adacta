import {NgModule} from '@angular/core';
import {Routes, RouterModule} from '@angular/router';
import {SearchComponent} from './modules/search/search.component';
import {TagsComponent} from './modules/tags/tags.component';

const routes: Routes = [
  {path: 'login', component: SearchComponent},
  {path: 'search', component: SearchComponent},
  {path: 'tags', component: TagsComponent},
];

@NgModule({
  imports: [RouterModule.forRoot(routes, {
    enableTracing: true,
  })],
  exports: [RouterModule]
})
export class AppRoutingModule {
}
