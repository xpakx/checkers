import { Injectable } from '@angular/core';
import { Observable, Subject } from 'rxjs';
import { environment } from 'src/environments/environment';
import { BoardMessage } from './dto/board-message';
import { MoveMessage } from './dto/move-message';
import { MoveRequest } from './dto/move-request';
import { ChatMessage } from './dto/chat-message';
import { SubscribeRequest } from './dto/subscribe-request';
import { ChatRequest } from './dto/chat-request';
import { AuthMessage } from './dto/auth-message';
import { AuthService } from '../auth/auth.service';
import { AuthResponse } from '../auth/dto/auth-response';

@Injectable({
  providedIn: 'root'
})
export class WebsocketService {
  private apiUrl: string;

  subject?: WebSocket;

  private boardSubject: Subject<BoardMessage> = new Subject<BoardMessage>();
  board$: Observable<BoardMessage> = this.boardSubject.asObservable();

  private moveSubject: Subject<MoveMessage> = new Subject<MoveMessage>();
  move$: Observable<MoveMessage> = this.moveSubject.asObservable();

  private chatSubject: Subject<ChatMessage> = new Subject<ChatMessage>();
  chat$: Observable<ChatMessage> = this.chatSubject.asObservable();

  private authenticated: boolean = false;
  private id?: number;

  constructor(private authService: AuthService) { 
    this.apiUrl = environment.apiUrl.replace(/^http/, 'ws');
    this.apiUrl = "ws://localhost:8081/ws";
    if (!this.apiUrl.startsWith("ws")) {
      let frontendUrl = window.location.origin.replace(/^http/, 'ws');
      this.apiUrl = frontendUrl + environment.apiUrl;
    }
    console.log(this.apiUrl);
  }

  connect() {
    if (this.subject?.readyState == WebSocket.OPEN) {
      return;
    }
    this.subject = new WebSocket(`${this.apiUrl}/ws`);
    this.subject.onmessage = (event: MessageEvent<any>) => this.onMessage(event);
    this.subject.onclose = () => this.onClose();
  }

  makeMove(move: String) {
    if (!this.subject) {
      return;
    }
    let request: MoveRequest = {path: "/move", move: move};
    this.subject.send(JSON.stringify(request));
  }

  sendChat(message: String) {
    if (!this.subject) {
      return;
    }
    let request: ChatRequest = {path: "/chat", message: message};
    this.subject.send(JSON.stringify(request));
  }

  subscribeGame(gameId: number) {
    this.id = gameId;
    if (!this.subject) {
      return;
    }
    if (this.subject.readyState == WebSocket.OPEN) {
      this.doSubscribe(gameId);
    } else {
      this.subject.onopen = () => this.doSubscribe(gameId);
    }

  }

  doSubscribe(gameId: number) {
    console.log(`trying to subscribe game ${gameId}`);
    let request: SubscribeRequest = { path: "/subscribe", game_id: gameId };
    this.subject!.send(JSON.stringify(request));
    if (!this.authenticated) {
      let authRequest = { path: "/auth", jwt: localStorage.getItem("token") };
      this.subject!.send(JSON.stringify(authRequest));
    }
  }

  disconnect() {
    this.subject?.close();
  }

  onMessage(event: MessageEvent<any>) {
    console.log(event);
    let response: any = JSON.parse(event.data);
    console.log(response);
    if ("move" in response) {
      console.log("Move message");
      this.moveSubject.next(response as MoveMessage);
    } else if ("message" in response) {
      console.log("Chat message");
      this.chatSubject.next(response as ChatMessage);
    } else if ("username1" in response) {
      console.log("Board message");
      this.boardSubject.next(response as BoardMessage);
    } else if ("authenticated" in response) {
      this.onAuth(response as AuthMessage);
    }
  }

  onClose() {
    console.log("Closed");
    this.authenticated = false;
    if (this.id) {
      this.connect();
      this.subscribeGame(this.id);
    }
  }

  onAuth(message: AuthMessage) {
    this.authenticated = message.authenticated;
    if (message.error && message.error.indexOf("expired") > 0) {
      let token = localStorage.getItem("refresh");
      if (!token) {
        this.clearStorage();
        return;
      }

      this.authService.refreshToken({ "token": token })
        .subscribe({
          next: (response: AuthResponse) => this.onRefresh(response),
          error: (_err: any) => this.clearStorage()
        });
    }
  }

  onRefresh(response: AuthResponse) {
    localStorage.setItem('refresh', response.refreshToken.toString());
    localStorage.setItem('token', response.token.toString());
    localStorage.setItem('username', response.username.toString());
    if (this.id) {
      this.subscribeGame(this.id);
    }
  }

  private clearStorage(): void {
    localStorage.removeItem('refresh');
    localStorage.removeItem('token');
    localStorage.removeItem('username');
  }
}
