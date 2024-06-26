import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';

import { AppComponent } from './app.component';
import { FormsModule, ReactiveFormsModule } from '@angular/forms';
import { HTTP_INTERCEPTORS, HttpClientModule } from '@angular/common/http';
import { ModalRegisterComponent } from './auth/modal-register/modal-register.component';
import { ModalLoginComponent } from './auth/modal-login/modal-login.component';
import { ErrorInterceptor } from './error/error.interceptor';
import { MenuComponent } from './main/menu/menu.component';
import { GameListComponent } from './main/game-list/game-list.component';
import { NewGameModalComponent } from './main/new-game-modal/new-game-modal.component';
import { BoardComponent } from './board/board/board.component';
import { FieldPipe } from './board/field.pipe';
import { ButtonComponent } from './elements/button/button.component';
import { ToastComponent } from './elements/toast/toast.component';
import { MiniboardComponent } from './board/miniboard/miniboard.component';
import { ChatComponent } from './chat/chat.component';

@NgModule({
  declarations: [
    AppComponent,
    ModalRegisterComponent,
    ModalLoginComponent,
    MenuComponent,
    GameListComponent,
    NewGameModalComponent,
    BoardComponent,
    FieldPipe,
    ButtonComponent,
    ToastComponent,
    MiniboardComponent,
    ChatComponent
  ],
  imports: [
    BrowserModule,
    HttpClientModule,
    FormsModule,
    ReactiveFormsModule,
  ],
  providers: [
    {
      provide: HTTP_INTERCEPTORS,
      useClass: ErrorInterceptor,
      multi: true
    }
  ],
  bootstrap: [AppComponent]
})
export class AppModule { }
