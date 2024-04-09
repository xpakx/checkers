import { HttpErrorResponse } from '@angular/common/http';
import { Component } from '@angular/core';
import { GameRequest } from './main/dto/game-request';
import { GameResponse } from './main/dto/game-response';
import { GameManagementService } from './main/game-management.service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})
export class AppComponent {
  title = 'checkers';
  registerCard = false;
  gameId?: number;
  openRequestModal: boolean = false;
  requestModalForAI: boolean = false;

  constructor(private gameService: GameManagementService) { }

  get logged(): boolean {
    return localStorage.getItem("username") != null;
  }

  changeRegisterCard(value: boolean) {
    this.registerCard = value;
  }
  
  open(gameId: number) {
    this.gameId = gameId;
  }

  closeRequestModal(request: GameRequest) {
    this.openRequestModal = false;
    this.gameService.newGame(request)
      .subscribe({
        next: (game: GameResponse) => this.onRequestSent(game, request),
        error: (err: HttpErrorResponse) => this.onError(err)
      });

  }

  openGameModal(aiGame: boolean) {
    this.requestModalForAI = aiGame;
    this.openRequestModal = true;
  }

  onRequestSent(game: GameResponse, request: GameRequest) {
    if (request.type == "AI") {
      this.open(game.gameId);
    } else {
      // TODO
      console.log(`${request.opponent} invited to game`)
    }
  }

  onError(err: HttpErrorResponse) {
    // TODO
    console.log(err);
  }
}
