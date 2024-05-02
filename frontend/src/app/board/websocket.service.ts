import { Injectable } from '@angular/core';
import { Observable, Subject } from 'rxjs';
import { environment } from 'src/environments/environment';
import { BoardMessage } from './dto/board-message';
import { MoveMessage } from './dto/move-message';
import { MoveRequest } from './dto/move-request';
import { ChatMessage } from './dto/chat-message';
import { SubscribeRequest } from './dto/subscribe-request';
import { ChatRequest } from './dto/chat-request';

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
    this.subject = new WebSocket(this.apiUrl);
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
    if (!this.subject) {
      return;
    }
    let request: SubscribeRequest = {path: "/subscribe", game_id: gameId};
    this.subject.send(JSON.stringify(request));
  }

  disconnect() {
    this.subject?.close();
  }

  onMessage(event: MessageEvent<any>) {
    console.log(event);
    let response: any = JSON.parse(event.data);
    if ("move" in response) {
      this.moveSubject.next(response as MoveMessage);
    } else if ("message" in response) {
      this.chatSubject.next(response as ChatMessage);
    } else if ("username1" in response) {
      this.boardSubject.next(response as BoardMessage);
    }
    console.log(event);
  }

  onClose() {
    // TODO: reconnect?
    console.log("Closed");
  }
}
