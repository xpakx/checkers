import { Component, Input, OnInit } from '@angular/core';

@Component({
  selector: 'app-board',
  templateUrl: './board.component.html',
  styleUrls: ['./board.component.css']
})
export class BoardComponent implements OnInit {
  board: ("WhiteKing" | "BlackKing" | "White" | "Black" | "Empty")[][] = 
  [
    ["Empty", "White", "Empty", "White", "Empty", "White", "Empty", "White"],
    ["White", "Empty", "White", "Empty", "White", "Empty", "White", "Empty"], 
    ["Empty", "White", "Empty", "White", "Empty", "White", "Empty", "White"],

    ["Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty"],
    ["Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty", "Empty"],

    ["Black", "Empty", "Black", "Empty", "Black", "Empty", "Black", "Empty"],
    ["Empty", "Black", "Empty", "Black", "Empty", "Black", "Empty", "Black"],
    ["Black", "Empty", "Black", "Empty", "Black", "Empty", "Black", "Empty"],

  ]
  _gameId?: number;

  @Input() set gameId(value: number | undefined) {
    this._gameId = value;
  }

  constructor() { }

  ngOnInit(): void {
  }

}
