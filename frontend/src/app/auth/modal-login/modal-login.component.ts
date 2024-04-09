import { HttpErrorResponse } from '@angular/common/http';
import { Component, EventEmitter, OnInit, Output } from '@angular/core';
import { FormGroup, FormBuilder } from '@angular/forms';
import { AuthService } from '../auth.service';
import { AuthRequest } from '../dto/auth-request';
import { AuthResponse } from '../dto/auth-response';

@Component({
  selector: 'app-modal-login',
  templateUrl: './modal-login.component.html',
  styleUrls: ['./modal-login.component.css']
})
export class ModalLoginComponent implements OnInit {
  loginForm: FormGroup;

  error: boolean = false;
  errorMsg: String = "";

  @Output() card: EventEmitter<boolean> = new EventEmitter<boolean>();

  constructor(private formBuilder: FormBuilder, private authService: AuthService) {
    this.loginForm = this.formBuilder.group({
      username: [''],
      password: [''],
    });
   }

  ngOnInit(): void {}

  login(): void {
    console.log(this.loginForm.value);
    if (this.loginForm.invalid) {
      return;
    }

    let request: AuthRequest = {
      username: this.loginForm.value.username,
      password: this.loginForm.value.password,
    };

    console.log(request);
    this.authService.login(request)
      .subscribe({
        next: (response: AuthResponse) => this.onLogin(response),
        error: (err: HttpErrorResponse) => this.onError(err)
      });
  }

  onLogin(response: AuthResponse) {
    this.error = false;
    localStorage.setItem('token', response.token.toString());
    localStorage.setItem('refresh', response.refreshToken.toString());
    localStorage.setItem('username', response.username.toString());
  }

  onError(err: HttpErrorResponse) {
    console.log(err);
    this.error = true;
    this.errorMsg = err.message;
  }

  goToRegistration() {
    this.card.emit(true);
  }
}
