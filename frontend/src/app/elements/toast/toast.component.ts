import { Component, OnInit } from '@angular/core';
import { Subscription } from 'rxjs';
import { ToastService } from '../toast.service';
import { Toast } from '../dto/toast';

@Component({
  selector: 'app-toast',
  templateUrl: './toast.component.html',
  styleUrls: ['./toast.component.css']
})
export class ToastComponent implements OnInit {
  private toastSub?: Subscription;
  toasts: Toast[] = [];

  constructor(private toastService: ToastService) { }


  ngOnInit(): void {
    this.toastSub = this.toastService.toast$
      .subscribe((toast: Toast) => this.onToast(toast));
  }

  ngOnDestroy(): void {
    this.toastSub?.unsubscribe;
  }

  onToast(toast: Toast) {
    this.toasts.push(toast);
  }

}
