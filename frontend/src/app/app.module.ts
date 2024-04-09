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

@NgModule({
  declarations: [
    AppComponent,
    ModalRegisterComponent,
    ModalLoginComponent,
    MenuComponent,
    GameListComponent,
    NewGameModalComponent
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
