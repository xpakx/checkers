import { HttpErrorResponse } from '@angular/common/http';
import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';
import { Game } from '../dto/game';
import { GameManagementService } from '../game-management.service';

@Component({
  selector: 'app-menu',
  templateUrl: './menu.component.html',
  styleUrls: ['./menu.component.css']
})
export class MenuComponent implements OnInit {
  games: Game[] = [];
  requestView: boolean = false;
  activeView: boolean = false;
  error: boolean = false;
  errorMsg: String = "";

  @Output() openGame: EventEmitter<number> = new EventEmitter<number>();
  @Output() openGameModal: EventEmitter<boolean> = new EventEmitter<boolean>();
  @Input() gameId?: number;

  constructor(private gameService: GameManagementService) { }

  ngOnInit(): void {
  }

  getRequests() {
    this.gameService.getGameRequests()
      .subscribe({
        next: (games: Game[]) => this.onRequests(games),
        error: (err: HttpErrorResponse) => this.onError(err)
      });
  }

  getGames() {
    this.gameService.getActiveGames()
      .subscribe({
        next: (games: Game[]) => this.onGames(games),
        error: (err: HttpErrorResponse) => this.onError(err)
      });
  }

  getArchive() {
    this.gameService.getFinishedGames()
      .subscribe({
        next: (games: Game[]) => this.onArchive(games),
        error: (err: HttpErrorResponse) => this.onError(err)
      });

  }

  onRequests(games: Game[]) {
    this.games = games;
    this.activeView = false;
    this.requestView = true;
  }

  onArchive(games: Game[]) {
    this.games = games;
    this.activeView = false;
    this.requestView = false;
  }

  onGames(games: Game[]) {
    this.games = games;
    this.activeView = true;
    this.requestView = false;
  }

  onError(err: HttpErrorResponse) {
    console.log(err);
    this.error = true;
    this.errorMsg = err.message;
  }
  
  newGame() {
    this.openGameModal.emit(false);
  }

  newAIGame() {
    this.openGameModal.emit(true);
  }

  open(gameId: number) {
    this.openGame.emit(gameId);
  }
}
