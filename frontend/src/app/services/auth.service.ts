import {Injectable} from '@angular/core';
import {HttpClient} from '@angular/common/http';
import {BehaviorSubject} from 'rxjs';
import {environment} from '../../environments/environment';
import {first} from 'rxjs/operators';

export class User {
  username: string;
  token?: string;
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private user$: BehaviorSubject<User | null>;

  constructor(private http: HttpClient) {
    this.user$ = new BehaviorSubject<User>(JSON.parse(localStorage.getItem('user')));
  }

  public login(username: string, password: string): Promise<User> {
    return this.http.post<User>(`/api/authenticate`, {username, password})
      .pipe(first())
      .toPromise()
      .then(user => {
        // Store user details and JWT token in local storage to keep user logged in between page refreshes
        localStorage.setItem('user', JSON.stringify(user));
        this.user$.next(user);
        return user;
      });
  }

  public logout() {
    // Remove user from local storage to log out
    localStorage.removeItem('user');
    this.user$.next(null);
  }

  public get user(): User | null {
    return this.user$.value;
  }

  public get authenticated(): boolean {
    return this.user$.value != null;
  }
}
