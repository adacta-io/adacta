import {Component, Input, OnInit, Output} from '@angular/core';
import {AuthService} from '../../shared/services/auth.service';

@Component({
  selector: 'app-login',
  templateUrl: './login.component.html',
  styleUrls: ['./login.component.less']
})
export class LoginComponent {

  public username = '';

  public password = '';

  constructor(private auth: AuthService) {
  }

  public submit(): void {
    this.auth.login(this.username, this.password);
  }
}
