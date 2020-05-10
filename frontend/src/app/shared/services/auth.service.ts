import {Injectable} from '@angular/core';
import {HttpClient} from '@angular/common/http';
import {BehaviorSubject, Observable} from 'rxjs';
import {first, map} from 'rxjs/operators';

export class Auth {
  username: string;
  token: string;
}

@Injectable()
export class AuthService {
  private token$: BehaviorSubject<string | null>;
  private username$: BehaviorSubject<string | null>;

  constructor(private http: HttpClient) {
    this.token$ = new BehaviorSubject(localStorage.getItem('token'));
    this.username$ = new BehaviorSubject(localStorage.getItem('username'));
  }

  public login(username: string, password: string) {
    this.http.post<Auth>(`/api/auth`, {username, password})
      .pipe(first())
      .subscribe(auth => {
        // Store auth state in local storage to keep user logged in between page refreshes
        localStorage.setItem('token', auth.token);
        localStorage.setItem('username', auth.username);
        this.token$.next(auth.token);
        this.username$.next(auth.username);
        return auth;
      });
  }

  public update(token: string) {
    this.token$.next(token);
    localStorage.setItem('token', token);
  }

  public logout() {
    // Remove auth state from local storage to log out
    localStorage.removeItem('token');
    localStorage.removeItem('username');
    this.token$.next(null);
    this.username$.next(null);
  }

  public get token(): string | null {
    return this.token$.value;
  }

  public get username(): string | null {
    return this.username$.value;
  }

  public get authenticated(): boolean {
    // TODO: Check if token is expired
    return this.token$.value !== null;
  }
}
