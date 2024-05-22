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
    console.log(start);
    console.log(end);
    let type = this.board[start[0]][start[1]];
    this.board[start[0]][start[1]] = "Empty";
    this.board[end[0]][end[1]] = type; // TODO: promotion
    
    for (let index of details.captures) {
      let indices = this.mapIndex(index);
      this.board[indices[0]][indices[1]] = "Empty";
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
  }

  onCell(i: number, j: number) {
    if (!this.moveStart) {
      this.moveStart = [i, j];
      this.currentMove.push([i, j]);
      console.log(`starting move from ${this.moveStart}.`);
      return;
    }

    this.currentMove.push([i, j]);
    console.log(`pushed ${[i, j]} to move.`);
    console.log(`move at the moment: ${this.currentMove}`);

    const len = this.currentMove.length
    if (len < 2) {
      return;
    }

    if (this.testCapture(this.currentMove[len-1], this.currentMove[len-2])) {
      console.log("move with capture");
      this.currentMoveCapturing = true;
    }

    if (this.testMoveEnd(this.moveStart, [i, j])) {
      let move = this.currentMove
        .map((p) => this.mapToIndex(p[0], p[1]))
        .join(this.currentMoveCapturing ? "x" : "-");
      this.currentMove = [];
      this.currentMoveCapturing = false;
      this.moveStart = undefined;
      this.websocket.makeMove(move);
    }
  }

  testMoveEnd(mover: number[], target: number[]): boolean {
    const field = this.board[mover[0]][mover[1]];
    const row = target[0];
    const column = target[1];

    if (field == "WhiteKing") {
      return !this.testNeighboursForKing(row, column, "Red");
    } else if (field == "WhitePawn") {
      if (row-1 < 0) {
        return true;
      }
      return !this.testNeighboursInRow(row-1, column, "Red");
    } else if (field == "RedKing") {
      return !this.testNeighboursForKing(row, column, "White");
    } else if (field == "RedPawn") {
      if (row+1 >= this.board.length) {
        return true;
      }
      return !this.testNeighboursInRow(row+1, column, "Red");
    }
    return true;
  }

  testNeighboursForKing(row: number, column: number, enemyColor: "Red" | "White"): boolean {
      if (row + 1 < this.board.length) {
        if (this.testNeighboursInRow(row + 1, column, enemyColor)) {
          return true;
        }
      }
      if (row-1 >= 0) {
        if(this.testNeighboursInRow(row-1, column, enemyColor)) {
          return true;
        }
      }
      return false;
  }

  testNeighboursInRow(row: number, column: number, enemyColor: "Red" | "White") {
    if (column + 1 < this.board[row].length) {
      const neighbourRight = this.board[row][column + 1];
      if (neighbourRight.startsWith(enemyColor)) {
        return true;
      }
    }
    if (column - 1 >= 0) {
      const neighbourLeft = this.board[row][column - 1];
      if (neighbourLeft.startsWith(enemyColor)) {
        return true;
      }
    }
    return false;
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
      return true;
    }
    return false;
  }
}
