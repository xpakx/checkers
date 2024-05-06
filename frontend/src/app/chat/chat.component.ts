import { Component, OnDestroy, OnInit } from '@angular/core';
import { ChatMessage } from '../board/dto/chat-message';
import { Subscription } from 'rxjs';
import { WebsocketService } from '../board/websocket.service';
import { FormBuilder, FormGroup } from '@angular/forms';

@Component({
  selector: 'app-chat',
  templateUrl: './chat.component.html',
  styleUrls: ['./chat.component.css']
})
export class ChatComponent implements OnInit, OnDestroy {
  chat: ChatMessage[] = [];
  private chatSub?: Subscription;
  chatForm: FormGroup;

  constructor(private formBuilder: FormBuilder, private websocket: WebsocketService) {
    this.chatForm = this.formBuilder.group({ message: [''] });
   }

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

  sendMessage() {
    if (this.chatForm.invalid) {
      return;
    }
    let message = this.chatForm.value.message;
    this.websocket.sendChat(message);
  }
}
