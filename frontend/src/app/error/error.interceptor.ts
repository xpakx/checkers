import { Injectable } from '@angular/core';
import {
  HttpRequest,
  HttpHandler,
  HttpEvent,
  HttpInterceptor,
  HttpErrorResponse
} from '@angular/common/http';
import { Observable, catchError, switchMap, throwError } from 'rxjs';
import { AuthService } from '../auth/auth.service';
import { AuthResponse } from '../auth/dto/auth-response';

@Injectable()
export class ErrorInterceptor implements HttpInterceptor {

  constructor(private authService: AuthService) {}

  intercept(request: HttpRequest<unknown>, next: HttpHandler): Observable<HttpEvent<unknown>> {
    if (request.url.includes('/refresh')) {
      return next.handle(request);
    }

    return next.handle(request).pipe(
      catchError(
        (error: HttpErrorResponse) => {
          return this.testAuthorization(request, next, error);
        }
      )
    );
  }

  private testAuthorization(request: HttpRequest<unknown>, next: HttpHandler, error: HttpErrorResponse): Observable<HttpEvent<unknown>>  {
    if (error.status === 401) {
      return this.handleUnauthorizedError(request, next, error)
    } else {
      return throwError(() => error)
    }
  }

  private handleUnauthorizedError(request: HttpRequest<unknown>, next: HttpHandler, error: HttpErrorResponse): Observable<HttpEvent<unknown>> {
    let token = localStorage.getItem("refresh");
    if (!token) {
        this.clearStorage();
        return throwError(() => error);
    }

    return this.authService.refreshToken({"token": token}).pipe(
      switchMap((response: AuthResponse) => {
        localStorage.setItem('refresh', response.refreshToken.toString());
        localStorage.setItem('token', response.token.toString());
        localStorage.setItem('username', response.username.toString());
        const newRequest = request.clone({
          setHeaders: {
            Authorization: `Bearer ${response.token}`
          }
        });
        return next.handle(newRequest);
      }),
      catchError((err: any) => {
        this.clearStorage();
        return throwError(() => error);
      })
    );
  }

  private clearStorage(): void {
    localStorage.removeItem('refresh');
    localStorage.removeItem('token');
    localStorage.removeItem('username');
  }
}
