import {Injectable} from '@angular/core';
import {HttpEvent, HttpHandler, HttpHeaderResponse, HttpInterceptor, HttpRequest} from '@angular/common/http';
import {Observable, throwError} from 'rxjs';
import {AuthService} from './services/auth.service';
import {catchError, tap} from 'rxjs/operators';

@Injectable()
export class AuthInterceptor implements HttpInterceptor {

  constructor(private authService: AuthService) {
  }

  intercept(request: HttpRequest<unknown>, next: HttpHandler): Observable<HttpEvent<unknown>> {
    const token = this.authService.token;
    if (token != null) {
      request = request.clone({ setHeaders: {Authentication: token}});
    }

    return next.handle(request)
      .pipe(tap(event => {
          if (event instanceof HttpHeaderResponse) {
            const token = event.headers.get('Authorization');
            this.authService.update(token);
          }
        }),
        catchError(err => {
          if (err.status === 401) {
            this.authService.logout();
            // location.reload();
          }

          const error = err.error.message || err.statusText;
          return throwError(error);
        }));
  }
}
