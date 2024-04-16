import { HttpErrorResponse } from '@angular/common/http';
import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';
import { Game } from '../dto/game';
import { GameManagementService } from '../game-management.service';
import { ToastService } from 'src/app/elements/toast.service';

@Component({
  selector: 'app-game-list',
  templateUrl: './game-list.component.html',
  styleUrls: ['./game-list.component.css']
})
export class GameListComponent implements OnInit {
  @Input() games: Game[] = [];
  @Input() active: boolean = true;
  @Input() requests: boolean = false;

  @Output() openGame: EventEmitter<number> = new EventEmitter<number>();

  constructor(private gameService: GameManagementService, private toast: ToastService) { }

  ngOnInit(): void {
  }

  accept(gameId: number) {
    this.gameService.acceptRequest(gameId, {status: "Accepted"})
      .subscribe({
        next: (value: Boolean) => this.onAccept(gameId),
        error: (err: HttpErrorResponse) => this.onError(err)
      });
  }

  onAccept(gameId: number) {
    this.open(gameId);
    this.toast.createToast({message: "Request accepted", id: `rejection-${gameId}`, type: "info"});
  }

  reject(gameId: number) {
    this.gameService.acceptRequest(gameId, {status: "Rejected"})
      .subscribe({
        next: (value: Boolean) => this.onReject(gameId),
        error: (err: HttpErrorResponse) => this.onError(err)
      });

  }

  onReject(gameId: number) {
    this.games = this.games.filter((game) => game.id != gameId);
    this.toast.createToast({message: "Request rejected", id: `rejection-${gameId}`, type: "info"});
  }

  onError(err: HttpErrorResponse) {
    this.toast.createToast({message: err.error, id: `error-${new Date().toTimeString}`, type: "error"});
  }

  open(gameId: number) {
    this.openGame.emit(gameId);
  }
}
