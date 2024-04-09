import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';

import { AppComponent } from './app.component';
import { FormsModule, ReactiveFormsModule } from '@angular/forms';
import { HttpClientModule } from '@angular/common/http';
import { ModalRegisterComponent } from './auth/modal-register/modal-register.component';
import { ModalLoginComponent } from './auth/modal-login/modal-login.component';

@NgModule({
  declarations: [
    AppComponent,
    ModalRegisterComponent,
    ModalLoginComponent
  ],
  imports: [
    BrowserModule,
    HttpClientModule,
    FormsModule,
    ReactiveFormsModule,
  ],
  providers: [],
  bootstrap: [AppComponent]
})
export class AppModule { }
