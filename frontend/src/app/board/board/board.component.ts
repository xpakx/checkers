import { Component, Input, OnDestroy, OnInit } from '@angular/core';
import { WebsocketService } from '../websocket.service';
import { BoardMessage } from '../dto/board-message';
import { MoveMessage } from '../dto/move-message';
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
  game?: BoardMessage;

  moveStart?: number[] = undefined;
  currentMove: number[][] = [];
  currentMoveCapturing: boolean = false;


  @Input() set gameId(value: number | undefined) {
    this._gameId = value;
    if (value) {
      this.websocket.connect();
      this.websocket.subscribeGame(value);
    }
  }

  constructor(private websocket: WebsocketService) { }

  ngOnInit(): void {
    this.boardSub = this.websocket.board$
      .subscribe((board: BoardMessage) => this.onBoard(board));

    this.moveSub = this.websocket.move$
    .subscribe((move: MoveMessage) => this.onMove(move));
  }

  ngOnDestroy() {
    this.websocket.disconnect();
    this.boardSub?.unsubscribe();
    this.moveSub?.unsubscribe();
  }

  onMove(move: MoveMessage) {
    if (!move.legal) {
      return;
    }
    let details = move.details; 
    if (!details) {
      return;
    }
    
    let start = this.mapIndex(details.start);
    let end = this.mapIndex(details.end);
    let type = this.board[start[0]][start[1]];
    this.board[start[0]][start[1]] = "Empty";
    this.board[end[0]][end[1]] = type; // TODO: promotion
    
    for (let index of details.captures) {
      let indices = this.mapIndex(index);
      this.board[indices[0]][indices[1]] = "Empty";
    }
  }

  mapIndex(index: number): number[] {
    var dim = this.board.length;

    // TODO: reversed board
    var rowIndex = Math.floor(index / dim);
    var colIndex = index % dim;

    return [rowIndex, colIndex];
  }

  mapToIndex(i: number, j: number): number {
    // TODO: reversed board
    var dim = this.board.length;
    return i * dim + j;
  }

  onBoard(board: BoardMessage) {
    console.log("Updating board");
    // TODO: errors?
    this.game = board;
    // TODO: reverse board for reds?
    this.board = board.currentState;
  }

  onCell(i: number, j: number) {
    if (!this.moveStart) {
      this.moveStart = [i, j];
      this.currentMove.push([i, j]);
      return;
    }

    this.currentMove.push([i, j]);

    if (this.testCapture.length < 2) {
      return;
    }

    if (this.testCapture(this.currentMove[-1], this.currentMove[-2])) {
      this.currentMoveCapturing = true;
    }

    if (this.testMoveEnd(i, j)) {
      let move = this.currentMove
        .map((p) => this.mapToIndex(p[0], p[1]))
        .join(this.currentMoveCapturing ? "x" : "-");
      this.currentMove = [];
      this.currentMoveCapturing = false;
      this.moveStart = undefined;
      this.websocket.makeMove(move);
    }
  }

  testMoveEnd(i: number, j: number): boolean {
    // TODO
    return true;
  }

  testCapture(lastPosition: number[], newPosition: number[]): boolean {
    // TODO
    return true;
  }
}
