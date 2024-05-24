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
    ["Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty"],
    ["Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty"],
    ["Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty"],
    ["Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty"],
    ["Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty"],
    ["Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty"],
    ["Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty"],
    ["Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty"],
  ]
  _gameId?: number; 
  private moveSub?: Subscription;
  private boardSub?: Subscription;
  game?: BoardMessage;
  viewClass: ("" | "mover" | "capture" | "target")[][] = Array(8).fill(Array(8).fill(""));

  moveStart?: number[] = undefined;
  currentMove: number[][] = [];
  currentMoveCapturing: boolean = false;

  myTurn: boolean = false;

  @Input() set gameId(value: number | undefined) {
    this._gameId = value;
    this.moveStart = undefined;
    this.currentMove = [];
    this.currentMoveCapturing = false
    this.myTurn = false;
    this.viewClass = Array(this.board.length).fill(null).map(() => Array(this.board.length).fill(""));
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
    console.log(start);
    console.log(end);
    let type = this.board[start[0]][start[1]];
    this.board[start[0]][start[1]] = "Empty";
    if (details.promotion) {
      type = type.replace("Pawn", "King") as "WhiteKing" | "RedKing";
    }
    this.board[end[0]][end[1]] = type;
    
    for (let index of details.captures) {
      let indices = this.mapIndex(index);
      this.board[indices[0]][indices[1]] = "Empty";
    }

    let username = localStorage.getItem("username");
    if (move.player == username) {
      this.myTurn = false;
    } else if (username == this.game?.username1 || username == this.game?.username2) {
      this.myTurn = true;
    }
  }

  mapIndex(index: number): number[] {
    var dim = this.board.length/2;

    // TODO: reversed board
    var rowIndex = Math.floor((index-1) / dim);
    var colIndex = (index-1) % dim;
    if (rowIndex % 2 == 0) {
      colIndex = colIndex*2+1;
    } else {
      colIndex = colIndex*2;
    }

    return [rowIndex, colIndex];
  }

  mapToIndex(i: number, j: number): number {
    var dim = this.board.length / 2;
    var colIndex = 0;

    if (i % 2 == 0) {
      colIndex = (j - 1) / 2;
    } else {
      colIndex = j / 2;
    }

    var index = i * dim + colIndex + 1;
    return index;
  }

  onBoard(board: BoardMessage) {
    console.log("Updating board");
    // TODO: errors?
    this.game = board;
    // TODO: reverse board for reds?
    this.board = board.currentState;
    this.viewClass = Array(this.board.length).fill(null).map(() => Array(this.board.length).fill(""));
    let username = localStorage.getItem("username");
    if (!username) {
      return;
    }
    if (board.userTurn) {
      this.myTurn = board.username1 == username;
    } else {
      this.myTurn = board.username2 == username;
    }
  }

  onCell(i: number, j: number) {
    if (!this.moveStart) {
      if (this.board[i][j] == "Empty") {
        return;
      }
      this.moveStart = [i, j];
      this.currentMove.push([i, j]);
      this.viewClass[i][j] = "mover";
      console.log(`starting move from ${this.moveStart}.`);
      return;
    }

    this.currentMove.push([i, j]);
    this.viewClass[i][j] = "target";
    console.log(`pushed ${[i, j]} to move.`);
    console.log(`move at the moment: ${this.currentMove}`);

    const len = this.currentMove.length
    if (len < 2) {
      return;
    }

    let capture = this.testCapture(this.currentMove[len-2], this.currentMove[len-1]);
    if (capture) {
      console.log("move with capture");
      this.currentMoveCapturing = true;
    }

    if (!capture || this.testMoveEnd(this.moveStart, [i, j])) {
      let move = this.currentMove
        .map((p) => this.mapToIndex(p[0], p[1]))
        .join(this.currentMoveCapturing ? "x" : "-");
      this.currentMove = [];
      this.currentMoveCapturing = false;
      this.moveStart = undefined;
      this.viewClass = Array(this.board.length).fill(null).map(() => Array(this.board.length).fill(""));
      this.websocket.makeMove(move);
    }
  }

  testMoveEnd(mover: number[], target: number[]): boolean {
    const field = this.board[mover[0]][mover[1]];
    const row = target[0];
    const column = target[1];

    if (field == "WhiteKing") {
      console.log("White king is moving");
      const rightDown = this.isCaptureable([row+1, column+1], target, "Red")
      const leftDown = this.isCaptureable([row+1, column-1], target, "Red")
      const rightUp = this.isCaptureable([row-1, column+1], target, "Red")
      const leftUp = this.isCaptureable([row-1, column-1], target, "Red")
      return !rightDown && !leftDown && !rightUp && !leftUp;
    } else if (field == "WhitePawn") {
      console.log("White pawn is moving");
      const right = this.isCaptureable([row+1, column+1], target, "Red")
      const left = this.isCaptureable([row+1, column-1], target, "Red")
      return !right && !left;
    } else if (field == "RedKing") {
      console.log("Red king is moving");
      const rightDown = this.isCaptureable([row+1, column+1], target, "White")
      const leftDown = this.isCaptureable([row+1, column-1], target, "White")
      const rightUp = this.isCaptureable([row-1, column+1], target, "White")
      const leftUp = this.isCaptureable([row-1, column-1], target, "White")
      return !rightDown && !leftDown && !rightUp && !leftUp;
    } else if (field == "RedPawn") {
      console.log("Red pawn is moving");
      const right = this.isCaptureable([row-1, column+1], target, "White")
      const left = this.isCaptureable([row-1, column-1], target, "White")
      return !right && !left;
    }
    return true;
  }

  isCaptureable(target: number[], capturer: number[], enemyColor: "Red" | "White"): boolean {
    const rowDist = Math.abs(target[0] - capturer[0]);
    const colDist = Math.abs(target[1] - capturer[1]);
    if (rowDist != 1 || colDist != 1) {
      return false;
    }
    if (target[0] < 0 || target[0] >= this.board.length) {
      return false;
    }
    if (target[1] < 0 || target[1] >= this.board[target[0]].length) {
      return false;
    }
    let targetPawn = this.board[target[0]][target[1]];
    let isEnemy = targetPawn.startsWith(enemyColor);
    if (!isEnemy) {
      return false;
    }

    const nextRow = target[0] > capturer[0] ? target[0]+1 : target[0]-1;
    const nextCol = target[1] > capturer[1] ? target[1]+1 : target[1]-1;
    if (nextRow < 0 || nextRow >= this.board.length) {
      return false;
    }
    if (nextCol < 0 || nextCol >= this.board[nextRow].length) {
      return false;
    }

    let fieldAfterTarget = this.board[nextRow][nextCol];
    return fieldAfterTarget == "Empty";
  }

  testCapture(lastPosition: number[], newPosition: number[]): boolean {
    const rowDiff = Math.abs(newPosition[0] - lastPosition[0]);
    const colDiff = Math.abs(newPosition[1] - lastPosition[1]);

    if (rowDiff !== 2 || colDiff !== 2) {
      return false;
    }

    const capturedRow = (lastPosition[0] + newPosition[0]) / 2;
    const capturedCol = (lastPosition[1] + newPosition[1]) / 2;

    const field = this.board[capturedRow][capturedCol];

    if (field === "RedKing" || field == "RedPawn") { // TODO: for reds
      this.viewClass[capturedRow][capturedCol] = "capture";
      return true;
    }
    return false;
  }
}
