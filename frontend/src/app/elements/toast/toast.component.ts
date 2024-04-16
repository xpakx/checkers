import { Component, OnDestroy, OnInit } from '@angular/core';
import { Subscription } from 'rxjs';
import { ToastService } from '../toast.service';
import { Toast } from '../dto/toast';

@Component({
  selector: 'app-toast',
  templateUrl: './toast.component.html',
  styleUrls: ['./toast.component.css']
})
export class ToastComponent implements OnInit, OnDestroy {
  private toastSub?: Subscription;
  toasts: Toast[] = [];

  constructor(private toastService: ToastService) { }

  ngOnInit(): void {
    this.toastSub = this.toastService.toast$
      .subscribe((toast: Toast) => this.onToast(toast));
  }

  ngOnDestroy(): void {
    this.toastSub?.unsubscribe();
  }

  onToast(toast: Toast) {
    this.toasts.push(toast);
    let time = toast.time !== undefined ? toast.time : 5000;
    this.prepareDeletion(toast.id, time)
  }

  prepareDeletion(id: String, time: number) {
    setTimeout(() => {
      const index = this.toasts.findIndex(item => item.id === id);
      if (index !== -1) {
        this.toasts.splice(index, 1);
      }
    }, time);
  }

}
