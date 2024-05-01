import { Injectable } from '@angular/core';
import { Observable, Subject } from 'rxjs';
import { environment } from 'src/environments/environment';
import { BoardMessage } from './dto/board-message';
import { MoveMessage } from './dto/move-message';
import { MoveRequest } from './dto/move-request';
import { ChatMessage } from './dto/chat-message';

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

  constructor() { 
    this.apiUrl = environment.apiUrl.replace(/^http/, 'ws');
    if (!this.apiUrl.startsWith("ws")) {
      let frontendUrl = window.location.origin.replace(/^http/, 'ws');
      this.apiUrl = frontendUrl + environment.apiUrl;
    }
    console.log(this.apiUrl);
  }

  connect() {
    // TODO
    this.subject = new WebSocket(this.apiUrl);

  }

  makeMove(gameId: number, move: MoveRequest) {
  }


  subscribeGame(gameId: number) {
  }

  disconnect() {
  }
}
