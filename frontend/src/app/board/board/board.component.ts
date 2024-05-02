import { Component, Input, OnDestroy, OnInit } from '@angular/core';
import { WebsocketService } from '../websocket.service';
import { BoardMessage } from '../dto/board-message';
import { MoveMessage } from '../dto/move-message';
import { ChatMessage } from '../dto/chat-message';
import { Subscription } from 'rxjs';

@Component({
  selector: 'app-board',
  templateUrl: './board.component.html',
  styleUrls: ['./board.component.css']
})
export class BoardComponent implements OnInit, OnDestroy {
  board: ("WhiteKing" | "RedKing" | "WhitePawn" | "RedPawn" | "Empty")[][] = 
  [
    ["Empty", "WhitePawn", "Empty", "WhitePawn", "Empty", "WhitePawn", "Empty", "WhitePawn"],
    ["WhitePawn", "Empty", "WhitePawn", "Empty", "WhitePawn", "Empty", "WhitePawn", "Empty"], 
    ["Empty", "WhitePawn", "Empty", "WhitePawn", "Empty", "WhitePawn", "Empty", "WhitePawn"],

    ["Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty"],
    ["Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty"],

    ["RedPawn", "Empty", "RedPawn", "Empty", "RedPawn", "Empty", "RedPawn", "Empty"],
    ["Empty", "RedPawn", "Empty", "RedPawn", "Empty", "RedPawn", "Empty", "RedPawn"],
    ["RedPawn", "Empty", "RedPawn", "Empty", "RedPawn", "Empty", "RedPawn", "Empty"],

  ]
  _gameId?: number; 
   private moveSub?: Subscription;
  private boardSub?: Subscription;
  private chatSub?: Subscription;
  game?: BoardMessage;

  @Input() set gameId(value: number | undefined) {
    this._gameId = value;
  }

  constructor(private websocket: WebsocketService) { }

  ngOnInit(): void {
    this.boardSub = this.websocket.board$
      .subscribe((board: BoardMessage) => this.onBoard(board));

    this.moveSub = this.websocket.move$
    .subscribe((move: MoveMessage) => this.onMove(move));

    this.chatSub = this.websocket.chat$
    .subscribe((placement: ChatMessage) => this.onChat(placement));
  }

  ngOnDestroy() {
    this.websocket.disconnect();
    this.boardSub?.unsubscribe();
    this.moveSub?.unsubscribe();
    this.chatSub?.unsubscribe();
  }

  onMove(move: MoveMessage) {
    if (!move.legal) {
      return;
    }
    // TODO
  }

  onChat(move: ChatMessage) {
    // TODO
  }

  onBoard(board: BoardMessage) {
    // TODO
  }

}
