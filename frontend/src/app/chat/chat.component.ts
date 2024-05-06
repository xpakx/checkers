import { Component, OnDestroy, OnInit } from '@angular/core';
import { ChatMessage } from '../board/dto/chat-message';
import { Subscription } from 'rxjs';
import { WebsocketService } from '../board/websocket.service';

@Component({
  selector: 'app-chat',
  templateUrl: './chat.component.html',
  styleUrls: ['./chat.component.css']
})
export class ChatComponent implements OnInit, OnDestroy {
  chat: ChatMessage[] = [];
  private chatSub?: Subscription;

  constructor(private websocket: WebsocketService) { }

  ngOnInit(): void {
    this.chatSub = this.websocket.chat$
    .subscribe((placement: ChatMessage) => this.onChat(placement));
  }

  onChat(message: ChatMessage) {
    this.chat.push(message);
  }

  ngOnDestroy(): void {
    this.chatSub?.unsubscribe();
  }
}
