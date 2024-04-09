import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { environment } from 'src/environments/environment';
import { AcceptRequest } from './dto/accept-request';
import { Game } from './dto/game';
import { GameRequest } from './dto/game-request';
import { GameResponse } from './dto/game-response';

@Injectable({
  providedIn: 'root'
})
export class GameManagementService {
  private apiUrl: String = environment.apiUrl + "/game";

  constructor(protected http: HttpClient) { }

  private getHeaders(): HttpHeaders {
    let token = localStorage.getItem("token");
    return new HttpHeaders({'Authorization':`Bearer ${token}`});
  }

  public newGame(request: GameRequest): Observable<GameResponse> {
    return this.http.post<GameResponse>(`${this.apiUrl}`, request, { headers: this.getHeaders() });
  }

  public getActiveGames(): Observable<Game[]> {
    return this.http.get<Game[]>(`${this.apiUrl}`, { headers: this.getHeaders() });
  }

  public getGameRequests(): Observable<Game[]> {
    return this.http.get<Game[]>(`${this.apiUrl}/request`, { headers: this.getHeaders() });
  }

  public getFinishedGames(): Observable<Game[]> {
    return this.http.get<Game[]>(`${this.apiUrl}/archive`, { headers: this.getHeaders() });
  }
  
  public acceptRequest(gameId: number, request: AcceptRequest): Observable<Boolean> {
    return this.http.post<Boolean>(`${this.apiUrl}/${gameId}/request`, request, { headers: this.getHeaders() });
  }

  public getGame(gameId: number): Observable<Game> {
    return this.http.get<Game>(`${this.apiUrl}/${gameId}`, { headers: this.getHeaders() });
  }
}
