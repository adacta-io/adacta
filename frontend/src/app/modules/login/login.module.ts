import {NgModule} from '@angular/core';
import {CommonModule} from '@angular/common';
import {LoginComponent} from './login.component';
import {ClarityModule} from '@clr/angular';
import {FormsModule} from '@angular/forms';


@NgModule({
  declarations: [
    LoginComponent,
  ],
  imports: [
    CommonModule,
    ClarityModule,
    FormsModule,
  ]
})
export class LoginModule {
}
