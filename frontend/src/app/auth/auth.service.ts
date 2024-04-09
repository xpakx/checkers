import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { environment } from 'src/environments/environment';
import { AuthRequest } from './dto/auth-request';
import { AuthResponse } from './dto/auth-response';
import { RefreshRequest } from './dto/refresh-request';
import { RegisterRequest } from './dto/register-request';

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private apiUrl: String = environment.apiUrl;

  constructor(protected http: HttpClient) { }

  public register(request: RegisterRequest): Observable<AuthResponse> {
    return this.http.post<AuthResponse>(`${this.apiUrl}/register`, request);
  }

  public login(request: AuthRequest): Observable<AuthResponse> {
    return this.http.post<AuthResponse>(`${this.apiUrl}/authenticate`, request);
  }

  public refreshToken(request: RefreshRequest): Observable<AuthResponse> {
    return this.http.post<AuthResponse>(`${this.apiUrl}/refresh`, request);
  }
}
